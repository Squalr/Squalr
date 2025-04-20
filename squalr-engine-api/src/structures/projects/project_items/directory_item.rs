use crate::structures::projects::project_items::project_item::ProjectItem;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DirectoryItem {
    project_item: ProjectItem,
}

impl DirectoryItem {
    pub fn new() {
        //
    }
}
