use crate::conversions::conversion_error::ConversionError;
use crate::conversions::conversions_from_binary::ConversionsFromBinary;
use crate::conversions::conversions_from_decimal::ConversionsFromDecimal;
use crate::conversions::conversions_from_hexadecimal::ConversionsFromHexadecimal;
use crate::structures::data_values::display_value_type::DisplayValueType;
use std::num::ParseIntError;

pub struct Conversions {}

impl Conversions {
    pub fn convert_data_value(
        data_value: &str,
        from_display_value_type: DisplayValueType,
        to_display_value_type: DisplayValueType,
    ) -> Result<String, ConversionError> {
        match from_display_value_type {
            DisplayValueType::Binary(display_container) => ConversionsFromBinary::convert_to_display_value(data_value, to_display_value_type),
            DisplayValueType::Decimal(display_container) => ConversionsFromDecimal::convert_to_display_value(data_value, to_display_value_type),
            DisplayValueType::Hexadecimal(display_container) => ConversionsFromHexadecimal::convert_to_display_value(data_value, to_display_value_type),
            DisplayValueType::Address(display_container) => ConversionsFromHexadecimal::convert_to_display_value(data_value, to_display_value_type),
            _ => Ok(data_value.to_string()),
        }
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

    pub fn binary_to_bytes(bin: &str) -> Result<Vec<u8>, &'static str> {
        // Strip optional 0b/0B prefix first, then every kind of whitespace or comma.
        let bin = if bin.to_lowercase().starts_with("0b") { &bin[2..] } else { bin };
        let bin: String = bin
            .chars()
            .filter(|ch| !ch.is_whitespace() && *ch != ',')
            .collect();

        if bin.is_empty() {
            return Ok(Vec::new());
        }

        // Validate characters and build bytes (least significant bit first, then reverse once at the end).
        let mut bytes = Vec::with_capacity((bin.len() + 7) / 8);
        let mut current = 0u8;
        let mut bit_pos = 0;

        // 0-7 inside current byte (least significant bit first).
        for ch in bin.as_bytes().iter().rev() {
            match *ch {
                // Leave bit clear.
                b'0' => {}
                // Set the bit.
                b'1' => current |= 1 << bit_pos,
                _ => return Err("Invalid binary character."),
            }

            bit_pos += 1;
            if bit_pos == 8 {
                bytes.push(current);
                current = 0;
                bit_pos = 0;
            }
        }

        // Handle the last partial byte.
        if bit_pos > 0 {
            bytes.push(current);
        }

        // restore most significant bit first order
        bytes.reverse();
        Ok(bytes)
    }

    pub fn binary_to_primitive_bytes<T: Copy + num_traits::ToBytes>(
        bin: &str,
        is_big_endian: bool,
    ) -> Result<Vec<u8>, &'static str> {
        // Remove optional prefix and whitespace (no commas inside primitives).
        let bin = if bin.to_lowercase().starts_with("0b") { &bin[2..] } else { bin };
        let bin: String = bin.chars().filter(|ch| !ch.is_whitespace()).collect();
        let max_size = std::mem::size_of::<T>();
        let max_bits = max_size * 8;

        if bin.is_empty() {
            return Ok(vec![0u8; max_size]);
        }

        if bin.len() > max_bits {
            return Err("Binary string length does not fit into the expected size of the primitive.");
        }

        // Parse the binary string into raw bytes first.
        let parsed = Self::binary_to_bytes(&bin)?;

        // Copy into fixed-size buffer, least-significant byte first.
        let used_size = parsed.len();
        let mut bytes = vec![0u8; max_size];
        for idx in 0..used_size {
            bytes[idx] = parsed[used_size - idx - 1];
        }

        // Handle endianness: big-endian means most-significant byte first.
        if is_big_endian {
            bytes.reverse();
        }

        Ok(bytes)
    }

    pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, &'static str> {
        let hex = if hex.to_lowercase().starts_with("0x") { &hex[2..] } else { hex };
        let hex = hex.replace(|next_char: char| next_char.is_whitespace() || next_char == ',', "");

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

        // Padding to an even number makes our algorithm a bit easier to manage.
        let hex_zero_padded = if hex.len() % 2 == 1 { format!("0{}", hex) } else { hex };
        let max_size = std::mem::size_of::<T>();
        let used_size = hex_zero_padded.len() / 2;

        if used_size > max_size {
            return Err("Hex string length does not fit into the expected size of the primitive.");
        }

        let mut bytes: Vec<u8> = vec![0u8; max_size];

        for index in 0..used_size {
            let high_nibble = Self::hex_char_to_byte(hex_zero_padded.chars().nth(2 * index).unwrap_or_default() as u8)?;
            let low_nibble = Self::hex_char_to_byte(hex_zero_padded.chars().nth(2 * index + 1).unwrap_or_default() as u8)?;

            // Combine the high and low nibble to form the next byte.
            bytes[used_size - index - 1] = (high_nibble << 4) | low_nibble;
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
