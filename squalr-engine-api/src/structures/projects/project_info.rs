use crate::structures::projects::project_items::project_item_ref::ProjectItemRef;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectInfo {
    name: String,
    items: Vec<ProjectItemRef>,
}

impl ProjectInfo {
    pub fn get_name(&self) -> &str {
        &self.name
    }
}
