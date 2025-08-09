use crate::registries::{
    data_types::data_type_registry::DataTypeRegistry, freeze_list::freeze_list_registry::FreezeListRegistry,
    project_item_types::project_item_type_registry::ProjectItemTypeRegistry, scan_rules::element_scan_rule_registry::ElementScanRuleRegistry,
    symbols::symbolic_struct_registry::SymbolicStructDefinitionRegistry,
};
use std::sync::{Arc, RwLock};

pub struct Registries {
    // The list of frozen scan results.
    freeze_list_registry: Arc<RwLock<FreezeListRegistry>>,

    /// The registry for data types.
    data_type_registry: Arc<RwLock<DataTypeRegistry>>,

    /// The registry for project item types.
    project_item_type_registry: Arc<RwLock<ProjectItemTypeRegistry>>,

    /// The registry for element scan rules.
    element_scan_rule_registry: Arc<RwLock<ElementScanRuleRegistry>>,

    /// The registry for symbolic struct definitions.
    symbolic_struct_definition_registry: Arc<RwLock<SymbolicStructDefinitionRegistry>>,
}

impl Registries {
    pub fn new() -> Self {
        let freeze_list_registry = Arc::new(RwLock::new(FreezeListRegistry::new()));
        let data_type_registry = Arc::new(RwLock::new(DataTypeRegistry::new()));
        let project_item_type_registry = Arc::new(RwLock::new(ProjectItemTypeRegistry::new()));
        let element_scan_rule_registry = Arc::new(RwLock::new(ElementScanRuleRegistry::new()));
        let symbolic_struct_definition_registry = Arc::new(RwLock::new(SymbolicStructDefinitionRegistry::new()));

        Self {
            freeze_list_registry,
            data_type_registry,
            project_item_type_registry,
            element_scan_rule_registry,
            symbolic_struct_definition_registry,
        }
    }

    /// Gets the registry for the list of addresses that have been marked as frozen.
    pub fn get_freeze_list_registry(&self) -> Arc<RwLock<FreezeListRegistry>> {
        self.freeze_list_registry.clone()
    }

    /// Gets the registry for data types.
    pub fn get_data_type_registry(&self) -> Arc<RwLock<DataTypeRegistry>> {
        self.data_type_registry.clone()
    }

    /// Gets the registry for project item types.
    pub fn get_project_item_type_registry(&self) -> Arc<RwLock<ProjectItemTypeRegistry>> {
        self.project_item_type_registry.clone()
    }

    /// Gets the registry for element scan rules.
    pub fn get_element_scan_rule_registry(&self) -> Arc<RwLock<ElementScanRuleRegistry>> {
        self.element_scan_rule_registry.clone()
    }

    /// Gets the registry for symbolic struct definitions.
    pub fn get_symbolic_struct_definition_registry(&self) -> Arc<RwLock<SymbolicStructDefinitionRegistry>> {
        self.symbolic_struct_definition_registry.clone()
    }
}
