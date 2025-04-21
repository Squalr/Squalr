use crate::structures::projects::project_item_ref::ProjectItemRef;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectRef {
    name: String,
    children: Vec<ProjectItemRef>,
}

impl ProjectRef {
    pub fn get_name(&self) -> &str {
        &self.name
    }
}
