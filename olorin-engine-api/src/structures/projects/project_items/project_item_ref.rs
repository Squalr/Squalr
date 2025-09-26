use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Represents a handle to a project item.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ProjectItemRef {
    project_item_id: String,
}

impl ProjectItemRef {
    pub fn new(project_item_id: String) -> Self {
        Self { project_item_id }
    }

    pub fn get_project_item_id(&self) -> &str {
        &self.project_item_id
    }
}
