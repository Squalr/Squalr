use crate::structures::pointer_scans::pointer_scan_node_type::PointerScanNodeType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PointerScanNode {
    node_id: u64,
    parent_node_id: Option<u64>,
    pointer_scan_node_type: PointerScanNodeType,
    depth: u64,
    pointer_address: u64,
    pointer_value: u64,
    resolved_target_address: u64,
    pointer_offset: i64,
    module_name: String,
    module_offset: u64,
    child_node_ids: Vec<u64>,
}

impl PointerScanNode {
    pub fn new(
        node_id: u64,
        parent_node_id: Option<u64>,
        pointer_scan_node_type: PointerScanNodeType,
        depth: u64,
        pointer_address: u64,
        pointer_value: u64,
        resolved_target_address: u64,
        pointer_offset: i64,
        module_name: String,
        module_offset: u64,
        child_node_ids: Vec<u64>,
    ) -> Self {
        Self {
            node_id,
            parent_node_id,
            pointer_scan_node_type,
            depth,
            pointer_address,
            pointer_value,
            resolved_target_address,
            pointer_offset,
            module_name,
            module_offset,
            child_node_ids,
        }
    }

    pub fn get_node_id(&self) -> u64 {
        self.node_id
    }

    pub fn get_parent_node_id(&self) -> Option<u64> {
        self.parent_node_id
    }

    pub fn get_pointer_scan_node_type(&self) -> PointerScanNodeType {
        self.pointer_scan_node_type
    }

    pub fn get_depth(&self) -> u64 {
        self.depth
    }

    pub fn get_pointer_address(&self) -> u64 {
        self.pointer_address
    }

    pub fn get_pointer_value(&self) -> u64 {
        self.pointer_value
    }

    pub fn get_resolved_target_address(&self) -> u64 {
        self.resolved_target_address
    }

    pub fn get_pointer_offset(&self) -> i64 {
        self.pointer_offset
    }

    pub fn get_module_name(&self) -> &str {
        &self.module_name
    }

    pub fn get_module_offset(&self) -> u64 {
        self.module_offset
    }

    pub fn get_child_node_ids(&self) -> &Vec<u64> {
        &self.child_node_ids
    }

    pub fn has_children(&self) -> bool {
        !self.child_node_ids.is_empty()
    }
}
