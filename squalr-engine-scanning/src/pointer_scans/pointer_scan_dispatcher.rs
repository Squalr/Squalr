use crate::pointer_scans::{
    search_kernels::{
        pointer_scan_scalar_binary_search_kernel::ScalarBinaryPointerScanKernel, pointer_scan_scalar_linear_search_kernel::ScalarLinearPointerScanKernel,
        pointer_scan_search_kernel::PointerScanSearchKernel, pointer_scan_simd_linear_search_kernel::SimdLinearPointerScanKernel,
    },
    structures::pointer_scan_target_ranges::PointerScanTargetRangeSet,
};
use squalr_engine_api::{
    registries::scan_rules::pointer_scan_rule_registry::PointerScanRuleRegistry,
    structures::{
        pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
        scanning::plans::pointer_scan::{
            planned_pointer_scan_kernel_kind::PlannedPointerScanKernelKind, pointer_scan_execution_plan::PointerScanExecutionPlan,
        },
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
    ) -> Box<dyn PointerScanSearchKernel + 'a> {
        match pointer_scan_execution_plan.get_planned_kernel_kind() {
            PlannedPointerScanKernelKind::ScalarLinear => Box::new(ScalarLinearPointerScanKernel::new(
                target_range_set,
                pointer_scan_execution_plan.get_pointer_size(),
            )),
            PlannedPointerScanKernelKind::ScalarBinary => Box::new(ScalarBinaryPointerScanKernel::new(
                target_range_set,
                pointer_scan_execution_plan.get_pointer_size(),
            )),
            PlannedPointerScanKernelKind::SimdLinear => Box::new(SimdLinearPointerScanKernel::new(
                target_range_set,
                pointer_scan_execution_plan.get_pointer_size(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PointerScanDispatcher;
    use crate::pointer_scans::structures::pointer_scan_target_ranges::PointerScanTargetRangeSet;
    use squalr_engine_api::structures::{
        pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
        scanning::plans::pointer_scan::{
            planned_pointer_scan_kernel_kind::PlannedPointerScanKernelKind, pointer_scan_execution_plan::PointerScanExecutionPlan,
        },
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

    #[test]
    fn pointer_scan_dispatcher_returns_concrete_kernel_that_scans_region() {
        let target_range_set = PointerScanTargetRangeSet::from_target_addresses(&[0x3000, 0x3010, 0x4000], 0x20);
        let mut current_values = Vec::new();
        current_values.extend_from_slice(&0x2FF0_u64.to_le_bytes());
        current_values.extend_from_slice(&0x3018_u64.to_le_bytes());
        current_values.extend_from_slice(&0x3500_u64.to_le_bytes());
        current_values.extend_from_slice(&0x4010_u64.to_le_bytes());

        for planned_kernel_kind in [
            PlannedPointerScanKernelKind::ScalarLinear,
            PlannedPointerScanKernelKind::ScalarBinary,
            PlannedPointerScanKernelKind::SimdLinear,
        ] {
            let mut pointer_scan_execution_plan = PointerScanExecutionPlan::new(PointerScanPointerSize::Pointer64, target_range_set.get_range_count(), 0x1000);
            pointer_scan_execution_plan.set_planned_kernel_kind(planned_kernel_kind);
            let range_search_kernel = PointerScanDispatcher::acquire_range_search_kernel(&target_range_set, &pointer_scan_execution_plan);
            let pointer_matches = range_search_kernel.scan_region(0x1000, &current_values);

            assert_eq!(pointer_matches.len(), 3, "Kernel {:?} returned an unexpected match count.", planned_kernel_kind);
            assert_eq!(pointer_matches[0].get_pointer_address(), 0x1000);
            assert_eq!(pointer_matches[1].get_pointer_address(), 0x1008);
            assert_eq!(pointer_matches[2].get_pointer_address(), 0x1018);
        }
    }
}
