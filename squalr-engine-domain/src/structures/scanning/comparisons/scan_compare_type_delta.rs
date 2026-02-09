use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScanCompareTypeDelta {
    IncreasedByX,
    DecreasedByX,
    MultipliedByX,
    DividedByX,
    ModuloByX,
    ShiftLeftByX,
    ShiftRightByX,
    LogicalAndByX,
    LogicalOrByX,
    LogicalXorByX,
}
