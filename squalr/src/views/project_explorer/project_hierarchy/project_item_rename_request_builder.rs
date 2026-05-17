use squalr_engine_api::commands::project_items::rename::project_items_rename_request::ProjectItemsRenameRequest;
use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_directory::ProjectItemTypeDirectory;
use std::path::Path;

pub struct ProjectItemRenameRequestBuilder;

impl ProjectItemRenameRequestBuilder {
    pub fn build(
        project_item_path: &Path,
        project_item_type_id: &str,
        edited_name: &str,
    ) -> Option<ProjectItemsRenameRequest> {
        let sanitized_file_name = Path::new(edited_name)
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .map(str::trim)
            .filter(|file_name| !file_name.is_empty())?
            .to_string();
        let is_directory_project_item = project_item_type_id == ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID;
        let renamed_project_item_name = if is_directory_project_item {
            sanitized_file_name
        } else {
            let mut file_name_with_extension = sanitized_file_name.clone();
            let expected_extension = Project::PROJECT_ITEM_EXTENSION.trim_start_matches('.');
            let has_expected_extension = Path::new(&sanitized_file_name)
                .extension()
                .and_then(|extension| extension.to_str())
                .map(|extension| extension.eq_ignore_ascii_case(expected_extension))
                .unwrap_or(false);

            if !has_expected_extension {
                file_name_with_extension.push('.');
                file_name_with_extension.push_str(expected_extension);
            }

            file_name_with_extension
        };
        let current_file_name = project_item_path
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .unwrap_or_default();

        if current_file_name == renamed_project_item_name {
            return None;
        }

        Some(ProjectItemsRenameRequest {
            project_item_path: project_item_path.to_path_buf(),
            project_item_name: renamed_project_item_name,
        })
    }
}
