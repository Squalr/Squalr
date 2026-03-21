use crate::structures::pointer_scans::pointer_scan_node_type::PointerScanNodeType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PointerScanNode {
    node_id: u64,
    graph_node_id: u64,
    discovery_depth: u64,
    branch_total_depth: u64,
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
    has_children: bool,
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
        let has_children = !child_node_ids.is_empty();

        Self {
            node_id,
            graph_node_id: node_id,
            discovery_depth: depth,
            branch_total_depth: depth,
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
            has_children,
        }
    }

    pub fn new_materialized(
        node_id: u64,
        graph_node_id: u64,
        discovery_depth: u64,
        branch_total_depth: u64,
        depth: u64,
        parent_node_id: Option<u64>,
        pointer_scan_node_type: PointerScanNodeType,
        pointer_address: u64,
        pointer_value: u64,
        resolved_target_address: u64,
        pointer_offset: i64,
        module_name: String,
        module_offset: u64,
        has_children: bool,
    ) -> Self {
        Self {
            node_id,
            graph_node_id,
            discovery_depth,
            branch_total_depth,
            parent_node_id,
            pointer_scan_node_type,
            depth,
            pointer_address,
            pointer_value,
            resolved_target_address,
            pointer_offset,
            module_name,
            module_offset,
            child_node_ids: Vec::new(),
            has_children,
        }
    }

    pub fn get_node_id(&self) -> u64 {
        self.node_id
    }

    pub fn get_graph_node_id(&self) -> u64 {
        self.graph_node_id
    }

    pub fn get_discovery_depth(&self) -> u64 {
        self.discovery_depth
    }

    pub fn get_branch_total_depth(&self) -> u64 {
        self.branch_total_depth
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

    pub fn set_depth(
        &mut self,
        depth: u64,
    ) {
        self.depth = depth;
    }

    pub fn set_discovery_depth(
        &mut self,
        discovery_depth: u64,
    ) {
        self.discovery_depth = discovery_depth;
    }

    pub fn set_branch_total_depth(
        &mut self,
        branch_total_depth: u64,
    ) {
        self.branch_total_depth = branch_total_depth;
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

    pub fn set_child_node_ids(
        &mut self,
        child_node_ids: Vec<u64>,
    ) {
        self.has_children = !child_node_ids.is_empty();
        self.child_node_ids = child_node_ids;
    }

    pub fn has_children(&self) -> bool {
        self.has_children
    }
}
