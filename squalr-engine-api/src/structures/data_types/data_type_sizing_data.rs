use serde::{Deserialize, Serialize};

/// Represents the size of an associated data type.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DataTypeSizingData {
    FixedSize,
    VariableSize(Option<u64>),
}
