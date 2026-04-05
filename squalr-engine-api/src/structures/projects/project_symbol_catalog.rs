use crate::registries::symbols::symbolic_struct_descriptor::SymbolicStructDescriptor;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectSymbolCatalog {
    symbolic_struct_descriptors: Vec<SymbolicStructDescriptor>,
}

impl ProjectSymbolCatalog {
    pub fn new(symbolic_struct_descriptors: Vec<SymbolicStructDescriptor>) -> Self {
        Self { symbolic_struct_descriptors }
    }

    pub fn get_symbolic_struct_descriptors(&self) -> &[SymbolicStructDescriptor] {
        &self.symbolic_struct_descriptors
    }

    pub fn set_symbolic_struct_descriptors(
        &mut self,
        symbolic_struct_descriptors: Vec<SymbolicStructDescriptor>,
    ) {
        self.symbolic_struct_descriptors = symbolic_struct_descriptors;
    }

    pub fn is_empty(&self) -> bool {
        self.symbolic_struct_descriptors.is_empty()
    }
}
