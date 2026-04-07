//! Структура `EntityHashTable` — хеш-таблица entity внутри `C_Game`.
//!
//! Внутри `C_Game` таких таблиц две:
//! - основная по `+0x1A0`
//! - вторичная по `+0x86D8`
//!
//! ## Layout (0x46B0 байт)
//!
//! ```text
//! +0x000  bucket_begin    *u16  (→ self + 0x3EB0)
//! +0x008  bucket_end      *u16  (→ self + 0x3EB0 + 0x800)
//! +0x010  entry_count     u32   (init=0)
//! +0x018  unk_18          u64   (init=0)
//! +0x020  unk_20          u64   (init=0)
//! +0x028  unk_28          u64   (init=0)
//! +0x030  entries[250]    Entry × 250  (250 × 64B = 0x3E80)
//! +0x3EB0 buckets[1024]   u16 × 1024  (0x800B, init=0xFFFF=empty)
//! ```
//!
//! ## Алгоритм поиска
//!
//! Ключ = `entity->table_id >> 8`.
//! Bucket index = `key % 1024`.
//! `buckets[idx]` = индекс первой записи в `entries[]` или `0xFFFF` (пусто).
//!
//! Конструктор: `M2DE_EntityHashTable_Constructor` (`0x1403D0F50`).

use crate::macros::assert_field_offsets;

/// Одна запись в `EntityHashTable`.
///
/// Размер: 64 байта (8 × u64).
#[repr(C)]
pub struct EntityHashEntry {
    _data: [u64; 8],
}

/// Хеш-таблица entity (inline подобъект внутри `C_Game`).
///
/// Конструктор: `M2DE_EntityHashTable_Constructor` (`0x1403D0F50`).
/// Размер: **0x46B0 байт**.
#[repr(C)]
pub struct EntityHashTable {
    /// `+0x000` Указатель на начало массива бакетов.
    ///
    /// Инициализируется как `self + 0x3EB0`.
    pub bucket_begin: *mut u16,

    /// `+0x008` Указатель на конец массива бакетов.
    ///
    /// Инициализируется как `self + 0x3EB0 + 0x800`.
    pub bucket_end: *mut u16,

    /// `+0x010` Количество записей в таблице (init=0).
    pub entry_count: u32,

    _pad_014: u32,

    /// `+0x018` Неизвестное поле (init=0).
    pub unk_18: u64,

    /// `+0x020` Неизвестное поле (init=0).
    pub unk_20: u64,

    /// `+0x028` Неизвестное поле (init=0).
    pub unk_28: u64,

    /// `+0x030` Массив из 250 записей (250 × 64B = 0x3E80).
    pub entries: [EntityHashEntry; 250],

    /// `+0x3EB0` Массив из 1024 бакетов (u16, init=0xFFFF=пусто).
    pub buckets: [u16; 1024],
}

assert_field_offsets!(EntityHashTable {
    bucket_begin == 0x000,
    bucket_end   == 0x008,
    entry_count  == 0x010,
    unk_18       == 0x018,
    entries      == 0x030,
    buckets      == 0x3EB0,
});

const _: () = {
    assert!(std::mem::size_of::<EntityHashTable>() == 0x46B0);
};

impl EntityHashTable {
    /// Таблица пуста.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entry_count == 0
    }

    /// Бакет пустой (значение 0xFFFF).
    #[inline]
    pub fn bucket_is_empty(bucket: u16) -> bool {
        bucket == 0xFFFF
    }
}
