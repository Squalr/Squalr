use crate::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MemoryFreezeTarget {
    pub address: u64,
    pub module_name: String,
    pub data_type_id: String,
    #[serde(default)]
    pub pointer_offsets: Vec<i64>,
    #[serde(default)]
    pub pointer_size: PointerScanPointerSize,
}
