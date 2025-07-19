use crate::structures::projects::project_items::project_item_type_ref::ProjectItemTypeRef;
use crate::structures::projects::project_items::{project_item::ProjectItem, project_item_type::ProjectItemType};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct ProjectItemTypeDirectory {}

impl ProjectItemTypeDirectory {
    pub const PROJECT_ITEM_TYPE_ID: &str = "directory";
}

impl ProjectItemType for ProjectItemTypeDirectory {
    fn get_project_item_type_id(&self) -> &str {
        &Self::PROJECT_ITEM_TYPE_ID
    }
}

impl ProjectItemTypeDirectory {
    pub fn new_project_item(directory: &Path) -> ProjectItem {
        let directory_type = ProjectItemTypeRef::new(Self::PROJECT_ITEM_TYPE_ID.to_string());

        ProjectItem::new(directory.to_path_buf(), directory_type, true)
    }
}
