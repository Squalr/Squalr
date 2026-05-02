use crate::structures::projects::project_symbol_locator::ProjectSymbolLocator;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSymbolClaim {
    display_name: String,
    locator: ProjectSymbolLocator,
    struct_layout_id: String,
    #[serde(default)]
    metadata: BTreeMap<String, String>,
}

impl ProjectSymbolClaim {
    pub fn new(
        display_name: String,
        locator: ProjectSymbolLocator,
        struct_layout_id: String,
    ) -> Self {
        Self {
            display_name,
            locator,
            struct_layout_id,
            metadata: BTreeMap::new(),
        }
    }

    pub fn new_absolute_address(
        display_name: String,
        address: u64,
        struct_layout_id: String,
    ) -> Self {
        Self::new(display_name, ProjectSymbolLocator::new_absolute_address(address), struct_layout_id)
    }

    pub fn new_module_offset(
        display_name: String,
        module_name: String,
        offset: u64,
        struct_layout_id: String,
    ) -> Self {
        Self::new(display_name, ProjectSymbolLocator::new_module_offset(module_name, offset), struct_layout_id)
    }

    pub fn get_symbol_locator_key(&self) -> String {
        self.locator.to_locator_key()
    }

    pub fn get_display_name(&self) -> &str {
        &self.display_name
    }

    pub fn set_display_name(
        &mut self,
        display_name: String,
    ) {
        self.display_name = display_name;
    }

    pub fn get_locator(&self) -> &ProjectSymbolLocator {
        &self.locator
    }

    pub fn get_struct_layout_id(&self) -> &str {
        &self.struct_layout_id
    }

    pub fn set_struct_layout_id(
        &mut self,
        struct_layout_id: String,
    ) {
        self.struct_layout_id = struct_layout_id;
    }

    pub fn get_metadata(&self) -> &BTreeMap<String, String> {
        &self.metadata
    }

    pub fn get_metadata_mut(&mut self) -> &mut BTreeMap<String, String> {
        &mut self.metadata
    }
}
