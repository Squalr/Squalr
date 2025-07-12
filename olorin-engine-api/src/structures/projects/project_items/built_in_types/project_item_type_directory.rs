use crate::structures::projects::project_items::project_item_type::ProjectItemType;
use serde::{Deserialize, Serialize};

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
