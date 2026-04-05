use crate::structures::structs::symbolic_struct_definition::SymbolicStructDefinition;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SymbolicStructDescriptor {
    symbolic_struct_id: String,
    symbolic_struct_definition: SymbolicStructDefinition,
}

impl SymbolicStructDescriptor {
    pub fn new(
        symbolic_struct_id: String,
        symbolic_struct_definition: SymbolicStructDefinition,
    ) -> Self {
        Self {
            symbolic_struct_id,
            symbolic_struct_definition,
        }
    }

    pub fn get_symbolic_struct_id(&self) -> &str {
        &self.symbolic_struct_id
    }

    pub fn get_symbolic_struct_definition(&self) -> &SymbolicStructDefinition {
        &self.symbolic_struct_definition
    }
}
