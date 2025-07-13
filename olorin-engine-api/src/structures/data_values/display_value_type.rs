use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisplayValueType {
    Bool,
    #[default]
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
        match input {
            "bool" | "boolean" => Ok(DisplayValueType::Bool),
            "string" => Ok(DisplayValueType::String),
            "bin" | "binary" => Ok(DisplayValueType::Binary),
            "dec" | "decimal" => Ok(DisplayValueType::Decimal),
            "hex" | "hexadecimal" => Ok(DisplayValueType::Hexadecimal),
            "address" => Ok(DisplayValueType::Address),
            "data_type_ref" => Ok(DisplayValueType::DataTypeRef),
            "enumeration" => Ok(DisplayValueType::Enumeration),
            _ => Err(format!("Unknown display type: {}", input)),
        }
    }
}
