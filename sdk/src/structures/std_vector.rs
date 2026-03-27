//! MSVC `std::vector<T>` layout (x64).
//!
//! Движок Illusion Engine использует стандартный MSVC STL.
//! Layout: три последовательных указателя — begin, end, capacity.
//!
//! ```text
//! struct vector<T> {
//!     T* _Myfirst;   // begin  (+0x00)
//!     T* _Mylast;    // end    (+0x08)
//!     T* _Myend;     // cap    (+0x10)
//! };
//! ```
//!
//! Размер: 24 байта на x64.

use std::fmt;

/// MSVC `std::vector<T>` — три указателя.
///
/// Используется в `repr(C)` структурах движка вместо
/// ручных троек `begin / end / capacity`.
///
/// # Пример
///
/// ```ignore
/// pub struct GameCallbackManager {
///     pub vtable: *const c_void,
///     pub entries: StdVector<CallbackEventDesc>,     // +0x08
///     pub pending: StdVector<PendingFunctionOp>,     // +0x20
///     pub current_dispatch_ctx: *mut DispatchContext, // +0x38
/// }
///
/// let count = manager.entries.len();
/// let slice = unsafe { manager.entries.as_slice() };
/// ```
#[repr(C)]
pub struct StdVector<T> {
    /// `_Myfirst` — указатель на первый элемент.
    pub begin: *mut T,
    /// `_Mylast` — указатель за последний элемент.
    pub end: *mut T,
    /// `_Myend` — указатель на конец выделенной памяти.
    pub capacity: *mut T,
}

// ---------------------------------------------------------------------------
//  Безопасные методы (не разыменовывают указатели)
// ---------------------------------------------------------------------------

impl<T> StdVector<T> {
    /// Количество элементов: `(end - begin) / size_of::<T>()`.
    ///
    /// Безопасно — только арифметика указателей.
    /// Возвращает 0 если begin == null, end <= begin, или T zero-sized.
    #[inline]
    pub fn len(&self) -> usize {
        let size = std::mem::size_of::<T>();
        if size == 0 {
            return 0;
        }
        let b = self.begin as usize;
        let e = self.end as usize;
        if b == 0 || e <= b { 0 } else { (e - b) / size }
    }

    /// Вектор пуст или невалиден.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Ёмкость (количество элементов до реаллокации).
    #[inline]
    pub fn capacity_count(&self) -> usize {
        let size = std::mem::size_of::<T>();
        if size == 0 {
            return 0;
        }
        let b = self.begin as usize;
        let c = self.capacity as usize;
        if b == 0 || c <= b { 0 } else { (c - b) / size }
    }

    /// `begin` как числовой адрес (для логов).
    #[inline]
    pub fn begin_addr(&self) -> usize {
        self.begin as usize
    }

    /// `end` как числовой адрес.
    #[inline]
    pub fn end_addr(&self) -> usize {
        self.end as usize
    }

    /// Указатель `begin` валиден (не null, в user-mode диапазоне).
    #[inline]
    pub fn is_valid(&self) -> bool {
        let b = self.begin as usize;
        let e = self.end as usize;
        let c = self.capacity as usize;
        b >= 0x10000 && e >= b && c >= e
    }
}

// ---------------------------------------------------------------------------
//  Unsafe методы (разыменовывают указатели)
// ---------------------------------------------------------------------------

impl<T> StdVector<T> {
    /// Слайс элементов.
    ///
    /// # Safety
    ///
    /// - `begin..end` должны указывать на committed readable память.
    /// - Данные не должны модифицироваться конкурентно (game thread only).
    /// - Lifetime возвращённого слайса не контролируется — движок может
    ///   реаллоцировать вектор в любой момент.
    #[inline]
    pub unsafe fn as_slice(&self) -> &[T] {
        let count = self.len();
        if count == 0 || self.begin.is_null() {
            return &[];
        }
        unsafe { std::slice::from_raw_parts(self.begin, count) }
    }

    /// Мутабельный слайс элементов.
    ///
    /// # Safety
    ///
    /// Те же требования что [`as_slice`], плюс не должно быть
    /// других ссылок на эти данные.
    #[inline]
    pub unsafe fn as_mut_slice(&mut self) -> &mut [T] {
        let count = self.len();
        if count == 0 || self.begin.is_null() {
            return &mut [];
        }
        unsafe { std::slice::from_raw_parts_mut(self.begin, count) }
    }

    /// Получить элемент по индексу.
    ///
    /// # Safety
    ///
    /// `index < self.len()` и те же требования что [`as_slice`].
    #[inline]
    pub unsafe fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len() || self.begin.is_null() {
            return None;
        }
        Some(unsafe { &*self.begin.add(index) })
    }
}

// ---------------------------------------------------------------------------
//  Trait impls
// ---------------------------------------------------------------------------

// Игровые структуры — просто layout, можно копировать побитово.
impl<T> Copy for StdVector<T> {}
impl<T> Clone for StdVector<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Debug for StdVector<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "StdVector<{}>(begin=0x{:X}, len={}, cap={})",
            std::any::type_name::<T>(),
            self.begin as usize,
            self.len(),
            self.capacity_count(),
        )
    }
}

// Нулевой вектор — полезно для Default-инициализации в тестах.
impl<T> Default for StdVector<T> {
    fn default() -> Self {
        Self {
            begin: std::ptr::null_mut(),
            end: std::ptr::null_mut(),
            capacity: std::ptr::null_mut(),
        }
    }
}

// ---------------------------------------------------------------------------
//  Compile-time layout проверки
// ---------------------------------------------------------------------------

const _: () = {
    // 3 указателя × 8 байт = 24 байта
    assert!(std::mem::size_of::<StdVector<u8>>() == 24);
    assert!(std::mem::align_of::<StdVector<u8>>() == 8);

    // Внутренние смещения
    assert!(std::mem::offset_of!(StdVector<u8>, begin) == 0);
    assert!(std::mem::offset_of!(StdVector<u8>, end) == 8);
    assert!(std::mem::offset_of!(StdVector<u8>, capacity) == 16);

    // Типизированные варианты имеют тот же размер
    assert!(std::mem::size_of::<StdVector<u64>>() == 24);
    assert!(std::mem::size_of::<StdVector<[u8; 64]>>() == 24);
};
