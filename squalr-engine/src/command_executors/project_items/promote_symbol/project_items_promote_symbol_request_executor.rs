use crate::command_executors::project::project_symbol_sync::sync_project_symbol_catalog;
use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use crate::services::projects::project_item_symbol_promotion::{
    NO_OPENED_PROCESS_STATUS_MESSAGE, apply_project_item_symbol_promotion, query_has_opened_process,
};
use squalr_engine_api::commands::project_items::promote_symbol::project_items_promote_symbol_request::ProjectItemsPromoteSymbolRequest;
use squalr_engine_api::commands::project_items::promote_symbol::project_items_promote_symbol_response::ProjectItemsPromoteSymbolResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectItemsPromoteSymbolRequest {
    type ResponseType = ProjectItemsPromoteSymbolResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        if self.project_item_paths.is_empty() {
            return ProjectItemsPromoteSymbolResponse {
                success: true,
                status_message: String::new(),
                promoted_symbol_count: 0,
                reused_symbol_count: 0,
                promoted_symbol_locator_keys: Vec::new(),
                conflicts: Vec::new(),
            };
        }

        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project = project_manager.get_opened_project();
        let mut opened_project_guard = match opened_project.write() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                log::error!("Failed to acquire opened project lock for promote-symbol command: {}", error);

                return ProjectItemsPromoteSymbolResponse::default();
            }
        };
        let Some(opened_project) = opened_project_guard.as_mut() else {
            log::warn!("Cannot promote project items to symbols without an opened project.");

            return ProjectItemsPromoteSymbolResponse::default();
        };
        let Some(project_directory_path) = opened_project.get_project_info().get_project_directory() else {
            log::error!("Failed to resolve opened project directory for promote-symbol operation.");

            return ProjectItemsPromoteSymbolResponse::default();
        };
        if !query_has_opened_process(engine_unprivileged_state) {
            log::warn!("{}", NO_OPENED_PROCESS_STATUS_MESSAGE);

            return ProjectItemsPromoteSymbolResponse {
                success: false,
                status_message: String::from(NO_OPENED_PROCESS_STATUS_MESSAGE),
                promoted_symbol_count: 0,
                reused_symbol_count: 0,
                promoted_symbol_locator_keys: Vec::new(),
                conflicts: Vec::new(),
            };
        }

        let promotion_change_set = apply_project_item_symbol_promotion(
            engine_unprivileged_state,
            opened_project,
            &project_directory_path,
            &self.project_item_paths,
            self.overwrite_conflicting_symbols,
        );

        if promotion_change_set.should_save_project {
            if let Err(error) = opened_project.save_to_path(&project_directory_path, false) {
                log::error!("Failed to save project after promote-symbol operation: {}", error);

                return ProjectItemsPromoteSymbolResponse::default();
            }
        }

        drop(opened_project_guard);

        if let Some(updated_project_symbol_catalog) = promotion_change_set.updated_project_symbol_catalog.clone() {
            if !sync_project_symbol_catalog(engine_unprivileged_state, updated_project_symbol_catalog) {
                log::error!("Failed to sync project symbol catalog after promote-symbol operation.");

                return ProjectItemsPromoteSymbolResponse {
                    success: false,
                    status_message: String::from("Failed to sync project symbol catalog after promote-symbol operation."),
                    promoted_symbol_count: promotion_change_set.promoted_symbol_count,
                    reused_symbol_count: promotion_change_set.reused_symbol_count,
                    promoted_symbol_locator_keys: promotion_change_set.promoted_symbol_locator_keys,
                    conflicts: promotion_change_set.conflicts,
                };
            }
        }

        ProjectItemsPromoteSymbolResponse {
            success: true,
            status_message: String::new(),
            promoted_symbol_count: promotion_change_set.promoted_symbol_count,
            reused_symbol_count: promotion_change_set.reused_symbol_count,
            promoted_symbol_locator_keys: promotion_change_set.promoted_symbol_locator_keys,
            conflicts: promotion_change_set.conflicts,
        }
    }
}
