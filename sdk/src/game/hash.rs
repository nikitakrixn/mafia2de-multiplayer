/// FNV-1 32-bit hash — используется для module ServiceIdentity.
///
/// Подтверждено из C_ServiceIdentity::Init:
/// ```asm
/// imul ebx, 1000193h   ; hash *= prime
/// xor  ebx, ecx        ; hash ^= byte
/// ```
///
/// Seed: 0x811C9DC5, Prime: 0x01000193
pub fn fnv1_32(data: &[u8]) -> u32 {
    let mut hash: u32 = 0x811C_9DC5;
    for &byte in data {
        hash = hash.wrapping_mul(0x0100_0193);
        hash ^= byte as u32;
    }
    hash
}

/// FNV-1 64-bit hash — используется движком для имён entity и ресурсов.
///
/// Порядок операций: multiply FIRST, then XOR (FNV-1, НЕ FNV-1a).
/// Подтверждено из inline asm в M2DE_EntityManager_FindByName:
/// ```asm
/// imul rdi, rcx   ; hash *= prime  (СНАЧАЛА)
/// xor  rdi, rax   ; hash ^= byte   (ПОТОМ)
/// ```
///
/// Seed: 0xCBF29CE484222325, Prime: 0x100000001B3
pub fn fnv1_64(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xCBF2_9CE4_8422_2325;
    for &byte in data {
        hash = hash.wrapping_mul(0x0100_0000_01B3);
        hash ^= byte as u64;
    }
    hash
}

/// FNV-1 64-bit hash with seed=0 — используется TypeRegistry для имён entity.
///
/// ОТЛИЧАЕТСЯ от стандартного fnv1_64 только seed'ом.
/// Подтверждено из M2DE_Register_CleanEntity_TypeDescriptor:
/// ```c
/// for (i = 0; *name; ) {       // seed = 0!
///     i = byte ^ (0x100000001B3 * i);
/// }
/// ```
pub fn fnv1_64_seed0(data: &[u8]) -> u64 {
    let mut hash: u64 = 0; // seed = 0, NOT 0xCBF29CE484222325
    for &byte in data {
        hash = hash.wrapping_mul(0x0100_0000_01B3);
        hash ^= byte as u64;
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fnv1_32_module_names() {
        // Можно проверить в runtime через breakpoint в C_ServiceIdentity::Init
        let hash = fnv1_32(b"ENTITY_LIST");
        println!("ENTITY_LIST FNV1-32: 0x{:08X}", hash);
    }

    #[test]
    fn test_fnv1_64_entity_names() {
        let hash = fnv1_64(b"EntityType");
        println!("EntityType FNV1-64: 0x{:016X}", hash);
    }
}
