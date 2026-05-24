use squalr_engine_api::commands::project_items::rename::project_items_rename_request::ProjectItemsRenameRequest;
use std::path::Path;

pub struct ProjectItemRenameRequestBuilder;

impl ProjectItemRenameRequestBuilder {
    pub fn build(
        project_item_path: &Path,
        _project_item_type_id: &str,
        edited_name: &str,
    ) -> Option<ProjectItemsRenameRequest> {
        let renamed_project_item_name = edited_name.trim();

        if renamed_project_item_name.is_empty() {
            return None;
        }

        Some(ProjectItemsRenameRequest {
            project_item_path: project_item_path.to_path_buf(),
            project_item_name: renamed_project_item_name.to_string(),
        })
    }
}
