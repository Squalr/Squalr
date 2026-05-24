pub fn parse_arm_memory_expression(expression_text: &str) -> Result<(String, i64), String> {
    let trimmed_expression_text = expression_text.trim();

    if trimmed_expression_text.is_empty() {
        return Err(String::from("ARM memory operand expression must not be empty."));
    }

    let mut expression_parts = trimmed_expression_text.splitn(2, ',');
    let base_register_name = expression_parts
        .next()
        .map(str::trim)
        .filter(|base_register_name| !base_register_name.is_empty())
        .ok_or_else(|| String::from("ARM memory operand is missing a base register."))?;
    let offset = expression_parts
        .next()
        .map(parse_arm_memory_offset)
        .transpose()?
        .unwrap_or(0);

    Ok((base_register_name.to_string(), offset))
}

fn parse_arm_memory_offset(offset_text: &str) -> Result<i64, String> {
    let trimmed_offset_text = offset_text.trim();

    if trimmed_offset_text.is_empty() {
        return Err(String::from("ARM memory operand offset must not be empty."));
    }

    parse_signed_immediate(trimmed_offset_text)
}

pub fn parse_signed_immediate(immediate_text: &str) -> Result<i64, String> {
    let trimmed_immediate_text = immediate_text.trim();
    let unsigned_prefixed_immediate_text = trimmed_immediate_text
        .strip_prefix('#')
        .unwrap_or(trimmed_immediate_text);

    if unsigned_prefixed_immediate_text.is_empty() {
        return Err(String::from("Immediate is missing digits."));
    }

    let (sign_multiplier, unsigned_immediate_text) = if let Some(stripped_immediate_text) = unsigned_prefixed_immediate_text.strip_prefix('-') {
        (-1_i64, stripped_immediate_text)
    } else if let Some(stripped_immediate_text) = unsigned_prefixed_immediate_text.strip_prefix('+') {
        (1_i64, stripped_immediate_text)
    } else {
        (1_i64, unsigned_prefixed_immediate_text)
    };

    let parsed_value = if let Some(hexadecimal_digits) = unsigned_immediate_text
        .strip_prefix("0x")
        .or_else(|| unsigned_immediate_text.strip_prefix("0X"))
    {
        i64::from_str_radix(hexadecimal_digits, 16).map_err(|error| format!("Invalid hexadecimal immediate '{}': {}.", immediate_text, error))?
    } else {
        unsigned_immediate_text
            .parse::<i64>()
            .map_err(|error| format!("Invalid immediate '{}': {}.", immediate_text, error))?
    };

    Ok(parsed_value.saturating_mul(sign_multiplier))
}
