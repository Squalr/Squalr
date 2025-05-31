use crate::structures::data_values::container_type::ContainerType;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DisplayValueType {
    Bool(ContainerType),
    String(ContainerType),
    Binary(ContainerType),
    Decimal(ContainerType),
    Hexadecimal(ContainerType),
    Address(ContainerType),
    DataTypeRef(ContainerType),
    Enumeration(ContainerType),
}

impl fmt::Display for DisplayValueType {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        fn container_str(container: &ContainerType) -> &'static str {
            match container {
                ContainerType::None => "",
                ContainerType::Array => "[]",
            }
        }

        let result = match self {
            DisplayValueType::Bool(container_type) => format!("boolean{}", container_str(container_type)),
            DisplayValueType::String(container_type) => format!("string{}", container_str(container_type)),
            DisplayValueType::Binary(container_type) => format!("binary{}", container_str(container_type)),
            DisplayValueType::Decimal(container_type) => format!("decimal{}", container_str(container_type)),
            DisplayValueType::Hexadecimal(container_type) => format!("hexadecimal{}", container_str(container_type)),
            DisplayValueType::Address(container_type) => format!("address{}", container_str(container_type)),
            DisplayValueType::DataTypeRef(container_type) => format!("data_type_ref{}", container_str(container_type)),
            DisplayValueType::Enumeration(container_type) => format!("enumeration{}", container_str(container_type)),
        };

        write!(formatter, "{}", result)
    }
}

impl FromStr for DisplayValueType {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        fn extract_container(input: &str) -> (ContainerType, &str) {
            if let Some(stripped_input) = input.strip_suffix("[]") {
                (ContainerType::Array, stripped_input)
            } else {
                (ContainerType::None, input)
            }
        }

        let (container, base_type) = extract_container(input);

        match base_type {
            "bool" | "boolean" => Ok(DisplayValueType::Bool(container)),
            "string" => Ok(DisplayValueType::String(container)),
            "bin" | "binary" => Ok(DisplayValueType::Binary(container)),
            "dec" | "decimal" => Ok(DisplayValueType::Decimal(container)),
            "hex" | "hexadecimal" => Ok(DisplayValueType::Hexadecimal(container)),
            "address" => Ok(DisplayValueType::Address(container)),
            "data_type_ref" => Ok(DisplayValueType::DataTypeRef(container)),
            "enumeration" => Ok(DisplayValueType::Enumeration(container)),
            _ => Err(()),
        }
    }
}
