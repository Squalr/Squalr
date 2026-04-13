use crate::pointer_scans::search_kernels::pointer_scan_scalar_binary_search_kernel::scan_region_scalar_binary;
use crate::pointer_scans::search_kernels::pointer_scan_scalar_linear_search_kernel::scan_region_scalar_linear;
use crate::pointer_scans::search_kernels::pointer_scan_simd_linear_search_kernel::scan_region_simd_linear;
pub use crate::pointer_scans::structures::pointer_scan_region_match::PointerScanRegionMatch;
use crate::pointer_scans::structures::pointer_scan_target_ranges::PointerScanTargetRangeSet;
use squalr_engine_api::structures::{
    pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
    scanning::plans::pointer_scan::{planned_pointer_scan_kernel_kind::PlannedPointerScanKernelKind, pointer_scan_execution_plan::PointerScanExecutionPlan},
};

pub struct PointerScanRangeSearchKernel<'a> {
    target_range_set: &'a PointerScanTargetRangeSet,
    pointer_size: PointerScanPointerSize,
    kernel_kind: PlannedPointerScanKernelKind,
}

impl<'a> PointerScanRangeSearchKernel<'a> {
    pub fn from_execution_plan(
        target_range_set: &'a PointerScanTargetRangeSet,
        pointer_scan_execution_plan: &PointerScanExecutionPlan,
    ) -> Self {
        Self {
            target_range_set,
            pointer_size: pointer_scan_execution_plan.get_pointer_size(),
            kernel_kind: pointer_scan_execution_plan.get_planned_kernel_kind(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.target_range_set.is_empty()
    }

    #[cfg(test)]
    pub fn scan_region(
        &self,
        base_address: u64,
        current_values: &[u8],
    ) -> Vec<PointerScanRegionMatch> {
        let mut pointer_matches = Vec::new();
        self.scan_region_with_visitor(base_address, current_values, |pointer_match| pointer_matches.push(pointer_match));

        pointer_matches
    }

    pub fn scan_region_with_visitor<VisitMatch>(
        &self,
        base_address: u64,
        current_values: &[u8],
        mut visit_match: VisitMatch,
    ) where
        VisitMatch: FnMut(PointerScanRegionMatch),
    {
        let Some(start_offset) = self.find_scan_start_offset(base_address, current_values) else {
            return;
        };

        match self.kernel_kind {
            PlannedPointerScanKernelKind::ScalarLinear => scan_region_scalar_linear(
                base_address,
                current_values,
                start_offset,
                self.pointer_size,
                self.target_range_set,
                &mut visit_match,
            ),
            PlannedPointerScanKernelKind::ScalarBinary => scan_region_scalar_binary(
                base_address,
                current_values,
                start_offset,
                self.pointer_size,
                self.target_range_set,
                &mut visit_match,
            ),
            PlannedPointerScanKernelKind::SimdLinear => scan_region_simd_linear(
                base_address,
                current_values,
                start_offset,
                self.pointer_size,
                self.target_range_set,
                &mut visit_match,
            ),
        }
    }

    fn find_scan_start_offset(
        &self,
        base_address: u64,
        current_values: &[u8],
    ) -> Option<usize> {
        let pointer_size_in_bytes = self.pointer_size.get_size_in_bytes() as usize;

        if current_values.len() < pointer_size_in_bytes {
            return None;
        }

        let pointer_alignment = pointer_size_in_bytes as u64;
        let alignment_remainder = base_address % pointer_alignment;
        let start_offset = if alignment_remainder == 0 {
            0_usize
        } else {
            pointer_alignment.saturating_sub(alignment_remainder) as usize
        };

        (start_offset.saturating_add(pointer_size_in_bytes) <= current_values.len()).then_some(start_offset)
    }
}

#[cfg(test)]
mod tests {
    use super::PointerScanRangeSearchKernel;
    use crate::pointer_scans::structures::pointer_scan_target_ranges::PointerScanTargetRangeSet;
    use squalr_engine_api::structures::{
        pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
        scanning::plans::pointer_scan::{
            planned_pointer_scan_kernel_kind::PlannedPointerScanKernelKind, pointer_scan_execution_plan::PointerScanExecutionPlan,
        },
    };

    #[test]
    fn range_search_kernel_scans_region_against_merged_ranges() {
        let target_range_set = PointerScanTargetRangeSet::from_target_addresses(&[0x3000, 0x3010, 0x4000], 0x20);
        let pointer_scan_execution_plan = PointerScanExecutionPlan::new(PointerScanPointerSize::Pointer64, target_range_set.get_range_count(), 0x1000);
        let search_kernel = PointerScanRangeSearchKernel::from_execution_plan(&target_range_set, &pointer_scan_execution_plan);
        let mut current_values = Vec::new();

        current_values.extend_from_slice(&0x2FF0_u64.to_le_bytes());
        current_values.extend_from_slice(&0x3018_u64.to_le_bytes());
        current_values.extend_from_slice(&0x3500_u64.to_le_bytes());
        current_values.extend_from_slice(&0x4010_u64.to_le_bytes());

        let pointer_matches = search_kernel.scan_region(0x1000, &current_values);

        assert_eq!(pointer_matches.len(), 3);
        assert_eq!(pointer_matches[0].get_pointer_address(), 0x1000);
        assert_eq!(pointer_matches[1].get_pointer_address(), 0x1008);
        assert_eq!(pointer_matches[2].get_pointer_address(), 0x1018);
    }

    #[test]
    fn range_search_kernel_scalar_variants_scan_region_against_merged_ranges() {
        let target_range_set = PointerScanTargetRangeSet::from_target_addresses(&[0x3000, 0x3010, 0x4000], 0x20);
        let mut current_values = Vec::new();

        current_values.extend_from_slice(&0x2FF0_u64.to_le_bytes());
        current_values.extend_from_slice(&0x3018_u64.to_le_bytes());
        current_values.extend_from_slice(&0x3500_u64.to_le_bytes());
        current_values.extend_from_slice(&0x4010_u64.to_le_bytes());

        for planned_kernel_kind in [
            PlannedPointerScanKernelKind::ScalarLinear,
            PlannedPointerScanKernelKind::ScalarBinary,
        ] {
            let mut pointer_scan_execution_plan = PointerScanExecutionPlan::new(PointerScanPointerSize::Pointer64, target_range_set.get_range_count(), 0x1000);
            pointer_scan_execution_plan.set_planned_kernel_kind(planned_kernel_kind);
            let search_kernel = PointerScanRangeSearchKernel::from_execution_plan(&target_range_set, &pointer_scan_execution_plan);
            let pointer_matches = search_kernel.scan_region(0x1000, &current_values);

            assert_eq!(pointer_matches.len(), 3, "Kernel {:?} returned an unexpected match count.", planned_kernel_kind);
            assert_eq!(pointer_matches[0].get_pointer_address(), 0x1000);
            assert_eq!(pointer_matches[1].get_pointer_address(), 0x1008);
            assert_eq!(pointer_matches[2].get_pointer_address(), 0x1018);
        }
    }
}
