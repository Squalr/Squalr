use crate::structures::projects::project_items::project_item_type::ProjectItemType;
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::{Arc, RwLock},
};
use typetag::serde;

#[derive(Serialize, Deserialize)]
pub struct ProjectItemTypeDirectory {
    name: String,

    #[serde(skip)]
    children: Arc<RwLock<Vec<Box<dyn ProjectItemType>>>>,

    #[serde(skip)]
    is_activated: bool,
}

impl ProjectItemTypeDirectory {
    pub fn new(path: &Path) -> Self {
        Self {
            name: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            children: Arc::new(RwLock::new(vec![])),
            is_activated: false,
        }
    }

    pub fn append_child(
        &mut self,
        child: Box<dyn ProjectItemType>,
    ) {
        if let Ok(mut children) = self.children.write() {
            children.push(child);
        }
    }
}

#[typetag::serde]
impl ProjectItemType for ProjectItemTypeDirectory {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_description(&self) -> &str {
        ""
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
