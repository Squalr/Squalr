use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Eq, Serialize, Deserialize)]
pub enum AnonymousValueStringFormat {
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

impl fmt::Display for AnonymousValueStringFormat {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let result = match self {
            AnonymousValueStringFormat::Bool => "boolean",
            AnonymousValueStringFormat::String => "string",
            AnonymousValueStringFormat::Binary => "binary",
            AnonymousValueStringFormat::Decimal => "decimal",
            AnonymousValueStringFormat::Hexadecimal => "hexadecimal",
            AnonymousValueStringFormat::Address => "address",
            AnonymousValueStringFormat::DataTypeRef => "data_type_ref",
            AnonymousValueStringFormat::Enumeration => "enumeration",
        };

        write!(formatter, "{}", result)
    }
}

impl FromStr for AnonymousValueStringFormat {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "bool" | "boolean" => Ok(AnonymousValueStringFormat::Bool),
            "string" => Ok(AnonymousValueStringFormat::String),
            "bin" | "binary" => Ok(AnonymousValueStringFormat::Binary),
            "dec" | "decimal" => Ok(AnonymousValueStringFormat::Decimal),
            "hex" | "hexadecimal" => Ok(AnonymousValueStringFormat::Hexadecimal),
            "address" => Ok(AnonymousValueStringFormat::Address),
            "data_type_ref" => Ok(AnonymousValueStringFormat::DataTypeRef),
            "enumeration" => Ok(AnonymousValueStringFormat::Enumeration),
            _ => Err(format!("Unknown display type: {}", input)),
        }
    }
}
