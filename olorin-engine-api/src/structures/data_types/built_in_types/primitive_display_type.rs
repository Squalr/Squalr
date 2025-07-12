use serde::{Deserialize, Serialize};

/// Represents the string encoding supported in scans.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PrimitiveDisplayType {
    Normal,
    AsHex,
    AsAddress,
}
