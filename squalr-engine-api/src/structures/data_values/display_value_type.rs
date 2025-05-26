use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DisplayValueType {
    Bool,
    String,
    Binary(bool),
    Decimal,
    Hexadecimal(bool),
    Address(bool),
    ByteArray,
    DataTypeRef,
    Enumeration,
}

impl fmt::Display for DisplayValueType {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let string = match self {
            DisplayValueType::Bool => "bool",
            DisplayValueType::String => "string",
            DisplayValueType::Binary(_) => "bin",
            DisplayValueType::Decimal => "dec",
            DisplayValueType::Hexadecimal(_) => "hex",
            DisplayValueType::Address(_) => "address",
            DisplayValueType::ByteArray => "byte_array",
            DisplayValueType::DataTypeRef => "data_type_ref",
            DisplayValueType::Enumeration => "enumeration",
        };
        write!(formatter, "{}", string)
    }
}

impl FromStr for DisplayValueType {
    type Err = ();

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        // JIRA: We could support an optional suffix to specify whether values have prefixes.
        match string {
            "bool" => Ok(DisplayValueType::Bool),
            "string" => Ok(DisplayValueType::String),
            "bin" => Ok(DisplayValueType::Binary(false)),
            "dec" => Ok(DisplayValueType::Decimal),
            "hex" => Ok(DisplayValueType::Hexadecimal(false)),
            "address" => Ok(DisplayValueType::Address(false)),
            "byte_array" => Ok(DisplayValueType::ByteArray),
            "data_type_ref" => Ok(DisplayValueType::DataTypeRef),
            "enumeration" => Ok(DisplayValueType::Enumeration),
            _ => Err(()),
        }
    }
}
