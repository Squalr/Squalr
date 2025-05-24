use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DisplayValueType {
    Bool,
    String,
    Bin,
    Dec,
    Hex,
    Address,
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
            DisplayValueType::Bin => "bin",
            DisplayValueType::Dec => "dec",
            DisplayValueType::Hex => "hex",
            DisplayValueType::Address => "address",
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
        match string {
            "bool" => Ok(DisplayValueType::Bool),
            "string" => Ok(DisplayValueType::String),
            "bin" => Ok(DisplayValueType::Bin),
            "dec" => Ok(DisplayValueType::Dec),
            "hex" => Ok(DisplayValueType::Hex),
            "address" => Ok(DisplayValueType::Address),
            "byte_array" => Ok(DisplayValueType::ByteArray),
            "data_type_ref" => Ok(DisplayValueType::DataTypeRef),
            "enumeration" => Ok(DisplayValueType::Enumeration),
            _ => Err(()),
        }
    }
}
