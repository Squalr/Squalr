use crate::structures::scanning::{
    plans::pointer_scan::{planned_pointer_scan_kernel_kind::PlannedPointerScanKernelKind, pointer_scan_execution_plan::PointerScanExecutionPlan},
    rules::pointer_scan_planning_rule::PointerScanPlanningRule,
};

pub struct RuleMapSearchKernel;

impl RuleMapSearchKernel {
    pub const RULE_ID: &str = "map_search_kernel";

    const SIMD_LINEAR_RANGE_THRESHOLD: usize = 8;
    const SCALAR_LINEAR_RANGE_THRESHOLD: usize = 64;
}

impl PointerScanPlanningRule for RuleMapSearchKernel {
    fn get_id(&self) -> &str {
        Self::RULE_ID
    }

    fn map_plan(
        &self,
        pointer_scan_execution_plan: &mut PointerScanExecutionPlan,
    ) {
        let planned_kernel_kind = if pointer_scan_execution_plan.get_frontier_target_range_count() <= Self::SIMD_LINEAR_RANGE_THRESHOLD {
            PlannedPointerScanKernelKind::SimdLinear
        } else if pointer_scan_execution_plan.get_frontier_target_range_count() <= Self::SCALAR_LINEAR_RANGE_THRESHOLD {
            PlannedPointerScanKernelKind::ScalarLinear
        } else {
            PlannedPointerScanKernelKind::ScalarBinary
        };

        pointer_scan_execution_plan.set_planned_kernel_kind(planned_kernel_kind);
    }
}

#[cfg(test)]
mod tests {
    use super::RuleMapSearchKernel;
    use crate::structures::{
        pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
        scanning::{
            plans::pointer_scan::{planned_pointer_scan_kernel_kind::PlannedPointerScanKernelKind, pointer_scan_execution_plan::PointerScanExecutionPlan},
            rules::pointer_scan_planning_rule::PointerScanPlanningRule,
        },
    };

    #[test]
    fn rule_map_search_kernel_selects_simd_linear_for_small_frontiers() {
        let mut pointer_scan_execution_plan = PointerScanExecutionPlan::new(PointerScanPointerSize::Pointer64, 8, 1024);

        RuleMapSearchKernel.map_plan(&mut pointer_scan_execution_plan);

        assert_eq!(pointer_scan_execution_plan.get_planned_kernel_kind(), PlannedPointerScanKernelKind::SimdLinear);
    }

    #[test]
    fn rule_map_search_kernel_selects_scalar_linear_for_medium_frontiers() {
        let mut pointer_scan_execution_plan = PointerScanExecutionPlan::new(PointerScanPointerSize::Pointer64, 32, 1024);

        RuleMapSearchKernel.map_plan(&mut pointer_scan_execution_plan);

        assert_eq!(
            pointer_scan_execution_plan.get_planned_kernel_kind(),
            PlannedPointerScanKernelKind::ScalarLinear
        );
    }

    #[test]
    fn rule_map_search_kernel_selects_scalar_binary_for_large_frontiers() {
        let mut pointer_scan_execution_plan = PointerScanExecutionPlan::new(PointerScanPointerSize::Pointer64, 128, 1024);

        RuleMapSearchKernel.map_plan(&mut pointer_scan_execution_plan);

        assert_eq!(
            pointer_scan_execution_plan.get_planned_kernel_kind(),
            PlannedPointerScanKernelKind::ScalarBinary
        );
    }
}
