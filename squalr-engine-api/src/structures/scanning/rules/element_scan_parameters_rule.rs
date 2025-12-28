use crate::{registries::symbols::symbol_registry::SymbolRegistry, structures::scanning::plans::element_scan::element_scan_parameters::ElementScanParameters};

pub trait ElementScanParametersRule: Send + Sync {
    fn get_id(&self) -> &str;
    fn map_parameters(
        &self,
        symbol_registry: &SymbolRegistry,
        element_scan_parameters: &mut ElementScanParameters,
    );
}
