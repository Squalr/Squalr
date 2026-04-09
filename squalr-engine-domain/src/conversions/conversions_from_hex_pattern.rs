/// Parses a masked hex byte pattern string in which individual nibbles may be wildcarded with
/// the character `x` or `X`.  All other nibbles must be valid hex digits `[0-9A-Fa-f]`.
///
/// Tokens are separated by any combination of whitespace, commas, or hyphens.
///
/// Nibble encoding rules:
/// - `AB` → exact byte 0xAB, mask byte 0xFF
/// - `Ax` → high nibble A exact, low nibble wildcard; pattern byte 0xA0, mask byte 0xF0
/// - `xB` → high nibble wildcard, low nibble B exact; pattern byte 0x0B, mask byte 0x0F
/// - `xx` → full byte wildcard; pattern byte 0x00, mask byte 0x00
///
/// Returns `(pattern_bytes, mask_bytes)` where a mask byte of `0xFF` means the corresponding
/// pattern byte must match exactly and lower mask values indicate partially or fully wildcarded
/// nibbles.
pub struct ConversionsFromHexPattern {}

impl ConversionsFromHexPattern {
    /// Parses a hex pattern string into `(pattern_bytes, mask_bytes)`.
    ///
    /// Returns an error string when any token is not exactly two characters, or when a
    /// non-wildcard character is not a valid hex digit.
    pub fn parse(input: &str) -> Result<(Vec<u8>, Vec<u8>), String> {
        if input.trim().is_empty() {
            return Err("Hex pattern must not be empty.".to_string());
        }

        let tokens: Vec<&str> = input
            .split(|character: char| character == ' ' || character == ',' || character == '-')
            .map(|token| token.trim())
            .filter(|token| !token.is_empty())
            .collect();

        if tokens.is_empty() {
            return Err("Hex pattern must contain at least one byte token.".to_string());
        }

        let mut pattern = Vec::with_capacity(tokens.len());
        let mut mask = Vec::with_capacity(tokens.len());

        for (token_index, token) in tokens.iter().enumerate() {
            match Self::parse_token(token) {
                Ok((byte_value, mask_byte)) => {
                    pattern.push(byte_value);
                    mask.push(mask_byte);
                }
                Err(error) => {
                    return Err(format!("Invalid token '{}' at position {}: {}", token, token_index, error));
                }
            }
        }

        Ok((pattern, mask))
    }

    /// Returns `true` if any mask byte is not `0xFF`, indicating at least one wildcarded nibble.
    pub fn has_wildcards(mask: &[u8]) -> bool {
        mask.iter().any(|&mask_byte| mask_byte != 0xFF)
    }

    /// Parses a single two-character hex token into `(pattern_byte, mask_byte)`.
    fn parse_token(token: &str) -> Result<(u8, u8), String> {
        let chars: Vec<char> = token.chars().collect();

        if chars.len() != 2 {
            return Err(format!("Expected exactly 2 nibble characters, got {} in '{}'.", chars.len(), token));
        }

        let hi_char = chars[0].to_ascii_uppercase();
        let lo_char = chars[1].to_ascii_uppercase();
        let hi_is_wild = hi_char == 'X';
        let lo_is_wild = lo_char == 'X';

        let hi_nibble = if hi_is_wild {
            0u8
        } else if hi_char.is_ascii_hexdigit() {
            hi_char.to_digit(16).expect("verified hex digit") as u8
        } else {
            return Err(format!("'{}' is not a valid hex nibble or wildcard 'x'.", hi_char));
        };

        let lo_nibble = if lo_is_wild {
            0u8
        } else if lo_char.is_ascii_hexdigit() {
            lo_char.to_digit(16).expect("verified hex digit") as u8
        } else {
            return Err(format!("'{}' is not a valid hex nibble or wildcard 'x'.", lo_char));
        };

        let byte_value = (hi_nibble << 4) | lo_nibble;
        let mask_byte = match (hi_is_wild, lo_is_wild) {
            (true, true) => 0x00,
            (true, false) => 0x0F,
            (false, true) => 0xF0,
            (false, false) => 0xFF,
        };

        Ok((byte_value, mask_byte))
    }
}

#[cfg(test)]
mod tests {
    use super::ConversionsFromHexPattern;

    #[test]
    fn parse_exact_bytes_space_separated() {
        let (pattern, mask) = ConversionsFromHexPattern::parse("DE AD BE EF").unwrap();
        assert_eq!(pattern, vec![0xDE, 0xAD, 0xBE, 0xEF]);
        assert_eq!(mask, vec![0xFF, 0xFF, 0xFF, 0xFF]);
        assert!(!ConversionsFromHexPattern::has_wildcards(&mask));
    }

    #[test]
    fn parse_exact_bytes_comma_separated() {
        let (pattern, mask) = ConversionsFromHexPattern::parse("DE,AD,BE,EF").unwrap();
        assert_eq!(pattern, vec![0xDE, 0xAD, 0xBE, 0xEF]);
        assert_eq!(mask, vec![0xFF, 0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn parse_mixed_pattern_with_full_wildcard() {
        let (pattern, mask) = ConversionsFromHexPattern::parse("00 xx 7x FB A1").unwrap();
        assert_eq!(pattern, vec![0x00, 0x00, 0x70, 0xFB, 0xA1]);
        assert_eq!(mask, vec![0xFF, 0x00, 0xF0, 0xFF, 0xFF]);
        assert!(ConversionsFromHexPattern::has_wildcards(&mask));
    }

    #[test]
    fn parse_low_nibble_wildcard() {
        let (pattern, mask) = ConversionsFromHexPattern::parse("Ax").unwrap();
        assert_eq!(pattern, vec![0xA0]);
        assert_eq!(mask, vec![0xF0]);
        assert!(ConversionsFromHexPattern::has_wildcards(&mask));
    }

    #[test]
    fn parse_high_nibble_wildcard() {
        let (pattern, mask) = ConversionsFromHexPattern::parse("xB").unwrap();
        assert_eq!(pattern, vec![0x0B]);
        assert_eq!(mask, vec![0x0F]);
        assert!(ConversionsFromHexPattern::has_wildcards(&mask));
    }

    #[test]
    fn parse_full_wildcard_single_token() {
        let (pattern, mask) = ConversionsFromHexPattern::parse("xx").unwrap();
        assert_eq!(pattern, vec![0x00]);
        assert_eq!(mask, vec![0x00]);
        assert!(ConversionsFromHexPattern::has_wildcards(&mask));
    }

    #[test]
    fn parse_is_case_insensitive() {
        let (pattern_upper, mask_upper) = ConversionsFromHexPattern::parse("FF XX 7X xF").unwrap();
        let (pattern_lower, mask_lower) = ConversionsFromHexPattern::parse("ff xx 7x xf").unwrap();
        assert_eq!(pattern_upper, pattern_lower);
        assert_eq!(mask_upper, mask_lower);
        assert_eq!(pattern_upper, vec![0xFF, 0x00, 0x70, 0x0F]);
        assert_eq!(mask_upper, vec![0xFF, 0x00, 0xF0, 0x0F]);
    }

    #[test]
    fn parse_rejects_empty_input() {
        assert!(ConversionsFromHexPattern::parse("").is_err());
        assert!(ConversionsFromHexPattern::parse("   ").is_err());
    }

    #[test]
    fn parse_rejects_invalid_nibble_characters() {
        assert!(ConversionsFromHexPattern::parse("GG").is_err());
        assert!(ConversionsFromHexPattern::parse("1G").is_err());
        assert!(ConversionsFromHexPattern::parse("ZZ").is_err());
    }

    #[test]
    fn parse_rejects_wrong_token_length() {
        assert!(ConversionsFromHexPattern::parse("ABC").is_err());
        assert!(ConversionsFromHexPattern::parse("A").is_err());
        assert!(ConversionsFromHexPattern::parse("ABCD").is_err());
    }

    #[test]
    fn parse_mixed_separators() {
        let (pattern, mask) = ConversionsFromHexPattern::parse("DE-AD xx,FF").unwrap();
        assert_eq!(pattern, vec![0xDE, 0xAD, 0x00, 0xFF]);
        assert_eq!(mask, vec![0xFF, 0xFF, 0x00, 0xFF]);
    }

    #[test]
    fn parse_no_wildcards_means_no_wildcards() {
        let (_, mask) = ConversionsFromHexPattern::parse("01 02 03 04").unwrap();
        assert!(!ConversionsFromHexPattern::has_wildcards(&mask));
    }
}
