use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScanCompareTypeRelative {
    Changed,
    Unchanged,
    Increased,
    Decreased,
}
