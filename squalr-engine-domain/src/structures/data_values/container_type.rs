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
    PointerArray(PointerScanPointerSize),
    PointerArrayFixed(PointerScanPointerSize, u64),
}

impl ContainerType {
    pub fn from_pointer_size(pointer_size: PointerScanPointerSize) -> Self {
        Self::Pointer(pointer_size)
    }

    pub fn get_pointer_size(&self) -> Option<PointerScanPointerSize> {
        match self {
            ContainerType::Pointer(pointer_size) => Some(*pointer_size),
            _ => None,
        }
    }

    pub fn get_total_size_in_bytes(
        &self,
        unit_size_in_bytes: u64,
    ) -> u64 {
        match self {
            ContainerType::None => unit_size_in_bytes,
            ContainerType::Array => unit_size_in_bytes,
            ContainerType::ArrayFixed(length) => unit_size_in_bytes.saturating_mul(*length),
            ContainerType::Pointer(pointer_size) | ContainerType::PointerArray(pointer_size) => pointer_size.get_size_in_bytes(),
            ContainerType::PointerArrayFixed(pointer_size, length) => pointer_size.get_size_in_bytes().saturating_mul(*length),
        }
    }

    pub fn with_fixed_element_count(
        &self,
        element_count: u64,
    ) -> Option<Self> {
        match self {
            ContainerType::Array | ContainerType::ArrayFixed(_) => Some(ContainerType::ArrayFixed(element_count)),
            ContainerType::PointerArray(pointer_size) | ContainerType::PointerArrayFixed(pointer_size, _) => {
                Some(ContainerType::PointerArrayFixed(*pointer_size, element_count))
            }
            ContainerType::None | ContainerType::Pointer(_) => None,
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
            ContainerType::PointerArray(pointer_size) => format!("*({})[]", pointer_size),
            ContainerType::PointerArrayFixed(pointer_size, length) => format!("*({})[{}]", pointer_size, length),
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

        if let Some(pointer_array_string) = trimmed.strip_prefix("*(") {
            let (pointer_size_string, array_string) = pointer_array_string
                .split_once(')')
                .ok_or_else(|| format!("Invalid pointer array container type: {}", trimmed))?;
            let pointer_size = PointerScanPointerSize::from_str(pointer_size_string)?;

            if let Some(length_string) = array_string
                .strip_prefix('[')
                .and_then(|string| string.strip_suffix(']'))
            {
                if length_string.is_empty() {
                    return Ok(ContainerType::PointerArray(pointer_size));
                }

                let length = length_string
                    .parse::<u64>()
                    .map_err(|_| format!("Invalid pointer array length: {}", length_string))?;

                return Ok(ContainerType::PointerArrayFixed(pointer_size, length));
            }

            if array_string.is_empty() {
                return Ok(ContainerType::Pointer(pointer_size));
            }

            return Err(format!("Invalid pointer container type: {}", trimmed));
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
    fn fixed_pointer_array_container_round_trips() {
        let container_type = ContainerType::from_str("*(u64)[1024]").expect("Expected fixed pointer array container to parse.");

        assert_eq!(container_type, ContainerType::PointerArrayFixed(PointerScanPointerSize::Pointer64, 1024));
        assert_eq!(container_type.to_string(), "*(u64)[1024]");
        assert_eq!(container_type.get_total_size_in_bytes(32), 8192);
    }
}
