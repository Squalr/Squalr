use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub enum ContainerType {
    #[default]
    None,
    Array,
    ArrayFixed(u64),
    Pointer32,
    Pointer64,
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
            ContainerType::Pointer32 => "*(32)".to_string(),
            ContainerType::Pointer64 => "*(64)".to_string(),
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
            match pointer_bits {
                "32" => return Ok(ContainerType::Pointer32),
                "64" => return Ok(ContainerType::Pointer64),
                _ => return Err(format!("Invalid pointer size: {}", pointer_bits)),
            }
        }

        Err(format!("Invalid container type: {}", trimmed))
    }
}
