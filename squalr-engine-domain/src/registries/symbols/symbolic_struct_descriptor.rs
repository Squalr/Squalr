use crate::structures::structs::symbolic_struct_definition::SymbolicStructDefinition;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StructLayoutDescriptor {
    struct_layout_id: String,
    struct_layout_definition: SymbolicStructDefinition,
}

impl StructLayoutDescriptor {
    pub fn new(
        struct_layout_id: String,
        struct_layout_definition: SymbolicStructDefinition,
    ) -> Self {
        Self {
            struct_layout_id,
            struct_layout_definition,
        }
    }

    pub fn get_struct_layout_id(&self) -> &str {
        &self.struct_layout_id
    }

    pub fn get_struct_layout_definition(&self) -> &SymbolicStructDefinition {
        &self.struct_layout_definition
    }
}
