use crate::structures::structs::symbolic_resolver_definition::SymbolicResolverDefinition;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolicResolverDescriptor {
    resolver_id: String,
    resolver_definition: SymbolicResolverDefinition,
}

impl SymbolicResolverDescriptor {
    pub fn new(
        resolver_id: String,
        resolver_definition: SymbolicResolverDefinition,
    ) -> Self {
        Self {
            resolver_id,
            resolver_definition,
        }
    }

    pub fn get_resolver_id(&self) -> &str {
        &self.resolver_id
    }

    pub fn get_resolver_definition(&self) -> &SymbolicResolverDefinition {
        &self.resolver_definition
    }
}
