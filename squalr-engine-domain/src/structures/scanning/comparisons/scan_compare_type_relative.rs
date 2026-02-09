use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScanCompareTypeRelative {
    Changed,
    Unchanged,
    Increased,
    Decreased,
}
