use crate::structures::projects::project_symbol_module_field::ProjectSymbolModuleField;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSymbolModule {
    module_name: String,
    size: u64,
    #[serde(default)]
    fields: Vec<ProjectSymbolModuleField>,
}

impl ProjectSymbolModule {
    pub fn new(
        module_name: String,
        size: u64,
    ) -> Self {
        Self {
            module_name,
            size,
            fields: Vec::new(),
        }
    }

    pub fn get_module_name(&self) -> &str {
        &self.module_name
    }

    pub fn set_module_name(
        &mut self,
        module_name: String,
    ) {
        self.module_name = module_name;
    }

    pub fn get_size(&self) -> u64 {
        self.size
    }

    pub fn set_size(
        &mut self,
        size: u64,
    ) {
        self.size = size;
    }

    pub fn get_fields(&self) -> &[ProjectSymbolModuleField] {
        &self.fields
    }

    pub fn get_fields_mut(&mut self) -> &mut Vec<ProjectSymbolModuleField> {
        &mut self.fields
    }

    pub fn find_field(
        &self,
        offset: u64,
    ) -> Option<&ProjectSymbolModuleField> {
        self.fields
            .iter()
            .find(|module_field| module_field.get_offset() == offset)
    }

    pub fn find_field_mut(
        &mut self,
        offset: u64,
    ) -> Option<&mut ProjectSymbolModuleField> {
        self.fields
            .iter_mut()
            .find(|module_field| module_field.get_offset() == offset)
    }
}
