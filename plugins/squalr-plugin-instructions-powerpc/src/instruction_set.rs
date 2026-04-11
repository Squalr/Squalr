use crate::{
    powerpc_memory_operand::parse_powerpc_memory_operand,
    powerpc_register::{format_powerpc_register_name, parse_powerpc_register_name},
};
use squalr_engine_api::plugins::instruction_set::{InstructionOperand, InstructionSet, ParsedInstruction, parse_instruction_sequence};
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Clone, Debug)]
enum DecodedInstruction {
    Plain(String),
    Branch { mnemonic: &'static str, target_address: i64 },
}

#[derive(Clone, Debug, Default)]
pub struct PowerPc32BeInstructionSet;

impl PowerPc32BeInstructionSet {
    pub fn new() -> Self {
        Self
    }
}

impl InstructionSet for PowerPc32BeInstructionSet {
    fn get_instruction_set_id(&self) -> &str {
        "ppc32be"
    }

    fn get_display_name(&self) -> &str {
        "PowerPC32 BE"
    }

    fn assemble(
        &self,
        assembly_source: &str,
    ) -> Result<Vec<u8>, String> {
        let parsed_instruction_sequence = parse_instruction_sequence(assembly_source).map_err(|error| error.to_string())?;
        let label_addresses = parsed_instruction_sequence
            .label_instruction_indices()
            .iter()
            .map(|(label_name, instruction_index)| (label_name.clone(), (*instruction_index as i64) * 4))
            .collect::<HashMap<_, _>>();
        let mut instruction_bytes = Vec::with_capacity(parsed_instruction_sequence.instructions().len() * 4);

        for (instruction_index, parsed_instruction) in parsed_instruction_sequence.instructions().iter().enumerate() {
            let instruction_word = encode_instruction(parsed_instruction, (instruction_index as i64) * 4, &label_addresses)?;

            instruction_bytes.extend_from_slice(&instruction_word.to_be_bytes());
        }

        Ok(instruction_bytes)
    }

    fn disassemble(
        &self,
        instruction_bytes: &[u8],
    ) -> Result<String, String> {
        if instruction_bytes.is_empty() {
            return Err(String::from("Instruction byte sequence must not be empty."));
        }

        if instruction_bytes.len() % 4 != 0 {
            return Err(format!(
                "Instruction byte sequence length '{}' must be a multiple of 4.",
                instruction_bytes.len()
            ));
        }

        let mut decoded_instructions = Vec::with_capacity(instruction_bytes.len() / 4);

        for (instruction_index, instruction_word_bytes) in instruction_bytes.chunks_exact(4).enumerate() {
            let instruction_word = u32::from_be_bytes([
                instruction_word_bytes[0],
                instruction_word_bytes[1],
                instruction_word_bytes[2],
                instruction_word_bytes[3],
            ]);

            decoded_instructions.push(decode_instruction(instruction_word, (instruction_index as i64) * 4)?);
        }

        Ok(format_decoded_instruction_sequence(&decoded_instructions, instruction_bytes.len()))
    }
}

fn encode_instruction(
    parsed_instruction: &ParsedInstruction,
    current_instruction_address: i64,
    label_addresses: &HashMap<String, i64>,
) -> Result<u32, String> {
    match parsed_instruction.mnemonic() {
        "nop" if parsed_instruction.operands().is_empty() => Ok(0x6000_0000),
        "blr" if parsed_instruction.operands().is_empty() => Ok(0x4E80_0020),
        "b" | "bl" => encode_branch(parsed_instruction, current_instruction_address, label_addresses),
        "li" => encode_li(parsed_instruction),
        "addi" => encode_addi(parsed_instruction),
        "lwz" | "stw" => encode_load_store(parsed_instruction),
        "mr" => encode_mr(parsed_instruction),
        unsupported_mnemonic => Err(format!("Unsupported PowerPC mnemonic '{}'.", unsupported_mnemonic)),
    }
}

fn decode_instruction(
    instruction_word: u32,
    current_instruction_address: i64,
) -> Result<DecodedInstruction, String> {
    if instruction_word == 0x6000_0000 {
        return Ok(DecodedInstruction::Plain(String::from("nop")));
    }

    if instruction_word == 0x4E80_0020 {
        return Ok(DecodedInstruction::Plain(String::from("blr")));
    }

    let primary_opcode = instruction_word >> 26;

    if primary_opcode == 18 {
        let branch_delta = sign_extend_branch_delta(instruction_word & 0x03FF_FFFC);
        let target_address = current_instruction_address + branch_delta;
        let mnemonic = if (instruction_word & 1) != 0 { "bl" } else { "b" };

        return Ok(DecodedInstruction::Branch { mnemonic, target_address });
    }

    if primary_opcode == 14 {
        let destination_register_index = ((instruction_word >> 21) & 0x1F) as u8;
        let left_register_index = ((instruction_word >> 16) & 0x1F) as u8;
        let immediate_value = instruction_word as i16;

        return if left_register_index == 0 {
            Ok(DecodedInstruction::Plain(format!(
                "li {}, {}",
                format_powerpc_register_name(destination_register_index),
                immediate_value
            )))
        } else {
            Ok(DecodedInstruction::Plain(format!(
                "addi {}, {}, {}",
                format_powerpc_register_name(destination_register_index),
                format_powerpc_register_name(left_register_index),
                immediate_value
            )))
        };
    }

    if primary_opcode == 32 || primary_opcode == 36 {
        let register_index = ((instruction_word >> 21) & 0x1F) as u8;
        let base_register_index = ((instruction_word >> 16) & 0x1F) as u8;
        let displacement = instruction_word as i16;
        let mnemonic = if primary_opcode == 32 { "lwz" } else { "stw" };

        return Ok(DecodedInstruction::Plain(format!(
            "{} {}, {}({})",
            mnemonic,
            format_powerpc_register_name(register_index),
            displacement,
            format_powerpc_register_name(base_register_index)
        )));
    }

    if primary_opcode == 31 {
        let source_register_index = ((instruction_word >> 21) & 0x1F) as u8;
        let destination_register_index = ((instruction_word >> 16) & 0x1F) as u8;
        let right_register_index = ((instruction_word >> 11) & 0x1F) as u8;
        let extended_opcode = (instruction_word >> 1) & 0x3FF;

        if extended_opcode == 444 && source_register_index == right_register_index {
            return Ok(DecodedInstruction::Plain(format!(
                "mr {}, {}",
                format_powerpc_register_name(destination_register_index),
                format_powerpc_register_name(source_register_index)
            )));
        }
    }

    Err(format!("Unsupported PowerPC instruction 0x{:08X}.", instruction_word))
}

fn encode_branch(
    parsed_instruction: &ParsedInstruction,
    current_instruction_address: i64,
    label_addresses: &HashMap<String, i64>,
) -> Result<u32, String> {
    let target_address = resolve_branch_target_address(parsed_instruction.operands(), label_addresses)?;
    let branch_delta = target_address - current_instruction_address;

    if branch_delta % 4 != 0 {
        return Err(format!("PowerPC branch target '{}' must be 4-byte aligned.", format_signed_hex(target_address)));
    }

    if !(-0x20_00000..=0x1F_FFFFC).contains(&branch_delta) {
        return Err(format!(
            "PowerPC branch target '{}' is out of range for the current relative branch encoding.",
            format_signed_hex(target_address)
        ));
    }

    let branch_opcode = if parsed_instruction.mnemonic() == "bl" { 0x4800_0001 } else { 0x4800_0000 };

    Ok(branch_opcode | ((branch_delta as i32 as u32) & 0x03FF_FFFC))
}

fn encode_li(parsed_instruction: &ParsedInstruction) -> Result<u32, String> {
    let destination_register_index = parse_register_operand(parsed_instruction.operands(), 0)?;
    let immediate_value = parse_i16_immediate_operand(parsed_instruction.operands(), 1, "li")?;

    Ok(0x3800_0000 | ((destination_register_index as u32) << 21) | ((immediate_value as u16) as u32))
}

fn encode_addi(parsed_instruction: &ParsedInstruction) -> Result<u32, String> {
    let destination_register_index = parse_register_operand(parsed_instruction.operands(), 0)?;
    let left_register_index = parse_register_operand(parsed_instruction.operands(), 1)?;
    let immediate_value = parse_i16_immediate_operand(parsed_instruction.operands(), 2, "addi")?;

    Ok(0x3800_0000 | ((destination_register_index as u32) << 21) | ((left_register_index as u32) << 16) | ((immediate_value as u16) as u32))
}

fn encode_load_store(parsed_instruction: &ParsedInstruction) -> Result<u32, String> {
    let register_index = parse_register_operand(parsed_instruction.operands(), 0)?;
    let (displacement, base_register_name) = parse_powerpc_memory_operand(parse_identifier_operand(parsed_instruction.operands(), 1)?)?;
    let base_register_index =
        parse_powerpc_register_name(&base_register_name).ok_or_else(|| format!("Unsupported PowerPC base register '{}'.", base_register_name))?;
    let instruction_base = if parsed_instruction.mnemonic() == "lwz" { 0x8000_0000 } else { 0x9000_0000 };

    Ok(instruction_base | ((register_index as u32) << 21) | ((base_register_index as u32) << 16) | ((displacement as u16) as u32))
}

fn encode_mr(parsed_instruction: &ParsedInstruction) -> Result<u32, String> {
    let destination_register_index = parse_register_operand(parsed_instruction.operands(), 0)?;
    let source_register_index = parse_register_operand(parsed_instruction.operands(), 1)?;

    Ok(0x7C00_0378 | ((source_register_index as u32) << 21) | ((destination_register_index as u32) << 16) | ((source_register_index as u32) << 11))
}

fn parse_register_operand(
    operands: &[InstructionOperand],
    operand_index: usize,
) -> Result<u8, String> {
    let register_name = parse_identifier_operand(operands, operand_index)?;

    parse_powerpc_register_name(register_name).ok_or_else(|| format!("Unsupported PowerPC register '{}'.", register_name))
}

fn parse_identifier_operand<'a>(
    operands: &'a [InstructionOperand],
    operand_index: usize,
) -> Result<&'a str, String> {
    match operands.get(operand_index) {
        Some(InstructionOperand::Identifier(identifier)) => Ok(identifier.as_str()),
        Some(unexpected_operand) => Err(format!(
            "Expected identifier operand at position '{}' but found '{:?}'.",
            operand_index, unexpected_operand
        )),
        None => Err(format!("Instruction is missing operand '{}'.", operand_index)),
    }
}

fn parse_i16_immediate_operand(
    operands: &[InstructionOperand],
    operand_index: usize,
    mnemonic: &str,
) -> Result<i16, String> {
    match operands.get(operand_index) {
        Some(InstructionOperand::Immediate(immediate_value)) => {
            i16::try_from(*immediate_value).map_err(|_| format!("PowerPC {} immediate '{}' is out of i16 range.", mnemonic, immediate_value))
        }
        Some(unexpected_operand) => Err(format!(
            "PowerPC {} expected an immediate operand at position '{}' but found '{:?}'.",
            mnemonic, operand_index, unexpected_operand
        )),
        None => Err(format!("PowerPC {} is missing operand '{}'.", mnemonic, operand_index)),
    }
}

fn resolve_branch_target_address(
    operands: &[InstructionOperand],
    label_addresses: &HashMap<String, i64>,
) -> Result<i64, String> {
    if operands.len() != 1 {
        return Err(String::from("PowerPC branch instructions require exactly one target operand."));
    }

    match &operands[0] {
        InstructionOperand::Identifier(label_name) => label_addresses
            .get(label_name)
            .copied()
            .ok_or_else(|| format!("Unknown instruction label '{}'.", label_name)),
        InstructionOperand::Immediate(immediate_value) => Ok(*immediate_value as i64),
        unexpected_operand => Err(format!(
            "Unsupported branch operand '{:?}'. Expected a label or absolute instruction offset.",
            unexpected_operand
        )),
    }
}

fn sign_extend_branch_delta(encoded_delta: u32) -> i64 {
    let shift_amount = 64 - 26;

    ((encoded_delta as i64) << shift_amount) >> shift_amount
}

fn format_decoded_instruction_sequence(
    decoded_instructions: &[DecodedInstruction],
    total_byte_len: usize,
) -> String {
    let mut label_addresses = BTreeSet::new();

    for decoded_instruction in decoded_instructions {
        if let DecodedInstruction::Branch { target_address, .. } = decoded_instruction {
            if *target_address >= 0 && (*target_address as usize) < total_byte_len && (*target_address as usize) % 4 == 0 {
                label_addresses.insert(*target_address as usize);
            }
        }
    }

    let label_names = label_addresses
        .into_iter()
        .enumerate()
        .map(|(label_index, label_address)| (label_address, format!("label_{}", label_index)))
        .collect::<BTreeMap<_, _>>();
    let mut instruction_texts = Vec::with_capacity(decoded_instructions.len());

    for (instruction_index, decoded_instruction) in decoded_instructions.iter().enumerate() {
        let instruction_address = instruction_index * 4;
        let label_prefix = label_names
            .get(&instruction_address)
            .map(|label_name| format!("{}: ", label_name))
            .unwrap_or_default();
        let instruction_text = match decoded_instruction {
            DecodedInstruction::Plain(instruction_text) => instruction_text.clone(),
            DecodedInstruction::Branch { mnemonic, target_address } => {
                let target_text = label_names
                    .get(&(*target_address as usize))
                    .cloned()
                    .unwrap_or_else(|| format_signed_hex(*target_address));

                format!("{} {}", mnemonic, target_text)
            }
        };

        instruction_texts.push(format!("{}{}", label_prefix, instruction_text));
    }

    instruction_texts.join("; ")
}

fn format_signed_hex(value: i64) -> String {
    if value < 0 {
        format!("-0x{:X}", value.unsigned_abs())
    } else {
        format!("0x{:X}", value as u64)
    }
}
