pub fn parse_powerpc_memory_operand(operand_text: &str) -> Result<(i16, String), String> {
    let trimmed_operand_text = operand_text.trim();
    let Some(open_paren_index) = trimmed_operand_text.find('(') else {
        return Err(format!("PowerPC memory operand '{}' is missing '(' .", operand_text));
    };
    let Some(close_paren_index) = trimmed_operand_text.rfind(')') else {
        return Err(format!("PowerPC memory operand '{}' is missing ')' .", operand_text));
    };

    if close_paren_index <= open_paren_index {
        return Err(format!("PowerPC memory operand '{}' is malformed.", operand_text));
    }

    let displacement_text = trimmed_operand_text[..open_paren_index].trim();
    let base_register_name = trimmed_operand_text[open_paren_index + 1..close_paren_index].trim();
    let displacement = if displacement_text.is_empty() {
        0
    } else if let Some(hexadecimal_digits) = displacement_text
        .strip_prefix("0x")
        .or_else(|| displacement_text.strip_prefix("0X"))
    {
        i16::from_str_radix(hexadecimal_digits, 16).map_err(|error| format!("Invalid PowerPC hexadecimal displacement '{}': {}.", displacement_text, error))?
    } else {
        displacement_text
            .parse::<i16>()
            .map_err(|error| format!("Invalid PowerPC displacement '{}': {}.", displacement_text, error))?
    };

    if base_register_name.is_empty() {
        return Err(format!("PowerPC memory operand '{}' is missing a base register.", operand_text));
    }

    Ok((displacement, base_register_name.to_string()))
}
