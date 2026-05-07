use crossbeam_channel::{Receiver, unbounded};
use squalr_engine_api::commands::{
    memory::{memory_command::MemoryCommand, read::memory_read_request::MemoryReadRequest, read::memory_read_response::MemoryReadResponse},
    privileged_command::PrivilegedCommand,
    privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse},
    registry::{registry_command::RegistryCommand, set_project_symbols::registry_set_project_symbols_response::RegistrySetProjectSymbolsResponse},
    unprivileged_command::UnprivilegedCommand,
    unprivileged_command_response::UnprivilegedCommandResponse,
};
use squalr_engine_api::engine::{
    engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings, engine_binding_error::EngineBindingError, engine_event_envelope::EngineEventEnvelope,
    engine_execution_context::EngineExecutionContext,
};
use squalr_engine_api::structures::projects::{
    project::Project, project_info::ProjectInfo, project_items::built_in_types::project_item_type_directory::ProjectItemTypeDirectory,
    project_items::project_item_ref::ProjectItemRef, project_manifest::ProjectManifest, project_symbol_catalog::ProjectSymbolCatalog,
};
use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
use squalr_engine_session::engine_unprivileged_state::{EngineUnprivilegedState, EngineUnprivilegedStateOptions};
use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex, RwLock},
};

pub struct MockProjectSymbolsBindings {
    captured_project_symbol_catalogs: Arc<Mutex<Vec<ProjectSymbolCatalog>>>,
    memory_read_response_factory: Option<Arc<dyn Fn(&MemoryReadRequest) -> MemoryReadResponse + Send + Sync>>,
}

impl MockProjectSymbolsBindings {
    pub fn new() -> Self {
        Self {
            captured_project_symbol_catalogs: Arc::new(Mutex::new(Vec::new())),
            memory_read_response_factory: None,
        }
    }

    pub fn new_with_memory_read_response_factory(
        memory_read_response_factory: impl Fn(&MemoryReadRequest) -> MemoryReadResponse + Send + Sync + 'static
    ) -> Self {
        Self {
            captured_project_symbol_catalogs: Arc::new(Mutex::new(Vec::new())),
            memory_read_response_factory: Some(Arc::new(memory_read_response_factory)),
        }
    }

    pub fn captured_project_symbol_catalogs(&self) -> Arc<Mutex<Vec<ProjectSymbolCatalog>>> {
        self.captured_project_symbol_catalogs.clone()
    }
}

impl EngineApiUnprivilegedBindings for MockProjectSymbolsBindings {
    fn dispatch_privileged_command(
        &self,
        engine_command: PrivilegedCommand,
        callback: Box<dyn FnOnce(PrivilegedCommandResponse) + Send + Sync + 'static>,
    ) -> Result<(), EngineBindingError> {
        match engine_command {
            PrivilegedCommand::Registry(RegistryCommand::SetProjectSymbols {
                registry_set_project_symbols_request,
            }) => {
                let mut captured_project_symbol_catalogs = self
                    .captured_project_symbol_catalogs
                    .lock()
                    .map_err(|error| EngineBindingError::lock_failure("capturing project symbol catalog in tests", error.to_string()))?;

                captured_project_symbol_catalogs.push(registry_set_project_symbols_request.project_symbol_catalog);
                drop(captured_project_symbol_catalogs);

                callback(RegistrySetProjectSymbolsResponse { success: true }.to_engine_response());

                Ok(())
            }
            PrivilegedCommand::Memory(MemoryCommand::Read { memory_read_request }) => {
                let Some(memory_read_response_factory) = self.memory_read_response_factory.as_ref() else {
                    return Err(EngineBindingError::unavailable("dispatching memory read in project-symbols tests"));
                };

                callback(memory_read_response_factory(&memory_read_request).to_engine_response());

                Ok(())
            }
            _ => Err(EngineBindingError::unavailable(
                "dispatching unsupported privileged command in project-symbols tests",
            )),
        }
    }

    fn dispatch_unprivileged_command(
        &self,
        _engine_command: UnprivilegedCommand,
        _engine_execution_context: &Arc<dyn EngineExecutionContext>,
        _callback: Box<dyn FnOnce(UnprivilegedCommandResponse) + Send + Sync + 'static>,
    ) -> Result<(), EngineBindingError> {
        Err(EngineBindingError::unavailable("dispatching unprivileged commands in project-symbols tests"))
    }

    fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEventEnvelope>, EngineBindingError> {
        let (_event_sender, event_receiver) = unbounded();

        Ok(event_receiver)
    }
}

pub fn create_engine_unprivileged_state(mock_project_symbols_bindings: MockProjectSymbolsBindings) -> Arc<EngineUnprivilegedState> {
    let engine_bindings: Arc<RwLock<dyn EngineApiUnprivilegedBindings>> = Arc::new(RwLock::new(mock_project_symbols_bindings));

    EngineUnprivilegedState::new_with_options(engine_bindings, EngineUnprivilegedStateOptions { enable_console_logging: false })
}

pub fn create_project_with_symbol_catalog(
    project_directory_path: &Path,
    project_symbol_catalog: ProjectSymbolCatalog,
) -> Project {
    let project_file_path = project_directory_path.join(Project::PROJECT_FILE);
    let root_directory_path = project_directory_path.join(Project::PROJECT_DIR);
    let project_root_ref = ProjectItemRef::new(root_directory_path.clone());
    let project_info = ProjectInfo::new_with_symbol_catalog(project_file_path, None, ProjectManifest::default(), project_symbol_catalog);
    let mut project_items = HashMap::new();

    project_items.insert(project_root_ref.clone(), ProjectItemTypeDirectory::new_project_item(&project_root_ref));

    let mut project = Project::new(project_info, project_items, project_root_ref);
    project
        .save_to_path(project_directory_path, true)
        .expect("Expected test project to save.");

    project
}
