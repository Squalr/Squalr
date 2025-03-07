use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemoryReadMode {
    Skip,
    ReadBeforeScan,
    ReadInterleavedWithScan,
}

impl Default for MemoryReadMode {
    fn default() -> Self {
        MemoryReadMode::ReadBeforeScan
    }
}

impl FromStr for MemoryReadMode {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "s" => Ok(MemoryReadMode::Skip),
            "b" => Ok(MemoryReadMode::ReadBeforeScan),
            "i" => Ok(MemoryReadMode::ReadInterleavedWithScan),
            _ => Err("Unknown memory reading mode.".to_string()),
        }
    }
}
