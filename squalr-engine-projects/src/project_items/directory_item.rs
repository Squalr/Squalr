use crate::project_items::project_item::ProjectItem;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use typetag::serde;

/*
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DirectoryItem {
    name: String,
    description: String,
    children: Arc<RwLock<Box<dyn ProjectItem>>>,

    #[serde(skip)]
    is_activated: bool,
}

impl DirectoryItem {
    pub fn new() {
        //
    }
}

impl ProjectItem for DirectoryItem {
    fn typetag_name(&self) -> &'static str {
        "directory"
    }

    fn typetag_deserialize(&self) {}

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
}
*/
