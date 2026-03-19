use crate::structures::pointer_scans::pointer_scan_node_type::PointerScanNodeType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PointerScanCandidate {
    candidate_id: u64,
    discovery_depth: u64,
    pointer_scan_node_type: PointerScanNodeType,
    pointer_address: u64,
    pointer_value: u64,
    module_name: String,
    module_offset: u64,
}

impl PointerScanCandidate {
    pub fn new(
        candidate_id: u64,
        discovery_depth: u64,
        pointer_scan_node_type: PointerScanNodeType,
        pointer_address: u64,
        pointer_value: u64,
        module_name: String,
        module_offset: u64,
    ) -> Self {
        Self {
            candidate_id,
            discovery_depth,
            pointer_scan_node_type,
            pointer_address,
            pointer_value,
            module_name,
            module_offset,
        }
    }

    pub fn get_candidate_id(&self) -> u64 {
        self.candidate_id
    }

    pub fn get_discovery_depth(&self) -> u64 {
        self.discovery_depth
    }

    pub fn get_pointer_scan_node_type(&self) -> PointerScanNodeType {
        self.pointer_scan_node_type
    }

    pub fn get_pointer_address(&self) -> u64 {
        self.pointer_address
    }

    pub fn get_pointer_value(&self) -> u64 {
        self.pointer_value
    }

    pub fn get_module_name(&self) -> &str {
        &self.module_name
    }

    pub fn get_module_offset(&self) -> u64 {
        self.module_offset
    }
}
