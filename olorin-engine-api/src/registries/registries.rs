use crate::registries::{
    data_types::data_type_registry::DataTypeRegistry, freeze_list::freeze_list_registry::FreezeListRegistry,
    project_item_types::project_item_type_registry::ProjectItemTypeRegistry, scan_rules::element_scan_rule_registry::ElementScanRuleRegistry,
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
}

impl Registries {
    pub fn new() -> Self {
        let freeze_list_registry = Arc::new(RwLock::new(FreezeListRegistry::new()));
        let data_type_registry = Arc::new(RwLock::new(DataTypeRegistry::new()));
        let project_item_type_registry = Arc::new(RwLock::new(ProjectItemTypeRegistry::new()));
        let element_scan_rule_registry = Arc::new(RwLock::new(ElementScanRuleRegistry::new()));

        Self {
            freeze_list_registry,
            data_type_registry,
            project_item_type_registry,
            element_scan_rule_registry,
        }
    }

    /// Gets the registry for the list of addresses that have been marked as frozen.
    pub fn get_freeze_list_registry(&self) -> Arc<RwLock<FreezeListRegistry>> {
        self.freeze_list_registry.clone()
    }

    /// Gets registry for data types.
    pub fn get_data_type_registry(&self) -> Arc<RwLock<DataTypeRegistry>> {
        self.data_type_registry.clone()
    }

    /// Gets registry for project item types.
    pub fn get_project_item_type_registry(&self) -> Arc<RwLock<ProjectItemTypeRegistry>> {
        self.project_item_type_registry.clone()
    }

    /// Gets registry for element scan rules.
    pub fn get_element_scan_rule_registry(&self) -> Arc<RwLock<ElementScanRuleRegistry>> {
        self.element_scan_rule_registry.clone()
    }
}
