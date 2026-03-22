//! Сканер сигнатур (паттернов) в памяти.
//!
//! Используется для поиска адресов по последовательности байт,
//! когда RVA может измениться между версиями игры.
//! На практике для Mafia II: DE адреса стабильны,
//! но сканер полезен как fallback и для верификации.
//!
//! # Формат паттерна
//!
//! Байты в hex через пробел, `??` — wildcard (любой байт):
//! ```text
//! "48 8B 05 ?? ?? ?? ?? 48 85 C0 74"
//! ```
//!
//! # Пример
//!
//! ```ignore
//! use sdk::{memory, patterns};
//!
//! let info = memory::get_module_info("Mafia II Definitive Edition.exe").unwrap();
//! if let Some(addr) = patterns::find(info.base, info.size, "48 8B 0D ?? ?? ?? ??") {
//!     let game_ptr = memory::resolve_rip_relative(addr, 3, 7);
//!     println!("C_Game* по адресу 0x{:X}", game_ptr);
//! }
//! ```

/// Результат парсинга одного токена паттерна.
/// `None` = wildcard (`?` или `??`), совпадает с любым байтом.
type PatternByte = Option<u8>;

/// Парсит строковый паттерн в вектор байтов.
///
/// Каждый токен через пробел — либо hex-байт ("48", "8B"),
/// либо wildcard ("?" или "??").
fn parse(pattern: &str) -> Vec<PatternByte> {
    pattern
        .split_whitespace()
        .map(|token| {
            if token == "?" || token == "??" {
                None
            } else {
                u8::from_str_radix(token, 16).ok()
            }
        })
        .collect()
}

/// Внутренняя реализация поиска паттерна.
///
/// Проходит по диапазону `[base .. base+size)` и сравнивает
/// каждую позицию с паттерном. Wildcard-байты пропускаются.
///
/// `first_only` = true -> возвращает сразу после первого совпадения.
/// `first_only` = false -> собирает все вхождения.
///
/// # Безопасность
///
/// Диапазон `[base .. base+size)` должен быть доступен для чтения.
/// Вызывающий обязан гарантировать это (обычно — через get_module_info).
fn scan_internal(base: usize, size: usize, pattern: &str, first_only: bool) -> Vec<usize> {
    let parsed = parse(pattern);
    if parsed.is_empty() {
        return Vec::new();
    }

    let data = unsafe { std::slice::from_raw_parts(base as *const u8, size) };
    let pat_len = parsed.len();
    let mut results = Vec::new();

    // Не выходим за границу: последний валидный старт —
    // (data.len() - pat_len), иначе при сравнении вылезем за буфер
    'outer: for i in 0..data.len().saturating_sub(pat_len) {
        for (j, expected) in parsed.iter().enumerate() {
            if let Some(byte) = expected
                && data[i + j] != *byte
            {
                continue 'outer;
            }
        }

        results.push(base + i);

        if first_only {
            break;
        }
    }

    results
}

/// Найти первое вхождение паттерна в указанном диапазоне памяти.
///
/// Возвращает абсолютный адрес первого совпадения или None.
///
/// # Пример
///
/// ```ignore
/// // Ищем mov rcx, [rip+????????]
/// let addr = patterns::find(base, size, "48 8B 0D ?? ?? ?? ??");
/// ```
pub fn find(base: usize, size: usize, pattern: &str) -> Option<usize> {
    scan_internal(base, size, pattern, true).into_iter().next()
}

/// Найти все вхождения паттерна в указанном диапазоне.
///
/// Полезно когда одна и та же инструкция встречается
/// в нескольких местах и нужно найти конкретную.
pub fn find_all(base: usize, size: usize, pattern: &str) -> Vec<usize> {
    scan_internal(base, size, pattern, false)
}
