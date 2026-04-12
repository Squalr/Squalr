use crate::structures::data_values::pointer_scan_pointer_size::PointerScanPointerSize;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub enum ContainerType {
    #[default]
    None,
    Array,
    ArrayFixed(u64),
    Pointer(PointerScanPointerSize),
    Pointer32,
    Pointer64,
}

impl ContainerType {
    pub fn from_pointer_size(pointer_size: PointerScanPointerSize) -> Self {
        Self::Pointer(pointer_size)
    }

    pub fn get_pointer_size(&self) -> Option<PointerScanPointerSize> {
        match self {
            ContainerType::Pointer(pointer_size) => Some(*pointer_size),
            ContainerType::Pointer32 => Some(PointerScanPointerSize::Pointer32),
            ContainerType::Pointer64 => Some(PointerScanPointerSize::Pointer64),
            _ => None,
        }
    }

    pub fn get_total_size_in_bytes(
        &self,
        unit_size_in_bytes: u64,
    ) -> u64 {
        if let Some(pointer_size) = self.get_pointer_size() {
            return pointer_size.get_size_in_bytes();
        }

        match self {
            ContainerType::None => unit_size_in_bytes,
            ContainerType::Array => unit_size_in_bytes,
            ContainerType::ArrayFixed(length) => unit_size_in_bytes.saturating_mul(*length),
            ContainerType::Pointer(_) | ContainerType::Pointer32 | ContainerType::Pointer64 => unit_size_in_bytes,
        }
    }
}

impl fmt::Display for ContainerType {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let container_type_str = match &self {
            ContainerType::None => String::new(),
            ContainerType::Array => "[]".to_string(),
            ContainerType::ArrayFixed(length) => format!("[{}]", length).to_string(),
            ContainerType::Pointer(pointer_size) => format!("*({})", pointer_size),
            ContainerType::Pointer32 => format!("*({})", PointerScanPointerSize::Pointer32),
            ContainerType::Pointer64 => format!("*({})", PointerScanPointerSize::Pointer64),
        };

        write!(formatter, "{}", container_type_str)
    }
}

impl FromStr for ContainerType {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Ok(ContainerType::None);
        }

        if let Some(length_string) = trimmed
            .strip_prefix('[')
            .and_then(|string| string.strip_suffix(']'))
        {
            if length_string.is_empty() {
                return Ok(ContainerType::Array);
            }

            let length = length_string
                .parse::<u64>()
                .map_err(|_| format!("Invalid array length: {}", length_string))?;

            return Ok(ContainerType::ArrayFixed(length));
        }

        if let Some(pointer_bits) = trimmed
            .strip_prefix("*(")
            .and_then(|string| string.strip_suffix(')'))
        {
            let pointer_size = PointerScanPointerSize::from_str(pointer_bits)?;

            return Ok(ContainerType::Pointer(pointer_size));
        }

        if trimmed == "*" {
            return Ok(ContainerType::Pointer(PointerScanPointerSize::Pointer64));
        }

        Err(format!("Invalid container type: {}", trimmed))
    }
}

#[cfg(test)]
mod tests {
    use super::ContainerType;
    use crate::structures::data_values::pointer_scan_pointer_size::PointerScanPointerSize;
    use std::str::FromStr;

    #[test]
    fn pointer_container_round_trips_extended_pointer_sizes() {
        let container_type = ContainerType::from_str("*(u24be)").expect("Expected pointer container to parse.");

        assert_eq!(container_type, ContainerType::Pointer(PointerScanPointerSize::Pointer24be));
        assert_eq!(container_type.to_string(), "*(u24be)");
    }

    #[test]
    fn legacy_pointer_container_strings_map_to_shared_pointer_sizes() {
        let container_type = ContainerType::from_str("*(32)").expect("Expected legacy pointer container to parse.");

        assert_eq!(container_type.get_pointer_size(), Some(PointerScanPointerSize::Pointer32));
    }
}
