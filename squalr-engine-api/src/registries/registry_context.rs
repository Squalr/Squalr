use crate::registries::freeze_list::freeze_list_registry::FreezeListRegistry;
use crate::registries::project_item_types::project_item_type_registry::ProjectItemTypeRegistry;
use crate::registries::scan_rules::element_scan_rule_registry::ElementScanRuleRegistry;
use crate::registries::symbols::symbol_registry::SymbolRegistry;
use std::sync::{Arc, RwLock};

/// Describes registry access required by API-level structures.
pub trait RegistryContext {
    fn get_freeze_list_registry(&self) -> Arc<RwLock<FreezeListRegistry>>;
    fn get_project_item_type_registry(&self) -> Arc<RwLock<ProjectItemTypeRegistry>>;
    fn get_element_scan_rule_registry(&self) -> Arc<RwLock<ElementScanRuleRegistry>>;
    fn get_symbol_registry(&self) -> Arc<RwLock<SymbolRegistry>>;
}
