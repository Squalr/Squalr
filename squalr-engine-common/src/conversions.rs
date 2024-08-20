use std::num::ParseIntError;

pub fn parse_hex_or_int(
    src: &str
) -> Result<u64, std::num::ParseIntError> {
    if src.starts_with("0x") || src.starts_with("0X") {
        return u64::from_str_radix(&src[2..], 16);
    } else {
        return src.parse::<u64>();
    }
}

/// Converts a given value into a metric information storage size (ie KB, MB, GB, TB, etc.)
pub fn value_to_metric_size(
    value: u64
) -> String {
    let suffix = ["B", "KB", "MB", "GB", "TB", "PB", "EB"];

    if value == 0 {
        return format!("0{}", suffix[0]);
    }

    let place = (value as f64).log(1024.0).floor() as usize;
    let number = (value as f64) / 1024f64.powi(place as i32);
    let rounded_number = (number * 10.0).round() / 10.0;

    return format!("{:.1}{}", rounded_number, suffix[place]);
}

// Converts an address string to a raw u64 value.
pub fn address_to_value(
    address: &str
) -> Result<u64, ParseIntError> {
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

    return u64::from_str_radix(trimmed_address, 16);
}
