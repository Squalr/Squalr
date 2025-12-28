use crate::structures::scanning::rules::{
    element_scan::{
        built_in_filter_rules::filter_rule_map_scan_type::RuleMapScanType,
        built_in_parameter_rules::{
            parameter_rule_map_to_primitive_type::RuleMapToPrimitiveType,
            parameter_rule_map_unsigned_greater_than_zero_to_not_equal::RuleMapUnsignedGreaterThanZeroToNotEqual,
        },
    },
    element_scan_filter_rule::ElementScanFilterRule,
    element_scan_parameters_rule::ElementScanParametersRule,
};
use std::{
    collections::HashMap,
    sync::{Arc, Once},
};

pub struct ElementScanRuleRegistry {
    element_scan_parameters_rule_registry: HashMap<String, Arc<dyn ElementScanParametersRule>>,
    element_scan_filters_rule_registry: HashMap<String, Arc<dyn ElementScanFilterRule>>,
}

impl ElementScanRuleRegistry {
    // JIRA: Deprecate this. Needs to support mutability, mirroring from client to server for non-standalone builds, etc.
    pub fn get_instance() -> &'static ElementScanRuleRegistry {
        static mut INSTANCE: Option<ElementScanRuleRegistry> = None;
        static ONCE: Once = Once::new();

        unsafe {
            ONCE.call_once(|| {
                let instance = ElementScanRuleRegistry::new();
                INSTANCE = Some(instance);
            });

            #[allow(static_mut_refs)]
            INSTANCE.as_ref().unwrap_unchecked()
        }
    }

    pub fn new() -> Self {
        Self {
            element_scan_parameters_rule_registry: Self::create_built_in_parameter_rules(),
            element_scan_filters_rule_registry: Self::create_built_in_filter_rules(),
        }
    }

    pub fn get_scan_parameters_rule_registry(&self) -> &HashMap<String, Arc<dyn ElementScanParametersRule>> {
        &self.element_scan_parameters_rule_registry
    }

    pub fn get_scan_filter_rule_registry(&self) -> &HashMap<String, Arc<dyn ElementScanFilterRule>> {
        &self.element_scan_filters_rule_registry
    }

    fn create_built_in_parameter_rules() -> HashMap<String, Arc<dyn ElementScanParametersRule>> {
        let mut registry: HashMap<String, Arc<dyn ElementScanParametersRule>> = HashMap::new();
        let built_in_rules: Vec<Arc<dyn ElementScanParametersRule>> = vec![
            Arc::new(RuleMapToPrimitiveType {}),
            Arc::new(RuleMapUnsignedGreaterThanZeroToNotEqual {}),
        ];

        for built_in_rule in built_in_rules.into_iter() {
            registry.insert(built_in_rule.get_id().to_string(), built_in_rule);
        }

        registry
    }

    fn create_built_in_filter_rules() -> HashMap<String, Arc<dyn ElementScanFilterRule>> {
        let mut registry: HashMap<String, Arc<dyn ElementScanFilterRule>> = HashMap::new();
        let built_in_rules: Vec<Arc<dyn ElementScanFilterRule>> = vec![Arc::new(RuleMapScanType {})];

        for built_in_rule in built_in_rules.into_iter() {
            registry.insert(built_in_rule.get_id().to_string(), built_in_rule);
        }

        registry
    }
}
