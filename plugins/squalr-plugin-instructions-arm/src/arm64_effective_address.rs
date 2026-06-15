//! Computes the effective memory address an ARM64 instruction accesses, from its disassembled text and a runtime
//! register lookup. Used by instruction-directed traces ("find what addresses this instruction accesses").
//!
//! The disassembled memory operand looks like one of:
//!   [xn]                     -> xn
//!   [xn, #imm]               -> xn + imm
//!   [xn, #imm]!              -> xn + imm        (pre-index)
//!   [xn], #imm               -> xn             (post-index: write-back happens after the access)
//!   [xn, xm]                 -> xn + xm
//!   [xn, xm, lsl #s]         -> xn + (xm << s)
//!   [xn, wm, sxtw #s]        -> xn + (sext32(wm) << s)
//!   [xn, wm, uxtw #s]        -> xn + (zext32(wm) << s)

/// Resolves the accessed address for an ARM64 instruction, or `None` if it has no resolvable memory operand.
pub fn resolve_arm64_accessed_address(
    instruction_text: &str,
    register_value_by_name: &dyn Fn(&str) -> Option<u64>,
) -> Option<u64> {
    let (inside_brackets, has_post_index) = extract_memory_operand(instruction_text)?;
    let mut operand_parts = inside_brackets
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty());

    let base_register_name = operand_parts.next()?;
    let base_value = lookup_register(register_value_by_name, base_register_name)?;

    // Post-index ([xn], #imm) accesses the base; the offset is applied to the register afterward.
    if has_post_index {
        return Some(base_value);
    }

    let Some(second_operand) = operand_parts.next() else {
        return Some(base_value);
    };

    // [xn, #imm] (and pre-index [xn, #imm]! which the bracket-extraction already stripped of the trailing '!').
    if let Some(immediate_text) = second_operand.strip_prefix('#') {
        let immediate = parse_immediate(immediate_text).ok()?;

        return Some(base_value.wrapping_add(immediate as u64));
    }

    // [xn, xm{, <extend|lsl> #s}]
    let index_value = lookup_register(register_value_by_name, second_operand)?;
    let (extended_index_value, shift_amount) = match operand_parts.next() {
        None => (index_value, 0),
        Some(shift_specifier) => apply_index_extend(second_operand, index_value, shift_specifier)?,
    };

    Some(base_value.wrapping_add(extended_index_value.wrapping_shl(shift_amount)))
}

/// Returns the text between the first `[` and its matching `]`, and whether the operand is post-indexed (text after the
/// `]` is `, #imm`). Pre-index `]!` is treated like a regular offset (the access uses base + offset).
fn extract_memory_operand(instruction_text: &str) -> Option<(&str, bool)> {
    let open_bracket_index = instruction_text.find('[')?;
    let close_bracket_relative_index = instruction_text[open_bracket_index..].find(']')?;
    let close_bracket_index = open_bracket_index + close_bracket_relative_index;
    let inside_brackets = &instruction_text[open_bracket_index + 1..close_bracket_index];
    let after_close_bracket = instruction_text[close_bracket_index + 1..].trim();
    let has_post_index = after_close_bracket.starts_with(',');

    Some((inside_brackets, has_post_index))
}

fn apply_index_extend(
    index_register_name: &str,
    index_value: u64,
    shift_specifier: &str,
) -> Option<(u64, u32)> {
    let mut shift_specifier_parts = shift_specifier.split_whitespace();
    let extend_operation = shift_specifier_parts.next()?.to_ascii_lowercase();
    let shift_amount = shift_specifier_parts
        .next()
        .and_then(|amount_text| parse_immediate(amount_text.trim_start_matches('#')).ok())
        .unwrap_or(0)
        .max(0) as u32;
    let is_word_register = index_register_name.trim().to_ascii_lowercase().starts_with('w');

    let extended_index_value = match extend_operation.as_str() {
        "lsl" => index_value,
        "uxtw" => index_value & 0xFFFF_FFFF,
        "sxtw" => ((index_value & 0xFFFF_FFFF) as u32 as i32 as i64) as u64,
        // uxtx/sxtx (and unknown) keep the full 64-bit value.
        _ if is_word_register => index_value & 0xFFFF_FFFF,
        _ => index_value,
    };

    Some((extended_index_value, shift_amount))
}

/// Looks up a register, treating `xzr`/`wzr` as zero and trying both the written name and its 64-bit `x` form.
fn lookup_register(
    register_value_by_name: &dyn Fn(&str) -> Option<u64>,
    register_name: &str,
) -> Option<u64> {
    let normalized_register_name = register_name.trim().to_ascii_lowercase();

    if normalized_register_name == "xzr" || normalized_register_name == "wzr" {
        return Some(0);
    }

    register_value_by_name(&normalized_register_name)
}

fn parse_immediate(immediate_text: &str) -> Result<i64, ()> {
    let trimmed = immediate_text.trim();
    let (is_negative, unsigned_text) = match trimmed.strip_prefix('-') {
        Some(stripped) => (true, stripped),
        None => (false, trimmed.strip_prefix('+').unwrap_or(trimmed)),
    };
    let magnitude = if let Some(hex_digits) = unsigned_text
        .strip_prefix("0x")
        .or_else(|| unsigned_text.strip_prefix("0X"))
    {
        i64::from_str_radix(hex_digits, 16).map_err(|_| ())?
    } else {
        unsigned_text.parse::<i64>().map_err(|_| ())?
    };

    Ok(if is_negative { -magnitude } else { magnitude })
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

    #[test]
    fn resolves_base_only() {
        let lookup = registers(&[("x19", 0x2A5F_C58)]);
        assert_eq!(resolve_arm64_accessed_address("stxr w9, x8, [x19]", &lookup), Some(0x2A5F_C58));
    }

    #[test]
    fn resolves_base_plus_immediate() {
        let lookup = registers(&[("x29", 0x1000)]);
        assert_eq!(resolve_arm64_accessed_address("ldr x0, [x29, #0x8]", &lookup), Some(0x1008));
        assert_eq!(resolve_arm64_accessed_address("ldr x0, [x29, #-0x8]", &lookup), Some(0xFF8));
    }

    #[test]
    fn pre_index_uses_base_plus_immediate() {
        let lookup = registers(&[("x1", 0x2000)]);
        assert_eq!(resolve_arm64_accessed_address("str x0, [x1, #0x10]!", &lookup), Some(0x2010));
    }

    #[test]
    fn post_index_uses_base_only() {
        let lookup = registers(&[("x1", 0x2000)]);
        assert_eq!(resolve_arm64_accessed_address("ldr x0, [x1], #0x10", &lookup), Some(0x2000));
    }

    #[test]
    fn resolves_base_plus_index() {
        let lookup = registers(&[("x1", 0x4000), ("x2", 0x8)]);
        assert_eq!(resolve_arm64_accessed_address("ldr x0, [x1, x2]", &lookup), Some(0x4008));
    }

    #[test]
    fn resolves_base_plus_scaled_index() {
        let lookup = registers(&[("x1", 0x4000), ("x2", 0x4)]);
        assert_eq!(resolve_arm64_accessed_address("ldr x0, [x1, x2, lsl #3]", &lookup), Some(0x4020));
    }

    #[test]
    fn resolves_sign_extended_word_index() {
        let lookup = registers(&[("x1", 0x4000), ("w2", 0xFFFF_FFFF)]);
        // sxtw of 0xFFFFFFFF == -1, shifted left 2 == -4.
        assert_eq!(resolve_arm64_accessed_address("ldr x0, [x1, w2, sxtw #2]", &lookup), Some(0x3FFC));
    }

    #[test]
    fn zero_register_is_zero() {
        let lookup = registers(&[("x1", 0x4000)]);
        assert_eq!(resolve_arm64_accessed_address("ldr x0, [x1, xzr]", &lookup), Some(0x4000));
    }

    #[test]
    fn no_memory_operand_returns_none() {
        let lookup = registers(&[("x0", 1), ("x1", 2)]);
        assert_eq!(resolve_arm64_accessed_address("add x0, x1, x2", &lookup), None);
    }
}
