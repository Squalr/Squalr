use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Represents a handle to a project item type.
#[derive(Reflect, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProjectItemTypeRef {
    project_item_type_id: String,
}

impl ProjectItemTypeRef {
    pub fn new(project_item_type_id: String) -> Self {
        Self { project_item_type_id }
    }

    pub fn get_project_item_type_id(&self) -> &str {
        &self.project_item_type_id
    }
}
