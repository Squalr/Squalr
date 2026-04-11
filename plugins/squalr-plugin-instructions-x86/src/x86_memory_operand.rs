use crate::x86_register::parse_register;
use iced_x86::{MemoryOperand, OpCodeOperandKind, Register};
use squalr_engine_api::plugins::instruction_set::InstructionMemoryOperand;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum X86InstructionMode {
    Bit32,
    Bit64,
}

impl X86InstructionMode {
    pub fn bitness(&self) -> u32 {
        match self {
            Self::Bit32 => 32,
            Self::Bit64 => 64,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ParsedMemoryTerm {
    register: Option<Register>,
    scale: u32,
    displacement: i64,
}

pub fn parse_memory_operand(
    memory_operand: &InstructionMemoryOperand,
    instruction_mode: X86InstructionMode,
) -> Result<MemoryOperand, String> {
    let expression_text = memory_operand.expression_text().replace(' ', "");

    if expression_text.is_empty() {
        return Err(String::from("Memory operand expression must not be empty."));
    }

    let mut displacement = 0_i64;
    let mut base_register = Register::None;
    let mut index_register = Register::None;
    let mut index_scale = 1_u32;

    for (term_sign, term_text) in split_signed_terms(&expression_text)? {
        let parsed_term = parse_memory_term(term_text)?;

        if let Some(parsed_register) = parsed_term.register {
            if parsed_term.scale > 1 {
                if index_register != Register::None {
                    return Err(format!(
                        "Memory operand '{}' currently supports at most one scaled index register.",
                        memory_operand.expression_text()
                    ));
                }

                index_register = parsed_register;
                index_scale = parsed_term.scale;
            } else if base_register == Register::None {
                base_register = parsed_register;
            } else if index_register == Register::None {
                index_register = parsed_register;
                index_scale = 1;
            } else {
                return Err(format!(
                    "Memory operand '{}' currently supports at most one base register and one index register.",
                    memory_operand.expression_text()
                ));
            }
        }

        let signed_displacement = if term_sign < 0 {
            parsed_term.displacement.saturating_neg()
        } else {
            parsed_term.displacement
        };

        displacement = displacement
            .checked_add(signed_displacement)
            .ok_or_else(|| format!("Memory displacement in '{}' overflowed the supported range.", memory_operand.expression_text()))?;
    }

    let segment_prefix = memory_operand
        .segment_override()
        .map(parse_required_segment_register)
        .transpose()?
        .unwrap_or(Register::None);
    let displacement_size = resolve_displacement_size(base_register, index_register, displacement, instruction_mode);

    Ok(MemoryOperand::new(
        base_register,
        index_register,
        index_scale,
        displacement,
        displacement_size,
        memory_operand.is_broadcast(),
        segment_prefix,
    ))
}

pub fn memory_operand_matches_operand_kind(
    memory_operand: &MemoryOperand,
    operand_kind: OpCodeOperandKind,
) -> bool {
    match operand_kind {
        OpCodeOperandKind::mem
        | OpCodeOperandKind::mem_mpx
        | OpCodeOperandKind::mem_mib
        | OpCodeOperandKind::sibmem
        | OpCodeOperandKind::r8_or_mem
        | OpCodeOperandKind::r16_or_mem
        | OpCodeOperandKind::r32_or_mem
        | OpCodeOperandKind::r32_or_mem_mpx
        | OpCodeOperandKind::r64_or_mem
        | OpCodeOperandKind::r64_or_mem_mpx
        | OpCodeOperandKind::mm_or_mem
        | OpCodeOperandKind::xmm_or_mem
        | OpCodeOperandKind::ymm_or_mem
        | OpCodeOperandKind::zmm_or_mem
        | OpCodeOperandKind::bnd_or_mem_mpx
        | OpCodeOperandKind::k_or_mem => true,
        OpCodeOperandKind::mem_offs => memory_operand.base == Register::None && memory_operand.index == Register::None,
        OpCodeOperandKind::mem_vsib32x | OpCodeOperandKind::mem_vsib64x => memory_operand.index.is_xmm(),
        OpCodeOperandKind::mem_vsib32y | OpCodeOperandKind::mem_vsib64y => memory_operand.index.is_ymm(),
        OpCodeOperandKind::mem_vsib32z | OpCodeOperandKind::mem_vsib64z => memory_operand.index.is_zmm(),
        _ => false,
    }
}

pub fn memory_operand_specificity_score(operand_kind: OpCodeOperandKind) -> u32 {
    match operand_kind {
        OpCodeOperandKind::mem_offs
        | OpCodeOperandKind::mem_mib
        | OpCodeOperandKind::mem_vsib32x
        | OpCodeOperandKind::mem_vsib64x
        | OpCodeOperandKind::mem_vsib32y
        | OpCodeOperandKind::mem_vsib64y
        | OpCodeOperandKind::mem_vsib32z
        | OpCodeOperandKind::mem_vsib64z
        | OpCodeOperandKind::sibmem => 55,
        OpCodeOperandKind::mem | OpCodeOperandKind::mem_mpx => 50,
        OpCodeOperandKind::r8_or_mem
        | OpCodeOperandKind::r16_or_mem
        | OpCodeOperandKind::r32_or_mem
        | OpCodeOperandKind::r32_or_mem_mpx
        | OpCodeOperandKind::r64_or_mem
        | OpCodeOperandKind::r64_or_mem_mpx
        | OpCodeOperandKind::mm_or_mem
        | OpCodeOperandKind::xmm_or_mem
        | OpCodeOperandKind::ymm_or_mem
        | OpCodeOperandKind::zmm_or_mem
        | OpCodeOperandKind::bnd_or_mem_mpx
        | OpCodeOperandKind::k_or_mem => 35,
        _ => 0,
    }
}

fn parse_required_segment_register(segment_register_name: &str) -> Result<Register, String> {
    match segment_register_name.trim().to_ascii_lowercase().as_str() {
        "es" => Ok(Register::ES),
        "cs" => Ok(Register::CS),
        "ss" => Ok(Register::SS),
        "ds" => Ok(Register::DS),
        "fs" => Ok(Register::FS),
        "gs" => Ok(Register::GS),
        _ => Err(format!("Unsupported segment override '{}'.", segment_register_name.trim())),
    }
}

fn split_signed_terms(expression_text: &str) -> Result<Vec<(i8, &str)>, String> {
    let mut signed_terms = Vec::new();
    let mut current_term_start_index = 0usize;
    let mut current_sign = 1_i8;

    for (character_index, current_character) in expression_text.char_indices() {
        if character_index == 0 {
            if current_character == '-' {
                current_sign = -1;
                current_term_start_index = current_character.len_utf8();
            } else if current_character == '+' {
                current_term_start_index = current_character.len_utf8();
            }

            continue;
        }

        if current_character == '+' || current_character == '-' {
            let current_term_text = &expression_text[current_term_start_index..character_index];

            if current_term_text.is_empty() {
                return Err(format!("Malformed memory expression '{}'.", expression_text));
            }

            signed_terms.push((current_sign, current_term_text));
            current_sign = if current_character == '-' { -1 } else { 1 };
            current_term_start_index = character_index + current_character.len_utf8();
        }
    }

    let trailing_term_text = &expression_text[current_term_start_index..];

    if trailing_term_text.is_empty() {
        return Err(format!("Malformed memory expression '{}'.", expression_text));
    }

    signed_terms.push((current_sign, trailing_term_text));

    Ok(signed_terms)
}

fn parse_memory_term(term_text: &str) -> Result<ParsedMemoryTerm, String> {
    if let Some((left_term, right_term)) = term_text.split_once('*') {
        let left_term = left_term.trim();
        let right_term = right_term.trim();

        if let Some(parsed_register) = parse_register(left_term) {
            let scale = parse_scale(right_term)?;

            return Ok(ParsedMemoryTerm {
                register: Some(parsed_register),
                scale,
                displacement: 0,
            });
        }

        if let Some(parsed_register) = parse_register(right_term) {
            let scale = parse_scale(left_term)?;

            return Ok(ParsedMemoryTerm {
                register: Some(parsed_register),
                scale,
                displacement: 0,
            });
        }

        return Err(format!("Unsupported scaled memory term '{}'.", term_text));
    }

    if let Some(parsed_register) = parse_register(term_text) {
        return Ok(ParsedMemoryTerm {
            register: Some(parsed_register),
            scale: 1,
            displacement: 0,
        });
    }

    let displacement = parse_memory_displacement(term_text).ok_or_else(|| format!("Unsupported memory expression term '{}'.", term_text))?;

    Ok(ParsedMemoryTerm {
        register: None,
        scale: 1,
        displacement,
    })
}

fn parse_scale(scale_text: &str) -> Result<u32, String> {
    match scale_text.parse::<u32>() {
        Ok(scale @ (1 | 2 | 4 | 8)) => Ok(scale),
        Ok(_) => Err(format!("Memory scale '{}' must be 1, 2, 4, or 8.", scale_text)),
        Err(_) => Err(format!("Invalid memory scale '{}'.", scale_text)),
    }
}

fn parse_memory_displacement(term_text: &str) -> Option<i64> {
    if let Some(hexadecimal_digits) = term_text
        .strip_prefix("0x")
        .or_else(|| term_text.strip_prefix("0X"))
    {
        i64::from_str_radix(hexadecimal_digits, 16).ok()
    } else {
        term_text.parse::<i64>().ok()
    }
}

fn resolve_displacement_size(
    base_register: Register,
    index_register: Register,
    displacement: i64,
    instruction_mode: X86InstructionMode,
) -> u32 {
    if base_register == Register::None && index_register == Register::None {
        return match instruction_mode {
            X86InstructionMode::Bit32 => 4,
            X86InstructionMode::Bit64 => {
                if i32::try_from(displacement).is_ok() || u32::try_from(displacement).is_ok() {
                    4
                } else {
                    8
                }
            }
        };
    }

    if displacement == 0 && base_register != Register::RBP && base_register != Register::R13 && base_register != Register::EBP {
        0
    } else if i8::try_from(displacement).is_ok() {
        1
    } else {
        4
    }
}

#[cfg(test)]
mod tests {
    use crate::x86_memory_operand::{X86InstructionMode, parse_memory_operand};
    use iced_x86::Register;
    use squalr_engine_api::plugins::instruction_set::{InstructionMemoryOperand, InstructionMemoryOperandSize};

    #[test]
    fn parse_memory_operand_supports_x86_absolute_addresses() {
        let parsed_operand = parse_memory_operand(
            &InstructionMemoryOperand::new(Some(InstructionMemoryOperandSize::Dword), "0x100579c"),
            X86InstructionMode::Bit32,
        )
        .expect("Expected x86 absolute memory operand to parse.");

        assert_eq!(parsed_operand.base, Register::None);
        assert_eq!(parsed_operand.displacement, 0x100579c);
        assert_eq!(parsed_operand.displ_size, 4);
    }

    #[test]
    fn parse_memory_operand_supports_base_register_plus_scaled_index() {
        let parsed_operand = parse_memory_operand(
            &InstructionMemoryOperand::new(Some(InstructionMemoryOperandSize::Dword), "eax+ecx*4+8"),
            X86InstructionMode::Bit32,
        )
        .expect("Expected scaled index memory operand to parse.");

        assert_eq!(parsed_operand.base, Register::EAX);
        assert_eq!(parsed_operand.index, Register::ECX);
        assert_eq!(parsed_operand.scale, 4);
        assert_eq!(parsed_operand.displacement, 8);
    }

    #[test]
    fn parse_memory_operand_supports_segment_override_and_broadcast() {
        let parsed_operand = parse_memory_operand(
            &InstructionMemoryOperand::with_metadata(Some(InstructionMemoryOperandSize::Dword), Some("fs"), "rax", true),
            X86InstructionMode::Bit64,
        )
        .expect("Expected segment override memory operand to parse.");

        assert_eq!(parsed_operand.base, Register::RAX);
        assert_eq!(parsed_operand.segment_prefix, Register::FS);
        assert!(parsed_operand.is_broadcast);
    }
}
