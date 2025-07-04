use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ContainerType {
    None,
    Array,
    Pointer,
}

impl fmt::Display for ContainerType {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let container_type_str = match &self {
            ContainerType::None => "",
            ContainerType::Array => "[]",
            ContainerType::Pointer => "*",
        };

        write!(formatter, "{}", container_type_str)
    }
}
