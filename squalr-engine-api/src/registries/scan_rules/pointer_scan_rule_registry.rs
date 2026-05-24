use crate::structures::scanning::rules::{
    pointer_scan::built_in_planning_rules::rule_map_search_kernel::RuleMapSearchKernel, pointer_scan_planning_rule::PointerScanPlanningRule,
};
use std::{
    collections::HashMap,
    sync::{Arc, Once},
};

pub struct PointerScanRuleRegistry {
    pointer_scan_planning_rule_registry: HashMap<String, Arc<dyn PointerScanPlanningRule>>,
}

impl PointerScanRuleRegistry {
    pub fn get_instance() -> &'static PointerScanRuleRegistry {
        static mut INSTANCE: Option<PointerScanRuleRegistry> = None;
        static ONCE: Once = Once::new();

        unsafe {
            ONCE.call_once(|| {
                let instance = PointerScanRuleRegistry::new();
                INSTANCE = Some(instance);
            });

            #[allow(static_mut_refs)]
            INSTANCE.as_ref().unwrap_unchecked()
        }
    }

    pub fn new() -> Self {
        Self {
            pointer_scan_planning_rule_registry: Self::create_built_in_planning_rules(),
        }
    }

    pub fn get_pointer_scan_planning_rule_registry(&self) -> &HashMap<String, Arc<dyn PointerScanPlanningRule>> {
        &self.pointer_scan_planning_rule_registry
    }

    fn create_built_in_planning_rules() -> HashMap<String, Arc<dyn PointerScanPlanningRule>> {
        let mut registry: HashMap<String, Arc<dyn PointerScanPlanningRule>> = HashMap::new();
        let built_in_rules: Vec<Arc<dyn PointerScanPlanningRule>> = vec![Arc::new(RuleMapSearchKernel)];

        for built_in_rule in built_in_rules {
            registry.insert(built_in_rule.get_id().to_string(), built_in_rule);
        }

        registry
    }
}
