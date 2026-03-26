//! Сканер сигнатур в памяти.
//!
//! Ищет последовательности байт в загруженных модулях.
//! Мы внутри процесса — модуль уже в нашем адресном пространстве,
//! читаем напрямую через слайс.
//!
//! # Формат паттерна
//!
//! Hex-байты через пробел, `??` — wildcard:
//! ```text
//! "48 8B 05 ?? ?? ?? ?? 48 85 C0 74"
//! ```
//!
//! # Пример
//!
//! ```ignore
//! let info = memory::get_module_info("Mafia II Definitive Edition.exe").unwrap();
//! let addr = unsafe { patterns::find(info.base, info.size, "48 8B 0D ?? ?? ?? ??") };
//! if let Some(addr) = addr {
//!     let target = unsafe { memory::resolve_rip_relative(addr, 3, 7) };
//! }
//! ```

use std::ptr;

/// `None` = wildcard, `Some(byte)` = конкретный байт.
type PatternByte = Option<u8>;

/// Парсит строковый паттерн.
///
/// # Panics
///
/// Паникует на невалидных hex-токенах — ошибки в паттернах
/// должны обнаруживаться при разработке, а не давать ложные результаты.
fn parse(pattern: &str) -> Vec<PatternByte> {
    pattern
        .split_whitespace()
        .map(|token| {
            if token == "?" || token == "??" {
                None
            } else {
                Some(u8::from_str_radix(token, 16).unwrap_or_else(|_| {
                    panic!("patterns::parse: invalid hex byte '{token}' in \"{pattern}\"")
                }))
            }
        })
        .collect()
}

/// Ищет первое вхождение паттерна в слайсе.
fn scan_first(data: &[u8], parsed: &[PatternByte]) -> Option<usize> {
    let pat_len = parsed.len();
    if pat_len == 0 || data.len() < pat_len {
        return None;
    }

    // Оптимизация: если паттерн полностью фиксированный (без wildcard),
    // используем windows() — компилятор может применить SIMD.
    let all_fixed: Option<Vec<u8>> = parsed.iter().copied().collect();
    if let Some(ref fixed) = all_fixed {
        return data.windows(pat_len).position(|w| w == fixed.as_slice());
    }

    // Первый не-wildcard байт для быстрого пропуска
    let first_fixed = parsed
        .iter()
        .enumerate()
        .find_map(|(i, b)| b.map(|v| (i, v)));

    let end = data.len() - pat_len;

    'outer: for i in 0..=end {
        // Быстрая проверка первого фиксированного байта
        if let Some((offset, byte)) = first_fixed {
            if data[i + offset] != byte {
                continue;
            }
        }

        for (j, expected) in parsed.iter().enumerate() {
            if let Some(byte) = expected {
                if data[i + j] != *byte {
                    continue 'outer;
                }
            }
        }

        return Some(i);
    }

    None
}

/// Ищет все вхождения паттерна в слайсе.
fn scan_all(data: &[u8], parsed: &[PatternByte]) -> Vec<usize> {
    let pat_len = parsed.len();
    if pat_len == 0 || data.len() < pat_len {
        return Vec::new();
    }

    let end = data.len() - pat_len;
    let mut results = Vec::new();

    'outer: for i in 0..=end {
        for (j, expected) in parsed.iter().enumerate() {
            if let Some(byte) = expected {
                if data[i + j] != *byte {
                    continue 'outer;
                }
            }
        }
        results.push(i);
    }

    results
}

/// Первое вхождение паттерна. Возвращает абсолютный адрес.
///
/// # Safety
///
/// `[base .. base+size)` должен быть readable.
/// Для загруженных модулей (через [`get_module_info`]) это гарантировано.
pub unsafe fn find(base: usize, size: usize, pattern: &str) -> Option<usize> {
    let parsed = parse(pattern);
    let data = unsafe {
        std::slice::from_raw_parts(ptr::with_exposed_provenance::<u8>(base), size)
    };
    scan_first(data, &parsed).map(|offset| base + offset)
}

/// Все вхождения паттерна. Возвращает абсолютные адреса.
///
/// # Safety
///
/// `[base .. base+size)` должен быть readable.
pub unsafe fn find_all(base: usize, size: usize, pattern: &str) -> Vec<usize> {
    let parsed = parse(pattern);
    let data = unsafe {
        std::slice::from_raw_parts(ptr::with_exposed_provenance::<u8>(base), size)
    };
    scan_all(data, &parsed)
        .into_iter()
        .map(|offset| base + offset)
        .collect()
}

// =============================================================================
//  Тесты
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid() {
        let p = parse("48 8B 05 ?? ??");
        assert_eq!(p, vec![Some(0x48), Some(0x8B), Some(0x05), None, None]);
    }

    #[test]
    fn parse_single_wildcard() {
        let p = parse("48 ? 05");
        assert_eq!(p, vec![Some(0x48), None, Some(0x05)]);
    }

    #[test]
    #[should_panic(expected = "invalid hex byte")]
    fn parse_invalid_panics() {
        parse("48 ZZ 05");
    }

    #[test]
    fn find_at_start() {
        let data = [0x48, 0x8B, 0x05, 0xAA, 0xBB];
        assert_eq!(scan_first(&data, &parse("48 8B 05")), Some(0));
    }

    #[test]
    fn find_at_end() {
        let data = [0x00, 0x00, 0x48, 0x8B, 0x05];
        assert_eq!(scan_first(&data, &parse("48 8B 05")), Some(2));
    }

    #[test]
    fn find_exact_size() {
        let data = [0x48, 0x8B, 0x05];
        assert_eq!(scan_first(&data, &parse("48 8B 05")), Some(0));
    }

    #[test]
    fn find_too_short() {
        let data = [0x48, 0x8B];
        assert_eq!(scan_first(&data, &parse("48 8B 05")), None);
    }

    #[test]
    fn find_not_found() {
        let data = [0x00; 5];
        assert_eq!(scan_first(&data, &parse("48 8B 05")), None);
    }

    #[test]
    fn find_with_wildcards() {
        let data = [0x48, 0x8B, 0x05, 0xDE, 0xAD, 0xBE, 0xEF];
        assert_eq!(scan_first(&data, &parse("48 8B 05 ?? ?? ?? ??")), Some(0));
    }

    #[test]
    fn find_all_multiple() {
        let data = [0xCC, 0xAA, 0xBB, 0xCC, 0xAA, 0xBB, 0xCC, 0xDD];
        assert_eq!(scan_all(&data, &parse("AA BB")), vec![1, 4]);
    }

    #[test]
    fn find_all_overlapping() {
        let data = [0xAA, 0xAA, 0xAA];
        assert_eq!(scan_all(&data, &parse("AA AA")), vec![0, 1]);
    }

    #[test]
    fn empty_pattern() {
        let data = [0x48, 0x8B];
        assert_eq!(scan_first(&data, &parse("")), None);
    }

    #[test]
    fn empty_data() {
        assert_eq!(scan_first(&[], &parse("48")), None);
    }
}