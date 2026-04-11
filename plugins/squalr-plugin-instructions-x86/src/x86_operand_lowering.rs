use crate::{
    x86_memory_operand::{X86InstructionMode, memory_operand_matches_operand_kind, memory_operand_specificity_score, parse_memory_operand},
    x86_opcode_index::get_opcode_candidates,
    x86_register::{parse_register, register_matches_operand_kind, register_operand_specificity_score},
};
use iced_x86::{Code, Instruction, OpCodeOperandKind, OpKind, Register, RoundingControl};
use squalr_engine_api::plugins::instruction_set::{
    InstructionDecorators, InstructionMemoryOperandSize, InstructionOperand, InstructionRoundingControl, ParsedInstruction,
};
use std::collections::HashMap;

pub fn build_candidate_instructions(
    parsed_instruction: &ParsedInstruction,
    instruction_mode: X86InstructionMode,
    display_name: &str,
    label_addresses: &HashMap<String, u64>,
) -> Result<Vec<Instruction>, String> {
    let mut candidate_codes = get_ranked_candidate_codes(parsed_instruction, instruction_mode, label_addresses);

    if candidate_codes.is_empty() {
        return Err(format!("Unsupported {} instruction '{}'.", display_name, parsed_instruction.source_text()));
    }

    let mut candidate_instructions = Vec::new();
    let mut candidate_errors = Vec::new();

    for candidate_code in candidate_codes.drain(..) {
        match build_candidate_instruction(candidate_code, parsed_instruction, instruction_mode, label_addresses) {
            Ok(instruction) => candidate_instructions.push(instruction),
            Err(candidate_error) => candidate_errors.push(candidate_error),
        }
    }

    if !candidate_instructions.is_empty() {
        return Ok(candidate_instructions);
    }

    let candidate_error_summary = candidate_errors
        .into_iter()
        .take(3)
        .collect::<Vec<_>>()
        .join(" ");

    Err(format!(
        "Failed to assemble {} '{}'. {}",
        display_name,
        parsed_instruction.source_text(),
        candidate_error_summary
    ))
}

fn get_ranked_candidate_codes(
    parsed_instruction: &ParsedInstruction,
    instruction_mode: X86InstructionMode,
    label_addresses: &HashMap<String, u64>,
) -> Vec<Code> {
    let mut scored_candidates = get_opcode_candidates(parsed_instruction.mnemonic())
        .into_iter()
        .filter(|candidate_code| {
            candidate_code
                .op_code()
                .is_available_in_mode(instruction_mode.bitness())
        })
        .filter_map(|candidate_code| {
            score_candidate_code(candidate_code, parsed_instruction, instruction_mode, label_addresses).map(|candidate_score| (candidate_score, candidate_code))
        })
        .collect::<Vec<_>>();

    scored_candidates.sort_by(|(left_score, left_code), (right_score, right_code)| {
        right_score
            .cmp(left_score)
            .then_with(|| (*left_code as usize).cmp(&(*right_code as usize)))
    });

    scored_candidates
        .into_iter()
        .map(|(_, candidate_code)| candidate_code)
        .collect()
}

fn score_candidate_code(
    candidate_code: Code,
    parsed_instruction: &ParsedInstruction,
    instruction_mode: X86InstructionMode,
    label_addresses: &HashMap<String, u64>,
) -> Option<u32> {
    let opcode_info = candidate_code.op_code();
    let operand_count = parsed_instruction.operands().len();

    if opcode_info.op_count() as usize != operand_count {
        return None;
    }

    if !instruction_decorators_match_candidate(parsed_instruction.decorators(), candidate_code) {
        return None;
    }

    let mut total_score = 0_u32;

    for (operand_index, parsed_operand) in parsed_instruction.operands().iter().enumerate() {
        let operand_kind = opcode_info.op_kind(operand_index as u32);
        total_score = total_score.checked_add(score_candidate_operand(
            parsed_operand,
            operand_kind,
            candidate_code,
            instruction_mode,
            label_addresses,
        )?)?;
    }

    Some(total_score)
}

fn score_candidate_operand(
    parsed_operand: &InstructionOperand,
    operand_kind: OpCodeOperandKind,
    candidate_code: Code,
    instruction_mode: X86InstructionMode,
    label_addresses: &HashMap<String, u64>,
) -> Option<u32> {
    match parsed_operand {
        InstructionOperand::Identifier(identifier_text) => {
            if let Some(parsed_register) = parse_register(identifier_text) {
                if !register_matches_operand_kind(parsed_register, operand_kind) {
                    return None;
                }

                return Some(register_operand_specificity_score(operand_kind));
            }

            if label_addresses.contains_key(identifier_text)
                && branch_operand_kind_accepts_label(operand_kind)
                && branch_operand_matches_instruction_mode(operand_kind, instruction_mode)
            {
                return Some(branch_operand_specificity_score(operand_kind));
            }

            None
        }
        InstructionOperand::Immediate(immediate_value) => {
            if branch_operand_kind_accepts_label(operand_kind) && !branch_operand_matches_instruction_mode(operand_kind, instruction_mode) {
                return None;
            }

            if !immediate_fits_operand_kind(*immediate_value, operand_kind) {
                return None;
            }

            Some(immediate_operand_specificity_score(operand_kind))
        }
        InstructionOperand::Memory(memory_operand) => {
            let lowered_memory_operand = parse_memory_operand(memory_operand, instruction_mode).ok()?;
            let mut operand_score = memory_operand_specificity_score(operand_kind);

            if !memory_operand_matches_operand_kind(&lowered_memory_operand, operand_kind) {
                return None;
            }

            if memory_operand.is_broadcast() && !candidate_code.op_code().can_broadcast() {
                return None;
            }

            if !memory_size_hint_matches_candidate(memory_operand.size(), candidate_code) {
                return None;
            }

            if memory_operand.size().is_some() {
                operand_score = operand_score.saturating_add(20);
            } else if candidate_code.op_code().memory_size().info().size() == default_implicit_memory_operand_size(instruction_mode) {
                operand_score = operand_score.saturating_add(10);
            }

            Some(operand_score)
        }
    }
}

fn build_candidate_instruction(
    candidate_code: Code,
    parsed_instruction: &ParsedInstruction,
    instruction_mode: X86InstructionMode,
    label_addresses: &HashMap<String, u64>,
) -> Result<Instruction, String> {
    let opcode_info = candidate_code.op_code();
    let mut instruction = Instruction::default();

    instruction.set_code(candidate_code);

    for (operand_index, parsed_operand) in parsed_instruction.operands().iter().enumerate() {
        let operand_kind = opcode_info.op_kind(operand_index as u32);
        apply_candidate_operand(
            &mut instruction,
            operand_index as u32,
            parsed_operand,
            operand_kind,
            instruction_mode,
            label_addresses,
        )?;
    }

    apply_instruction_decorators(&mut instruction, parsed_instruction.decorators(), candidate_code)?;

    Ok(instruction)
}

fn apply_candidate_operand(
    instruction: &mut Instruction,
    operand_index: u32,
    parsed_operand: &InstructionOperand,
    operand_kind: OpCodeOperandKind,
    instruction_mode: X86InstructionMode,
    label_addresses: &HashMap<String, u64>,
) -> Result<(), String> {
    match parsed_operand {
        InstructionOperand::Identifier(identifier_text) => {
            if let Some(parsed_register) = parse_register(identifier_text) {
                if !register_matches_operand_kind(parsed_register, operand_kind) {
                    return Err(format!("Register '{}' does not match operand kind {:?}.", identifier_text, operand_kind));
                }

                instruction
                    .try_set_op_kind(operand_index, OpKind::Register)
                    .map_err(|instruction_error| instruction_error.to_string())?;
                instruction
                    .try_set_op_register(operand_index, parsed_register)
                    .map_err(|instruction_error| instruction_error.to_string())?;

                return Ok(());
            }

            if let Some(label_target_address) = label_addresses.get(identifier_text).copied() {
                return apply_label_operand(instruction, operand_index, label_target_address, operand_kind, identifier_text);
            }

            Err(format!("Unsupported identifier '{}' in instruction operand.", identifier_text))
        }
        InstructionOperand::Immediate(immediate_value) => apply_immediate_operand(instruction, operand_index, *immediate_value, operand_kind),
        InstructionOperand::Memory(memory_operand) => {
            let lowered_memory_operand = parse_memory_operand(memory_operand, instruction_mode)?;
            let instruction_operand_kind = operand_kind_to_instruction_memory_kind(operand_kind, instruction_mode)
                .ok_or_else(|| format!("Operand kind {:?} does not accept a memory operand.", operand_kind))?;

            if !memory_operand_matches_operand_kind(&lowered_memory_operand, operand_kind) {
                return Err(format!(
                    "Memory operand '{}' does not match operand kind {:?}.",
                    memory_operand.expression_text(),
                    operand_kind
                ));
            }

            instruction
                .try_set_op_kind(operand_index, instruction_operand_kind)
                .map_err(|instruction_error| instruction_error.to_string())?;
            instruction.set_memory_base(lowered_memory_operand.base);
            instruction.set_memory_index(lowered_memory_operand.index);
            instruction.set_memory_index_scale(lowered_memory_operand.scale);
            instruction.set_memory_displ_size(lowered_memory_operand.displ_size);
            instruction.set_memory_displacement64(lowered_memory_operand.displacement as u64);
            instruction.set_is_broadcast(lowered_memory_operand.is_broadcast);
            instruction.set_segment_prefix(lowered_memory_operand.segment_prefix);

            Ok(())
        }
    }
}

fn apply_immediate_operand(
    instruction: &mut Instruction,
    operand_index: u32,
    immediate_value: i128,
    operand_kind: OpCodeOperandKind,
) -> Result<(), String> {
    match operand_kind {
        OpCodeOperandKind::imm4_m2z | OpCodeOperandKind::imm8 | OpCodeOperandKind::imm8_const_1 => {
            let immediate_value =
                u8::try_from(immediate_value).map_err(|_| format!("Immediate '{}' does not fit an unsigned 8-bit operand.", immediate_value))?;
            let instruction_operand_kind = if operand_kind == OpCodeOperandKind::imm8 && has_previous_immediate_operand(instruction, operand_index) {
                OpKind::Immediate8_2nd
            } else {
                OpKind::Immediate8
            };

            instruction
                .try_set_op_kind(operand_index, instruction_operand_kind)
                .map_err(|instruction_error| instruction_error.to_string())?;
            if instruction_operand_kind == OpKind::Immediate8_2nd {
                instruction.set_immediate8_2nd(immediate_value);
            } else {
                instruction.set_immediate8(immediate_value);
            }

            Ok(())
        }
        OpCodeOperandKind::imm8sex16 => {
            let immediate_value =
                i16::try_from(immediate_value).map_err(|_| format!("Immediate '{}' does not fit a sign-extended 8-bit operand.", immediate_value))?;

            instruction
                .try_set_op_kind(operand_index, OpKind::Immediate8to16)
                .map_err(|instruction_error| instruction_error.to_string())?;
            instruction.set_immediate8to16(immediate_value);

            Ok(())
        }
        OpCodeOperandKind::imm8sex32 => {
            let immediate_value =
                i32::try_from(immediate_value).map_err(|_| format!("Immediate '{}' does not fit a sign-extended 8-bit operand.", immediate_value))?;

            instruction
                .try_set_op_kind(operand_index, OpKind::Immediate8to32)
                .map_err(|instruction_error| instruction_error.to_string())?;
            instruction.set_immediate8to32(immediate_value);

            Ok(())
        }
        OpCodeOperandKind::imm8sex64 => {
            let immediate_value =
                i64::try_from(immediate_value).map_err(|_| format!("Immediate '{}' does not fit a sign-extended 8-bit operand.", immediate_value))?;

            instruction
                .try_set_op_kind(operand_index, OpKind::Immediate8to64)
                .map_err(|instruction_error| instruction_error.to_string())?;
            instruction.set_immediate8to64(immediate_value);

            Ok(())
        }
        OpCodeOperandKind::imm16 => {
            let immediate_value =
                u16::try_from(immediate_value).map_err(|_| format!("Immediate '{}' does not fit an unsigned 16-bit operand.", immediate_value))?;

            instruction
                .try_set_op_kind(operand_index, OpKind::Immediate16)
                .map_err(|instruction_error| instruction_error.to_string())?;
            instruction.set_immediate16(immediate_value);

            Ok(())
        }
        OpCodeOperandKind::imm32 => {
            let immediate_value =
                u32::try_from(immediate_value).map_err(|_| format!("Immediate '{}' does not fit an unsigned 32-bit operand.", immediate_value))?;

            instruction
                .try_set_op_kind(operand_index, OpKind::Immediate32)
                .map_err(|instruction_error| instruction_error.to_string())?;
            instruction.set_immediate32(immediate_value);

            Ok(())
        }
        OpCodeOperandKind::imm32sex64 => {
            let immediate_value =
                i64::try_from(immediate_value).map_err(|_| format!("Immediate '{}' does not fit a sign-extended 32-bit operand.", immediate_value))?;

            instruction
                .try_set_op_kind(operand_index, OpKind::Immediate32to64)
                .map_err(|instruction_error| instruction_error.to_string())?;
            instruction.set_immediate32to64(immediate_value);

            Ok(())
        }
        OpCodeOperandKind::imm64 => {
            let immediate_value =
                u64::try_from(immediate_value).map_err(|_| format!("Immediate '{}' does not fit an unsigned 64-bit operand.", immediate_value))?;

            instruction
                .try_set_op_kind(operand_index, OpKind::Immediate64)
                .map_err(|instruction_error| instruction_error.to_string())?;
            instruction.set_immediate64(immediate_value);

            Ok(())
        }
        OpCodeOperandKind::br16_1 | OpCodeOperandKind::br16_2 | OpCodeOperandKind::xbegin_2 | OpCodeOperandKind::brdisp_2 => {
            let immediate_value =
                u16::try_from(immediate_value).map_err(|_| format!("Branch target '{}' does not fit a 16-bit branch operand.", immediate_value))?;

            instruction
                .try_set_op_kind(operand_index, OpKind::NearBranch16)
                .map_err(|instruction_error| instruction_error.to_string())?;
            instruction.set_near_branch16(immediate_value);

            Ok(())
        }
        OpCodeOperandKind::br32_1 | OpCodeOperandKind::br32_4 | OpCodeOperandKind::xbegin_4 | OpCodeOperandKind::brdisp_4 => {
            let immediate_value =
                u32::try_from(immediate_value).map_err(|_| format!("Branch target '{}' does not fit a 32-bit branch operand.", immediate_value))?;

            instruction
                .try_set_op_kind(operand_index, OpKind::NearBranch32)
                .map_err(|instruction_error| instruction_error.to_string())?;
            instruction.set_near_branch32(immediate_value);

            Ok(())
        }
        OpCodeOperandKind::br64_1 | OpCodeOperandKind::br64_4 => {
            let immediate_value =
                u64::try_from(immediate_value).map_err(|_| format!("Branch target '{}' does not fit a 64-bit branch operand.", immediate_value))?;

            instruction
                .try_set_op_kind(operand_index, OpKind::NearBranch64)
                .map_err(|instruction_error| instruction_error.to_string())?;
            instruction.set_near_branch64(immediate_value);

            Ok(())
        }
        _ => Err(format!("Operand kind {:?} does not accept an immediate operand.", operand_kind)),
    }
}

fn apply_label_operand(
    instruction: &mut Instruction,
    operand_index: u32,
    label_target_address: u64,
    operand_kind: OpCodeOperandKind,
    label_name: &str,
) -> Result<(), String> {
    match operand_kind {
        OpCodeOperandKind::br16_1 | OpCodeOperandKind::br16_2 | OpCodeOperandKind::xbegin_2 | OpCodeOperandKind::brdisp_2 => {
            let branch_target = u16::try_from(label_target_address)
                .map_err(|_| format!("Label '{}' target '{}' does not fit a 16-bit branch operand.", label_name, label_target_address))?;

            instruction
                .try_set_op_kind(operand_index, OpKind::NearBranch16)
                .map_err(|instruction_error| instruction_error.to_string())?;
            instruction.set_near_branch16(branch_target);

            Ok(())
        }
        OpCodeOperandKind::br32_1 | OpCodeOperandKind::br32_4 | OpCodeOperandKind::xbegin_4 | OpCodeOperandKind::brdisp_4 => {
            let branch_target = u32::try_from(label_target_address)
                .map_err(|_| format!("Label '{}' target '{}' does not fit a 32-bit branch operand.", label_name, label_target_address))?;

            instruction
                .try_set_op_kind(operand_index, OpKind::NearBranch32)
                .map_err(|instruction_error| instruction_error.to_string())?;
            instruction.set_near_branch32(branch_target);

            Ok(())
        }
        OpCodeOperandKind::br64_1 | OpCodeOperandKind::br64_4 => {
            instruction
                .try_set_op_kind(operand_index, OpKind::NearBranch64)
                .map_err(|instruction_error| instruction_error.to_string())?;
            instruction.set_near_branch64(label_target_address);

            Ok(())
        }
        _ => Err(format!("Label '{}' does not match operand kind {:?}.", label_name, operand_kind)),
    }
}

fn has_previous_immediate_operand(
    instruction: &Instruction,
    operand_index: u32,
) -> bool {
    (0..operand_index).any(|previous_operand_index| {
        matches!(
            instruction.op_kind(previous_operand_index),
            OpKind::Immediate8
                | OpKind::Immediate8_2nd
                | OpKind::Immediate16
                | OpKind::Immediate32
                | OpKind::Immediate64
                | OpKind::Immediate8to16
                | OpKind::Immediate8to32
                | OpKind::Immediate8to64
                | OpKind::Immediate32to64
        )
    })
}

fn operand_kind_to_instruction_memory_kind(
    operand_kind: OpCodeOperandKind,
    instruction_mode: X86InstructionMode,
) -> Option<OpKind> {
    match operand_kind {
        OpCodeOperandKind::mem
        | OpCodeOperandKind::mem_offs
        | OpCodeOperandKind::mem_mpx
        | OpCodeOperandKind::mem_mib
        | OpCodeOperandKind::mem_vsib32x
        | OpCodeOperandKind::mem_vsib64x
        | OpCodeOperandKind::mem_vsib32y
        | OpCodeOperandKind::mem_vsib64y
        | OpCodeOperandKind::mem_vsib32z
        | OpCodeOperandKind::mem_vsib64z
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
        | OpCodeOperandKind::k_or_mem => Some(OpKind::Memory),
        OpCodeOperandKind::seg_rSI => Some(match instruction_mode {
            X86InstructionMode::Bit32 => OpKind::MemorySegESI,
            X86InstructionMode::Bit64 => OpKind::MemorySegRSI,
        }),
        OpCodeOperandKind::es_rDI => Some(match instruction_mode {
            X86InstructionMode::Bit32 => OpKind::MemoryESEDI,
            X86InstructionMode::Bit64 => OpKind::MemoryESRDI,
        }),
        OpCodeOperandKind::seg_rDI => Some(match instruction_mode {
            X86InstructionMode::Bit32 => OpKind::MemorySegEDI,
            X86InstructionMode::Bit64 => OpKind::MemorySegRDI,
        }),
        _ => None,
    }
}

fn immediate_fits_operand_kind(
    immediate_value: i128,
    operand_kind: OpCodeOperandKind,
) -> bool {
    match operand_kind {
        OpCodeOperandKind::imm4_m2z => (0..=0xF).contains(&immediate_value),
        OpCodeOperandKind::imm8 => u8::try_from(immediate_value).is_ok(),
        OpCodeOperandKind::imm8_const_1 => immediate_value == 1,
        OpCodeOperandKind::imm8sex16 | OpCodeOperandKind::imm8sex32 | OpCodeOperandKind::imm8sex64 => i8::try_from(immediate_value).is_ok(),
        OpCodeOperandKind::imm16 => u16::try_from(immediate_value).is_ok(),
        OpCodeOperandKind::imm32 => u32::try_from(immediate_value).is_ok(),
        OpCodeOperandKind::imm32sex64 => i32::try_from(immediate_value).is_ok(),
        OpCodeOperandKind::imm64 => u64::try_from(immediate_value).is_ok(),
        OpCodeOperandKind::br16_1 | OpCodeOperandKind::br16_2 | OpCodeOperandKind::xbegin_2 | OpCodeOperandKind::brdisp_2 => {
            u16::try_from(immediate_value).is_ok()
        }
        OpCodeOperandKind::br32_1 | OpCodeOperandKind::br32_4 | OpCodeOperandKind::xbegin_4 | OpCodeOperandKind::brdisp_4 => {
            u32::try_from(immediate_value).is_ok()
        }
        OpCodeOperandKind::br64_1 | OpCodeOperandKind::br64_4 => u64::try_from(immediate_value).is_ok(),
        _ => false,
    }
}

fn immediate_operand_specificity_score(operand_kind: OpCodeOperandKind) -> u32 {
    match operand_kind {
        OpCodeOperandKind::imm8_const_1 => 60,
        OpCodeOperandKind::imm4_m2z => 55,
        OpCodeOperandKind::imm8 | OpCodeOperandKind::imm8sex16 | OpCodeOperandKind::imm8sex32 | OpCodeOperandKind::imm8sex64 => 50,
        OpCodeOperandKind::imm16 => 45,
        OpCodeOperandKind::imm32 | OpCodeOperandKind::imm32sex64 => 40,
        OpCodeOperandKind::imm64 => 35,
        OpCodeOperandKind::br16_1 | OpCodeOperandKind::br32_1 | OpCodeOperandKind::br64_1 => 60,
        OpCodeOperandKind::br16_2 | OpCodeOperandKind::xbegin_2 | OpCodeOperandKind::brdisp_2 => 50,
        OpCodeOperandKind::br32_4 | OpCodeOperandKind::br64_4 | OpCodeOperandKind::xbegin_4 | OpCodeOperandKind::brdisp_4 => 40,
        _ => 0,
    }
}

fn branch_operand_specificity_score(operand_kind: OpCodeOperandKind) -> u32 {
    immediate_operand_specificity_score(operand_kind)
}

fn branch_operand_kind_accepts_label(operand_kind: OpCodeOperandKind) -> bool {
    matches!(
        operand_kind,
        OpCodeOperandKind::br16_1
            | OpCodeOperandKind::br16_2
            | OpCodeOperandKind::br32_1
            | OpCodeOperandKind::br32_4
            | OpCodeOperandKind::br64_1
            | OpCodeOperandKind::br64_4
            | OpCodeOperandKind::xbegin_2
            | OpCodeOperandKind::xbegin_4
            | OpCodeOperandKind::brdisp_2
            | OpCodeOperandKind::brdisp_4
    )
}

fn branch_operand_matches_instruction_mode(
    operand_kind: OpCodeOperandKind,
    instruction_mode: X86InstructionMode,
) -> bool {
    match instruction_mode {
        X86InstructionMode::Bit32 => matches!(
            operand_kind,
            OpCodeOperandKind::br32_1 | OpCodeOperandKind::br32_4 | OpCodeOperandKind::xbegin_4 | OpCodeOperandKind::brdisp_4
        ),
        X86InstructionMode::Bit64 => matches!(
            operand_kind,
            OpCodeOperandKind::br64_1 | OpCodeOperandKind::br64_4 | OpCodeOperandKind::xbegin_4 | OpCodeOperandKind::brdisp_4
        ),
    }
}

fn instruction_decorators_match_candidate(
    instruction_decorators: &InstructionDecorators,
    candidate_code: Code,
) -> bool {
    let opcode_info = candidate_code.op_code();

    (!instruction_decorators.zeroing_masking() || opcode_info.can_use_zeroing_masking())
        && (instruction_decorators.op_mask_register_name().is_none() || opcode_info.can_use_op_mask_register())
        && (!instruction_decorators.suppress_all_exceptions() || opcode_info.can_suppress_all_exceptions())
        && (instruction_decorators.rounding_control().is_none() || opcode_info.can_use_rounding_control())
}

fn apply_instruction_decorators(
    instruction: &mut Instruction,
    instruction_decorators: &InstructionDecorators,
    candidate_code: Code,
) -> Result<(), String> {
    let opcode_info = candidate_code.op_code();

    if let Some(op_mask_register_name) = instruction_decorators.op_mask_register_name() {
        if !opcode_info.can_use_op_mask_register() {
            return Err(format!(
                "Instruction '{}' does not support an opmask register.",
                candidate_code.op_code().instruction_string()
            ));
        }

        let op_mask_register = parse_register(op_mask_register_name)
            .filter(|parsed_register| {
                matches!(
                    parsed_register,
                    Register::K1 | Register::K2 | Register::K3 | Register::K4 | Register::K5 | Register::K6 | Register::K7
                )
            })
            .ok_or_else(|| format!("Unsupported opmask register '{}'.", op_mask_register_name))?;

        instruction.set_op_mask(op_mask_register);
    }

    if instruction_decorators.zeroing_masking() {
        if instruction.op_mask() == Register::None {
            return Err(String::from("Zeroing masking requires an opmask register."));
        }

        if !opcode_info.can_use_zeroing_masking() {
            return Err(format!(
                "Instruction '{}' does not support zeroing masking.",
                candidate_code.op_code().instruction_string()
            ));
        }

        instruction.set_zeroing_masking(true);
    }

    if instruction_decorators.suppress_all_exceptions() {
        if !opcode_info.can_suppress_all_exceptions() {
            return Err(format!("Instruction '{}' does not support SAE.", candidate_code.op_code().instruction_string()));
        }

        instruction.set_suppress_all_exceptions(true);
    }

    if let Some(rounding_control) = instruction_decorators.rounding_control() {
        if !opcode_info.can_use_rounding_control() {
            return Err(format!(
                "Instruction '{}' does not support rounding control.",
                candidate_code.op_code().instruction_string()
            ));
        }

        instruction.set_rounding_control(map_rounding_control(rounding_control));
    }

    Ok(())
}

fn map_rounding_control(rounding_control: InstructionRoundingControl) -> RoundingControl {
    match rounding_control {
        InstructionRoundingControl::RoundToNearest => RoundingControl::RoundToNearest,
        InstructionRoundingControl::RoundDown => RoundingControl::RoundDown,
        InstructionRoundingControl::RoundUp => RoundingControl::RoundUp,
        InstructionRoundingControl::RoundTowardZero => RoundingControl::RoundTowardZero,
    }
}

fn memory_size_hint_matches_candidate(
    explicit_memory_size: Option<InstructionMemoryOperandSize>,
    candidate_code: Code,
) -> bool {
    let Some(explicit_memory_size) = explicit_memory_size else {
        return true;
    };
    let Some(explicit_size_in_bytes) = explicit_memory_size.size_in_bytes() else {
        return true;
    };
    let candidate_size_in_bytes = candidate_code.op_code().memory_size().info().size();

    let memory_size_matches = candidate_size_in_bytes == 0 || candidate_size_in_bytes == explicit_size_in_bytes;
    memory_size_matches
}

fn default_implicit_memory_operand_size(instruction_mode: X86InstructionMode) -> usize {
    match instruction_mode {
        X86InstructionMode::Bit32 => 4,
        X86InstructionMode::Bit64 => 8,
    }
}
