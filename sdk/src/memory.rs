//! Утилиты для работы с памятью процесса.

use std::ffi::CString;

use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::core::PCSTR;

/// Информация о загруженном модуле.
#[derive(Debug, Clone, Copy)]
pub struct ModuleInfo {
    pub base: usize,
    pub size: usize,
}

/// Получает базовый адрес загруженного модуля по имени.
pub fn get_module_base(module_name: &str) -> Option<usize> {
    let c_name = CString::new(module_name).ok()?;
    unsafe {
        let handle = GetModuleHandleA(PCSTR(c_name.as_ptr() as *const u8)).ok()?;
        Some(handle.0 as usize)
    }
}

/// Получает базовый адрес и размер модуля через PE-заголовок.
pub fn get_module_info(module_name: &str) -> Option<ModuleInfo> {
    let base = get_module_base(module_name)?;

    unsafe {
        let dos_magic = std::ptr::read(base as *const u16);
        if dos_magic != 0x5A4D {
            return None;
        }

        let pe_offset = std::ptr::read((base + 0x3C) as *const u32) as usize;
        let pe_sig = std::ptr::read((base + pe_offset) as *const u32);
        if pe_sig != 0x0000_4550 {
            return None;
        }

        let size = std::ptr::read((base + pe_offset + 0x50) as *const u32) as usize;
        Some(ModuleInfo { base, size })
    }
}

// =============================================================================
//  Проверка указателей
// =============================================================================

const MIN_VALID_ADDR: usize = 0x10000;
const MAX_VALID_ADDR: usize = 0x7FFF_FFFF_FFFF;

/// Проверяет, выглядит ли адрес как валидный user-mode указатель.
///
/// Не проверяет выравнивание — это зависит от типа данных.
/// Для указателей на 8-байтные структуры используй is_aligned_ptr.
pub fn is_valid_ptr(addr: usize) -> bool {
    (MIN_VALID_ADDR..=MAX_VALID_ADDR).contains(&addr)
}

/// Проверка с учётом выравнивания.
/// Полезно для указателей на vtable, структуры, COM-объекты.
pub fn is_aligned_ptr(addr: usize, align: usize) -> bool {
    is_valid_ptr(addr) && addr.is_multiple_of(align)
}

/// Безопасно читает указатель (8 байт).
/// Проверяет что и адрес чтения, и прочитанное значение —
/// валидные user-mode адреса.
pub unsafe fn read_ptr(addr: usize) -> Option<usize> {
    if !is_valid_ptr(addr) {
        return None;
    }
    let value = unsafe { std::ptr::read(addr as *const usize) };
    if is_valid_ptr(value) {
        Some(value)
    } else {
        None
    }
}

/// Читает указатель **без проверки результата**.
///
/// Возвращает сырое значение (может быть 0, мусор и т.д.).
/// Проверяет только что адрес для чтения валиден.
pub unsafe fn read_ptr_raw(addr: usize) -> Option<usize> {
    if !is_valid_ptr(addr) {
        return None;
    }
    Some(unsafe { std::ptr::read(addr as *const usize) })
}

/// Безопасно читает значение типа `T`.
pub unsafe fn read_value<T: Copy>(addr: usize) -> Option<T> {
    if !is_valid_ptr(addr) {
        return None;
    }
    Some(unsafe { std::ptr::read(addr as *const T) })
}

/// Безопасно читает значение типа `T` без требования выравнивания.
///
/// Полезно для packed-структур, стековых временных объектов движка
/// и вообще любых полей, где игра может передать не-aligned адрес.
///
/// # Safety
/// Вызывающий всё ещё обязан убедиться, что адрес вообще читаем.
pub unsafe fn read_value_unaligned<T: Copy>(addr: usize) -> Option<T> {
    if !is_valid_ptr(addr) {
        return None;
    }
    Some(unsafe { std::ptr::read_unaligned(addr as *const T) })
}

/// Читает указатель без требования выравнивания.
///
/// Возвращает сырое значение без проверки, указывает ли оно на валидную память.
pub unsafe fn read_ptr_raw_unaligned(addr: usize) -> Option<usize> {
    if !is_valid_ptr(addr) {
        return None;
    }
    Some(unsafe { std::ptr::read_unaligned(addr as *const usize) })
}

/// Читает указатель без требования выравнивания и проверяет результат.
pub unsafe fn read_ptr_unaligned(addr: usize) -> Option<usize> {
    unsafe {
        let value = read_ptr_raw_unaligned(addr)?;
        if is_valid_ptr(value) {
            Some(value)
        } else {
            None
        }
    }
}

// =============================================================================
//  Проверка доступности страниц памяти
// =============================================================================

/// Проверяет что диапазон памяти `addr..addr+size` реально доступен для чтения.
///
/// Использует `VirtualQuery` для проверки что страница committed и readable.
/// Это **тяжелее** чем `is_valid_ptr`, но гарантирует отсутствие access violation.
///
/// Использовать для:
/// - дампов неизвестных структур
/// - итерации по hash table с произвольными указателями
/// - любых операций где нет 100% уверенности в валидности указателя
pub fn is_readable(addr: usize, size: usize) -> bool {
    use windows::Win32::System::Memory::{
        VirtualQuery, MEMORY_BASIC_INFORMATION, MEM_COMMIT,
        PAGE_READONLY, PAGE_READWRITE, PAGE_EXECUTE_READ,
        PAGE_EXECUTE_READWRITE, PAGE_WRITECOPY, PAGE_EXECUTE_WRITECOPY,
    };

    if addr == 0 || addr < MIN_VALID_ADDR || size == 0 {
        return false;
    }

    let mut mbi = MEMORY_BASIC_INFORMATION::default();
    let result = unsafe {
        VirtualQuery(
            Some(addr as *const std::ffi::c_void),
            &mut mbi,
            std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
        )
    };

    if result == 0 {
        return false;
    }

    if mbi.State != MEM_COMMIT {
        return false;
    }

    let readable = mbi.Protect == PAGE_READONLY
        || mbi.Protect == PAGE_READWRITE
        || mbi.Protect == PAGE_EXECUTE_READ
        || mbi.Protect == PAGE_EXECUTE_READWRITE
        || mbi.Protect == PAGE_WRITECOPY
        || mbi.Protect == PAGE_EXECUTE_WRITECOPY;

    if !readable {
        return false;
    }

    // Проверяем что весь запрошенный диапазон укладывается в один регион
    let region_end = mbi.BaseAddress as usize + mbi.RegionSize;
    addr + size <= region_end
}

/// Безопасно читает значение типа `T` с проверкой доступности страницы.
///
/// Медленнее чем `read_value`, но гарантирует отсутствие краша.
pub unsafe fn read_value_safe<T: Copy>(addr: usize) -> Option<T> {
    if !is_readable(addr, std::mem::size_of::<T>()) {
        return None;
    }
    Some(std::ptr::read(addr as *const T))
}

/// Безопасно читает указатель с проверкой доступности страницы.
pub unsafe fn read_ptr_safe(addr: usize) -> Option<usize> {
    let value = read_value_safe::<usize>(addr)?;
    if is_valid_ptr(value) { Some(value) } else { None }
}

// =============================================================================
//  Отладка
// =============================================================================

/// Дампит `count` байт начиная с `addr` в hex-формате.
///
/// Вывод в формате:
/// ```text
/// 0x51E552B0: 48 8B 05 AA BB CC DD 00  00 00 00 00 00 00 00 00
/// ```
///
/// Возвращает строку. Если адрес невалиден — возвращает сообщение об ошибке.
pub fn hex_dump(addr: usize, count: usize) -> String {
    if !is_valid_ptr(addr) {
        return format!("0x{addr:X}: <invalid address>");
    }

    let mut result = String::new();
    let bytes = unsafe { std::slice::from_raw_parts(addr as *const u8, count) };

    for (i, chunk) in bytes.chunks(16).enumerate() {
        let line_addr = addr + i * 16;
        result.push_str(&format!("  0x{line_addr:X}: "));

        // Hex
        for (j, byte) in chunk.iter().enumerate() {
            if j == 8 {
                result.push(' ');
            }
            result.push_str(&format!("{byte:02X} "));
        }

        // Padding если меньше 16 байт
        for j in chunk.len()..16 {
            if j == 8 {
                result.push(' ');
            }
            result.push_str("   ");
        }

        // ASCII
        result.push('|');
        for byte in chunk {
            if byte.is_ascii_graphic() || *byte == b' ' {
                result.push(*byte as char);
            } else {
                result.push('.');
            }
        }
        result.push('|');
        result.push('\n');
    }

    result
}

/// Разрешает RIP-relative адрес.
pub fn resolve_rip_relative(
    instruction_addr: usize,
    offset_pos: usize,
    instruction_len: usize,
) -> usize {
    let disp = unsafe { std::ptr::read((instruction_addr + offset_pos) as *const i32) };
    let rip = instruction_addr + instruction_len;
    (rip as isize + disp as isize) as usize
}

/// Безопасно записывает значение типа `T`.
pub unsafe fn write_value<T: Copy>(addr: usize, value: T) -> bool {
    if !is_valid_ptr(addr) {
        return false;
    }
    unsafe { std::ptr::write(addr as *mut T, value) };
    true
}

/// Преобразует абсолютный адрес функции в typed function pointer.
///
/// Использовать только для адресов кода, например:
/// `unsafe extern "C" fn(...) -> ...`.
///
/// # Safety
///
/// Вызывающий обязан гарантировать, что:
/// - `addr` указывает на корректную функцию
/// - сигнатура `T` точно соответствует реальной calling convention и параметрам
pub unsafe fn fn_at<T: Copy>(addr: usize) -> T {
    debug_assert!(
        is_valid_ptr(addr),
        "fn_at: invalid function address 0x{addr:X}"
    );
    let ptr = addr as *const ();
    unsafe { std::mem::transmute_copy(&ptr) }
}
