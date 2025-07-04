use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ContainerType {
    None,
    Array,
    Pointer,
}
