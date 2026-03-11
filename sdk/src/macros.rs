//! Вспомогательные макросы SDK.
//!
//! Используются для compile-time проверки layout'ов repr(C) структур.
//! Если реверс поплывёт после патча игры — ошибка компиляции
//! укажет на конкретное поле с неправильным смещением.

/// Проверяет смещения полей структуры на этапе компиляции.
///
/// ```ignore
/// assert_field_offsets!(CCar {
///     important_data == 0x38,
/// });
/// ```
macro_rules! assert_field_offsets {
    ($ty:ty { $( $field:ident == $offset:expr ),* $(,)? }) => {
        const _: () = {
            $(
                assert!(std::mem::offset_of!($ty, $field) == $offset);
            )*
        };
    };
}

/// Проверяет размер структуры и смещения полей.
///
/// ```ignore
/// assert_layout!(VehicleWrapper, size = 32, {
///     refcount == 0x08,
///     vehicle  == 0x18,
/// });
/// ```
macro_rules! assert_layout {
    ($ty:ty, size = $size:expr, { $( $field:ident == $offset:expr ),* $(,)? }) => {
        const _: () = {
            assert!(std::mem::size_of::<$ty>() == $size);
            $(
                assert!(std::mem::offset_of!($ty, $field) == $offset);
            )*
        };
    };
}

pub(crate) use assert_field_offsets;
pub(crate) use assert_layout;