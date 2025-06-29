use crate::{
    registries::scan_rules::element_scan_mapping_rule::ElementScanMappingRule,
    structures::scanning::parameters::element_scan::built_in_rules::map_unsigned_greater_than_zero_to_not_equal::MapUnsignedGreaterThanZeroToNotEqual,
};
use std::{
    collections::HashMap,
    sync::{Arc, Once, RwLock},
};

pub struct ElementScanRuleRegistry {
    registry: RwLock<HashMap<String, Arc<dyn ElementScanMappingRule>>>,
}

impl ElementScanRuleRegistry {
    pub fn get_instance() -> &'static ElementScanRuleRegistry {
        static mut INSTANCE: Option<ElementScanRuleRegistry> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = ElementScanRuleRegistry::new();
                INSTANCE = Some(instance);
            });

            #[allow(static_mut_refs)]
            INSTANCE.as_ref().unwrap_unchecked()
        }
    }

    fn new() -> Self {
        Self {
            registry: Self::create_built_in_types(),
        }
    }

    pub fn get_registry(&self) -> &RwLock<HashMap<String, Arc<dyn ElementScanMappingRule>>> {
        &self.registry
    }

    fn create_built_in_types() -> RwLock<HashMap<String, Arc<dyn ElementScanMappingRule>>> {
        let mut registry: HashMap<String, Arc<dyn ElementScanMappingRule>> = HashMap::new();

        let built_in_project_item_types: Vec<Arc<dyn ElementScanMappingRule>> = vec![Arc::new(MapUnsignedGreaterThanZeroToNotEqual {})];

        for built_in_project_item_type in built_in_project_item_types.into_iter() {
            registry.insert(built_in_project_item_type.get_id().to_string(), built_in_project_item_type);
        }

        RwLock::new(registry)
    }
}
