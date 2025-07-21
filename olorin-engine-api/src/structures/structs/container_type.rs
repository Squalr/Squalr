use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub enum ContainerType {
    #[default]
    None,
    Array(u64),
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
            ContainerType::Array(length) => format!("[{}]", length).to_string(),
            ContainerType::Pointer32 => "*(32)".to_string(),
            ContainerType::Pointer64 => "*(64)".to_string(),
        };

        write!(formatter, "{}", container_type_str)
    }
}
