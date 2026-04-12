use crate::{
    arm_memory_operand::parse_arm_memory_expression,
    arm32_register::{format_arm32_register_name, parse_arm32_register_name},
    arm64_register::{
        Arm64Register, Arm64RegisterKind, Arm64RegisterWidth, format_arm64_base_register, format_arm64_general_register, parse_arm64_register_name,
    },
};
use squalr_engine_api::plugins::instruction_set::{
    InstructionMemoryOperand, InstructionOperand, InstructionSet, ParsedInstruction, parse_instruction_sequence,
};
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ArmMode {
    Arm32,
    Arm64,
}

#[derive(Clone, Debug)]
enum DecodedInstruction {
    Plain(String),
    Branch { mnemonic: &'static str, target_address: i64 },
}

#[derive(Clone, Debug, Default)]
pub struct Arm32InstructionSet;

#[derive(Clone, Debug, Default)]
pub struct Arm64InstructionSet;

impl Arm32InstructionSet {
    pub fn new() -> Self {
        Self
    }
}

impl Arm64InstructionSet {
    pub fn new() -> Self {
        Self
    }
}

impl InstructionSet for Arm32InstructionSet {
    fn get_instruction_set_id(&self) -> &str {
        "arm"
    }

    fn get_display_name(&self) -> &str {
        "ARM"
    }

    fn assemble(
        &self,
        assembly_source: &str,
    ) -> Result<Vec<u8>, String> {
        assemble_instruction_sequence(ArmMode::Arm32, assembly_source)
    }

    fn disassemble(
        &self,
        instruction_bytes: &[u8],
    ) -> Result<String, String> {
        disassemble_instruction_sequence(ArmMode::Arm32, instruction_bytes)
    }

    fn build_no_operation_fill(
        &self,
        byte_count: usize,
    ) -> Result<Vec<u8>, String> {
        build_arm_no_operation_fill(ArmMode::Arm32, byte_count)
    }
}

impl InstructionSet for Arm64InstructionSet {
    fn get_instruction_set_id(&self) -> &str {
        "arm64"
    }

    fn get_display_name(&self) -> &str {
        "ARM64"
    }

    fn assemble(
        &self,
        assembly_source: &str,
    ) -> Result<Vec<u8>, String> {
        assemble_instruction_sequence(ArmMode::Arm64, assembly_source)
    }

    fn disassemble(
        &self,
        instruction_bytes: &[u8],
    ) -> Result<String, String> {
        disassemble_instruction_sequence(ArmMode::Arm64, instruction_bytes)
    }

    fn build_no_operation_fill(
        &self,
        byte_count: usize,
    ) -> Result<Vec<u8>, String> {
        build_arm_no_operation_fill(ArmMode::Arm64, byte_count)
    }
}

fn build_arm_no_operation_fill(
    arm_mode: ArmMode,
    byte_count: usize,
) -> Result<Vec<u8>, String> {
    if byte_count == 0 {
        return Ok(Vec::new());
    }

    let nop_bytes = assemble_instruction_sequence(arm_mode, "nop")?;

    if nop_bytes.is_empty() || byte_count % nop_bytes.len() != 0 {
        return Err(format!(
            "{:?} no-operation fill requires a byte count aligned to {} bytes.",
            arm_mode,
            nop_bytes.len().max(1)
        ));
    }

    let mut fill_bytes = Vec::with_capacity(byte_count);

    while fill_bytes.len() < byte_count {
        fill_bytes.extend_from_slice(&nop_bytes);
    }

    Ok(fill_bytes)
}

fn assemble_instruction_sequence(
    arm_mode: ArmMode,
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
        let current_instruction_address = (instruction_index as i64) * 4;
        let instruction_word = match arm_mode {
            ArmMode::Arm32 => encode_arm32_instruction(parsed_instruction, current_instruction_address, &label_addresses)?,
            ArmMode::Arm64 => encode_arm64_instruction(parsed_instruction, current_instruction_address, &label_addresses)?,
        };

        instruction_bytes.extend_from_slice(&instruction_word.to_le_bytes());
    }

    Ok(instruction_bytes)
}

fn disassemble_instruction_sequence(
    arm_mode: ArmMode,
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
        let instruction_word = u32::from_le_bytes([
            instruction_word_bytes[0],
            instruction_word_bytes[1],
            instruction_word_bytes[2],
            instruction_word_bytes[3],
        ]);
        let current_instruction_address = (instruction_index as i64) * 4;
        let decoded_instruction = match arm_mode {
            ArmMode::Arm32 => decode_arm32_instruction(instruction_word, current_instruction_address)?,
            ArmMode::Arm64 => decode_arm64_instruction(instruction_word, current_instruction_address)?,
        };

        decoded_instructions.push(decoded_instruction);
    }

    Ok(format_decoded_instruction_sequence(&decoded_instructions, instruction_bytes.len()))
}

fn encode_arm32_instruction(
    parsed_instruction: &ParsedInstruction,
    current_instruction_address: i64,
    label_addresses: &HashMap<String, i64>,
) -> Result<u32, String> {
    match parsed_instruction.mnemonic() {
        "nop" if parsed_instruction.operands().is_empty() => Ok(0xE320_F000),
        "bx" => Ok(0xE12F_FF10 | (parse_arm32_register_operand(parsed_instruction.operands(), 0)? as u32)),
        "b" | "bl" => encode_arm32_branch(parsed_instruction, current_instruction_address, label_addresses),
        "mov" => encode_arm32_mov(parsed_instruction),
        "add" => encode_arm32_add(parsed_instruction),
        "ldr" | "str" => encode_arm32_load_store(parsed_instruction),
        unsupported_mnemonic => Err(format!("Unsupported ARM mnemonic '{}'.", unsupported_mnemonic)),
    }
}

fn encode_arm64_instruction(
    parsed_instruction: &ParsedInstruction,
    current_instruction_address: i64,
    label_addresses: &HashMap<String, i64>,
) -> Result<u32, String> {
    match parsed_instruction.mnemonic() {
        "nop" if parsed_instruction.operands().is_empty() => Ok(0xD503_201F),
        "ret" => encode_arm64_ret(parsed_instruction),
        "b" | "bl" => encode_arm64_branch(parsed_instruction, current_instruction_address, label_addresses),
        "mov" => encode_arm64_mov(parsed_instruction),
        "add" => encode_arm64_add(parsed_instruction),
        "ldr" | "str" => encode_arm64_load_store(parsed_instruction),
        unsupported_mnemonic => Err(format!("Unsupported ARM64 mnemonic '{}'.", unsupported_mnemonic)),
    }
}

fn encode_arm32_branch(
    parsed_instruction: &ParsedInstruction,
    current_instruction_address: i64,
    label_addresses: &HashMap<String, i64>,
) -> Result<u32, String> {
    let target_address = resolve_branch_target_address(parsed_instruction.operands(), label_addresses)?;
    let branch_delta_bytes = target_address - (current_instruction_address + 8);

    if branch_delta_bytes % 4 != 0 {
        return Err(format!("ARM branch target '{}' must be 4-byte aligned.", format_signed_hex(target_address)));
    }

    let branch_delta_words = branch_delta_bytes / 4;

    if !(-0x80_0000..=0x7F_FFFF).contains(&branch_delta_words) {
        return Err(format!(
            "ARM branch target '{}' is out of range for a 24-bit branch immediate.",
            format_signed_hex(target_address)
        ));
    }

    let branch_opcode = if parsed_instruction.mnemonic() == "bl" { 0xEB00_0000 } else { 0xEA00_0000 };

    Ok(branch_opcode | ((branch_delta_words as i32 as u32) & 0x00FF_FFFF))
}

fn encode_arm64_branch(
    parsed_instruction: &ParsedInstruction,
    current_instruction_address: i64,
    label_addresses: &HashMap<String, i64>,
) -> Result<u32, String> {
    let target_address = resolve_branch_target_address(parsed_instruction.operands(), label_addresses)?;
    let branch_delta_bytes = target_address - current_instruction_address;

    if branch_delta_bytes % 4 != 0 {
        return Err(format!("ARM64 branch target '{}' must be 4-byte aligned.", format_signed_hex(target_address)));
    }

    let branch_delta_words = branch_delta_bytes / 4;

    if !(-0x20_00000..=0x1F_FFFFF).contains(&branch_delta_words) {
        return Err(format!(
            "ARM64 branch target '{}' is out of range for a 26-bit branch immediate.",
            format_signed_hex(target_address)
        ));
    }

    let branch_opcode = if parsed_instruction.mnemonic() == "bl" { 0x9400_0000 } else { 0x1400_0000 };

    Ok(branch_opcode | ((branch_delta_words as i32 as u32) & 0x03FF_FFFF))
}

fn encode_arm32_mov(parsed_instruction: &ParsedInstruction) -> Result<u32, String> {
    let destination_register_index = parse_arm32_register_operand(parsed_instruction.operands(), 0)?;
    let immediate_value = parse_non_negative_immediate_operand(parsed_instruction.operands(), 1, "ARM mov")? as u32;
    let (rotation, immediate8) = encode_arm32_immediate(immediate_value)
        .ok_or_else(|| format!("ARM mov immediate '{}' cannot be encoded as a rotated 8-bit immediate.", immediate_value))?;

    Ok(0xE3A0_0000 | ((destination_register_index as u32) << 12) | ((rotation as u32) << 8) | (immediate8 as u32))
}

fn encode_arm32_add(parsed_instruction: &ParsedInstruction) -> Result<u32, String> {
    let destination_register_index = parse_arm32_register_operand(parsed_instruction.operands(), 0)?;
    let left_register_index = parse_arm32_register_operand(parsed_instruction.operands(), 1)?;
    let immediate_value = parse_non_negative_immediate_operand(parsed_instruction.operands(), 2, "ARM add")? as u32;
    let (rotation, immediate8) = encode_arm32_immediate(immediate_value)
        .ok_or_else(|| format!("ARM add immediate '{}' cannot be encoded as a rotated 8-bit immediate.", immediate_value))?;

    Ok(0xE280_0000 | ((left_register_index as u32) << 16) | ((destination_register_index as u32) << 12) | ((rotation as u32) << 8) | (immediate8 as u32))
}

fn encode_arm32_load_store(parsed_instruction: &ParsedInstruction) -> Result<u32, String> {
    let data_register_index = parse_arm32_register_operand(parsed_instruction.operands(), 0)?;
    let memory_operand = parse_memory_operand(parsed_instruction.operands(), 1, parsed_instruction.mnemonic())?;

    if memory_operand.size().is_some() {
        return Err(format!(
            "ARM '{}' does not support explicit memory size prefixes in this plugin.",
            parsed_instruction.mnemonic()
        ));
    }

    let (base_register_name, offset) = parse_arm_memory_expression(memory_operand.expression_text())?;
    let base_register_index =
        parse_arm32_register_name(&base_register_name).ok_or_else(|| format!("Unsupported ARM base register '{}'.", base_register_name))?;
    let absolute_offset = offset.unsigned_abs();

    if absolute_offset > 0xFFF {
        return Err(format!("ARM {} offset '{}' is out of range.", parsed_instruction.mnemonic(), offset));
    }

    let load_store_base = if parsed_instruction.mnemonic() == "ldr" { 0xE590_0000 } else { 0xE580_0000 };
    let offset_direction_bit = if offset >= 0 { 1_u32 << 23 } else { 0 };

    Ok(load_store_base | offset_direction_bit | ((base_register_index as u32) << 16) | ((data_register_index as u32) << 12) | (absolute_offset as u32))
}

fn encode_arm64_ret(parsed_instruction: &ParsedInstruction) -> Result<u32, String> {
    let return_register_index = if parsed_instruction.operands().is_empty() {
        30
    } else {
        let return_register = parse_arm64_general_register_operand(parsed_instruction.operands(), 0, true)?;

        if return_register.width() != Arm64RegisterWidth::X {
            return Err(String::from("ARM64 ret requires an X register operand."));
        }

        return_register.index()
    };

    Ok(0xD65F_0000 | ((return_register_index as u32) << 5))
}

fn encode_arm64_mov(parsed_instruction: &ParsedInstruction) -> Result<u32, String> {
    let destination_register = parse_arm64_general_register_operand(parsed_instruction.operands(), 0, false)?;

    if destination_register.kind() == Arm64RegisterKind::StackPointer {
        return Err(String::from("ARM64 mov immediate does not support SP/WSP in this plugin."));
    }

    let immediate_value = parse_non_negative_immediate_operand(parsed_instruction.operands(), 1, "ARM64 mov")? as u64;

    if immediate_value > 0xFFFF {
        return Err(format!(
            "ARM64 mov immediate '{}' is out of range for the current MOVZ-only implementation.",
            immediate_value
        ));
    }

    let instruction_base = match destination_register.width() {
        Arm64RegisterWidth::W => 0x5280_0000,
        Arm64RegisterWidth::X => 0xD280_0000,
    };

    Ok(instruction_base | ((immediate_value as u32) << 5) | (destination_register.index() as u32))
}

fn encode_arm64_add(parsed_instruction: &ParsedInstruction) -> Result<u32, String> {
    let destination_register = parse_arm64_general_register_operand(parsed_instruction.operands(), 0, true)?;
    let left_register = parse_arm64_general_register_operand(parsed_instruction.operands(), 1, true)?;

    if destination_register.width() != left_register.width() {
        return Err(String::from("ARM64 add requires both registers to have the same width."));
    }

    let immediate_value = parse_non_negative_immediate_operand(parsed_instruction.operands(), 2, "ARM64 add")? as u64;

    if immediate_value > 0xFFF {
        return Err(format!("ARM64 add immediate '{}' is out of range.", immediate_value));
    }

    let instruction_base = match destination_register.width() {
        Arm64RegisterWidth::W => 0x1100_0000,
        Arm64RegisterWidth::X => 0x9100_0000,
    };

    Ok(instruction_base | ((immediate_value as u32) << 10) | ((left_register.index() as u32) << 5) | (destination_register.index() as u32))
}

fn encode_arm64_load_store(parsed_instruction: &ParsedInstruction) -> Result<u32, String> {
    let data_register = parse_arm64_general_register_operand(parsed_instruction.operands(), 0, false)?;
    let memory_operand = parse_memory_operand(parsed_instruction.operands(), 1, parsed_instruction.mnemonic())?;

    if memory_operand.size().is_some() {
        return Err(format!(
            "ARM64 '{}' does not support explicit memory size prefixes in this plugin.",
            parsed_instruction.mnemonic()
        ));
    }

    let (base_register_name, offset) = parse_arm_memory_expression(memory_operand.expression_text())?;
    let base_register = parse_arm64_register_name(&base_register_name).ok_or_else(|| format!("Unsupported ARM64 base register '{}'.", base_register_name))?;

    if base_register.width() != Arm64RegisterWidth::X || base_register.kind() == Arm64RegisterKind::Zero {
        return Err(format!("ARM64 base register '{}' is not valid for memory addressing.", base_register_name));
    }

    if offset < 0 {
        return Err(format!(
            "ARM64 {} does not support negative offsets in this plugin.",
            parsed_instruction.mnemonic()
        ));
    }

    let scale = match data_register.width() {
        Arm64RegisterWidth::W => 4_i64,
        Arm64RegisterWidth::X => 8_i64,
    };

    if offset % scale != 0 {
        return Err(format!(
            "ARM64 {} offset '{}' must be aligned to {} bytes.",
            parsed_instruction.mnemonic(),
            offset,
            scale
        ));
    }

    let scaled_offset = offset / scale;

    if scaled_offset > 0xFFF {
        return Err(format!("ARM64 {} offset '{}' is out of range.", parsed_instruction.mnemonic(), offset));
    }

    let instruction_base = match (parsed_instruction.mnemonic(), data_register.width()) {
        ("ldr", Arm64RegisterWidth::W) => 0xB940_0000,
        ("str", Arm64RegisterWidth::W) => 0xB900_0000,
        ("ldr", Arm64RegisterWidth::X) => 0xF940_0000,
        ("str", Arm64RegisterWidth::X) => 0xF900_0000,
        _ => unreachable!(),
    };

    Ok(instruction_base | ((scaled_offset as u32) << 10) | ((base_register.index() as u32) << 5) | (data_register.index() as u32))
}

fn decode_arm32_instruction(
    instruction_word: u32,
    current_instruction_address: i64,
) -> Result<DecodedInstruction, String> {
    let condition_code = instruction_word >> 28;

    if condition_code != 0xE {
        return Err(format!("Unsupported ARM condition code in instruction 0x{:08X}.", instruction_word));
    }

    if instruction_word == 0xE320_F000 {
        return Ok(DecodedInstruction::Plain(String::from("nop")));
    }

    if instruction_word & 0x0FFF_FFF0 == 0x012F_FF10 {
        let register_index = (instruction_word & 0xF) as u8;

        return Ok(DecodedInstruction::Plain(format!("bx {}", format_arm32_register_name(register_index))));
    }

    let op27_25 = (instruction_word >> 25) & 0b111;

    if op27_25 == 0b101 {
        let branch_delta_words = sign_extend(instruction_word & 0x00FF_FFFF, 24);
        let target_address = current_instruction_address + 8 + branch_delta_words * 4;
        let mnemonic = if (instruction_word & (1 << 24)) != 0 { "bl" } else { "b" };

        return Ok(DecodedInstruction::Branch { mnemonic, target_address });
    }

    if op27_25 == 0b001 {
        let opcode = (instruction_word >> 21) & 0xF;
        let left_register_index = ((instruction_word >> 16) & 0xF) as u8;
        let destination_register_index = ((instruction_word >> 12) & 0xF) as u8;
        let rotate_right_bits = (((instruction_word >> 8) & 0xF) * 2) as u32;
        let immediate8 = instruction_word & 0xFF;
        let immediate_value = immediate8.rotate_right(rotate_right_bits);

        return match opcode {
            0xD if left_register_index == 0 => Ok(DecodedInstruction::Plain(format!(
                "mov {}, #{}",
                format_arm32_register_name(destination_register_index),
                immediate_value
            ))),
            0x4 => Ok(DecodedInstruction::Plain(format!(
                "add {}, {}, #{}",
                format_arm32_register_name(destination_register_index),
                format_arm32_register_name(left_register_index),
                immediate_value
            ))),
            _ => Err(format!("Unsupported ARM data-processing immediate instruction 0x{:08X}.", instruction_word)),
        };
    }

    if ((instruction_word >> 26) & 0b11) == 0b01 && ((instruction_word >> 25) & 0b1) == 0 {
        let pre_indexed = (instruction_word & (1 << 24)) != 0;
        let add_offset = (instruction_word & (1 << 23)) != 0;
        let byte_transfer = (instruction_word & (1 << 22)) != 0;
        let write_back = (instruction_word & (1 << 21)) != 0;
        let is_load = (instruction_word & (1 << 20)) != 0;

        if !pre_indexed || byte_transfer || write_back {
            return Err(format!("Unsupported ARM load/store addressing mode in 0x{:08X}.", instruction_word));
        }

        let base_register_index = ((instruction_word >> 16) & 0xF) as u8;
        let data_register_index = ((instruction_word >> 12) & 0xF) as u8;
        let offset = (instruction_word & 0xFFF) as i64;
        let signed_offset = if add_offset { offset } else { -offset };
        let mnemonic = if is_load { "ldr" } else { "str" };

        return Ok(DecodedInstruction::Plain(format!(
            "{} {}, [{}{}]",
            mnemonic,
            format_arm32_register_name(data_register_index),
            format_arm32_register_name(base_register_index),
            format_arm_memory_offset_suffix(signed_offset)
        )));
    }

    Err(format!("Unsupported ARM instruction 0x{:08X}.", instruction_word))
}

fn decode_arm64_instruction(
    instruction_word: u32,
    current_instruction_address: i64,
) -> Result<DecodedInstruction, String> {
    if instruction_word == 0xD503_201F {
        return Ok(DecodedInstruction::Plain(String::from("nop")));
    }

    if instruction_word & 0xFC00_0000 == 0x1400_0000 {
        let branch_delta_words = sign_extend(instruction_word & 0x03FF_FFFF, 26);
        let target_address = current_instruction_address + branch_delta_words * 4;
        let mnemonic = if (instruction_word & 0x8000_0000) != 0 { "bl" } else { "b" };

        return Ok(DecodedInstruction::Branch { mnemonic, target_address });
    }

    if instruction_word & 0xFFFF_FC1F == 0xD65F_0000 {
        let return_register_index = ((instruction_word >> 5) & 0x1F) as u8;

        return if return_register_index == 30 {
            Ok(DecodedInstruction::Plain(String::from("ret")))
        } else {
            Ok(DecodedInstruction::Plain(format!(
                "ret {}",
                format_arm64_general_register(Arm64RegisterWidth::X, return_register_index)
            )))
        };
    }

    if instruction_word & 0xFFE0_0000 == 0xD280_0000 {
        return Ok(DecodedInstruction::Plain(format!(
            "mov {}, #{}",
            format_arm64_general_register(Arm64RegisterWidth::X, (instruction_word & 0x1F) as u8),
            (instruction_word >> 5) & 0xFFFF
        )));
    }

    if instruction_word & 0xFFE0_0000 == 0x5280_0000 {
        return Ok(DecodedInstruction::Plain(format!(
            "mov {}, #{}",
            format_arm64_general_register(Arm64RegisterWidth::W, (instruction_word & 0x1F) as u8),
            (instruction_word >> 5) & 0xFFFF
        )));
    }

    if instruction_word & 0xFFC0_0000 == 0x9100_0000 {
        let left_register_index = ((instruction_word >> 5) & 0x1F) as u8;

        return Ok(DecodedInstruction::Plain(format!(
            "add {}, {}, #{}",
            format_arm64_general_register(Arm64RegisterWidth::X, (instruction_word & 0x1F) as u8),
            if left_register_index == 31 {
                String::from("sp")
            } else {
                format_arm64_general_register(Arm64RegisterWidth::X, left_register_index)
            },
            (instruction_word >> 10) & 0xFFF
        )));
    }

    if instruction_word & 0xFFC0_0000 == 0x1100_0000 {
        return Ok(DecodedInstruction::Plain(format!(
            "add {}, {}, #{}",
            format_arm64_general_register(Arm64RegisterWidth::W, (instruction_word & 0x1F) as u8),
            format_arm64_general_register(Arm64RegisterWidth::W, ((instruction_word >> 5) & 0x1F) as u8),
            (instruction_word >> 10) & 0xFFF
        )));
    }

    if instruction_word & 0xFFC0_0000 == 0xB940_0000 {
        return Ok(decode_arm64_unsigned_immediate_load_store(instruction_word, "ldr", Arm64RegisterWidth::W, 4));
    }

    if instruction_word & 0xFFC0_0000 == 0xB900_0000 {
        return Ok(decode_arm64_unsigned_immediate_load_store(instruction_word, "str", Arm64RegisterWidth::W, 4));
    }

    if instruction_word & 0xFFC0_0000 == 0xF940_0000 {
        return Ok(decode_arm64_unsigned_immediate_load_store(instruction_word, "ldr", Arm64RegisterWidth::X, 8));
    }

    if instruction_word & 0xFFC0_0000 == 0xF900_0000 {
        return Ok(decode_arm64_unsigned_immediate_load_store(instruction_word, "str", Arm64RegisterWidth::X, 8));
    }

    Err(format!("Unsupported ARM64 instruction 0x{:08X}.", instruction_word))
}

fn decode_arm64_unsigned_immediate_load_store(
    instruction_word: u32,
    mnemonic: &'static str,
    register_width: Arm64RegisterWidth,
    scale: i64,
) -> DecodedInstruction {
    let data_register_index = (instruction_word & 0x1F) as u8;
    let base_register_index = ((instruction_word >> 5) & 0x1F) as u8;
    let offset = (((instruction_word >> 10) & 0xFFF) as i64) * scale;

    DecodedInstruction::Plain(format!(
        "{} {}, [{}{}]",
        mnemonic,
        format_arm64_general_register(register_width, data_register_index),
        format_arm64_base_register(base_register_index),
        format_arm_memory_offset_suffix(offset)
    ))
}

fn parse_arm32_register_operand(
    operands: &[InstructionOperand],
    operand_index: usize,
) -> Result<u8, String> {
    let register_name = parse_identifier_operand(operands, operand_index)?;

    parse_arm32_register_name(register_name).ok_or_else(|| format!("Unsupported ARM register '{}'.", register_name))
}

fn parse_arm64_general_register_operand(
    operands: &[InstructionOperand],
    operand_index: usize,
    allow_stack_pointer: bool,
) -> Result<Arm64Register, String> {
    let register_name = parse_identifier_operand(operands, operand_index)?;
    let register = parse_arm64_register_name(register_name).ok_or_else(|| format!("Unsupported ARM64 register '{}'.", register_name))?;

    if !allow_stack_pointer && register.kind() == Arm64RegisterKind::StackPointer {
        return Err(format!("Register '{}' is not valid in this ARM64 operand position.", register_name));
    }

    Ok(register)
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

fn parse_non_negative_immediate_operand(
    operands: &[InstructionOperand],
    operand_index: usize,
    instruction_name: &str,
) -> Result<i128, String> {
    match operands.get(operand_index) {
        Some(InstructionOperand::Immediate(immediate_value)) if *immediate_value >= 0 => Ok(*immediate_value),
        Some(InstructionOperand::Immediate(immediate_value)) => Err(format!(
            "{} does not support negative immediates in this plugin (got '{}').",
            instruction_name, immediate_value
        )),
        Some(unexpected_operand) => Err(format!(
            "{} expected an immediate operand at position '{}' but found '{:?}'.",
            instruction_name, operand_index, unexpected_operand
        )),
        None => Err(format!("{} is missing operand '{}'.", instruction_name, operand_index)),
    }
}

fn parse_memory_operand<'a>(
    operands: &'a [InstructionOperand],
    operand_index: usize,
    mnemonic: &str,
) -> Result<&'a InstructionMemoryOperand, String> {
    match operands.get(operand_index) {
        Some(InstructionOperand::Memory(memory_operand)) => Ok(memory_operand),
        Some(unexpected_operand) => Err(format!(
            "{} expected a memory operand at position '{}' but found '{:?}'.",
            mnemonic, operand_index, unexpected_operand
        )),
        None => Err(format!("{} is missing operand '{}'.", mnemonic, operand_index)),
    }
}

fn resolve_branch_target_address(
    operands: &[InstructionOperand],
    label_addresses: &HashMap<String, i64>,
) -> Result<i64, String> {
    if operands.len() != 1 {
        return Err(String::from("Branch instructions require exactly one target operand."));
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

fn encode_arm32_immediate(value: u32) -> Option<(u8, u8)> {
    for rotation in 0..=15_u8 {
        let rotated_value = value.rotate_left((rotation as u32) * 2);

        if rotated_value <= 0xFF {
            return Some((rotation, rotated_value as u8));
        }
    }

    None
}

fn sign_extend(
    value: u32,
    bit_width: u32,
) -> i64 {
    let shift_amount = 64 - bit_width;

    ((value as i64) << shift_amount) >> shift_amount
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

fn format_arm_memory_offset_suffix(offset: i64) -> String {
    if offset == 0 { String::new() } else { format!(", #{}", offset) }
}

fn format_signed_hex(value: i64) -> String {
    if value < 0 {
        format!("-0x{:X}", value.unsigned_abs())
    } else {
        format!("0x{:X}", value as u64)
    }
}
