use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project_symbols::list::project_symbols_list_request::ProjectSymbolsListRequest;
use squalr_engine_api::commands::project_symbols::list::project_symbols_list_response::ProjectSymbolsListResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectSymbolsListRequest {
    type ResponseType = ProjectSymbolsListResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        let opened_project = engine_unprivileged_state
            .get_project_manager()
            .get_opened_project();
        let opened_project = match opened_project.read() {
            Ok(opened_project) => opened_project,
            Err(error) => {
                log::error!("Failed to acquire opened project lock for project-symbols list command: {}", error);
                return ProjectSymbolsListResponse::default();
            }
        };

        ProjectSymbolsListResponse {
            opened_project_info: opened_project
                .as_ref()
                .map(|opened_project| opened_project.get_project_info().clone()),
            project_symbol_catalog: opened_project.as_ref().map(|opened_project| {
                opened_project
                    .get_project_info()
                    .get_project_symbol_catalog()
                    .clone()
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectSymbolsListRequest;
    use crate::command_executors::project_symbols::test_support::{
        MockProjectSymbolsBindings, create_engine_unprivileged_state, create_project_with_symbol_catalog,
    };
    use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
    use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
    use squalr_engine_api::structures::projects::{project_symbol_catalog::ProjectSymbolCatalog, project_symbol_claim::ProjectSymbolClaim};
    use std::sync::Arc;

    #[test]
    fn list_project_symbols_request_returns_opened_project_symbol_catalog() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_symbol_claims(
            Vec::new(),
            vec![ProjectSymbolClaim::new_absolute_address(
                String::from("Player"),
                0x1234,
                String::from("player"),
            )],
        );
        let project = create_project_with_symbol_catalog(temp_directory.path(), project_symbol_catalog.clone());
        let engine_unprivileged_state = create_engine_unprivileged_state(MockProjectSymbolsBindings::new());

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let project_symbols_list_response = ProjectSymbolsListRequest::default().execute(&engine_execution_context);

        assert!(project_symbols_list_response.opened_project_info.is_some());
        let listed_project_symbol_catalog = project_symbols_list_response
            .project_symbol_catalog
            .expect("Expected project symbol catalog in list response.");
        assert_eq!(
            listed_project_symbol_catalog
                .get_struct_layout_descriptors()
                .len(),
            0
        );
        assert_eq!(listed_project_symbol_catalog.get_symbol_claims().len(), 1);
        assert_eq!(
            listed_project_symbol_catalog.get_symbol_claims()[0].get_symbol_locator_key(),
            project_symbol_catalog.get_symbol_claims()[0].get_symbol_locator_key()
        );
        assert_eq!(
            listed_project_symbol_catalog.get_symbol_claims()[0].get_display_name(),
            project_symbol_catalog.get_symbol_claims()[0].get_display_name()
        );
    }
}
