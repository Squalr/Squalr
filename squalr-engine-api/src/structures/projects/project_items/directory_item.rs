use crate::structures::projects::project_items::project_item::ProjectItem;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DirectoryItem {
    name: String,
    description: String,

    #[serde(skip_serializing)]
    is_activated: bool,
}

impl DirectoryItem {
    pub fn new() {
        //
    }
}

impl ProjectItem for DirectoryItem {
    fn get_name(&self) -> &'static str {
        ""
    }
    fn get_description(&self) -> &'static str {
        ""
    }
    fn is_activated(&self) -> bool {
        false
    }
    fn toggle_activated(&self) {}
    fn set_activated(
        &self,
        is_activated: bool,
    ) {
    }
}
