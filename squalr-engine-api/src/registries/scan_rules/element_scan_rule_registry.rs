use crate::structures::scanning::{
    parameters::element_scan::built_in_rules::{
        rule_map_periodic_scans::RuleMapPeriodicScans, rule_map_scan_type::RuleMapScanType, rule_map_to_primitive_type::RuleMapToPrimitiveType,
        rule_map_unsigned_greater_than_zero_to_not_equal::RuleMapUnsignedGreaterThanZeroToNotEqual,
    },
    rules::element_scan_mapping_rule::ElementScanMappingRule,
};
use std::{collections::HashMap, sync::Arc};

pub struct ElementScanRuleRegistry {
    registry: HashMap<String, Arc<dyn ElementScanMappingRule>>,
}

impl ElementScanRuleRegistry {
    pub fn new() -> Self {
        Self {
            registry: Self::create_built_in_types(),
        }
    }

    pub fn get_registry(&self) -> &HashMap<String, Arc<dyn ElementScanMappingRule>> {
        &self.registry
    }

    fn create_built_in_types() -> HashMap<String, Arc<dyn ElementScanMappingRule>> {
        let mut registry: HashMap<String, Arc<dyn ElementScanMappingRule>> = HashMap::new();

        let built_in_project_item_types: Vec<Arc<dyn ElementScanMappingRule>> = vec![
            Arc::new(RuleMapToPrimitiveType {}),
            Arc::new(RuleMapPeriodicScans {}),
            Arc::new(RuleMapScanType {}),
            Arc::new(RuleMapUnsignedGreaterThanZeroToNotEqual {}),
        ];

        for built_in_project_item_type in built_in_project_item_types.into_iter() {
            registry.insert(built_in_project_item_type.get_id().to_string(), built_in_project_item_type);
        }

        registry
    }
}
