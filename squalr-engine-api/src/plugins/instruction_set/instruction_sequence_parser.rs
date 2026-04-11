use crate::plugins::instruction_set::{
    InstructionDecorators, InstructionMemoryOperand, InstructionMemoryOperandSize, InstructionOperand, InstructionRoundingControl, InstructionSyntaxError,
    ParsedInstruction, ParsedInstructionSequence,
};
use std::collections::HashMap;

pub fn parse_instruction_sequence(assembly_source: &str) -> Result<ParsedInstructionSequence, InstructionSyntaxError> {
    let mut parsed_instructions = Vec::new();
    let mut label_instruction_indices = HashMap::new();

    for instruction_text in split_instruction_sequence(assembly_source) {
        let (label_names, remaining_instruction_text) = extract_leading_labels(instruction_text)?;

        for label_name in label_names {
            let previous_label = label_instruction_indices.insert(label_name.clone(), parsed_instructions.len());

            if previous_label.is_some() {
                return Err(InstructionSyntaxError::new(format!(
                    "Instruction label '{}' is defined more than once.",
                    label_name
                )));
            }
        }

        if !remaining_instruction_text.is_empty() {
            parsed_instructions.push(parse_instruction(remaining_instruction_text)?);
        }
    }

    if parsed_instructions.is_empty() {
        return Err(InstructionSyntaxError::new("Assembly source must not be empty."));
    }

    Ok(ParsedInstructionSequence::new(parsed_instructions, label_instruction_indices))
}

pub fn normalize_instruction_text(instruction_text: &str) -> String {
    let trimmed_instruction_text = instruction_text.trim();
    let mut normalized_instruction_text = String::with_capacity(trimmed_instruction_text.len() + 4);
    let mut previous_character = '\0';

    for current_character in trimmed_instruction_text.chars() {
        normalized_instruction_text.push(current_character);

        if current_character == ',' && previous_character != ' ' {
            normalized_instruction_text.push(' ');
        }

        previous_character = current_character;
    }

    normalized_instruction_text
}

fn split_instruction_sequence(assembly_source: &str) -> Vec<&str> {
    let mut instruction_texts = Vec::new();
    let mut instruction_start_index = 0usize;
    let mut bracket_depth = 0u32;

    for (character_index, current_character) in assembly_source.char_indices() {
        match current_character {
            '[' => bracket_depth = bracket_depth.saturating_add(1),
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            ';' | '\n' | '\r' if bracket_depth == 0 => {
                let instruction_text = assembly_source[instruction_start_index..character_index].trim();

                if !instruction_text.is_empty() {
                    instruction_texts.push(instruction_text);
                }

                instruction_start_index = character_index + current_character.len_utf8();
            }
            _ => {}
        }
    }

    let trailing_instruction_text = assembly_source[instruction_start_index..].trim();

    if !trailing_instruction_text.is_empty() {
        instruction_texts.push(trailing_instruction_text);
    }

    instruction_texts
}

fn parse_instruction(instruction_text: &str) -> Result<ParsedInstruction, InstructionSyntaxError> {
    let trimmed_instruction_text = instruction_text.trim();

    if trimmed_instruction_text.is_empty() {
        return Err(InstructionSyntaxError::new("Instruction text must not be empty."));
    }

    let (mnemonic, operands_text) = split_mnemonic_and_operands(trimmed_instruction_text);
    let (parsed_operands, instruction_decorators) = parse_operands(operands_text)?;

    Ok(ParsedInstruction::new(
        mnemonic,
        parsed_operands,
        instruction_decorators,
        normalize_instruction_text(trimmed_instruction_text),
    ))
}

fn extract_leading_labels(instruction_text: &str) -> Result<(Vec<String>, &str), InstructionSyntaxError> {
    let mut remaining_instruction_text = instruction_text.trim();
    let mut label_names = Vec::new();

    loop {
        let Some(label_separator_index) = find_label_separator(remaining_instruction_text) else {
            break;
        };
        let label_text = remaining_instruction_text[..label_separator_index].trim();

        if !is_valid_instruction_label(label_text) {
            break;
        }

        label_names.push(label_text.to_ascii_lowercase());
        remaining_instruction_text = remaining_instruction_text[label_separator_index + 1..].trim_start();
    }

    Ok((label_names, remaining_instruction_text))
}

fn find_label_separator(instruction_text: &str) -> Option<usize> {
    let mut bracket_depth = 0u32;

    for (character_index, current_character) in instruction_text.char_indices() {
        match current_character {
            '[' => bracket_depth = bracket_depth.saturating_add(1),
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            ':' if bracket_depth == 0 => return Some(character_index),
            _ => {}
        }
    }

    None
}

fn is_valid_instruction_label(label_text: &str) -> bool {
    let mut label_characters = label_text.chars();
    let Some(first_character) = label_characters.next() else {
        return false;
    };

    if !matches!(first_character, 'a'..='z' | 'A'..='Z' | '_' | '.' | '$' | '?' | '@') {
        return false;
    }

    label_characters.all(|label_character| matches!(label_character, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.' | '$' | '?' | '@'))
}

fn split_mnemonic_and_operands(instruction_text: &str) -> (String, &str) {
    if let Some(mnemonic_end_index) = instruction_text.find(char::is_whitespace) {
        (
            instruction_text[..mnemonic_end_index]
                .trim()
                .to_ascii_lowercase(),
            instruction_text[mnemonic_end_index..].trim(),
        )
    } else {
        (instruction_text.to_ascii_lowercase(), "")
    }
}

fn parse_operands(operands_text: &str) -> Result<(Vec<InstructionOperand>, InstructionDecorators), InstructionSyntaxError> {
    let trimmed_operands_text = operands_text.trim();

    if trimmed_operands_text.is_empty() {
        return Ok((Vec::new(), InstructionDecorators::default()));
    }

    let mut operands = Vec::new();
    let mut instruction_decorators = InstructionDecorators::default();
    let mut operand_start_index = 0usize;
    let mut bracket_depth = 0u32;

    for (character_index, current_character) in trimmed_operands_text.char_indices() {
        match current_character {
            '[' => bracket_depth = bracket_depth.saturating_add(1),
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            ',' if bracket_depth == 0 => {
                let operand_text = trimmed_operands_text[operand_start_index..character_index].trim();
                let (parsed_operand, operand_decorators) = parse_operand(operand_text)?;
                merge_instruction_decorators(&mut instruction_decorators, operand_decorators, operand_text)?;
                if let Some(parsed_operand) = parsed_operand {
                    operands.push(parsed_operand);
                }
                operand_start_index = character_index + 1;
            }
            _ => {}
        }
    }

    let trailing_operand_text = trimmed_operands_text[operand_start_index..].trim();
    let (parsed_operand, operand_decorators) = parse_operand(trailing_operand_text)?;
    merge_instruction_decorators(&mut instruction_decorators, operand_decorators, trailing_operand_text)?;
    if let Some(parsed_operand) = parsed_operand {
        operands.push(parsed_operand);
    }

    Ok((operands, instruction_decorators))
}

fn parse_operand(operand_text: &str) -> Result<(Option<InstructionOperand>, InstructionDecorators), InstructionSyntaxError> {
    let trimmed_operand_text = operand_text.trim();

    if trimmed_operand_text.is_empty() {
        return Err(InstructionSyntaxError::new("Instruction operand must not be empty."));
    }

    let (operand_core_text, decorator_texts) = split_operand_core_and_decorators(trimmed_operand_text)?;
    let (memory_operand_broadcast, instruction_decorators) = parse_operand_decorators(&decorator_texts, operand_text)?;

    if operand_core_text.is_empty() {
        if memory_operand_broadcast {
            return Err(InstructionSyntaxError::new(format!(
                "Instruction operand '{}' cannot use a broadcast decorator without a memory operand.",
                operand_text
            )));
        }

        return Ok((None, instruction_decorators));
    }

    if let Some(memory_operand) = parse_memory_operand(operand_core_text, memory_operand_broadcast)? {
        return Ok((Some(InstructionOperand::Memory(memory_operand)), instruction_decorators));
    }

    if let Some(immediate_value) = parse_immediate(operand_core_text)? {
        return Ok((Some(InstructionOperand::Immediate(immediate_value)), instruction_decorators));
    }

    Ok((
        Some(InstructionOperand::Identifier(operand_core_text.to_ascii_lowercase())),
        instruction_decorators,
    ))
}

fn parse_memory_operand(
    operand_text: &str,
    is_broadcast: bool,
) -> Result<Option<InstructionMemoryOperand>, InstructionSyntaxError> {
    let lowercase_operand_text = operand_text.to_ascii_lowercase();
    let (size, memory_operand_text) = parse_memory_operand_size_prefix(&lowercase_operand_text, operand_text);
    let trimmed_memory_operand_text = memory_operand_text.trim();

    if !trimmed_memory_operand_text.contains('[') && !trimmed_memory_operand_text.contains(']') {
        return Ok(None);
    }

    let Some(open_bracket_index) = trimmed_memory_operand_text.find('[') else {
        return Err(InstructionSyntaxError::new(format!("Memory operand '{}' is missing '['.", operand_text)));
    };
    let Some(close_bracket_index) = trimmed_memory_operand_text.rfind(']') else {
        return Err(InstructionSyntaxError::new(format!("Memory operand '{}' is missing ']'.", operand_text)));
    };

    if open_bracket_index > close_bracket_index {
        return Err(InstructionSyntaxError::new(format!(
            "Memory operand '{}' has malformed brackets.",
            operand_text
        )));
    }

    let prefix_text = trimmed_memory_operand_text[..open_bracket_index].trim();
    let suffix_text = trimmed_memory_operand_text[close_bracket_index + 1..].trim();

    let segment_override = if prefix_text.is_empty() {
        None
    } else if let Some(segment_override) = prefix_text.strip_suffix(':') {
        let trimmed_segment_override = segment_override.trim();

        if trimmed_segment_override.is_empty() {
            return Err(InstructionSyntaxError::new(format!(
                "Memory operand '{}' has an empty segment override.",
                operand_text
            )));
        }

        Some(trimmed_segment_override.to_ascii_lowercase())
    } else {
        return Err(InstructionSyntaxError::new(format!(
            "Memory operand '{}' has unsupported text outside brackets.",
            operand_text
        )));
    };
    let is_broadcast = if suffix_text.is_empty() {
        is_broadcast
    } else if is_broadcast_suffix(suffix_text) {
        true
    } else {
        return Err(InstructionSyntaxError::new(format!(
            "Memory operand '{}' has unsupported suffix '{}'.",
            operand_text, suffix_text
        )));
    };

    let expression_text = trimmed_memory_operand_text[open_bracket_index + 1..close_bracket_index].trim();

    if expression_text.is_empty() {
        return Err(InstructionSyntaxError::new(format!("Memory operand '{}' must not be empty.", operand_text)));
    }

    Ok(Some(InstructionMemoryOperand::with_metadata(
        size,
        segment_override,
        expression_text,
        is_broadcast,
    )))
}

fn split_operand_core_and_decorators<'a>(operand_text: &'a str) -> Result<(&'a str, Vec<&'a str>), InstructionSyntaxError> {
    let mut remaining_operand_text = operand_text.trim_end();
    let mut decorator_texts = Vec::new();

    while remaining_operand_text.ends_with('}') {
        let Some(decorator_start_index) = remaining_operand_text.rfind('{') else {
            return Err(InstructionSyntaxError::new(format!(
                "Operand '{}' has a closing decorator brace without an opening brace.",
                operand_text
            )));
        };
        let decorator_text = remaining_operand_text[decorator_start_index + 1..remaining_operand_text.len() - 1].trim();

        if decorator_text.is_empty() {
            return Err(InstructionSyntaxError::new(format!("Operand '{}' contains an empty decorator.", operand_text)));
        }

        decorator_texts.push(decorator_text);
        remaining_operand_text = remaining_operand_text[..decorator_start_index].trim_end();
    }

    decorator_texts.reverse();

    Ok((remaining_operand_text.trim(), decorator_texts))
}

fn parse_operand_decorators(
    decorator_texts: &[&str],
    operand_text: &str,
) -> Result<(bool, InstructionDecorators), InstructionSyntaxError> {
    let mut is_broadcast = false;
    let mut op_mask_register_name = None;
    let mut zeroing_masking = false;
    let mut suppress_all_exceptions = false;
    let mut rounding_control = None;

    for decorator_text in decorator_texts {
        let lowercase_decorator_text = decorator_text.trim().to_ascii_lowercase();

        if is_broadcast_decorator(&lowercase_decorator_text) {
            if is_broadcast {
                return Err(InstructionSyntaxError::new(format!(
                    "Operand '{}' repeats broadcast decorator '{{{}}}'.",
                    operand_text, decorator_text
                )));
            }

            is_broadcast = true;
            continue;
        }

        if matches!(lowercase_decorator_text.as_str(), "k1" | "k2" | "k3" | "k4" | "k5" | "k6" | "k7") {
            if op_mask_register_name.is_some() {
                return Err(InstructionSyntaxError::new(format!(
                    "Operand '{}' repeats opmask decorator '{{{}}}'.",
                    operand_text, decorator_text
                )));
            }

            op_mask_register_name = Some(lowercase_decorator_text);
            continue;
        }

        if lowercase_decorator_text == "z" {
            if zeroing_masking {
                return Err(InstructionSyntaxError::new(format!(
                    "Operand '{}' repeats zeroing decorator '{{{}}}'.",
                    operand_text, decorator_text
                )));
            }

            zeroing_masking = true;
            continue;
        }

        if lowercase_decorator_text == "sae" {
            if suppress_all_exceptions {
                return Err(InstructionSyntaxError::new(format!(
                    "Operand '{}' repeats SAE decorator '{{{}}}'.",
                    operand_text, decorator_text
                )));
            }

            suppress_all_exceptions = true;
            continue;
        }

        if let Some(parsed_rounding_control) = parse_rounding_control(&lowercase_decorator_text) {
            if rounding_control.is_some() {
                return Err(InstructionSyntaxError::new(format!(
                    "Operand '{}' repeats rounding decorator '{{{}}}'.",
                    operand_text, decorator_text
                )));
            }

            rounding_control = Some(parsed_rounding_control);
            continue;
        }

        return Err(InstructionSyntaxError::new(format!(
            "Operand '{}' has unsupported decorator '{{{}}}'.",
            operand_text, decorator_text
        )));
    }

    Ok((
        is_broadcast,
        InstructionDecorators::new(op_mask_register_name, zeroing_masking, suppress_all_exceptions, rounding_control),
    ))
}

fn parse_rounding_control(decorator_text: &str) -> Option<InstructionRoundingControl> {
    match decorator_text {
        "rn-sae" | "rn" => Some(InstructionRoundingControl::RoundToNearest),
        "rd-sae" | "rd" => Some(InstructionRoundingControl::RoundDown),
        "ru-sae" | "ru" => Some(InstructionRoundingControl::RoundUp),
        "rz-sae" | "rz" => Some(InstructionRoundingControl::RoundTowardZero),
        _ => None,
    }
}

fn merge_instruction_decorators(
    instruction_decorators: &mut InstructionDecorators,
    operand_decorators: InstructionDecorators,
    operand_text: &str,
) -> Result<(), InstructionSyntaxError> {
    if let Some(op_mask_register_name) = operand_decorators.op_mask_register_name() {
        if instruction_decorators.op_mask_register_name().is_some() {
            return Err(InstructionSyntaxError::new(format!(
                "Instruction '{}' repeats opmask decorators across operands.",
                operand_text
            )));
        }

        *instruction_decorators = InstructionDecorators::new(
            Some(op_mask_register_name),
            instruction_decorators.zeroing_masking(),
            instruction_decorators.suppress_all_exceptions(),
            instruction_decorators.rounding_control(),
        );
    }

    if operand_decorators.zeroing_masking() {
        if instruction_decorators.zeroing_masking() {
            return Err(InstructionSyntaxError::new(format!(
                "Instruction '{}' repeats zeroing decorators across operands.",
                operand_text
            )));
        }

        *instruction_decorators = InstructionDecorators::new(
            instruction_decorators.op_mask_register_name(),
            true,
            instruction_decorators.suppress_all_exceptions(),
            instruction_decorators.rounding_control(),
        );
    }

    if operand_decorators.suppress_all_exceptions() {
        if instruction_decorators.suppress_all_exceptions() {
            return Err(InstructionSyntaxError::new(format!(
                "Instruction '{}' repeats SAE decorators across operands.",
                operand_text
            )));
        }

        *instruction_decorators = InstructionDecorators::new(
            instruction_decorators.op_mask_register_name(),
            instruction_decorators.zeroing_masking(),
            true,
            instruction_decorators.rounding_control(),
        );
    }

    if let Some(rounding_control) = operand_decorators.rounding_control() {
        if instruction_decorators.rounding_control().is_some() {
            return Err(InstructionSyntaxError::new(format!(
                "Instruction '{}' repeats rounding decorators across operands.",
                operand_text
            )));
        }

        *instruction_decorators = InstructionDecorators::new(
            instruction_decorators.op_mask_register_name(),
            instruction_decorators.zeroing_masking(),
            instruction_decorators.suppress_all_exceptions(),
            Some(rounding_control),
        );
    }

    Ok(())
}

fn parse_memory_operand_size_prefix<'a>(
    lowercase_operand_text: &str,
    operand_text: &'a str,
) -> (Option<InstructionMemoryOperandSize>, &'a str) {
    const SIZE_PREFIXES: [(&str, InstructionMemoryOperandSize); 18] = [
        ("xmmword ptr ", InstructionMemoryOperandSize::Xmmword),
        ("ymmword ptr ", InstructionMemoryOperandSize::Ymmword),
        ("zmmword ptr ", InstructionMemoryOperandSize::Zmmword),
        ("oword ptr ", InstructionMemoryOperandSize::Xmmword),
        ("tbyte ptr ", InstructionMemoryOperandSize::Tbyte),
        ("fword ptr ", InstructionMemoryOperandSize::Fword),
        ("qword ptr ", InstructionMemoryOperandSize::Qword),
        ("dword ptr ", InstructionMemoryOperandSize::Dword),
        ("word ptr ", InstructionMemoryOperandSize::Word),
        ("byte ptr ", InstructionMemoryOperandSize::Byte),
        ("xmmword ", InstructionMemoryOperandSize::Xmmword),
        ("ymmword ", InstructionMemoryOperandSize::Ymmword),
        ("zmmword ", InstructionMemoryOperandSize::Zmmword),
        ("oword ", InstructionMemoryOperandSize::Xmmword),
        ("tbyte ", InstructionMemoryOperandSize::Tbyte),
        ("fword ", InstructionMemoryOperandSize::Fword),
        ("qword ", InstructionMemoryOperandSize::Qword),
        ("dword ", InstructionMemoryOperandSize::Dword),
    ];

    for (prefix_text, size) in SIZE_PREFIXES {
        if let Some(stripped_operand_text) = lowercase_operand_text.strip_prefix(prefix_text) {
            let operand_start_index = operand_text.len() - stripped_operand_text.len();

            return (Some(size), &operand_text[operand_start_index..]);
        }
    }

    if let Some(stripped_operand_text) = lowercase_operand_text.strip_prefix("word ") {
        let operand_start_index = operand_text.len() - stripped_operand_text.len();

        return (Some(InstructionMemoryOperandSize::Word), &operand_text[operand_start_index..]);
    }

    if let Some(stripped_operand_text) = lowercase_operand_text.strip_prefix("byte ") {
        let operand_start_index = operand_text.len() - stripped_operand_text.len();

        return (Some(InstructionMemoryOperandSize::Byte), &operand_text[operand_start_index..]);
    }

    (None, operand_text)
}

fn is_broadcast_suffix(suffix_text: &str) -> bool {
    let lowercase_suffix_text = suffix_text.to_ascii_lowercase();

    matches!(lowercase_suffix_text.as_str(), "{1to2}" | "{1to4}" | "{1to8}" | "{1to16}" | "{1to32}")
}

fn is_broadcast_decorator(decorator_text: &str) -> bool {
    matches!(decorator_text, "1to2" | "1to4" | "1to8" | "1to16" | "1to32")
}

fn parse_immediate(immediate_text: &str) -> Result<Option<i128>, InstructionSyntaxError> {
    let trimmed_immediate_text = immediate_text.trim();

    if trimmed_immediate_text.is_empty() {
        return Ok(None);
    }

    let unsigned_prefixed_immediate_text = trimmed_immediate_text
        .strip_prefix('#')
        .unwrap_or(trimmed_immediate_text);
    let (sign_multiplier, unsigned_immediate_text) = if let Some(stripped_immediate_text) = unsigned_prefixed_immediate_text.strip_prefix('-') {
        (-1_i128, stripped_immediate_text)
    } else if let Some(stripped_immediate_text) = unsigned_prefixed_immediate_text.strip_prefix('+') {
        (1_i128, stripped_immediate_text)
    } else {
        (1_i128, unsigned_prefixed_immediate_text)
    };

    if unsigned_immediate_text.is_empty() {
        return Err(InstructionSyntaxError::new(format!("Immediate '{}' is missing digits.", immediate_text)));
    }

    let parsed_value = if let Some(hexadecimal_digits) = unsigned_immediate_text
        .strip_prefix("0x")
        .or_else(|| unsigned_immediate_text.strip_prefix("0X"))
    {
        if hexadecimal_digits.is_empty() {
            return Err(InstructionSyntaxError::new(format!(
                "Immediate '{}' is missing hexadecimal digits.",
                immediate_text
            )));
        }

        match i128::from_str_radix(hexadecimal_digits, 16) {
            Ok(parsed_value) => Some(parsed_value.saturating_mul(sign_multiplier)),
            Err(_) => None,
        }
    } else {
        match unsigned_immediate_text.parse::<i128>() {
            Ok(parsed_value) => Some(parsed_value.saturating_mul(sign_multiplier)),
            Err(_) => None,
        }
    };

    Ok(parsed_value)
}

#[cfg(test)]
mod tests {
    use crate::plugins::instruction_set::{
        InstructionDecorators, InstructionMemoryOperand, InstructionMemoryOperandSize, InstructionOperand, InstructionRoundingControl,
        parse_instruction_sequence,
    };

    #[test]
    fn parse_instruction_sequence_supports_semicolon_and_newline_separators() {
        let parsed_instructions = parse_instruction_sequence("mov eax, 5; inc eax\nret").expect("Expected instruction sequence to parse.");

        assert_eq!(parsed_instructions.instructions().len(), 3);
        assert_eq!(parsed_instructions.instructions()[0].mnemonic(), "mov");
        assert_eq!(parsed_instructions.instructions()[1].mnemonic(), "inc");
        assert_eq!(parsed_instructions.instructions()[2].mnemonic(), "ret");
    }

    #[test]
    fn parse_instruction_sequence_parses_memory_operands_with_size_hints() {
        let parsed_instructions = parse_instruction_sequence("inc dword ptr [0x100579c]").expect("Expected memory operand to parse.");

        assert_eq!(
            parsed_instructions.instructions()[0].operands(),
            &[InstructionOperand::Memory(InstructionMemoryOperand::new(
                Some(InstructionMemoryOperandSize::Dword),
                "0x100579c"
            ))]
        );
    }

    #[test]
    fn parse_instruction_sequence_parses_immediate_operands() {
        let parsed_instructions = parse_instruction_sequence("mov eax, -5").expect("Expected immediate operand to parse.");

        assert_eq!(
            parsed_instructions.instructions()[0].operands(),
            &[
                InstructionOperand::Identifier(String::from("eax")),
                InstructionOperand::Immediate(-5)
            ]
        );
    }

    #[test]
    fn parse_instruction_sequence_parses_hash_prefixed_immediates() {
        let parsed_instructions = parse_instruction_sequence("mov x0, #5").expect("Expected hash-prefixed immediate operand to parse.");

        assert_eq!(
            parsed_instructions.instructions()[0].operands(),
            &[
                InstructionOperand::Identifier(String::from("x0")),
                InstructionOperand::Immediate(5)
            ]
        );
    }

    #[test]
    fn parse_instruction_sequence_parses_segmented_broadcast_memory_operands() {
        let parsed_instructions =
            parse_instruction_sequence("vaddps zmm0, zmm1, dword ptr fs:[rax+4]{1to16}").expect("Expected segmented broadcast memory operand to parse.");

        assert_eq!(
            parsed_instructions.instructions()[0].operands()[2],
            InstructionOperand::Memory(InstructionMemoryOperand::with_metadata(
                Some(InstructionMemoryOperandSize::Dword),
                Some("fs"),
                "rax+4",
                true
            ))
        );
    }

    #[test]
    fn parse_instruction_sequence_parses_avx512_register_decorators() {
        let parsed_instructions = parse_instruction_sequence("vaddps zmm1{k1}{z}, zmm2, zmm3").expect("Expected AVX-512 decorators to parse.");

        assert_eq!(
            parsed_instructions.instructions()[0].decorators(),
            &InstructionDecorators::new(Some("k1"), true, false, None)
        );
    }

    #[test]
    fn parse_instruction_sequence_parses_rounding_and_sae_decorators() {
        let parsed_instructions = parse_instruction_sequence("vsqrtps zmm1{k2}{z}, zmm23{rd-sae}").expect("Expected rounding decorators to parse.");

        assert_eq!(
            parsed_instructions.instructions()[0].decorators(),
            &InstructionDecorators::new(Some("k2"), true, false, Some(InstructionRoundingControl::RoundDown))
        );
    }

    #[test]
    fn parse_instruction_sequence_parses_sae_only_decorators() {
        let parsed_instructions = parse_instruction_sequence("vucomiss xmm31, xmm15{sae}").expect("Expected SAE decorator to parse.");

        assert_eq!(
            parsed_instructions.instructions()[0].decorators(),
            &InstructionDecorators::new(None::<String>, false, true, None)
        );
    }

    #[test]
    fn parse_instruction_sequence_parses_label_only_lines_and_inline_labels() {
        let parsed_instruction_sequence = parse_instruction_sequence("start:\nloop_label: inc eax\njne loop_label").expect("Expected label syntax to parse.");

        assert_eq!(parsed_instruction_sequence.instructions().len(), 2);
        assert_eq!(
            parsed_instruction_sequence
                .label_instruction_indices()
                .get("start"),
            Some(&0)
        );
        assert_eq!(
            parsed_instruction_sequence
                .label_instruction_indices()
                .get("loop_label"),
            Some(&0)
        );
        assert_eq!(parsed_instruction_sequence.instructions()[1].mnemonic(), "jne");
        assert_eq!(
            parsed_instruction_sequence.instructions()[1].operands(),
            &[InstructionOperand::Identifier(String::from("loop_label"))]
        );
    }
}
