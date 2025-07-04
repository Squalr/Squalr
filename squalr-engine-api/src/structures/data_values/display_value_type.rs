use crate::structures::structs::container_type::ContainerType;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum DisplayValueType {
    Bool,
    String,
    Binary,
    Decimal,
    Hexadecimal,
    Address,
    DataTypeRef,
    Enumeration,
}

impl fmt::Display for DisplayValueType {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let result = match self {
            DisplayValueType::Bool => "boolean",
            DisplayValueType::String => "string",
            DisplayValueType::Binary => "binary",
            DisplayValueType::Decimal => "decimal",
            DisplayValueType::Hexadecimal => "hexadecimal",
            DisplayValueType::Address => "address",
            DisplayValueType::DataTypeRef => "data_type_ref",
            DisplayValueType::Enumeration => "enumeration",
        };

        write!(formatter, "{}", result)
    }
}

impl FromStr for DisplayValueType {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        fn extract_container(input: &str) -> (ContainerType, &str) {
            if let Some(stripped_input) = input.strip_suffix("[]") {
                (ContainerType::Array, stripped_input)
            } else if let Some(stripped_input) = input.strip_suffix("*") {
                (ContainerType::Pointer, stripped_input)
            } else {
                (ContainerType::None, input)
            }
        }

        let (container, base_type) = extract_container(input);

        match base_type {
            "bool" | "boolean" => Ok(DisplayValueType::Bool),
            "string" => Ok(DisplayValueType::String),
            "bin" | "binary" => Ok(DisplayValueType::Binary),
            "dec" | "decimal" => Ok(DisplayValueType::Decimal),
            "hex" | "hexadecimal" => Ok(DisplayValueType::Hexadecimal),
            "address" => Ok(DisplayValueType::Address),
            "data_type_ref" => Ok(DisplayValueType::DataTypeRef),
            "enumeration" => Ok(DisplayValueType::Enumeration),
            _ => Err(format!("Unknown display type: {}", base_type)),
        }
    }
}
