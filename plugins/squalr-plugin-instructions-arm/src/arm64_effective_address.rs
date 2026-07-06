//! Computes the effective memory address an ARM64 instruction accesses, by decoding the instruction bytes with
//! `yaxpeax-arm` and reading its structured memory operand, then combining it with a runtime register lookup. Used by
//! instruction-directed traces ("find what addresses this instruction accesses").
//!
//! This mirrors the x86 path (which re-decodes the bytes with `iced` and asks it for the operand's virtual address)
//! rather than string-parsing the disassembly text, so it is exact for every addressing form the decoder understands
//! (including the atomics like `LDXR`/`STXR` that the in-house decoder can only render as text).

use yaxpeax_arch::{Decoder, U8Reader};
use yaxpeax_arm::armv8::a64::{InstDecoder, Operand, ShiftStyle};

const ARM64_INSTRUCTION_BYTE_LENGTH: usize = 4;

/// Resolves the accessed memory address for the ARM64 instruction at the start of `instruction_bytes`, or `None` if the
/// instruction has no resolvable memory operand (or cannot be decoded). `register_value_by_name` looks up the runtime
/// value of a register by lowercase name (e.g. `"x19"`, `"sp"`).
pub fn resolve_arm64_accessed_address(
    instruction_bytes: &[u8],
    register_value_by_name: &dyn Fn(&str) -> Option<u64>,
) -> Option<u64> {
    if instruction_bytes.len() < ARM64_INSTRUCTION_BYTE_LENGTH {
        return None;
    }

    let decoder = InstDecoder::default();
    let mut reader = U8Reader::new(&instruction_bytes[..ARM64_INSTRUCTION_BYTE_LENGTH]);
    let instruction = decoder.decode(&mut reader).ok()?;

    instruction
        .operands
        .iter()
        .find_map(|operand| resolve_memory_operand_address(operand, register_value_by_name))
}

fn resolve_memory_operand_address(
    operand: &Operand,
    register_value_by_name: &dyn Fn(&str) -> Option<u64>,
) -> Option<u64> {
    match operand {
        // `[xn]`, `[xn, #imm]`, and pre-index `[xn, #imm]!`: the access uses base + immediate (the optional write-back
        // updates the base register, but the accessed address is still base + immediate).
        Operand::RegPreIndex(base_register, immediate_offset, _has_writeback) => {
            let base_value = base_register_value(*base_register, register_value_by_name)?;

            Some(base_value.wrapping_add(*immediate_offset as i64 as u64))
        }
        // Post-index `[xn], #imm` / `[xn], xm`: the access uses the bare base; the offset is applied to the register
        // afterward.
        Operand::RegPostIndex(base_register, _immediate_offset) => base_register_value(*base_register, register_value_by_name),
        Operand::RegPostIndexReg(base_register, _index_register) => base_register_value(*base_register, register_value_by_name),
        // `[xn, xm{, <extend|lsl> #shift}]`: base + extend(index) << shift.
        Operand::RegRegOffset(base_register, index_register, _index_size, shift_style, shift_amount) => {
            let base_value = base_register_value(*base_register, register_value_by_name)?;
            let index_value = index_register_value(*index_register, register_value_by_name)?;

            Some(base_value.wrapping_add(extend_index_value(index_value, *shift_style, *shift_amount)))
        }
        _ => None,
    }
}

/// The base register of a memory operand uses the stack pointer for register encoding 31 (not the zero register).
fn base_register_value(
    register_number: u16,
    register_value_by_name: &dyn Fn(&str) -> Option<u64>,
) -> Option<u64> {
    if register_number == 31 {
        register_value_by_name("sp")
    } else {
        register_value_by_name(&format!("x{}", register_number))
    }
}

/// The index register uses the zero register (value 0) for register encoding 31.
fn index_register_value(
    register_number: u16,
    register_value_by_name: &dyn Fn(&str) -> Option<u64>,
) -> Option<u64> {
    if register_number == 31 {
        Some(0)
    } else {
        register_value_by_name(&format!("x{}", register_number))
    }
}

/// Applies the index extend/shift: extends the index value per the extend operation, then left-shifts by `shift_amount`.
/// The index is always read as the full 64-bit register; the `W`-width extends (`UXTW`/`SXTW`) operate on its low 32
/// bits, which is exactly the architectural `Wn` value.
fn extend_index_value(
    index_value: u64,
    shift_style: ShiftStyle,
    shift_amount: u8,
) -> u64 {
    let extended_index_value = match shift_style {
        ShiftStyle::UXTB => index_value & 0xFF,
        ShiftStyle::UXTH => index_value & 0xFFFF,
        ShiftStyle::UXTW => index_value & 0xFFFF_FFFF,
        ShiftStyle::SXTB => index_value as u8 as i8 as i64 as u64,
        ShiftStyle::SXTH => index_value as u16 as i16 as i64 as u64,
        ShiftStyle::SXTW => index_value as u32 as i32 as i64 as u64,
        // LSL / UXTX / SXTX use the full 64-bit value; the data-processing shift styles do not appear in memory operands
        // but are handled defensively as no-op extends.
        _ => index_value,
    };

    extended_index_value.wrapping_shl(shift_amount as u32)
}

#[cfg(test)]
mod tests {
    use super::resolve_arm64_accessed_address;
    use std::collections::HashMap;

    fn registers(pairs: &[(&str, u64)]) -> impl Fn(&str) -> Option<u64> {
        let map = pairs
            .iter()
            .map(|(name, value)| (name.to_string(), *value))
            .collect::<HashMap<_, _>>();

        move |register_name: &str| map.get(register_name).copied()
    }

    fn instruction_bytes(encoding: u32) -> [u8; 4] {
        encoding.to_le_bytes()
    }

    #[test]
    fn resolves_base_only_atomic_store() {
        // stxr w9, x8, [x19] — the canonical "find what writes" case; only the full decoder handles this form.
        let lookup = registers(&[("x19", 0x2A5F_C58)]);
        assert_eq!(resolve_arm64_accessed_address(&instruction_bytes(0xC809_7E68), &lookup), Some(0x2A5F_C58));
    }

    #[test]
    fn resolves_base_only_load() {
        // ldr x0, [x1]
        let lookup = registers(&[("x1", 0x4000)]);
        assert_eq!(resolve_arm64_accessed_address(&instruction_bytes(0xF940_0020), &lookup), Some(0x4000));
    }

    #[test]
    fn resolves_base_plus_immediate() {
        // ldr x0, [x1, #8]
        let lookup = registers(&[("x1", 0x4000)]);
        assert_eq!(resolve_arm64_accessed_address(&instruction_bytes(0xF940_0420), &lookup), Some(0x4008));
    }

    #[test]
    fn resolves_base_plus_negative_immediate() {
        // ldur x0, [x1, #-8]
        let lookup = registers(&[("x1", 0x1000)]);
        assert_eq!(resolve_arm64_accessed_address(&instruction_bytes(0xF85F_8020), &lookup), Some(0xFF8));
    }

    #[test]
    fn resolves_base_plus_index() {
        // ldr x0, [x1, x2]
        let lookup = registers(&[("x1", 0x4000), ("x2", 0x8)]);
        assert_eq!(resolve_arm64_accessed_address(&instruction_bytes(0xF862_6820), &lookup), Some(0x4008));
    }

    #[test]
    fn resolves_base_plus_scaled_index() {
        // ldr x0, [x1, x2, lsl #3]
        let lookup = registers(&[("x1", 0x4000), ("x2", 0x4)]);
        assert_eq!(resolve_arm64_accessed_address(&instruction_bytes(0xF862_7820), &lookup), Some(0x4020));
    }

    #[test]
    fn resolves_sign_extended_word_index() {
        // ldr w0, [x1, w2, sxtw #2] — sxtw(0xFFFF_FFFF) == -1, shifted left 2 == -4.
        let lookup = registers(&[("x1", 0x4000), ("x2", 0xFFFF_FFFF)]);
        assert_eq!(resolve_arm64_accessed_address(&instruction_bytes(0xB862_D820), &lookup), Some(0x3FFC));
    }

    #[test]
    fn post_index_uses_base_only() {
        // ldr x0, [x1], #16
        let lookup = registers(&[("x1", 0x2000)]);
        assert_eq!(resolve_arm64_accessed_address(&instruction_bytes(0xF841_0420), &lookup), Some(0x2000));
    }

    #[test]
    fn pre_index_uses_base_plus_immediate() {
        // ldr x0, [x1, #16]!
        let lookup = registers(&[("x1", 0x2000)]);
        assert_eq!(resolve_arm64_accessed_address(&instruction_bytes(0xF841_0C20), &lookup), Some(0x2010));
    }

    #[test]
    fn missing_register_value_resolves_to_none() {
        let lookup = registers(&[]);
        assert_eq!(resolve_arm64_accessed_address(&instruction_bytes(0xF940_0020), &lookup), None);
    }

    #[test]
    fn non_memory_instruction_resolves_to_none() {
        // add x0, x1, x2 — no memory operand.
        let lookup = registers(&[("x1", 0x10), ("x2", 0x20)]);
        assert_eq!(resolve_arm64_accessed_address(&instruction_bytes(0x8B02_0020), &lookup), None);
    }
}
