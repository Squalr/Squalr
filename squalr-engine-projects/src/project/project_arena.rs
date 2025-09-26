use serde::{Deserialize, Serialize};
use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectArena {
    /// The child project items underneath the referenced project item.
    project_items: HashMap<String, ProjectItem>,
}

impl ProjectArena {
    pub fn new() -> Self {
        Self { project_items: HashMap::new() }
    }
}
