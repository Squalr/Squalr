use crate::registries::symbols::symbolic_struct_descriptor::StructLayoutDescriptor;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectSymbolCatalog {
    struct_layout_descriptors: Vec<StructLayoutDescriptor>,
}

impl ProjectSymbolCatalog {
    pub fn new(struct_layout_descriptors: Vec<StructLayoutDescriptor>) -> Self {
        Self { struct_layout_descriptors }
    }

    pub fn get_struct_layout_descriptors(&self) -> &[StructLayoutDescriptor] {
        &self.struct_layout_descriptors
    }

    pub fn set_struct_layout_descriptors(
        &mut self,
        struct_layout_descriptors: Vec<StructLayoutDescriptor>,
    ) {
        self.struct_layout_descriptors = struct_layout_descriptors;
    }

    pub fn is_empty(&self) -> bool {
        self.struct_layout_descriptors.is_empty()
    }
}
