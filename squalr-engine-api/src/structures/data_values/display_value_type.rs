use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DisplayContainer {
    None,
    Array,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DisplayValueType {
    Bool(DisplayContainer),
    String(DisplayContainer),
    Binary(DisplayContainer),
    Decimal(DisplayContainer),
    Hexadecimal(DisplayContainer),
    Address(DisplayContainer),
    DataTypeRef(DisplayContainer),
    Enumeration,
}

impl fmt::Display for DisplayValueType {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        fn container_str(container: &DisplayContainer) -> &'static str {
            match container {
                DisplayContainer::None => "",
                DisplayContainer::Array => "[]",
            }
        }

        let result = match self {
            DisplayValueType::Bool(display_container) => format!("bool{}", container_str(display_container)),
            DisplayValueType::String(display_container) => format!("string{}", container_str(display_container)),
            DisplayValueType::Binary(display_container) => format!("bin{}", container_str(display_container)),
            DisplayValueType::Decimal(display_container) => format!("dec{}", container_str(display_container)),
            DisplayValueType::Hexadecimal(display_container) => format!("hex{}", container_str(display_container)),
            DisplayValueType::Address(display_container) => format!("address{}", container_str(display_container)),
            DisplayValueType::DataTypeRef(display_container) => format!("data_type_ref{}", container_str(display_container)),
            DisplayValueType::Enumeration => "enumeration".to_string(),
        };

        write!(formatter, "{}", result)
    }
}

impl FromStr for DisplayValueType {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        fn extract_container(input: &str) -> (DisplayContainer, &str) {
            if let Some(stripped_input) = input.strip_suffix("[]") {
                (DisplayContainer::Array, stripped_input)
            } else {
                (DisplayContainer::None, input)
            }
        }

        let (container, base_type) = extract_container(input);

        match base_type {
            "bool" => Ok(DisplayValueType::Bool(container)),
            "string" => Ok(DisplayValueType::String(container)),
            "bin" => Ok(DisplayValueType::Binary(container)),
            "dec" => Ok(DisplayValueType::Decimal(container)),
            "hex" => Ok(DisplayValueType::Hexadecimal(container)),
            "address" => Ok(DisplayValueType::Address(container)),
            "data_type_ref" => Ok(DisplayValueType::DataTypeRef(container)),
            "enumeration" => Ok(DisplayValueType::Enumeration),
            _ => Err(()),
        }
    }
}
