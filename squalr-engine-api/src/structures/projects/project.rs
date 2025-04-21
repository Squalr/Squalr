use crate::structures::projects::project_item::ProjectItem;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    name: String,
    children: Vec<ProjectItem>,
}

impl Project {
    pub fn get_name(&self) -> &str {
        &self.name
    }
}
