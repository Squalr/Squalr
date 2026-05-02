use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSymbolModule {
    module_name: String,
    size: u64,
}

impl ProjectSymbolModule {
    pub fn new(
        module_name: String,
        size: u64,
    ) -> Self {
        Self { module_name, size }
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
}
