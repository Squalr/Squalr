use crate::structures::pointer_scans::pointer_scan_level_summary::PointerScanLevelSummary;
use crate::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use crate::structures::pointer_scans::pointer_scan_target_descriptor::PointerScanTargetDescriptor;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct PointerScanSummary {
    session_id: u64,
    target_descriptor: PointerScanTargetDescriptor,
    pointer_size: PointerScanPointerSize,
    max_depth: u64,
    offset_radius: u64,
    root_node_count: u64,
    total_node_count: u64,
    total_static_node_count: u64,
    total_heap_node_count: u64,
    pointer_scan_level_summaries: Vec<PointerScanLevelSummary>,
}

impl PointerScanSummary {
    pub fn new(
        session_id: u64,
        target_descriptor: PointerScanTargetDescriptor,
        pointer_size: PointerScanPointerSize,
        max_depth: u64,
        offset_radius: u64,
        root_node_count: u64,
        total_node_count: u64,
        total_static_node_count: u64,
        total_heap_node_count: u64,
        pointer_scan_level_summaries: Vec<PointerScanLevelSummary>,
    ) -> Self {
        Self {
            session_id,
            target_descriptor,
            pointer_size,
            max_depth,
            offset_radius,
            root_node_count,
            total_node_count,
            total_static_node_count,
            total_heap_node_count,
            pointer_scan_level_summaries,
        }
    }

    pub fn get_session_id(&self) -> u64 {
        self.session_id
    }

    pub fn get_target_descriptor(&self) -> &PointerScanTargetDescriptor {
        &self.target_descriptor
    }

    pub fn get_pointer_size(&self) -> PointerScanPointerSize {
        self.pointer_size
    }

    pub fn get_max_depth(&self) -> u64 {
        self.max_depth
    }

    pub fn get_offset_radius(&self) -> u64 {
        self.offset_radius
    }

    pub fn get_root_node_count(&self) -> u64 {
        self.root_node_count
    }

    pub fn get_total_node_count(&self) -> u64 {
        self.total_node_count
    }

    pub fn get_total_static_node_count(&self) -> u64 {
        self.total_static_node_count
    }

    pub fn get_total_heap_node_count(&self) -> u64 {
        self.total_heap_node_count
    }

    pub fn get_pointer_scan_level_summaries(&self) -> &Vec<PointerScanLevelSummary> {
        &self.pointer_scan_level_summaries
    }
}
