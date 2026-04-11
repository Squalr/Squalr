use crate::{x86_memory_operand::X86InstructionMode, x86_operand_lowering::build_candidate_instructions};
use iced_x86::{Decoder, DecoderOptions, Encoder, FlowControl, Formatter, NasmFormatter};
use squalr_engine_api::plugins::instruction_set::{InstructionSet, ParsedInstruction, normalize_instruction_text, parse_instruction_sequence};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisassembledInstruction {
    pub address: u64,
    pub length: usize,
    pub bytes: Vec<u8>,
    pub text: String,
    pub branch_target_address: Option<u64>,
    pub is_control_flow: bool,
}

#[derive(Debug)]
struct X86InstructionSetBase {
    instruction_set_id: &'static str,
    display_name: &'static str,
    mode: X86InstructionMode,
}

impl X86InstructionSetBase {
    fn new(
        instruction_set_id: &'static str,
        display_name: &'static str,
        mode: X86InstructionMode,
    ) -> Self {
        Self {
            instruction_set_id,
            display_name,
            mode,
        }
    }

    fn assemble_instruction_sequence(
        &self,
        assembly_source: &str,
    ) -> Result<Vec<u8>, String> {
        let parsed_instruction_sequence = parse_instruction_sequence(assembly_source).map_err(|instruction_error| instruction_error.to_string())?;
        let parsed_instructions = parsed_instruction_sequence.instructions();
        let label_instruction_indices = parsed_instruction_sequence.label_instruction_indices();
        let mut instruction_lengths = vec![1usize; parsed_instructions.len()];
        let mut selected_instructions = Vec::new();

        for _pass_index in 0..16 {
            let label_addresses = build_label_addresses(label_instruction_indices, &instruction_lengths)?;
            let mut current_ip = 0u64;
            let mut next_selected_instructions = Vec::with_capacity(parsed_instructions.len());
            let mut next_instruction_lengths = Vec::with_capacity(parsed_instructions.len());

            for parsed_instruction in parsed_instructions {
                let candidate_instructions = build_candidate_instructions(parsed_instruction, self.mode, self.display_name, &label_addresses)?;
                let (selected_instruction, selected_instruction_length) =
                    select_encodable_instruction(&candidate_instructions, self.mode, current_ip, parsed_instruction, self.display_name)?;

                current_ip = current_ip.saturating_add(selected_instruction_length as u64);
                next_instruction_lengths.push(selected_instruction_length);
                next_selected_instructions.push(selected_instruction);
            }

            let lengths_stabilized = next_instruction_lengths == instruction_lengths;
            instruction_lengths = next_instruction_lengths;
            selected_instructions = next_selected_instructions;

            if lengths_stabilized {
                break;
            }
        }

        if selected_instructions.len() != parsed_instructions.len() {
            return Err(format!(
                "Failed to stabilize {} instruction layout for '{}'.",
                self.display_name, assembly_source
            ));
        }

        let mut encoder = Encoder::new(self.mode.bitness());
        let mut current_ip = 0u64;

        for (parsed_instruction, selected_instruction) in parsed_instructions.iter().zip(selected_instructions.iter()) {
            let instruction_length = encoder
                .encode(selected_instruction, current_ip)
                .map_err(|instruction_error| {
                    format!(
                        "Failed to encode {} instruction '{}': {}.",
                        self.display_name,
                        parsed_instruction.source_text(),
                        instruction_error
                    )
                })?;

            current_ip = current_ip.saturating_add(instruction_length as u64);
        }

        Ok(encoder.take_buffer())
    }

    fn disassemble_instruction_sequence(
        &self,
        instruction_bytes: &[u8],
    ) -> Result<String, String> {
        if instruction_bytes.is_empty() {
            return Ok(String::new());
        }

        let instruction_lines = self.disassemble_instruction_block(instruction_bytes, 0)?;
        let instruction_texts = instruction_lines
            .into_iter()
            .map(|instruction_line| instruction_line.text)
            .collect::<Vec<_>>();

        Ok(instruction_texts.join("; "))
    }

    fn disassemble_instruction_block(
        &self,
        instruction_bytes: &[u8],
        base_address: u64,
    ) -> Result<Vec<DisassembledInstruction>, String> {
        if instruction_bytes.is_empty() {
            return Ok(Vec::new());
        }

        let mut instruction_lines = Vec::new();
        let mut formatter = NasmFormatter::new();
        let mut byte_offset = 0usize;

        while byte_offset < instruction_bytes.len() {
            let instruction_address = base_address.saturating_add(byte_offset as u64);
            let mut decoder = Decoder::with_ip(
                self.mode.bitness(),
                &instruction_bytes[byte_offset..],
                instruction_address,
                DecoderOptions::NONE,
            );
            let instruction = decoder.decode();
            let instruction_length = instruction.len();

            if instruction.is_invalid() || instruction_length == 0 {
                instruction_lines.push(DisassembledInstruction {
                    address: instruction_address,
                    length: 1,
                    bytes: vec![instruction_bytes[byte_offset]],
                    text: format!("db 0x{:02X}", instruction_bytes[byte_offset]),
                    branch_target_address: None,
                    is_control_flow: false,
                });
                byte_offset += 1;
                continue;
            }

            let mut instruction_text = String::new();
            formatter.format(&instruction, &mut instruction_text);
            let branch_target_address = resolve_branch_target_address(&instruction);
            let is_control_flow = !matches!(instruction.flow_control(), FlowControl::Next);
            let instruction_end_offset = byte_offset
                .saturating_add(instruction_length)
                .min(instruction_bytes.len());

            instruction_lines.push(DisassembledInstruction {
                address: instruction_address,
                length: instruction_length,
                bytes: instruction_bytes[byte_offset..instruction_end_offset].to_vec(),
                text: normalize_instruction_text(&instruction_text),
                branch_target_address,
                is_control_flow,
            });
            byte_offset += instruction_length;
        }

        Ok(instruction_lines)
    }
}

fn resolve_branch_target_address(instruction: &iced_x86::Instruction) -> Option<u64> {
    match instruction.flow_control() {
        FlowControl::ConditionalBranch | FlowControl::UnconditionalBranch => Some(instruction.near_branch_target()),
        _ => None,
    }
}

#[derive(Debug)]
pub struct X86InstructionSet {
    inner: X86InstructionSetBase,
}

impl X86InstructionSet {
    pub fn new() -> Self {
        Self {
            inner: X86InstructionSetBase::new("x86", "x86", X86InstructionMode::Bit32),
        }
    }

    pub fn disassemble_block(
        &self,
        instruction_bytes: &[u8],
        base_address: u64,
    ) -> Result<Vec<DisassembledInstruction>, String> {
        self.inner
            .disassemble_instruction_block(instruction_bytes, base_address)
    }
}

impl Default for X86InstructionSet {
    fn default() -> Self {
        Self::new()
    }
}

impl InstructionSet for X86InstructionSet {
    fn get_instruction_set_id(&self) -> &str {
        self.inner.instruction_set_id
    }

    fn get_display_name(&self) -> &str {
        self.inner.display_name
    }

    fn assemble(
        &self,
        assembly_source: &str,
    ) -> Result<Vec<u8>, String> {
        self.inner.assemble_instruction_sequence(assembly_source)
    }

    fn disassemble(
        &self,
        instruction_bytes: &[u8],
    ) -> Result<String, String> {
        self.inner.disassemble_instruction_sequence(instruction_bytes)
    }
}

#[derive(Debug)]
pub struct X64InstructionSet {
    inner: X86InstructionSetBase,
}

impl X64InstructionSet {
    pub fn new() -> Self {
        Self {
            inner: X86InstructionSetBase::new("x64", "x64", X86InstructionMode::Bit64),
        }
    }

    pub fn disassemble_block(
        &self,
        instruction_bytes: &[u8],
        base_address: u64,
    ) -> Result<Vec<DisassembledInstruction>, String> {
        self.inner
            .disassemble_instruction_block(instruction_bytes, base_address)
    }
}

fn build_label_addresses(
    label_instruction_indices: &HashMap<String, usize>,
    instruction_lengths: &[usize],
) -> Result<HashMap<String, u64>, String> {
    let mut instruction_addresses = Vec::with_capacity(instruction_lengths.len() + 1);
    let mut current_ip = 0u64;
    instruction_addresses.push(current_ip);

    for instruction_length in instruction_lengths {
        current_ip = current_ip.saturating_add(*instruction_length as u64);
        instruction_addresses.push(current_ip);
    }

    let mut label_addresses = HashMap::new();

    for (label_name, instruction_index) in label_instruction_indices {
        let Some(label_address) = instruction_addresses.get(*instruction_index).copied() else {
            return Err(format!(
                "Instruction label '{}' points to invalid instruction index '{}'.",
                label_name, instruction_index
            ));
        };

        label_addresses.insert(label_name.clone(), label_address);
    }

    Ok(label_addresses)
}

fn select_encodable_instruction(
    candidate_instructions: &[iced_x86::Instruction],
    instruction_mode: X86InstructionMode,
    current_ip: u64,
    parsed_instruction: &ParsedInstruction,
    display_name: &str,
) -> Result<(iced_x86::Instruction, usize), String> {
    let mut candidate_errors = Vec::new();

    for candidate_instruction in candidate_instructions {
        let mut probe_encoder = Encoder::new(instruction_mode.bitness());

        match probe_encoder.encode(candidate_instruction, current_ip) {
            Ok(instruction_length) => return Ok((*candidate_instruction, instruction_length)),
            Err(candidate_error) => candidate_errors.push(candidate_error.to_string()),
        }
    }

    let candidate_error_summary = candidate_errors
        .into_iter()
        .take(3)
        .collect::<Vec<_>>()
        .join(" ");

    Err(format!(
        "Failed to encode {} instruction '{}'. {}",
        display_name,
        parsed_instruction.source_text(),
        candidate_error_summary
    ))
}

impl Default for X64InstructionSet {
    fn default() -> Self {
        Self::new()
    }
}

impl InstructionSet for X64InstructionSet {
    fn get_instruction_set_id(&self) -> &str {
        self.inner.instruction_set_id
    }

    fn get_display_name(&self) -> &str {
        self.inner.display_name
    }

    fn assemble(
        &self,
        assembly_source: &str,
    ) -> Result<Vec<u8>, String> {
        self.inner.assemble_instruction_sequence(assembly_source)
    }

    fn disassemble(
        &self,
        instruction_bytes: &[u8],
    ) -> Result<String, String> {
        self.inner.disassemble_instruction_sequence(instruction_bytes)
    }
}
