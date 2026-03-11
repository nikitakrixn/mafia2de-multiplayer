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
/// `first_only` = true → возвращает сразу после первого совпадения.
/// `first_only` = false → собирает все вхождения.
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

// ═══════════════════════════════════════════════════════════════════
//  Тесты
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_простой_паттерн() {
        let p = parse("48 8B ?? 00");
        assert_eq!(p, vec![Some(0x48), Some(0x8B), None, Some(0x00)]);
    }

    #[test]
    fn parse_пустая_строка() {
        let p = parse("");
        assert!(p.is_empty());
    }

    #[test]
    fn parse_только_wildcard() {
        let p = parse("?? ?? ??");
        assert_eq!(p, vec![None, None, None]);
    }

    #[test]
    fn parse_одинарный_вопрос() {
        // Поддерживаем и "?" и "??" как wildcard
        let p = parse("48 ? 00");
        assert_eq!(p, vec![Some(0x48), None, Some(0x00)]);
    }

    #[test]
    fn find_в_буфере() {
        let buffer: Vec<u8> = vec![
            0x00, 0x11, 0x48, 0x8B, 0x05, 0xAA, 0xBB, 0xCC, 0xDD, 0x48, 0x85,
        ];
        let base = buffer.as_ptr() as usize;
        let size = buffer.len();

        let result = find(base, size, "48 8B 05 ?? ?? ?? ??");
        assert_eq!(result, Some(base + 2));
    }

    #[test]
    fn find_нет_совпадения() {
        let buffer: Vec<u8> = vec![0x00, 0x11, 0x22, 0x33];
        let base = buffer.as_ptr() as usize;

        let result = find(base, buffer.len(), "FF FF FF");
        assert_eq!(result, None);
    }

    #[test]
    fn find_в_начале() {
        let buffer: Vec<u8> = vec![0xAA, 0xBB, 0xCC];
        let base = buffer.as_ptr() as usize;

        let result = find(base, buffer.len(), "AA BB CC");
        assert_eq!(result, Some(base));
    }

    #[test]
    fn find_all_несколько_вхождений() {
        let buffer: Vec<u8> = vec![
            0xAA, 0xBB, 0x00, 0xAA, 0xBB, 0x01, 0xAA, 0xBB, 0x02,
        ];
        let base = buffer.as_ptr() as usize;

        let results = find_all(base, buffer.len(), "AA BB ??");
        assert_eq!(results.len(), 3);
        assert_eq!(results[0], base);
        assert_eq!(results[1], base + 3);
        assert_eq!(results[2], base + 6);
    }

    #[test]
    fn find_all_пустой_паттерн() {
        let buffer: Vec<u8> = vec![0x00, 0x11];
        let base = buffer.as_ptr() as usize;

        let results = find_all(base, buffer.len(), "");
        assert!(results.is_empty());
    }

    #[test]
    fn find_паттерн_длиннее_буфера() {
        let buffer: Vec<u8> = vec![0xAA, 0xBB];
        let base = buffer.as_ptr() as usize;

        let result = find(base, buffer.len(), "AA BB CC DD");
        assert_eq!(result, None);
    }
}