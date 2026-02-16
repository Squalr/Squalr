use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project_items::reorder::project_items_reorder_request::ProjectItemsReorderRequest;
use squalr_engine_api::commands::project_items::reorder::project_items_reorder_response::ProjectItemsReorderResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
use std::path::{Path, PathBuf};
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectItemsReorderRequest {
    type ResponseType = ProjectItemsReorderResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project = project_manager.get_opened_project();
        let mut opened_project_guard = match opened_project.write() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                log::error!("Failed to acquire opened project lock for reorder command: {}", error);

                return ProjectItemsReorderResponse {
                    success: false,
                    reordered_project_item_count: 0,
                };
            }
        };
        let opened_project = match opened_project_guard.as_mut() {
            Some(opened_project) => opened_project,
            None => {
                log::warn!("Cannot reorder project items without an opened project.");

                return ProjectItemsReorderResponse {
                    success: false,
                    reordered_project_item_count: 0,
                };
            }
        };
        let project_directory_path = match opened_project.get_project_info().get_project_directory() {
            Some(project_directory_path) => project_directory_path,
            None => {
                log::error!("Failed to resolve opened project directory for reorder operation.");

                return ProjectItemsReorderResponse {
                    success: false,
                    reordered_project_item_count: 0,
                };
            }
        };
        let sort_order_paths: Vec<PathBuf> = self
            .project_item_paths
            .iter()
            .map(|project_item_path| to_manifest_path(&project_directory_path, project_item_path))
            .collect();

        opened_project
            .get_project_manifest_mut()
            .set_project_item_sort_order(sort_order_paths);
        opened_project
            .get_project_info_mut()
            .set_has_unsaved_changes(true);

        if let Err(error) = opened_project.save_to_path(&project_directory_path, false) {
            log::error!("Failed to persist project item reorder metadata: {}", error);
            return ProjectItemsReorderResponse {
                success: false,
                reordered_project_item_count: 0,
            };
        }

        project_manager.notify_project_items_changed();

        ProjectItemsReorderResponse {
            success: true,
            reordered_project_item_count: self.project_item_paths.len() as u64,
        }
    }
}

fn to_manifest_path(
    project_directory_path: &Path,
    project_item_path: &Path,
) -> PathBuf {
    let resolved_project_item_path = if project_item_path.is_absolute() {
        project_item_path.to_path_buf()
    } else {
        project_directory_path.join(project_item_path)
    };

    match resolved_project_item_path.strip_prefix(project_directory_path) {
        Ok(relative_project_item_path) => relative_project_item_path.to_path_buf(),
        Err(_) => resolved_project_item_path,
    }
}

#[cfg(test)]
mod tests {
    use super::to_manifest_path;
    use std::path::{Path, PathBuf};

    #[test]
    fn to_manifest_path_converts_absolute_path_inside_project_to_relative_path() {
        let project_directory_path = Path::new("C:/Projects/TestProject");
        let project_item_path = Path::new("C:/Projects/TestProject/Addresses/health.json");

        let manifest_path = to_manifest_path(project_directory_path, project_item_path);

        assert_eq!(manifest_path, PathBuf::from("Addresses/health.json"));
    }

    #[test]
    fn to_manifest_path_leaves_relative_path_relative_to_project_directory() {
        let project_directory_path = Path::new("C:/Projects/TestProject");
        let project_item_path = Path::new("Addresses/health.json");

        let manifest_path = to_manifest_path(project_directory_path, project_item_path);

        assert_eq!(manifest_path, PathBuf::from("Addresses/health.json"));
    }

    #[test]
    fn to_manifest_path_returns_absolute_path_when_outside_project_directory() {
        let project_directory_path = Path::new("C:/Projects/TestProject");
        let project_item_path = Path::new("C:/Projects/OtherProject/Addresses/health.json");

        let manifest_path = to_manifest_path(project_directory_path, project_item_path);

        assert_eq!(manifest_path, PathBuf::from("C:/Projects/OtherProject/Addresses/health.json"));
    }
}
