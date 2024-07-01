use std::num::ParseIntError;

pub fn parse_hex_or_int(src: &str) -> Result<u64, std::num::ParseIntError> {
    if src.starts_with("0x") || src.starts_with("0X") {
        u64::from_str_radix(&src[2..], 16)
    } else {
        src.parse::<u64>()
    }
}

// Converts an address string to a raw u64 value.
pub fn address_to_value(address: &str) -> Result<u64, ParseIntError> {
    if address.is_empty() {
        return Ok(0);
    }

    let trimmed_address = if address.to_lowercase().starts_with("0x") {
        &address[2..]
    } else {
        address
    }.trim_start_matches('0');

    if trimmed_address.is_empty() {
        return Ok(0);
    }

    u64::from_str_radix(trimmed_address, 16)
}