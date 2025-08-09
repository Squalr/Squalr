use crate::structures::structs::symbolic_struct_definition::SymbolicStructDefinition;
use std::{collections::HashMap, sync::Arc};

pub struct SymbolicStructDefinitionRegistry {
    registry: HashMap<String, Arc<SymbolicStructDefinition>>,
}

impl SymbolicStructDefinitionRegistry {
    pub fn new() -> Self {
        Self { registry: HashMap::new() }
    }

    pub fn get_registry(&self) -> &HashMap<String, Arc<SymbolicStructDefinition>> {
        &self.registry
    }

    pub fn get(
        &self,
        symbolic_struct_ref_id: &str,
    ) -> Option<Arc<SymbolicStructDefinition>> {
        self.registry.get(symbolic_struct_ref_id).cloned()
    }
}
