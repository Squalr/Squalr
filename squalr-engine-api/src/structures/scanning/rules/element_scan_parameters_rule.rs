use crate::structures::scanning::constraints::scan_constraint::ScanConstraint;

pub trait ElementScanParametersRule: Send + Sync {
    fn get_id(&self) -> &str;
    fn map_parameters(
        &self,
        scan_constraints: &mut Vec<ScanConstraint>,
    );
}
