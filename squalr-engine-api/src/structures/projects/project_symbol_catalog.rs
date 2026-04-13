use crate::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
use crate::structures::projects::project_root_symbol::ProjectRootSymbol;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectSymbolCatalog {
    #[serde(default)]
    struct_layout_descriptors: Vec<StructLayoutDescriptor>,
    #[serde(default)]
    rooted_symbols: Vec<ProjectRootSymbol>,
}

impl ProjectSymbolCatalog {
    pub fn new(struct_layout_descriptors: Vec<StructLayoutDescriptor>) -> Self {
        Self::new_with_rooted_symbols(struct_layout_descriptors, Vec::new())
    }

    pub fn new_with_rooted_symbols(
        struct_layout_descriptors: Vec<StructLayoutDescriptor>,
        rooted_symbols: Vec<ProjectRootSymbol>,
    ) -> Self {
        Self {
            struct_layout_descriptors,
            rooted_symbols,
        }
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

    pub fn get_rooted_symbols(&self) -> &[ProjectRootSymbol] {
        &self.rooted_symbols
    }

    pub fn get_rooted_symbols_mut(&mut self) -> &mut Vec<ProjectRootSymbol> {
        &mut self.rooted_symbols
    }

    pub fn set_rooted_symbols(
        &mut self,
        rooted_symbols: Vec<ProjectRootSymbol>,
    ) {
        self.rooted_symbols = rooted_symbols;
    }

    pub fn is_empty(&self) -> bool {
        self.struct_layout_descriptors.is_empty() && self.rooted_symbols.is_empty()
    }
}
