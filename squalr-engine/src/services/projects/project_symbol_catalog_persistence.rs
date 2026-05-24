use crate::command_executors::project::project_symbol_sync::sync_project_symbol_catalog;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
use std::path::PathBuf;
use std::sync::Arc;

pub fn save_and_sync_project_symbol_catalog(
    engine_execution_context: &Arc<dyn EngineExecutionContext>,
    opened_project: &mut Project,
    project_directory_path: &PathBuf,
) -> bool {
    let updated_project_symbol_catalog = opened_project
        .get_project_info()
        .get_project_symbol_catalog()
        .clone();

    opened_project
        .get_project_info_mut()
        .set_has_unsaved_changes(true);

    if let Err(error) = opened_project.save_to_path(project_directory_path, false) {
        log::error!("Failed to save project after project symbol mutation: {}", error);
        return false;
    }

    if !sync_project_symbol_catalog(engine_execution_context, updated_project_symbol_catalog) {
        log::error!("Failed to sync project symbol catalog after project symbol mutation.");
        return false;
    }

    true
}
