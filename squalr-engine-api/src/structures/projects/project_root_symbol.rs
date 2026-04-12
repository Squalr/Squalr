use crate::structures::projects::project_root_symbol_locator::ProjectRootSymbolLocator;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectRootSymbol {
    symbol_key: String,
    display_name: String,
    root_locator: ProjectRootSymbolLocator,
    struct_layout_id: String,
    #[serde(default)]
    metadata: BTreeMap<String, String>,
}

impl ProjectRootSymbol {
    pub fn new(
        symbol_key: String,
        display_name: String,
        root_locator: ProjectRootSymbolLocator,
        struct_layout_id: String,
    ) -> Self {
        Self {
            symbol_key,
            display_name,
            root_locator,
            struct_layout_id,
            metadata: BTreeMap::new(),
        }
    }

    pub fn new_absolute_address(
        symbol_key: String,
        display_name: String,
        address: u64,
        struct_layout_id: String,
    ) -> Self {
        Self::new(
            symbol_key,
            display_name,
            ProjectRootSymbolLocator::new_absolute_address(address),
            struct_layout_id,
        )
    }

    pub fn new_module_offset(
        symbol_key: String,
        display_name: String,
        module_name: String,
        offset: u64,
        struct_layout_id: String,
    ) -> Self {
        Self::new(
            symbol_key,
            display_name,
            ProjectRootSymbolLocator::new_module_offset(module_name, offset),
            struct_layout_id,
        )
    }

    pub fn get_symbol_key(&self) -> &str {
        &self.symbol_key
    }

    pub fn get_display_name(&self) -> &str {
        &self.display_name
    }

    pub fn get_root_locator(&self) -> &ProjectRootSymbolLocator {
        &self.root_locator
    }

    pub fn get_struct_layout_id(&self) -> &str {
        &self.struct_layout_id
    }

    pub fn get_metadata(&self) -> &BTreeMap<String, String> {
        &self.metadata
    }

    pub fn get_metadata_mut(&mut self) -> &mut BTreeMap<String, String> {
        &mut self.metadata
    }
}
