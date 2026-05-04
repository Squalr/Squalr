use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PointerChainSegment {
    Offset(i64),
    Symbol(String),
}

impl PointerChainSegment {
    pub fn new_offset(offset: i64) -> Self {
        Self::Offset(offset)
    }

    pub fn new_symbol(symbol_name: String) -> Option<Self> {
        if Self::is_valid_symbol_name(&symbol_name) {
            Some(Self::Symbol(symbol_name))
        } else {
            None
        }
    }

    pub fn as_offset(&self) -> Option<i64> {
        match self {
            Self::Offset(offset) => Some(*offset),
            Self::Symbol(_) => None,
        }
    }

    pub fn symbol_name(&self) -> Option<&str> {
        match self {
            Self::Offset(_) => None,
            Self::Symbol(symbol_name) => Some(symbol_name),
        }
    }

    pub fn display_text(&self) -> String {
        match self {
            Self::Offset(offset) => Self::format_offset(*offset),
            Self::Symbol(symbol_name) => symbol_name.clone(),
        }
    }

    pub fn format_offset(offset: i64) -> String {
        if offset < 0 {
            format!("-0x{:X}", offset.saturating_abs())
        } else {
            format!("0x{:X}", offset)
        }
    }

    pub fn parse_text_list(pointer_chain_text: &str) -> Vec<Self> {
        if let Ok(pointer_chain_segments) = serde_json::from_str::<Vec<Self>>(pointer_chain_text) {
            return pointer_chain_segments;
        }

        if let Ok(pointer_offsets) = serde_json::from_str::<Vec<i64>>(pointer_chain_text) {
            return pointer_offsets.into_iter().map(Self::Offset).collect();
        }

        pointer_chain_text
            .split(',')
            .filter_map(|pointer_chain_segment_text| Self::from_str(pointer_chain_segment_text).ok())
            .collect()
    }

    pub fn display_text_list(pointer_chain_segments: &[Self]) -> String {
        pointer_chain_segments
            .iter()
            .map(Self::display_text)
            .collect::<Vec<String>>()
            .join(", ")
    }

    pub fn is_valid_symbol_name(symbol_name: &str) -> bool {
        let symbol_name = symbol_name.trim();

        if symbol_name.is_empty() || Self::parse_offset(symbol_name).is_some() {
            return false;
        }

        let mut symbol_name_characters = symbol_name.chars();
        let Some(first_character) = symbol_name_characters.next() else {
            return false;
        };

        if !(first_character == '_' || first_character.is_ascii_alphabetic()) {
            return false;
        }

        symbol_name_characters.all(|symbol_name_character| symbol_name_character == '_' || symbol_name_character.is_ascii_alphanumeric())
    }

    fn parse_offset(pointer_offset_text: &str) -> Option<i64> {
        let pointer_offset_text = pointer_offset_text.trim();

        if pointer_offset_text.is_empty() {
            return None;
        }

        let (sign, pointer_offset_text) = pointer_offset_text
            .strip_prefix('-')
            .map(|pointer_offset_text| (-1_i64, pointer_offset_text.trim()))
            .unwrap_or((1_i64, pointer_offset_text));
        let pointer_offset_hex_text = pointer_offset_text
            .strip_prefix("0x")
            .or_else(|| pointer_offset_text.strip_prefix("0X"));

        if let Some(pointer_offset_hex_text) = pointer_offset_hex_text {
            i64::from_str_radix(pointer_offset_hex_text, 16)
                .ok()
                .and_then(|pointer_offset| pointer_offset.checked_mul(sign))
        } else {
            pointer_offset_text
                .parse::<i64>()
                .ok()
                .and_then(|pointer_offset| pointer_offset.checked_mul(sign))
        }
    }
}

impl From<i64> for PointerChainSegment {
    fn from(offset: i64) -> Self {
        Self::Offset(offset)
    }
}

impl FromStr for PointerChainSegment {
    type Err = String;

    fn from_str(pointer_chain_segment_text: &str) -> Result<Self, Self::Err> {
        let pointer_chain_segment_text = pointer_chain_segment_text.trim();

        if let Some(pointer_offset) = Self::parse_offset(pointer_chain_segment_text) {
            return Ok(Self::Offset(pointer_offset));
        }

        if Self::is_valid_symbol_name(pointer_chain_segment_text) {
            return Ok(Self::Symbol(pointer_chain_segment_text.to_string()));
        }

        Err(format!("Invalid pointer chain segment: {}", pointer_chain_segment_text))
    }
}

impl fmt::Display for PointerChainSegment {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        formatter.write_str(&self.display_text())
    }
}

pub trait IntoPointerChainSegments {
    fn into_pointer_chain_segments(self) -> Vec<PointerChainSegment>;
}

impl IntoPointerChainSegments for Vec<i64> {
    fn into_pointer_chain_segments(self) -> Vec<PointerChainSegment> {
        self.into_iter().map(PointerChainSegment::Offset).collect()
    }
}

impl IntoPointerChainSegments for Vec<PointerChainSegment> {
    fn into_pointer_chain_segments(self) -> Vec<PointerChainSegment> {
        self
    }
}

impl IntoPointerChainSegments for &[i64] {
    fn into_pointer_chain_segments(self) -> Vec<PointerChainSegment> {
        self.iter().copied().map(PointerChainSegment::Offset).collect()
    }
}

impl IntoPointerChainSegments for &[PointerChainSegment] {
    fn into_pointer_chain_segments(self) -> Vec<PointerChainSegment> {
        self.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::PointerChainSegment;
    use std::str::FromStr;

    #[test]
    fn parses_numeric_and_symbolic_segments() {
        assert_eq!(PointerChainSegment::from_str("0x20"), Ok(PointerChainSegment::Offset(0x20)));
        assert_eq!(PointerChainSegment::from_str("-10"), Ok(PointerChainSegment::Offset(-10)));
        assert_eq!(PointerChainSegment::from_str("health"), Ok(PointerChainSegment::Symbol(String::from("health"))));
    }

    #[test]
    fn rejects_number_like_or_invalid_symbol_names() {
        assert!(PointerChainSegment::from_str("123health").is_err());
        assert!(PointerChainSegment::from_str("0x20").is_ok());
        assert!(PointerChainSegment::new_symbol(String::from("123")).is_none());
        assert!(PointerChainSegment::new_symbol(String::from("bad-name")).is_none());
    }

    #[test]
    fn deserializes_mixed_json_segments() {
        let pointer_chain_segments = PointerChainSegment::parse_text_list(r#"[16,"health",-4]"#);

        assert_eq!(
            pointer_chain_segments,
            vec![
                PointerChainSegment::Offset(16),
                PointerChainSegment::Symbol(String::from("health")),
                PointerChainSegment::Offset(-4)
            ]
        );
    }
}
