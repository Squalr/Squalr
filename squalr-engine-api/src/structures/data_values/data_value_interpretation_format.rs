use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataValueInterpretationFormat {
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

impl fmt::Display for DataValueInterpretationFormat {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let result = match self {
            DataValueInterpretationFormat::Bool => "boolean",
            DataValueInterpretationFormat::String => "string",
            DataValueInterpretationFormat::Binary => "binary",
            DataValueInterpretationFormat::Decimal => "decimal",
            DataValueInterpretationFormat::Hexadecimal => "hexadecimal",
            DataValueInterpretationFormat::Address => "address",
            DataValueInterpretationFormat::DataTypeRef => "data_type_ref",
            DataValueInterpretationFormat::Enumeration => "enumeration",
        };

        write!(formatter, "{}", result)
    }
}

impl FromStr for DataValueInterpretationFormat {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "bool" | "boolean" => Ok(DataValueInterpretationFormat::Bool),
            "string" => Ok(DataValueInterpretationFormat::String),
            "bin" | "binary" => Ok(DataValueInterpretationFormat::Binary),
            "dec" | "decimal" => Ok(DataValueInterpretationFormat::Decimal),
            "hex" | "hexadecimal" => Ok(DataValueInterpretationFormat::Hexadecimal),
            "address" => Ok(DataValueInterpretationFormat::Address),
            "data_type_ref" => Ok(DataValueInterpretationFormat::DataTypeRef),
            "enumeration" => Ok(DataValueInterpretationFormat::Enumeration),
            _ => Err(format!("Unknown display type: {}", input)),
        }
    }
}
