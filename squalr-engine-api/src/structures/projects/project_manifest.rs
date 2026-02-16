use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectManifest {
    #[serde(rename = "sort_order")]
    project_item_sort_order: Vec<PathBuf>,
}

impl ProjectManifest {
    pub fn new(project_item_sort_order: Vec<PathBuf>) -> Self {
        Self { project_item_sort_order }
    }

    pub fn get_project_item_sort_order(&self) -> &Vec<PathBuf> {
        &self.project_item_sort_order
    }

    pub fn set_project_item_sort_order(
        &mut self,
        project_item_sort_order: Vec<PathBuf>,
    ) {
        self.project_item_sort_order = project_item_sort_order;
    }
}
