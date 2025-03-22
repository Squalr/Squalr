use std::num::ParseIntError;

pub struct Conversions {}

impl Conversions {
    /// Converts a hexadecimal string to a decimal string.
    pub fn hex_to_dec(hex: &str) -> Result<String, ParseIntError> {
        if hex.starts_with("0x") || hex.starts_with("0X") {
            u64::from_str_radix(&hex[2..], 16).map(|val| val.to_string())
        } else {
            u64::from_str_radix(hex, 16).map(|val| val.to_string())
        }
    }

    /// Converts a decimal string to a hexadecimal string.
    pub fn dec_to_hex(
        dec: &str,
        prepend_prefix: bool,
    ) -> Result<String, ParseIntError> {
        // Parse the decimal string to a u64 string.
        dec.parse::<u64>()
            .map(|val| if prepend_prefix { format!("0x{:X}", val) } else { format!("{:X}", val) })
    }

    pub fn parse_hex_or_int(src: &str) -> Result<u64, std::num::ParseIntError> {
        if src.starts_with("0x") || src.starts_with("0X") {
            u64::from_str_radix(&src[2..], 16)
        } else {
            src.parse::<u64>()
        }
    }

    /// Converts a given value into a metric information storage size (ie KB, MB, GB, TB, etc.).
    pub fn value_to_metric_size(value: u64) -> String {
        // Note: u64 runs out around EB.
        let suffix = ["B", "KB", "MB", "GB", "TB", "PB", "EB"];

        if value == 0 {
            return format!("0{}", suffix[0]);
        }

        let place = (value as f64).log(1024.0).floor() as usize;
        let number = (value as f64) / 1024f64.powi(place as i32);
        let rounded_number = (number * 10.0).round() / 10.0;

        format!("{:.1}{}", rounded_number, suffix[place])
    }

    // Converts an address string to a raw u64 value.
    pub fn address_to_value(address: &str) -> Result<u64, ParseIntError> {
        if address.is_empty() {
            return Ok(0);
        }

        let trimmed_address = if address.to_lowercase().starts_with("0x") { &address[2..] } else { address }.trim_start_matches('0');

        if trimmed_address.is_empty() {
            return Ok(0);
        }

        u64::from_str_radix(trimmed_address, 16)
    }

    pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, &'static str> {
        let hex = if hex.to_lowercase().starts_with("0x") { &hex[2..] } else { hex };
        let hex: String = hex
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect();

        if hex.is_empty() {
            return Ok(Vec::new());
        }

        // Round up capacity.
        let mut bytes = Vec::with_capacity((hex.len() + 1) / 2);
        let mut current_byte = 0u8;

        // Start with high nibble if odd length.
        let mut is_high_nibble = hex.len() % 2 == 1;

        for next_character in hex.as_bytes() {
            let nibble = Self::hex_char_to_byte(*next_character)?;

            if is_high_nibble {
                current_byte = nibble << 4;
            } else {
                current_byte |= nibble;
                bytes.push(current_byte);
                current_byte = 0;
            }
            is_high_nibble = !is_high_nibble;
        }

        // If we have a leftover high nibble (odd length), push it.
        if is_high_nibble {
            bytes.push(current_byte);
        }

        Ok(bytes)
    }

    pub fn hex_to_primitive_bytes<T: Copy + num_traits::ToBytes>(
        hex: &str,
        is_big_endian: bool,
    ) -> Result<Vec<u8>, &'static str> {
        let hex = if hex.to_lowercase().starts_with("0x") { &hex[2..] } else { hex };
        let hex: String = hex
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect();

        if hex.is_empty() {
            return Ok(vec![0; std::mem::size_of::<T>()]);
        }

        let required_size = std::mem::size_of::<T>();
        let hex_len = hex.len();
        let hex_half_len = hex_len / 2;

        if hex_len > required_size * 2 {
            return Err("Hex string length does not fit into the expected size of the primitive.");
        }

        // Round up capacity.
        let mut bytes: Vec<u8> = vec![0u8; required_size];

        for index in 0..hex_half_len {
            let high_nibble = Self::hex_char_to_byte(hex.chars().nth(2 * index).unwrap_or_default() as u8)?;
            let low_nibble = Self::hex_char_to_byte(hex.chars().nth(2 * index + 1).unwrap_or_default() as u8)?;

            // Combine the high and low nibble to form a byte
            bytes[index] = (high_nibble << 4) | low_nibble;
        }

        // Handle the last character explicitly if the length is odd.
        if hex_len % 2 == 1 {
            let high_nibble = Self::hex_char_to_byte(hex.chars().nth(hex_len - 1).unwrap_or_default() as u8)?;
            bytes[hex_len / 2] = high_nibble << 4;
        }

        // Finally handle endianness.
        if is_big_endian {
            bytes.reverse();
        }

        Ok(bytes)
    }

    // Helper function to convert a hex character to its byte value.
    pub fn hex_char_to_byte(hex_character: u8) -> Result<u8, &'static str> {
        match hex_character {
            b'0'..=b'9' => Ok(hex_character - b'0'),
            b'a'..=b'f' => Ok(hex_character - b'a' + 10),
            b'A'..=b'F' => Ok(hex_character - b'A' + 10),
            _ => Err("Invalid hex character."),
        }
    }
}
