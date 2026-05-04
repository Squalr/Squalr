use crate::structures::{
    pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
    scanning::plans::pointer_scan::planned_pointer_scan_kernel_kind::PlannedPointerScanKernelKind,
};

/// Represents the execution plan for a single pointer-scan traversal pass.
#[derive(Clone, Debug)]
pub struct PointerScanExecutionPlan {
    pointer_size: PointerScanPointerSize,
    frontier_target_range_count: usize,
    scan_region_byte_count: usize,
    planned_kernel_kind: PlannedPointerScanKernelKind,
}

impl PointerScanExecutionPlan {
    pub fn new(
        pointer_size: PointerScanPointerSize,
        frontier_target_range_count: usize,
        scan_region_byte_count: usize,
    ) -> Self {
        Self {
            pointer_size,
            frontier_target_range_count,
            scan_region_byte_count,
            planned_kernel_kind: PlannedPointerScanKernelKind::ScalarBinary,
        }
    }

    pub fn get_pointer_size(&self) -> PointerScanPointerSize {
        self.pointer_size
    }

    pub fn get_frontier_target_range_count(&self) -> usize {
        self.frontier_target_range_count
    }

    pub fn get_scan_region_byte_count(&self) -> usize {
        self.scan_region_byte_count
    }

    pub fn get_planned_kernel_kind(&self) -> PlannedPointerScanKernelKind {
        self.planned_kernel_kind
    }

    pub fn set_planned_kernel_kind(
        &mut self,
        planned_kernel_kind: PlannedPointerScanKernelKind,
    ) {
        self.planned_kernel_kind = planned_kernel_kind;
    }
}
