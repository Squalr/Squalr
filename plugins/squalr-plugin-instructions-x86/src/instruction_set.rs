use crate::x86_operand_lowering::build_candidate_instructions;
use iced_x86::{Decoder, DecoderOptions, Encoder, FlowControl, Formatter, NasmFormatter};
use squalr_engine_api::{
    plugins::instruction_set::{InstructionSet, ParsedInstruction, normalize_instruction_text, parse_instruction_sequence},
    structures::memory::bitness::Bitness,
};
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
    instruction_bitness: Bitness,
}

#[derive(Clone, Debug)]
enum AssembledSequenceItem {
    EncodedInstruction {
        parsed_instruction: ParsedInstruction,
        instruction: iced_x86::Instruction,
    },
    DataDirective {
        bytes: Vec<u8>,
    },
}

impl X86InstructionSetBase {
    fn new(
        instruction_set_id: &'static str,
        display_name: &'static str,
        instruction_bitness: Bitness,
    ) -> Self {
        Self {
            instruction_set_id,
            display_name,
            instruction_bitness,
        }
    }

    fn assemble_instruction_sequence(
        &self,
        assembly_source: &str,
    ) -> Result<Vec<u8>, String> {
        let parsed_instruction_sequence = parse_instruction_sequence(assembly_source).map_err(|instruction_error| instruction_error.to_string())?;
        let parsed_instructions = parsed_instruction_sequence.instructions();
        let label_instruction_indices = parsed_instruction_sequence.label_instruction_indices();
        let mut item_lengths = parsed_instructions
            .iter()
            .map(|parsed_instruction| {
                try_build_data_directive_bytes(parsed_instruction)
                    .map(|optional_directive_bytes| optional_directive_bytes.map_or(1usize, |directive_bytes| directive_bytes.len()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut assembled_sequence_items = Vec::new();

        for _pass_index in 0..16 {
            let label_addresses = build_label_addresses(label_instruction_indices, &item_lengths)?;
            let mut current_ip = 0u64;
            let mut next_assembled_sequence_items = Vec::with_capacity(parsed_instructions.len());
            let mut next_item_lengths = Vec::with_capacity(parsed_instructions.len());

            for parsed_instruction in parsed_instructions {
                if let Some(directive_bytes) = try_build_data_directive_bytes(parsed_instruction)? {
                    current_ip = current_ip.saturating_add(directive_bytes.len() as u64);
                    next_item_lengths.push(directive_bytes.len());
                    next_assembled_sequence_items.push(AssembledSequenceItem::DataDirective { bytes: directive_bytes });
                    continue;
                }

                let candidate_instructions = build_candidate_instructions(parsed_instruction, self.instruction_bitness, self.display_name, &label_addresses)?;
                let (selected_instruction, selected_instruction_length) = select_encodable_instruction(
                    &candidate_instructions,
                    self.instruction_bitness,
                    current_ip,
                    parsed_instruction,
                    self.display_name,
                )?;

                current_ip = current_ip.saturating_add(selected_instruction_length as u64);
                next_item_lengths.push(selected_instruction_length);
                next_assembled_sequence_items.push(AssembledSequenceItem::EncodedInstruction {
                    parsed_instruction: parsed_instruction.clone(),
                    instruction: selected_instruction,
                });
            }

            let lengths_stabilized = next_item_lengths == item_lengths;
            item_lengths = next_item_lengths;
            assembled_sequence_items = next_assembled_sequence_items;

            if lengths_stabilized {
                break;
            }
        }

        if assembled_sequence_items.len() != parsed_instructions.len() {
            return Err(format!(
                "Failed to stabilize {} instruction layout for '{}'.",
                self.display_name, assembly_source
            ));
        }

        let mut instruction_encoder = Encoder::new(bitness_as_u32(self.instruction_bitness));
        let mut current_ip = 0u64;
        let mut assembled_bytes = Vec::new();

        for assembled_sequence_item in &assembled_sequence_items {
            match assembled_sequence_item {
                AssembledSequenceItem::EncodedInstruction {
                    parsed_instruction,
                    instruction,
                } => {
                    let instruction_length = instruction_encoder
                        .encode(instruction, current_ip)
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
                AssembledSequenceItem::DataDirective { bytes } => {
                    assembled_bytes.extend(instruction_encoder.take_buffer());
                    assembled_bytes.extend(bytes);
                    current_ip = current_ip.saturating_add(bytes.len() as u64);
                }
            }
        }

        assembled_bytes.extend(instruction_encoder.take_buffer());

        Ok(assembled_bytes)
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
                bitness_as_u32(self.instruction_bitness),
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
            inner: X86InstructionSetBase::new("x86", "x86", Bitness::Bit32),
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

    fn build_no_operation_fill(
        &self,
        byte_count: usize,
    ) -> Result<Vec<u8>, String> {
        Ok(vec![0x90; byte_count])
    }
}

#[derive(Debug)]
pub struct X64InstructionSet {
    inner: X86InstructionSetBase,
}

impl X64InstructionSet {
    pub fn new() -> Self {
        Self {
            inner: X86InstructionSetBase::new("x64", "x64", Bitness::Bit64),
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
    instruction_bitness: Bitness,
    current_ip: u64,
    parsed_instruction: &ParsedInstruction,
    display_name: &str,
) -> Result<(iced_x86::Instruction, usize), String> {
    let prefer_shortest_encoding = matches!(parsed_instruction.mnemonic(), "ret" | "retn" | "retf");
    let mut shortest_candidate_instruction = None;
    let mut candidate_errors = Vec::new();

    for candidate_instruction in candidate_instructions {
        let mut probe_encoder = Encoder::new(bitness_as_u32(instruction_bitness));

        match probe_encoder.encode(candidate_instruction, current_ip) {
            Ok(instruction_length) => {
                if !prefer_shortest_encoding {
                    return Ok((*candidate_instruction, instruction_length));
                }

                if shortest_candidate_instruction
                    .as_ref()
                    .map(|(_instruction, selected_instruction_length)| instruction_length < *selected_instruction_length)
                    .unwrap_or(true)
                {
                    shortest_candidate_instruction = Some((*candidate_instruction, instruction_length));
                }
            }
            Err(candidate_error) => candidate_errors.push(candidate_error.to_string()),
        }
    }

    if let Some(shortest_candidate_instruction) = shortest_candidate_instruction {
        return Ok(shortest_candidate_instruction);
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

fn try_build_data_directive_bytes(parsed_instruction: &ParsedInstruction) -> Result<Option<Vec<u8>>, String> {
    let directive_unit_size_in_bytes = match parsed_instruction.mnemonic() {
        "db" => 1usize,
        "dw" => 2usize,
        "dd" => 4usize,
        "dq" => 8usize,
        _ => return Ok(None),
    };

    let mut directive_bytes = Vec::new();

    for directive_operand in parsed_instruction.operands() {
        let directive_value = match directive_operand {
            squalr_engine_api::plugins::instruction_set::InstructionOperand::Immediate(immediate_value) => *immediate_value,
            _ => {
                return Err(format!(
                    "Failed to encode {} directive '{}'. Expected only immediate operands.",
                    parsed_instruction.mnemonic(),
                    parsed_instruction.source_text()
                ));
            }
        };

        directive_bytes.extend(encode_data_directive_value(
            parsed_instruction.mnemonic(),
            parsed_instruction.source_text(),
            directive_value,
            directive_unit_size_in_bytes,
        )?);
    }

    if directive_bytes.is_empty() {
        return Err(format!(
            "Failed to encode {} directive '{}'. Expected at least one operand.",
            parsed_instruction.mnemonic(),
            parsed_instruction.source_text()
        ));
    }

    Ok(Some(directive_bytes))
}

fn encode_data_directive_value(
    directive_mnemonic: &str,
    source_text: &str,
    directive_value: i128,
    directive_unit_size_in_bytes: usize,
) -> Result<Vec<u8>, String> {
    match directive_unit_size_in_bytes {
        1 => {
            let encoded_value = u8::try_from(directive_value).map_err(|_| {
                format!(
                    "Failed to encode {} directive '{}'. Immediate {} is out of range for one byte.",
                    directive_mnemonic, source_text, directive_value
                )
            })?;

            Ok(vec![encoded_value])
        }
        2 => {
            let encoded_value = u16::try_from(directive_value).map_err(|_| {
                format!(
                    "Failed to encode {} directive '{}'. Immediate {} is out of range for two bytes.",
                    directive_mnemonic, source_text, directive_value
                )
            })?;

            Ok(encoded_value.to_le_bytes().to_vec())
        }
        4 => {
            let encoded_value = u32::try_from(directive_value).map_err(|_| {
                format!(
                    "Failed to encode {} directive '{}'. Immediate {} is out of range for four bytes.",
                    directive_mnemonic, source_text, directive_value
                )
            })?;

            Ok(encoded_value.to_le_bytes().to_vec())
        }
        8 => {
            let encoded_value = u64::try_from(directive_value).map_err(|_| {
                format!(
                    "Failed to encode {} directive '{}'. Immediate {} is out of range for eight bytes.",
                    directive_mnemonic, source_text, directive_value
                )
            })?;

            Ok(encoded_value.to_le_bytes().to_vec())
        }
        _ => Err(format!(
            "Failed to encode {} directive '{}'. Unsupported directive unit size {}.",
            directive_mnemonic, source_text, directive_unit_size_in_bytes
        )),
    }
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

    fn build_no_operation_fill(
        &self,
        byte_count: usize,
    ) -> Result<Vec<u8>, String> {
        Ok(vec![0x90; byte_count])
    }
}

fn bitness_as_u32(instruction_bitness: Bitness) -> u32 {
    match instruction_bitness {
        Bitness::Bit32 => 32,
        Bitness::Bit64 => 64,
    }
}
