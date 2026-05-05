use crate::command_executors::project_symbols::project_symbol_store_mutation::save_and_sync_project_symbol_catalog;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::plugins::symbol_tree::symbol_tree_action::ProjectSymbolStore;
use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
use std::sync::Arc;

pub struct EngineProjectSymbolStore {
    engine_execution_context: Arc<dyn EngineExecutionContext>,
}

impl EngineProjectSymbolStore {
    pub fn new(engine_execution_context: Arc<dyn EngineExecutionContext>) -> Self {
        Self { engine_execution_context }
    }
}

impl ProjectSymbolStore for EngineProjectSymbolStore {
    fn read_catalog(&self) -> Result<ProjectSymbolCatalog, String> {
        let opened_project = self
            .engine_execution_context
            .get_project_manager()
            .get_opened_project();
        let opened_project_guard = opened_project
            .read()
            .map_err(|error| format!("Failed to acquire opened project lock while reading symbols: {error}"))?;
        let opened_project = opened_project_guard
            .as_ref()
            .ok_or_else(|| String::from("Cannot read plugin symbols without an opened project."))?;

        Ok(opened_project
            .get_project_info()
            .get_project_symbol_catalog()
            .clone())
    }

    fn write_catalog(
        &self,
        reason: &str,
        update_catalog: Box<dyn FnOnce(&mut ProjectSymbolCatalog) -> Result<(), String> + Send>,
    ) -> Result<(), String> {
        let opened_project = self
            .engine_execution_context
            .get_project_manager()
            .get_opened_project();
        let mut opened_project_guard = opened_project
            .write()
            .map_err(|error| format!("Failed to acquire opened project lock while writing symbols: {error}"))?;
        let opened_project = opened_project_guard
            .as_mut()
            .ok_or_else(|| String::from("Cannot write plugin symbols without an opened project."))?;
        let project_directory_path = opened_project
            .get_project_info()
            .get_project_directory()
            .ok_or_else(|| String::from("Failed to resolve opened project directory while writing plugin symbols."))?;

        update_catalog(
            opened_project
                .get_project_info_mut()
                .get_project_symbol_catalog_mut(),
        )
        .map_err(|error| format!("Plugin symbol update failed during `{reason}`: {error}"))?;

        if !save_and_sync_project_symbol_catalog(&self.engine_execution_context, opened_project, &project_directory_path) {
            return Err(format!("Failed to save and sync plugin symbol update `{reason}`."));
        }

        Ok(())
    }
}
