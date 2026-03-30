//! Утилиты для работы с памятью процесса.
//!
//! ## Internal DLL
//!
//! Мы **внутри** процесса игры. Память игры — наша память.
//! Для объектов с известной структурой (`repr(C)`) кастим указатель
//! и работаем с полями напрямую. `read`/`write` нужны для:
//! - глобальных указателей (двойная косвенность из `globals.rs`)
//! - pointer-chasing по неотреверсенным цепочкам (inventory, frame)
//! - отладочных дампов и сканирования
//!
//! ## Основные инструменты
//!
//! - [`Ptr<T>`] — типизированная обёртка над игровым указателем
//! - [`read`] / [`write`] — чтение/запись по сырому адресу
//! - [`read_ptr`] — чтение указателя с валидацией
//! - [`follow_chain`] — проход по цепочке указателей
//! - [`fn_at`] — каст адреса в указатель на функцию

use std::ffi::CString;
use std::fmt;
use std::ptr;

use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::core::PCSTR;

// =============================================================================
//  Константы
// =============================================================================

/// Минимальный валидный user-mode адрес (Windows x64).
/// Первые 64 KiB зарезервированы системой.
pub const MIN_VALID_ADDR: usize = 0x10000;

/// Максимальный валидный user-mode адрес (Windows x64).
pub const MAX_VALID_ADDR: usize = 0x7FFF_FFFF_FFFF;

// =============================================================================
//  Provenance-корректные конверсии
// =============================================================================

/// Создаёт `*const T` из числового адреса с exposed provenance.
///
/// Заменяет `addr as *const T`. Старый способ теряет provenance
/// и вызывает lint `integer_to_ptr_transmutes`
#[inline(always)]
fn ptr_from_addr<T>(addr: usize) -> *const T {
    ptr::with_exposed_provenance(addr)
}

/// Создаёт `*mut T` из числового адреса с exposed provenance.
#[inline(always)]
fn ptr_mut_from_addr<T>(addr: usize) -> *mut T {
    ptr::with_exposed_provenance_mut(addr)
}

// =============================================================================
//  Информация о модулях
// =============================================================================

/// Информация о загруженном PE-модуле.
#[derive(Debug, Clone, Copy)]
pub struct ModuleInfo {
    /// Базовый адрес (адрес DOS-заголовка).
    pub base: usize,
    /// Размер образа в памяти (`SizeOfImage`).
    pub size: usize,
}

/// Получает базовый адрес модуля по имени.
///
/// Мы уже внутри процесса, поэтому `GetModuleHandle`
/// просто ищет в списке загруженных модулей — быстро.
pub fn get_module_base(module_name: &str) -> Option<usize> {
    let c_name = CString::new(module_name).ok()?;
    unsafe {
        let handle = GetModuleHandleA(PCSTR(c_name.as_ptr() as *const u8)).ok()?;
        Some(handle.0 as usize)
    }
}

/// Получает базовый адрес и размер модуля через PE-заголовок.
///
/// Парсит DOS -> PE -> Optional Header для `SizeOfImage`.
/// Модуль уже загружен в наше адресное пространство — читаем напрямую.
pub fn get_module_info(module_name: &str) -> Option<ModuleInfo> {
    let base = get_module_base(module_name)?;

    unsafe {
        let dos_magic = *ptr_from_addr::<u16>(base);
        if dos_magic != 0x5A4D {
            return None;
        }

        let pe_offset = *ptr_from_addr::<u32>(base + 0x3C) as usize;
        if pe_offset == 0 || pe_offset > 0x1000 {
            return None;
        }

        let pe_sig = *ptr_from_addr::<u32>(base + pe_offset);
        if pe_sig != 0x0000_4550 {
            return None;
        }

        // PE32+: COFF header (24) + offset 0x38 в Optional Header = 0x50
        let size = *ptr_from_addr::<u32>(base + pe_offset + 0x50) as usize;
        if size == 0 {
            return None;
        }

        Some(ModuleInfo { base, size })
    }
}

// =============================================================================
//  Проверка указателей
// =============================================================================

/// Быстрая проверка что адрес в user-mode диапазоне.
///
/// **Не проверяет** доступность страницы — для этого есть [`is_readable`].
/// Это дешёвая проверка (две инструкции cmp), подходит для горячих путей.
#[inline(always)]
pub fn is_valid_ptr(addr: usize) -> bool {
    (MIN_VALID_ADDR..=MAX_VALID_ADDR).contains(&addr)
}

/// Проверка адреса + выравнивание.
///
/// Типичное использование — проверка vtable-указателей (align = 8).
#[inline(always)]
pub fn is_aligned_ptr(addr: usize, align: usize) -> bool {
    debug_assert!(align.is_power_of_two(), "alignment must be a power of two");
    is_valid_ptr(addr) && (addr & (align - 1)) == 0
}

// =============================================================================
//  Чтение / запись по сырому адресу
// =============================================================================

/// Читает значение типа `T` по адресу. Проверяет диапазон.
///
/// Использует `read_unaligned` — выравнивание полей в реверснутых
/// структурах не всегда гарантировано.
///
/// ## Когда использовать
///
/// - Чтение глобальных указателей: `read::<usize>(base + GAME_MANAGER)`
/// - Pointer-chasing по неизвестным структурам (inventory chain)
/// - Поля из frame_node (нет `repr(C)` структуры)
///
/// ## Когда НЕ использовать
///
/// - Поля из `repr(C)` структур -> кастить через `Ptr<T>::as_ref()`
///
/// # Safety
///
/// Адрес должен указывать на валидную committed-память.
/// Байты должны представлять корректное значение `T`.
#[inline]
pub unsafe fn read<T: Copy>(addr: usize) -> Option<T> {
    if !is_valid_ptr(addr) {
        return None;
    }
    Some(unsafe { ptr::read_unaligned(ptr_from_addr::<T>(addr)) })
}

/// Записывает значение типа `T` по адресу.
///
/// # Safety
///
/// Адрес должен указывать на writable-память.
/// Запись не должна нарушать инварианты структур движка.
#[inline]
pub unsafe fn write<T: Copy>(addr: usize, value: T) -> bool {
    if !is_valid_ptr(addr) {
        return false;
    }
    unsafe { ptr::write_unaligned(ptr_mut_from_addr::<T>(addr), value) };
    true
}

/// Читает `usize` по адресу и проверяет что результат — валидный
/// user-mode указатель.
///
/// Основной инструмент для pointer-chasing:
/// ```ignore
/// let mgr = memory::read_validated_ptr(base + GAME_MANAGER)?;
/// let player = memory::read_validated_ptr(mgr + 0x180)?;
/// ```
///
/// # Safety
///
/// `addr` должен указывать на committed readable-память.
#[inline]
pub unsafe fn read_validated_ptr(addr: usize) -> Option<usize> {
    let value = unsafe { read::<usize>(addr)? };
    if is_valid_ptr(value) {
        Some(value)
    } else {
        None
    }
}

// Алиасы для удобства

/// Алиас для [`read_validated_ptr`]. Короткое имя для частого паттерна.
///
/// # Safety
///
/// `addr` должен указывать на committed readable-память.
#[inline]
pub unsafe fn read_ptr(addr: usize) -> Option<usize> {
    unsafe { read_validated_ptr(addr) }
}

/// Читает `usize` по адресу **без** проверки валидности результата.
///
/// В отличие от [`read_ptr`], возвращает значение даже если оно NULL
/// или kernel-адрес. Полезно когда нужно отличить NULL от ошибки чтения:
/// - `read_ptr_raw` -> `Some(0)` для NULL pointer
/// - `read_ptr` -> `None` для NULL pointer
///
/// # Safety
///
/// `addr` должен указывать на committed readable-память.
#[inline]
pub unsafe fn read_ptr_raw(addr: usize) -> Option<usize> {
    unsafe { read::<usize>(addr) }
}

/// Алиас для [`read`]. Явно показывает что читаем значение, а не указатель.
///
/// Семантически идентичен `read::<T>(addr)`, но лучше читается
/// в контексте полей, для которых нет `repr(C)` структуры:
/// ```ignore
/// // Поле frame_node — нет структуры, используем offset
/// let x = memory::read_value::<f32>(frame + 0x64)?;
/// ```
///
/// # Safety
///
/// Адрес должен указывать на валидную committed-память.
#[inline]
pub unsafe fn read_value<T: Copy>(addr: usize) -> Option<T> {
    unsafe { read::<T>(addr) }
}

// =============================================================================
//  Цепочки указателей
// =============================================================================

/// Проходит по цепочке указателей.
///
/// Типичный паттерн в движке Illusion Engine:
/// `[[base + 0x10] + 0x20] + 0x30`
///
/// ```ignore
/// // Эквивалентно: *(*(*(base + 0x10) + 0x20) + 0x30)
/// let addr = memory::follow_chain(base, &[0x10, 0x20, 0x30])?;
/// ```
///
/// Каждый шаг:
/// 1. Прибавляет смещение к текущему адресу
/// 2. Читает `usize` (указатель) по этому адресу
/// 3. Проверяет что прочитанное значение — валидный указатель
///
/// # Safety
///
/// Все промежуточные адреса должны быть readable.
pub unsafe fn follow_chain(base: usize, offsets: &[usize]) -> Option<usize> {
    if !is_valid_ptr(base) {
        return None;
    }

    let mut current = base;

    for &offset in offsets {
        let addr = current.checked_add(offset)?;
        if !is_valid_ptr(addr) {
            return None;
        }
        current = unsafe { ptr::read_unaligned(ptr_from_addr::<usize>(addr)) };
        if !is_valid_ptr(current) {
            return None;
        }
    }

    Some(current)
}

// =============================================================================
//  Указатели на функции
// =============================================================================

/// Каст абсолютного адреса в typed function pointer.
///
/// Для вызова **НЕвиртуальных** функций движка по RVA.
/// Виртуальные функции вызываются через vtable-структуры.
///
/// Mafia II: DE (x64) — Microsoft x64 calling convention.
/// В Rust это `unsafe extern "C" fn(...)`.
///
/// ```ignore
/// type AddWeaponFn = unsafe extern "C" fn(usize, u32, i32) -> u8;
/// let add_weapon: AddWeaponFn = unsafe { memory::fn_at(base + 0xD7_EF30) };
/// ```
///
/// # Safety
///
/// - `addr` должен указывать на начало корректной функции.
/// - Сигнатура `T` должна **точно** соответствовать реальной функции.
#[inline]
pub unsafe fn fn_at<T: Copy>(addr: usize) -> T {
    debug_assert!(
        is_valid_ptr(addr),
        "fn_at: invalid function address 0x{addr:X}"
    );
    debug_assert!(
        std::mem::size_of::<T>() == std::mem::size_of::<usize>(),
        "fn_at: T must be a function pointer"
    );
    let fn_ptr: *const () = ptr_from_addr(addr);
    unsafe { std::mem::transmute_copy(&fn_ptr) }
}

// =============================================================================
//  VTable (ad-hoc доступ)
// =============================================================================

/// Читает виртуальную функцию из vtable объекта по индексу.
///
/// Для ad-hoc доступа когда vtable **не описана** как `repr(C)` struct.
/// Если vtable описана — используй `(*entity.vtable).method(entity)`.
///
/// Layout (Illusion Engine, MSVC x64):
/// ```text
/// object -> [vtable_ptr][fields...]
/// vtable -> [vfn_0][vfn_1][vfn_2]...
/// ```
///
/// # Safety
///
/// - `object_addr` — объект с vtable в первых 8 байтах.
/// - `index` не должен выходить за пределы vtable.
pub unsafe fn vtable_fn<T: Copy>(object_addr: usize, index: usize) -> Option<T> {
    debug_assert!(
        std::mem::size_of::<T>() == std::mem::size_of::<usize>(),
        "vtable_fn: T must be a function pointer"
    );

    let vtable: usize = unsafe { read(object_addr)? };
    if !is_aligned_ptr(vtable, 8) {
        return None;
    }

    let fn_addr: usize = unsafe { read(vtable + index * 8)? };
    if !is_valid_ptr(fn_addr) {
        return None;
    }

    Some(unsafe { fn_at(fn_addr) })
}

/// Возвращает адрес vtable объекта (первые 8 байт).
///
/// # Safety
///
/// `object_addr` должен указывать на объект с vtable.
#[inline]
pub unsafe fn vtable_addr(object_addr: usize) -> Option<usize> {
    let vtable: usize = unsafe { read(object_addr)? };
    if is_aligned_ptr(vtable, 8) {
        Some(vtable)
    } else {
        None
    }
}

// =============================================================================
//  RIP-relative адресация
// =============================================================================

/// Разрешает RIP-relative адрес из x86-64 инструкции.
///
/// `effective = instruction_addr + instruction_len + disp32`
///
/// ```ignore
/// // 48 8B 0D [XX XX XX XX]  — mov rcx, [rip+disp32]
/// // offset_pos = 3, instruction_len = 7
/// let target = unsafe { memory::resolve_rip_relative(addr, 3, 7) };
/// ```
///
/// # Safety
///
/// `instruction_addr + offset_pos` должен указывать на 4 читаемых байта.
#[inline]
pub unsafe fn resolve_rip_relative(
    instruction_addr: usize,
    offset_pos: usize,
    instruction_len: usize,
) -> usize {
    let disp = unsafe { ptr::read_unaligned(ptr_from_addr::<i32>(instruction_addr + offset_pos)) };
    let rip = instruction_addr + instruction_len;
    (rip as isize + disp as isize) as usize
}

// =============================================================================
//  Ptr<T> — типизированная обёртка над игровым указателем
// =============================================================================

/// Типизированный указатель на объект в памяти игры.
///
/// Основной тип для работы с реверснутыми структурами движка.
/// Обёртка над `*mut T` с проверками и удобными методами.
///
/// ```ignore
/// // Получить указатель через pointer-chasing
/// let player = Ptr::<CHuman>::new(player_addr);
///
/// // Прямой доступ к полям структуры
/// let hp = unsafe { player.as_ref()?.current_health };
///
/// // Для неизвестных полей — read_at (пока нет структуры)
/// let unknown: u32 = unsafe { player.read_at(0x3D8)? };
/// ```
///
/// ## Когда что использовать
///
/// | Ситуация | Метод |
/// |----------|-------|
/// | Поле есть в `repr(C)` | `as_ref()?.field` |
/// | Поля нет в структуре | `read_at::<T>(offset)` |
/// | Нужен указатель на subobject | `deref_at::<U>(offset)` |
/// | Pointer-chain | `chain::<U>(&[0x10, 0x20])` |
#[repr(transparent)]
pub struct Ptr<T> {
    ptr: *mut T,
}

// Игровые указатели — просто адреса, можно передавать между потоками.
unsafe impl<T> Send for Ptr<T> {}
unsafe impl<T> Sync for Ptr<T> {}

impl<T> Copy for Ptr<T> {}
impl<T> Clone for Ptr<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> PartialEq for Ptr<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}
impl<T> Eq for Ptr<T> {}

impl<T> fmt::Debug for Ptr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Ptr<{}>(0x{:X})",
            std::any::type_name::<T>(),
            self.addr()
        )
    }
}

impl<T> fmt::Display for Ptr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:X}", self.addr())
    }
}

impl<T> fmt::LowerHex for Ptr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.addr(), f)
    }
}

impl<T> fmt::UpperHex for Ptr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(&self.addr(), f)
    }
}

impl<T> From<usize> for Ptr<T> {
    #[inline]
    fn from(addr: usize) -> Self {
        Self::new(addr)
    }
}

impl<T> From<*mut T> for Ptr<T> {
    #[inline]
    fn from(ptr: *mut T) -> Self {
        Self { ptr }
    }
}

impl<T> From<*const T> for Ptr<T> {
    #[inline]
    fn from(ptr: *const T) -> Self {
        Self {
            ptr: ptr.cast_mut(),
        }
    }
}

impl<T> Ptr<T> {
    /// Нулевой указатель.
    #[inline]
    pub const fn null() -> Self {
        Self {
            ptr: ptr::null_mut(),
        }
    }

    /// Создаёт Ptr из числового адреса.
    ///
    /// Использует `with_exposed_provenance_mut` (Rust 1.84+)
    /// вместо `addr as *mut T`.
    #[inline]
    pub fn new(addr: usize) -> Self {
        Self {
            ptr: ptr::with_exposed_provenance_mut(addr),
        }
    }

    /// Числовой адрес.
    ///
    /// Использует `expose_provenance` (Rust 1.84+)
    /// вместо `ptr as usize`.
    #[inline]
    pub fn addr(&self) -> usize {
        self.ptr.expose_provenance()
    }

    /// Сырой указатель (для передачи в функции движка).
    #[inline]
    pub fn raw(&self) -> *mut T {
        self.ptr
    }

    #[inline]
    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }

    /// Проверка что адрес в user-mode диапазоне.
    #[inline]
    pub fn is_valid(&self) -> bool {
        is_valid_ptr(self.addr())
    }

    // Доступ к объекту

    /// Получить `&T`. **Основной способ** работы с `repr(C)` структурами.
    ///
    /// ```ignore
    /// let entity = unsafe { ptr.as_ref()? };
    /// let tid = entity.table_id;       // прямое чтение
    /// let ft = entity.factory_type();  // вызов метода
    /// ```
    ///
    /// # Safety
    ///
    /// - Указатель должен быть aligned и dereferenceable.
    /// - Данные должны быть валидным `T`.
    /// - Время жизни ссылки не контролируется — игра может
    ///   освободить объект в любой момент.
    #[inline]
    pub unsafe fn as_ref(&self) -> Option<&T> {
        if !self.is_valid() {
            return None;
        }
        debug_assert!(
            self.ptr.is_aligned(),
            "Ptr::as_ref on unaligned pointer {self}"
        );
        Some(unsafe { &*self.ptr })
    }

    /// Получить `&mut T`.
    ///
    /// # Safety
    ///
    /// Те же требования что и для [`as_ref`](Self::as_ref),
    /// плюс не должно быть других ссылок на этот объект.
    #[inline]
    pub unsafe fn as_mut(&self) -> Option<&mut T> {
        if !self.is_valid() {
            return None;
        }
        debug_assert!(
            self.ptr.is_aligned(),
            "Ptr::as_mut on unaligned pointer {self}"
        );
        Some(unsafe { &mut *self.ptr })
    }

    /// Прочитать значение (копию). Использует `read_unaligned`.
    ///
    /// # Safety
    ///
    /// Указатель должен быть dereferenceable, данные — валидный `T`.
    #[inline]
    pub unsafe fn read(&self) -> Option<T>
    where
        T: Copy,
    {
        if !self.is_valid() {
            return None;
        }
        Some(unsafe { ptr::read_unaligned(self.ptr) })
    }

    /// Записать значение.
    ///
    /// # Safety
    ///
    /// Указатель должен быть dereferenceable и writable.
    #[inline]
    pub unsafe fn write(&self, value: T)
    where
        T: Copy,
    {
        debug_assert!(self.is_valid(), "Ptr::write to invalid address {self}");
        unsafe { ptr::write_unaligned(self.ptr, value) };
    }

    // Доступ по смещению

    /// Читает значение типа `U` по смещению от начала объекта.
    ///
    /// Для полей которых **нет** в `repr(C)` структуре.
    /// Если поле есть — используй `as_ref()?.field`.
    ///
    /// ```ignore
    /// // Поле известно но структура не обновлена:
    /// let flags: u32 = unsafe { player.read_at(0x3D8)? };
    /// ```
    ///
    /// # Safety
    ///
    /// `self.addr() + offset` должен быть readable.
    #[inline]
    pub unsafe fn read_at<U: Copy>(&self, offset: usize) -> Option<U> {
        let field_addr = self.addr().checked_add(offset)?;
        unsafe { read(field_addr) }
    }

    /// Записывает значение типа `U` по смещению.
    ///
    /// # Safety
    ///
    /// `self.addr() + offset` должен быть writable.
    #[inline]
    pub unsafe fn write_at<U: Copy>(&self, offset: usize, value: U) -> bool {
        let field_addr = self.addr().checked_add(offset).unwrap_or(0);
        unsafe { write(field_addr, value) }
    }

    /// Получить `Ptr<U>` к полю по смещению (**без** разыменования).
    ///
    /// Просто арифметика адреса, ничего не читает.
    #[inline]
    pub fn at<U>(&self, offset: usize) -> Ptr<U> {
        Ptr::new(self.addr() + offset)
    }

    /// Читает указатель по смещению и возвращает `Ptr<U>`.
    ///
    /// ```text
    /// self ──offset──-> [ptr_value] ──-> U
    /// ```
    ///
    /// # Safety
    ///
    /// `self.addr() + offset` должен содержать валидный указатель.
    #[inline]
    pub unsafe fn deref_at<U>(&self, offset: usize) -> Option<Ptr<U>> {
        let ptr_addr = self.addr().checked_add(offset)?;
        let target: usize = unsafe { read(ptr_addr)? };
        if is_valid_ptr(target) {
            Some(Ptr::new(target))
        } else {
            None
        }
    }

    // Цепочки указателей

    /// Проходит по цепочке указателей начиная от текущего адреса.
    ///
    /// ```ignore
    /// // [[player + 0x10] + 0x20]
    /// let weapon: Ptr<C_Weapon> = unsafe { player.chain(&[0x10, 0x20])? };
    /// ```
    ///
    /// # Safety
    ///
    /// Все промежуточные адреса должны быть readable.
    pub unsafe fn chain<U>(&self, offsets: &[usize]) -> Option<Ptr<U>> {
        let result = unsafe { follow_chain(self.addr(), offsets)? };
        Some(Ptr::new(result))
    }

    // VTable (ad-hoc)

    /// Адрес vtable объекта (первые 8 байт).
    ///
    /// Для ad-hoc доступа. Если vtable описана как struct —
    /// используй `as_ref()?.vtable` напрямую.
    ///
    /// # Safety
    ///
    /// Объект должен иметь vtable (полиморфный C++ класс).
    #[inline]
    pub unsafe fn vtable(&self) -> Option<usize> {
        unsafe { vtable_addr(self.addr()) }
    }

    /// Читает виртуальную функцию по индексу.
    ///
    /// Для ad-hoc доступа когда vtable **не описана** как struct.
    ///
    /// # Safety
    ///
    /// - Объект должен иметь vtable.
    /// - `index` не должен выходить за пределы vtable.
    /// - `F` должен точно соответствовать сигнатуре функции.
    #[inline]
    pub unsafe fn vtable_fn<F: Copy>(&self, index: usize) -> Option<F> {
        unsafe { crate::memory::vtable_fn(self.addr(), index) }
    }

    // Каст

    /// Каст к другому типу. Просто меняет фантомный тип, адрес тот же.
    ///
    /// ```ignore
    /// let base: Ptr<CEntity> = player_ptr.cast();
    /// ```
    #[inline]
    pub fn cast<U>(&self) -> Ptr<U> {
        Ptr::new(self.addr())
    }

    /// Добавляет байтовое смещение к адресу. Возвращает тот же тип.
    #[inline]
    pub fn offset(&self, bytes: isize) -> Self {
        Ptr::new((self.addr() as isize + bytes) as usize)
    }
}

// =============================================================================
//  Отладка / тяжёлые проверки (VirtualQuery)
// =============================================================================

/// Проверяет что `[addr .. addr+size)` реально committed и readable.
///
/// Использует `VirtualQuery` — **syscall**, медленнее чем [`is_valid_ptr`].
///
/// ## Когда использовать
///
/// - Дампы неизвестных структур
/// - Итерация по hash-таблицам EntityDatabase
/// - Debug-утилиты
/// - Первый доступ к подозрительному указателю
///
/// Для горячих путей используй [`is_valid_ptr`] + доверяй движку.
pub fn is_readable(addr: usize, size: usize) -> bool {
    use windows::Win32::System::Memory::{
        MEM_COMMIT, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE,
        PAGE_EXECUTE_WRITECOPY, PAGE_READONLY, PAGE_READWRITE, PAGE_WRITECOPY, VirtualQuery,
    };

    if !is_valid_ptr(addr) || size == 0 {
        return false;
    }

    if addr.checked_add(size).is_none() {
        return false;
    }

    let mut mbi = MEMORY_BASIC_INFORMATION::default();
    let result = unsafe {
        VirtualQuery(
            Some(ptr_from_addr::<std::ffi::c_void>(addr)),
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

    let readable = matches!(
        mbi.Protect,
        PAGE_READONLY
            | PAGE_READWRITE
            | PAGE_EXECUTE_READ
            | PAGE_EXECUTE_READWRITE
            | PAGE_WRITECOPY
            | PAGE_EXECUTE_WRITECOPY
    );

    if !readable {
        return false;
    }

    let region_end = mbi.BaseAddress as usize + mbi.RegionSize;
    addr + size <= region_end
}

/// Дампит `count` байт в hex+ASCII формат.
///
/// Проверяет доступность через [`is_readable`] перед чтением.
///
/// ```text
///   0x7FF6A000: 48 8B 05 AA BB CC DD 00  00 00 00 00 00 00 00 00 |H...............|
/// ```
pub fn hex_dump(addr: usize, count: usize) -> String {
    if count == 0 {
        return format!("  0x{addr:X}: <zero bytes>");
    }
    if !is_readable(addr, count) {
        return format!("  0x{addr:X}: <unreadable, {count} bytes>");
    }

    let bytes = unsafe { std::slice::from_raw_parts(ptr_from_addr::<u8>(addr), count) };
    let mut result = String::with_capacity(count / 16 * 80 + 80);

    for (i, chunk) in bytes.chunks(16).enumerate() {
        let line_addr = addr + i * 16;
        result.push_str(&format!("  0x{line_addr:X}: "));

        for (j, byte) in chunk.iter().enumerate() {
            if j == 8 {
                result.push(' ');
            }
            result.push_str(&format!("{byte:02X} "));
        }

        for j in chunk.len()..16 {
            if j == 8 {
                result.push(' ');
            }
            result.push_str("   ");
        }

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

/// Читает `T` с проверкой доступности страницы через `VirtualQuery`.
///
/// Медленнее чем [`read`], но гарантирует отсутствие access violation.
///
/// ## Когда использовать
///
/// - Обход hash-таблиц EntityDatabase (указатели могут быть stale)
/// - Первый доступ к подозрительному указателю
/// - Debug-утилиты
///
/// # Safety
///
/// Байты по адресу должны представлять валидное значение `T`.
pub unsafe fn read_safe<T: Copy>(addr: usize) -> Option<T> {
    if !is_readable(addr, std::mem::size_of::<T>()) {
        return None;
    }
    Some(unsafe { ptr::read_unaligned(ptr_from_addr::<T>(addr)) })
}

/// Читает указатель с `VirtualQuery` + проверкой что результат — валидный адрес.
///
/// # Safety
///
/// Доступность памяти проверяется автоматически.
pub unsafe fn read_validated_ptr_safe(addr: usize) -> Option<usize> {
    let value = unsafe { read_safe::<usize>(addr)? };
    if is_valid_ptr(value) {
        Some(value)
    } else {
        None
    }
}

impl<T> Ptr<T> {
    /// Получить `&T` с lifetime, не привязанным к `&self`.
    ///
    /// В отличие от [`as_ref`](Self::as_ref), этот метод подходит
    /// для случаев, когда ссылку нужно вернуть наружу из функции.
    ///
    /// ## Когда использовать
    ///
    /// - высокоуровневые обёртки (`Game::active_player()`)
    /// - функции, возвращающие `Option<&T>`
    ///
    /// ## Когда НЕ использовать
    ///
    /// - обычный локальный доступ к полям (`ptr.as_ref()?.field`)
    ///
    /// В локальном коде предпочтительнее [`as_ref`](Self::as_ref),
    /// потому что она даёт более строгие гарантии borrow checker'у.
    ///
    /// # Safety
    ///
    /// - указатель должен указывать на валидный живой объект `T`
    /// - объект не должен быть уничтожен движком в течение жизни ссылки
    /// - вызывающий полностью отвечает за корректность lifetime
    #[inline]
    pub unsafe fn to_ref<'a>(self) -> Option<&'a T> {
        if !self.is_valid() {
            return None;
        }
        debug_assert!(
            self.ptr.is_aligned(),
            "Ptr::to_ref on unaligned pointer {self}"
        );
        Some(unsafe { &*self.ptr })
    }

    /// Получить `&mut T` с lifetime, не привязанным к `&self`.
    ///
    /// Более опасный вариант [`as_mut`](Self::as_mut), нужен только там,
    /// где мутабельную ссылку необходимо вернуть наружу.
    ///
    /// # Safety
    ///
    /// - указатель должен быть валиден
    /// - объект должен быть writable
    /// - не должно существовать других ссылок на этот объект
    /// - вызывающий отвечает за корректность lifetime
    #[inline]
    pub unsafe fn to_mut<'a>(self) -> Option<&'a mut T> {
        if !self.is_valid() {
            return None;
        }
        debug_assert!(
            self.ptr.is_aligned(),
            "Ptr::to_mut on unaligned pointer {self}"
        );
        Some(unsafe { &mut *self.ptr })
    }
}