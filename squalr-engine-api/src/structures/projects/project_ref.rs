use crate::structures::projects::project_items::project_item::ProjectItem;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectRef {
    name: String,
    children: Vec<ProjectItem>,
}

impl ProjectRef {
    pub fn get_name(&self) -> &str {
        &self.name
    }
}
