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

// ═══════════════════════════════════════════════════════════════════════════
//  Проверка указателей
// ═══════════════════════════════════════════════════════════════════════════

const MIN_VALID_ADDR: usize = 0x10000;
const MAX_VALID_ADDR: usize = 0x7FFF_FFFF_FFFF;

/// Проверяет, выглядит ли адрес как валидный user-mode указатель.
pub fn is_valid_ptr(addr: usize) -> bool {
    (MIN_VALID_ADDR..=MAX_VALID_ADDR).contains(&addr) && addr.is_multiple_of(8)
}

/// Безопасно читает указатель. Проверяет и адрес, и результат.
///
/// Возвращает `None` если адрес или прочитанное значение невалидны.
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

// ═══════════════════════════════════════════════════════════════════════════
//  Отладка
// ═══════════════════════════════════════════════════════════════════════════

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

pub unsafe fn fn_at<T: Copy>(addr: usize) -> T {
    let ptr = addr as *const ();
    unsafe { std::mem::transmute_copy(&ptr) }
}