use crate::structures::projects::project_items::project_item_type::ProjectItemType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ProjectItemTypePointer {
    /*
    name: String,

    description: String,

    data_type: DataTypeRef,

    is_value_hex: bool,

    module_name: String,

    module_offset: u64,

    pointer_offsets: Vec<i32>,

    #[serde(skip)]
    is_activated: bool,

    #[serde(skip)]
    has_unsaved_changes: bool,*/
}

impl ProjectItemTypePointer {
    pub const PROJECT_ITEM_TYPE_ID: &str = "pointer";
    /*
    pub fn new(
        name: String,
        description: String,
        data_type: DataTypeRef,
        is_value_hex: bool,
        module_name: String,
        module_offset: u64,
        pointer_offsets: Vec<i32>,
    ) -> Self {
        Self {
            name,
            description,
            data_type,
            is_value_hex,
            module_name,
            module_offset,
            pointer_offsets,
            is_activated: false,
            has_unsaved_changes: true,
        }
    }*/
}

impl ProjectItemType for ProjectItemTypePointer {
    fn get_project_item_type_id(&self) -> &str {
        &Self::PROJECT_ITEM_TYPE_ID
    }
    /*
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

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

    fn get_has_unsaved_changes(&self) -> bool {
        self.has_unsaved_changes
    }

    fn set_has_unsaved_changes(
        &mut self,
        has_unsaved_changes: bool,
    ) {
        self.has_unsaved_changes = has_unsaved_changes;
    }*/
}
