use crate::structures::scanning::plans::pointer_scan::pointer_scan_execution_plan::PointerScanExecutionPlan;

pub trait PointerScanPlanningRule: Send + Sync {
    fn get_id(&self) -> &str;
    fn map_plan(
        &self,
        pointer_scan_execution_plan: &mut PointerScanExecutionPlan,
    );
}
