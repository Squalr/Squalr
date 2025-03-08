use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScanMemoryReadMode {
    Skip,
    ReadBeforeScan,
    ReadInterleavedWithScan,
}

impl Default for ScanMemoryReadMode {
    fn default() -> Self {
        ScanMemoryReadMode::ReadBeforeScan
    }
}

impl FromStr for ScanMemoryReadMode {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "s" => Ok(ScanMemoryReadMode::Skip),
            "b" => Ok(ScanMemoryReadMode::ReadBeforeScan),
            "i" => Ok(ScanMemoryReadMode::ReadInterleavedWithScan),
            _ => Err("Unknown memory reading mode.".to_string()),
        }
    }
}
