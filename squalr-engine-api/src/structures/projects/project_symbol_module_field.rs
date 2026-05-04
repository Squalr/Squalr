use crate::structures::projects::project_symbol_locator::ProjectSymbolLocator;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSymbolModuleField {
    display_name: String,
    offset: u64,
    struct_layout_id: String,
}

impl ProjectSymbolModuleField {
    pub fn new(
        display_name: String,
        offset: u64,
        struct_layout_id: String,
    ) -> Self {
        Self {
            display_name,
            offset,
            struct_layout_id,
        }
    }

    pub fn get_symbol_locator_key(
        &self,
        module_name: &str,
    ) -> String {
        ProjectSymbolLocator::new_module_offset(module_name.to_string(), self.offset).to_locator_key()
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

    pub fn get_offset(&self) -> u64 {
        self.offset
    }

    pub fn set_offset(
        &mut self,
        offset: u64,
    ) {
        self.offset = offset;
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
}
