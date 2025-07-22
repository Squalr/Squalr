use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanResultRef {
    scan_result_index: u64,
}

impl ScanResultRef {
    pub fn new(scan_result_index: u64) -> Self {
        Self { scan_result_index }
    }

    pub fn get_scan_result_index(&self) -> u64 {
        self.scan_result_index
    }
}

impl FromStr for ScanResultRef {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let scan_result_index = string.parse::<u64>().map_err(|error| error.to_string())?;

        Ok(ScanResultRef { scan_result_index })
    }
}
