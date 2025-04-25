use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::projects::project_items::project_item_type::ProjectItemType;
use serde::{Deserialize, Serialize};
use std::any::Any;
use typetag::serde;

#[derive(Serialize, Deserialize)]
pub struct ProjectItemTypeAddress {
    name: String,

    description: String,

    data_type: DataTypeRef,

    is_value_hex: bool,

    #[serde(skip)]
    is_activated: bool,

    #[serde(skip)]
    has_unsaved_changes: bool,
}

impl ProjectItemTypeAddress {
    pub fn new(
        name: String,
        description: String,
        data_type: DataTypeRef,
        is_value_hex: bool,
    ) -> Self {
        Self {
            name,
            description,
            data_type,
            is_value_hex,
            is_activated: false,
            has_unsaved_changes: true,
        }
    }
}

#[typetag::serde]
impl ProjectItemType for ProjectItemTypeAddress {
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
    }
}
