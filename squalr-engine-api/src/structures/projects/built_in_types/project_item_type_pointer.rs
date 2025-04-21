use crate::structures::{data_types::data_type_ref::DataTypeRef, projects::project_item_type::ProjectItemType};
use serde::{Deserialize, Serialize};
use typetag::serde;

#[derive(Serialize, Deserialize)]
pub struct ProjectItemTypePointer {
    name: String,

    description: String,

    data_type: DataTypeRef,

    is_value_hex: bool,

    module_name: String,

    module_offset: u64,

    pointer_offsets: Vec<i32>,

    #[serde(skip)]
    is_activated: bool,
}

impl ProjectItemTypePointer {
    pub fn new() {
        //
    }
}

#[typetag::serde]
impl ProjectItemType for ProjectItemTypePointer {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_description(&self) -> &str {
        &self.description
    }

    fn is_activated(&self) -> bool {
        self.is_activated
    }

    fn toggle_activated(&mut self) {
        self.is_activated = !self.is_activated
    }

    fn set_activated(
        &mut self,
        is_activated: bool,
    ) {
        self.is_activated = is_activated;
    }
}
