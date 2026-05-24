use crate::{registries::symbols::symbol_registry::SymbolRegistry, structures::scanning::constraints::scan_constraint::ScanConstraint};

pub trait ElementScanParametersRule: Send + Sync {
    fn get_id(&self) -> &str;
    fn map_parameters(
        &self,
        symbol_registry: &SymbolRegistry,
        scan_constraints: &mut Vec<ScanConstraint>,
    );
}
