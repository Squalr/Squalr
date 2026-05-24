use crate::command_executors::project_items::project_item_sort_order::apply_reorder_subset_to_sort_order;
use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project_items::reorder::project_items_reorder_request::ProjectItemsReorderRequest;
use squalr_engine_api::commands::project_items::reorder::project_items_reorder_response::ProjectItemsReorderResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
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
        apply_reorder_subset_to_sort_order(opened_project, &project_directory_path, &self.project_item_paths);

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
