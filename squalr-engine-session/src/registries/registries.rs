use squalr_engine_api::registries::freeze_list::freeze_list_registry::FreezeListRegistry;
use squalr_engine_api::registries::project_item_types::project_item_type_registry::ProjectItemTypeRegistry;
use squalr_engine_api::registries::registry_context::RegistryContext;
use squalr_engine_api::registries::scan_rules::element_scan_rule_registry::ElementScanRuleRegistry;
use squalr_engine_api::registries::symbols::symbol_registry::SymbolRegistry;
use std::sync::{Arc, RwLock};

pub struct Registries {
    // The list of frozen scan results.
    freeze_list_registry: Arc<RwLock<FreezeListRegistry>>,

    /// The registry for project item types.
    project_item_type_registry: Arc<RwLock<ProjectItemTypeRegistry>>,

    /// The registry for element scan rules.
    element_scan_rule_registry: Arc<RwLock<ElementScanRuleRegistry>>,

    /// The registry for symbolic struct definitions.
    symbol_registry: Arc<RwLock<SymbolRegistry>>,
}

impl Registries {
    pub fn new() -> Self {
        let freeze_list_registry = Arc::new(RwLock::new(FreezeListRegistry::new()));
        let project_item_type_registry = Arc::new(RwLock::new(ProjectItemTypeRegistry::new()));
        let element_scan_rule_registry = Arc::new(RwLock::new(ElementScanRuleRegistry::new()));
        let symbol_registry = Arc::new(RwLock::new(SymbolRegistry::new()));

        Self {
            freeze_list_registry,
            project_item_type_registry,
            element_scan_rule_registry,
            symbol_registry,
        }
    }
}

impl RegistryContext for Registries {
    /// Gets the registry for the list of addresses that have been marked as frozen.
    fn get_freeze_list_registry(&self) -> Arc<RwLock<FreezeListRegistry>> {
        self.freeze_list_registry.clone()
    }

    /// Gets the registry for project item types.
    fn get_project_item_type_registry(&self) -> Arc<RwLock<ProjectItemTypeRegistry>> {
        self.project_item_type_registry.clone()
    }

    /// Gets the registry for element scan rules.
    fn get_element_scan_rule_registry(&self) -> Arc<RwLock<ElementScanRuleRegistry>> {
        self.element_scan_rule_registry.clone()
    }

    /// Gets the registry for symbolic struct definitions.
    fn get_symbol_registry(&self) -> Arc<RwLock<SymbolRegistry>> {
        self.symbol_registry.clone()
    }
}
