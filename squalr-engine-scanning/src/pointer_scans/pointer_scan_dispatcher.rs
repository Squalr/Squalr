use crate::pointer_scans::{
    search_kernels::pointer_scan_range_search_kernel::PointerScanRangeSearchKernel, structures::pointer_scan_target_ranges::PointerScanTargetRangeSet,
};
use squalr_engine_api::{
    registries::scan_rules::pointer_scan_rule_registry::PointerScanRuleRegistry,
    structures::{
        pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize, scanning::plans::pointer_scan::pointer_scan_execution_plan::PointerScanExecutionPlan,
    },
};

pub(crate) struct PointerScanDispatcher;

impl PointerScanDispatcher {
    pub(crate) fn build_execution_plan(
        target_range_set: &PointerScanTargetRangeSet,
        pointer_size: PointerScanPointerSize,
        scan_region_byte_count: usize,
    ) -> PointerScanExecutionPlan {
        let mut pointer_scan_execution_plan = PointerScanExecutionPlan::new(pointer_size, target_range_set.get_range_count(), scan_region_byte_count);

        for (_rule_id, pointer_scan_planning_rule) in PointerScanRuleRegistry::get_instance()
            .get_pointer_scan_planning_rule_registry()
            .iter()
        {
            pointer_scan_planning_rule.map_plan(&mut pointer_scan_execution_plan);
        }

        pointer_scan_execution_plan
    }

    pub(crate) fn acquire_range_search_kernel<'a>(
        target_range_set: &'a PointerScanTargetRangeSet,
        pointer_scan_execution_plan: &PointerScanExecutionPlan,
    ) -> PointerScanRangeSearchKernel<'a> {
        PointerScanRangeSearchKernel::from_execution_plan(target_range_set, pointer_scan_execution_plan)
    }
}

#[cfg(test)]
mod tests {
    use super::PointerScanDispatcher;
    use crate::pointer_scans::structures::pointer_scan_target_ranges::PointerScanTargetRangeSet;
    use squalr_engine_api::structures::{
        pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
        scanning::plans::pointer_scan::planned_pointer_scan_kernel_kind::PlannedPointerScanKernelKind,
    };

    #[test]
    fn pointer_scan_dispatcher_builds_simd_linear_plan_for_small_range_sets() {
        let target_range_set = PointerScanTargetRangeSet::from_target_addresses(&[0x1000, 0x2000], 0x20);
        let pointer_scan_execution_plan = PointerScanDispatcher::build_execution_plan(&target_range_set, PointerScanPointerSize::Pointer64, 0x1000);

        assert_eq!(pointer_scan_execution_plan.get_planned_kernel_kind(), PlannedPointerScanKernelKind::SimdLinear);
    }

    #[test]
    fn pointer_scan_dispatcher_builds_scalar_binary_plan_for_large_range_sets() {
        let target_addresses = (0_u64..128)
            .map(|target_index| 0x1000_0000_u64.saturating_add(target_index.saturating_mul(0x20_000)))
            .collect::<Vec<_>>();
        let target_range_set = PointerScanTargetRangeSet::from_target_addresses(&target_addresses, 0x20);
        let pointer_scan_execution_plan = PointerScanDispatcher::build_execution_plan(&target_range_set, PointerScanPointerSize::Pointer64, 0x1000);

        assert_eq!(
            pointer_scan_execution_plan.get_planned_kernel_kind(),
            PlannedPointerScanKernelKind::ScalarBinary
        );
    }
}
