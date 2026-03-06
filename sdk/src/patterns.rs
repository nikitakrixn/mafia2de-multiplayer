//! Сканер сигнатур (паттернов) в памяти.
//!
//! # Формат паттерна
//! Байты в hex через пробел, `??` — wildcard:
//! ```text
//! "48 8B 05 ?? ?? ?? ?? 48 85 C0 74"
//! ```
//!
//! # Пример
//! ```ignore
//! use sdk::{memory, patterns};
//!
//! let info = memory::get_module_info("Mafia II Definitive Edition.exe").unwrap();
//! if let Some(addr) = patterns::find(info.base, info.size, "48 8B 0D ?? ?? ?? ??") {
//!     let game_ptr = memory::resolve_rip_relative(addr, 3, 7);
//!     println!("C_Game* at 0x{:X}", game_ptr);
//! }
//! ```

/// Результат парсинга одного байта паттерна.
/// `None` = wildcard (`??`).
type PatternByte = Option<u8>;

/// Парсит строку паттерна в вектор байтов.
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

/// Ищет паттерн в указанном диапазоне памяти.
///
/// Возвращает абсолютный адрес первого вхождения.
///
/// # Safety (неявная)
/// Диапазон `[base .. base+size)` должен быть доступен для чтения.
pub fn find(base: usize, size: usize, pattern: &str) -> Option<usize> {
    let parsed = parse(pattern);
    if parsed.is_empty() {
        return None;
    }

    let data = unsafe { std::slice::from_raw_parts(base as *const u8, size) };
    let len = parsed.len();

    'outer: for i in 0..data.len().saturating_sub(len) {
        for (j, expected) in parsed.iter().enumerate() {
            if let Some(byte) = expected
                && data[i + j] != *byte {
                    continue 'outer;
                }
        }
        return Some(base + i);
    }

    None
}

/// Ищет **все** вхождения паттерна в указанном диапазоне.
pub fn find_all(base: usize, size: usize, pattern: &str) -> Vec<usize> {
    let parsed = parse(pattern);
    if parsed.is_empty() {
        return Vec::new();
    }

    let data = unsafe { std::slice::from_raw_parts(base as *const u8, size) };
    let len = parsed.len();
    let mut results = Vec::new();

    'outer: for i in 0..data.len().saturating_sub(len) {
        for (j, expected) in parsed.iter().enumerate() {
            if let Some(byte) = expected
                && data[i + j] != *byte {
                    continue 'outer;
                }
        }
        results.push(base + i);
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pattern() {
        let p = parse("48 8B ?? 00");
        assert_eq!(p, vec![Some(0x48), Some(0x8B), None, Some(0x00)]);
    }

    #[test]
    fn test_find_in_buffer() {
        let buffer: Vec<u8> = vec![
            0x00, 0x11, 0x48, 0x8B, 0x05, 0xAA, 0xBB, 0xCC, 0xDD, 0x48, 0x85,
        ];
        let base = buffer.as_ptr() as usize;
        let size = buffer.len();

        let result = find(base, size, "48 8B 05 ?? ?? ?? ??");
        assert_eq!(result, Some(base + 2));
    }

    #[test]
    fn test_find_all() {
        let buffer: Vec<u8> = vec![
            0xAA, 0xBB, 0x00, 0xAA, 0xBB, 0x01, 0xAA, 0xBB, 0x02,
        ];
        let base = buffer.as_ptr() as usize;
        let results = find_all(base, buffer.len(), "AA BB ??");
        assert_eq!(results.len(), 3);
    }
}