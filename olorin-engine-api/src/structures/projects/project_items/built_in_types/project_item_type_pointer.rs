use crate::structures::projects::project_items::{project_item::ProjectItem, project_item_type::ProjectItemType};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ProjectItemTypePointer {
    /*
    module_name: String,
    module_offset: u64,
    pointer_offsets: Vec<i32>,
    */
}

impl ProjectItemTypePointer {
    pub const PROJECT_ITEM_TYPE_ID: &str = "pointer";
}

impl ProjectItemType for ProjectItemTypePointer {
    fn get_project_item_type_id(&self) -> &str {
        &Self::PROJECT_ITEM_TYPE_ID
    }

    fn on_activated_changed(
        &self,
        project_item: &ProjectItem,
    ) {
        // JIRA: Implement
    }
}
