use crate::command_executors::project_symbols::project_symbol_plugin_store::EngineProjectSymbolStore;
use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::{
    commands::{
        memory::read::{memory_read_request::MemoryReadRequest, memory_read_response::MemoryReadResponse},
        privileged_command_request::PrivilegedCommandRequest,
        privileged_command_response::TypedPrivilegedCommandResponse,
        project_symbols::execute_plugin_action::{
            project_symbols_execute_plugin_action_request::ProjectSymbolsExecutePluginActionRequest,
            project_symbols_execute_plugin_action_response::ProjectSymbolsExecutePluginActionResponse,
        },
    },
    engine::engine_execution_context::EngineExecutionContext,
    plugins::symbol_tree::symbol_tree_action::{
        DataTypeRegistryStore, ProcessMemoryStore, ProjectSymbolStore, SymbolTreeActionServices, SymbolTreeWindowStore,
    },
    structures::{
        data_types::data_type_ref::DataTypeRef,
        data_values::container_type::ContainerType,
        structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition},
    },
};
use squalr_engine_session::plugins::plugin_registry::PluginRegistry;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;

impl UnprivilegedCommandRequestExecutor for ProjectSymbolsExecutePluginActionRequest {
    type ResponseType = ProjectSymbolsExecutePluginActionResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        let plugin_registry = PluginRegistry::new();
        let Some((plugin_id, symbol_tree_action)) = plugin_registry
            .get_enabled_symbol_tree_actions()
            .into_iter()
            .find(|(plugin_id, symbol_tree_action)| plugin_id == &self.plugin_id && symbol_tree_action.action_id() == self.action_id)
        else {
            return ProjectSymbolsExecutePluginActionResponse {
                success: false,
                error: Some(format!(
                    "Could not resolve enabled Symbol Tree action `{}` from plugin `{}`.",
                    self.action_id, self.plugin_id
                )),
            };
        };

        if !plugin_registry.plugin_action_has_required_permissions(&plugin_id, symbol_tree_action.as_ref()) {
            return ProjectSymbolsExecutePluginActionResponse {
                success: false,
                error: Some(format!(
                    "Plugin `{}` does not declare the permissions required by `{}`.",
                    plugin_id, self.action_id
                )),
            };
        }

        let symbol_tree_action_services = EngineSymbolTreeActionServices::new(engine_unprivileged_state.clone());

        match symbol_tree_action.execute(&self.context, &symbol_tree_action_services) {
            Ok(()) => ProjectSymbolsExecutePluginActionResponse { success: true, error: None },
            Err(error) => ProjectSymbolsExecutePluginActionResponse {
                success: false,
                error: Some(error),
            },
        }
    }
}

struct EngineSymbolTreeActionServices {
    engine_execution_context: Arc<dyn EngineExecutionContext>,
    project_symbol_store: EngineProjectSymbolStore,
    symbol_tree_window_store: EngineSymbolTreeWindowStore,
}

impl EngineSymbolTreeActionServices {
    fn new(engine_unprivileged_state: Arc<dyn EngineExecutionContext>) -> Self {
        Self {
            engine_execution_context: engine_unprivileged_state.clone(),
            project_symbol_store: EngineProjectSymbolStore::new(engine_unprivileged_state),
            symbol_tree_window_store: EngineSymbolTreeWindowStore,
        }
    }
}

impl SymbolTreeActionServices for EngineSymbolTreeActionServices {
    fn symbol_store(&self) -> &dyn ProjectSymbolStore {
        &self.project_symbol_store
    }

    fn process_memory(&self) -> &dyn ProcessMemoryStore {
        self
    }

    fn data_type_registry(&self) -> &dyn DataTypeRegistryStore {
        self
    }

    fn symbol_tree_window(&self) -> &dyn SymbolTreeWindowStore {
        &self.symbol_tree_window_store
    }
}

impl DataTypeRegistryStore for EngineSymbolTreeActionServices {
    fn get_registered_data_type_refs(&self) -> Vec<DataTypeRef> {
        self.engine_execution_context.get_registered_data_type_refs()
    }

    fn get_unit_size_in_bytes(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> u64 {
        self.engine_execution_context
            .get_unit_size_in_bytes(data_type_ref)
    }
}

impl ProcessMemoryStore for EngineSymbolTreeActionServices {
    fn read_module_bytes(
        &self,
        module_name: &str,
        offset: u64,
        length: u64,
    ) -> Result<Vec<u8>, String> {
        if length == 0 {
            return Ok(Vec::new());
        }

        let memory_read_request = MemoryReadRequest {
            address: offset,
            module_name: module_name.to_string(),
            symbolic_struct_definition: SymbolicStructDefinition::new_anonymous(vec![SymbolicFieldDefinition::new(
                DataTypeRef::new("u8"),
                ContainerType::ArrayFixed(length),
            )]),
            suppress_logging: true,
        };
        let memory_read_command = memory_read_request.to_engine_command();
        let (memory_read_response_sender, memory_read_response_receiver) = mpsc::channel();

        self.engine_execution_context
            .get_bindings()
            .read()
            .map_err(|error| format!("Failed to acquire engine bindings while reading process memory: {error}"))?
            .dispatch_privileged_command(
                memory_read_command,
                Box::new(move |memory_read_response| {
                    let conversion_result = MemoryReadResponse::from_engine_response(memory_read_response);
                    let _ = memory_read_response_sender.send(conversion_result);
                }),
            )
            .map_err(|error| format!("Failed to dispatch process memory read for Symbol Tree plugin action: {error}"))?;

        let memory_read_response = memory_read_response_receiver
            .recv_timeout(Duration::from_secs(2))
            .map_err(|error| format!("Timed out reading process memory for Symbol Tree plugin action: {error}"))?
            .map_err(|_| String::from("Process memory read returned an unexpected response type."))?;

        if !memory_read_response.success {
            return Err(format!("Failed to read {} byte(s) from {}+0x{:X}.", length, module_name, offset));
        }

        Ok(memory_read_response.valued_struct.get_bytes())
    }
}

struct EngineSymbolTreeWindowStore;

impl SymbolTreeWindowStore for EngineSymbolTreeWindowStore {
    fn request_refresh(&self) {}

    fn focus_tree_node(
        &self,
        _tree_node_key: &str,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectSymbolsExecutePluginActionRequest;
    use crate::command_executors::project_symbols::test_support::{
        MockProjectSymbolsBindings, create_engine_unprivileged_state, create_project_with_symbol_catalog,
    };
    use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
    use squalr_engine_api::{
        commands::memory::read::{memory_read_request::MemoryReadRequest, memory_read_response::MemoryReadResponse},
        engine::engine_execution_context::EngineExecutionContext,
        plugins::symbol_tree::symbol_tree_action::{SymbolTreeActionContext, SymbolTreeActionSelection},
        registries::symbols::symbol_registry::SymbolRegistry,
        structures::projects::{project::Project, project_symbol_catalog::ProjectSymbolCatalog, project_symbol_module::ProjectSymbolModule},
    };
    use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
    use std::sync::Arc;

    #[test]
    fn execute_plugin_action_populates_pe_symbols() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let project_symbol_catalog =
            ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![ProjectSymbolModule::new(String::from("game.exe"), 0x2000)], Vec::new(), Vec::new());
        let project = create_project_with_symbol_catalog(temp_directory.path(), project_symbol_catalog);
        let engine_unprivileged_state = create_engine_unprivileged_state(MockProjectSymbolsBindings::new_with_memory_read_response_factory(
            create_test_pe_memory_read_response,
        ));

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let response = ProjectSymbolsExecutePluginActionRequest {
            plugin_id: String::from("builtin.symbols.binary"),
            action_id: String::from("builtin.symbols.binary.populate-binary-symbols"),
            context: SymbolTreeActionContext::new(SymbolTreeActionSelection::ModuleRoot {
                module_name: String::from("game.exe"),
            }),
        }
        .execute(&engine_execution_context);

        assert!(response.success, "Expected plugin action to succeed: {:?}", response.error);

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected plugin-updated project to load from disk.");
        let symbol_module = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .find_symbol_module("game.exe")
            .expect("Expected module to remain in catalog.");
        let project_symbol_catalog = loaded_project.get_project_info().get_project_symbol_catalog();
        let pe_headers_descriptor = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == "win.pe.PE_HEADERS32")
            .expect("Expected PE headers layout descriptor.");
        let pe_header_field_names = pe_headers_descriptor
            .get_struct_layout_definition()
            .get_fields()
            .iter()
            .map(|field_definition| field_definition.get_field_name())
            .collect::<Vec<_>>();

        assert_eq!(symbol_module.get_fields().len(), 1);
        assert_eq!(symbol_module.get_fields()[0].get_display_name(), "PE Headers");
        assert_eq!(symbol_module.get_fields()[0].get_struct_layout_id(), "win.pe.PE_HEADERS32");
        assert_eq!(pe_header_field_names, vec!["DOSHeader", "DOSStub", "NTHeaders", "SectionHeaders"]);
    }

    #[test]
    fn execute_plugin_action_populates_macho_symbols() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let project_symbol_catalog =
            ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![ProjectSymbolModule::new(String::from("Finder"), 0x2000)], Vec::new(), Vec::new());
        let project = create_project_with_symbol_catalog(temp_directory.path(), project_symbol_catalog);
        let engine_unprivileged_state = create_engine_unprivileged_state(MockProjectSymbolsBindings::new_with_memory_read_response_factory(
            create_test_macho_memory_read_response,
        ));

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let response = ProjectSymbolsExecutePluginActionRequest {
            plugin_id: String::from("builtin.symbols.binary"),
            action_id: String::from("builtin.symbols.binary.populate-binary-symbols"),
            context: SymbolTreeActionContext::new(SymbolTreeActionSelection::ModuleRoot {
                module_name: String::from("Finder"),
            }),
        }
        .execute(&engine_execution_context);

        assert!(response.success, "Expected plugin action to succeed: {:?}", response.error);

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected plugin-updated project to load from disk.");
        let symbol_module = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .find_symbol_module("Finder")
            .expect("Expected module to remain in catalog.");
        let project_symbol_catalog = loaded_project.get_project_info().get_project_symbol_catalog();
        let macho_headers_descriptor = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == "mac.macho.finder.headers")
            .expect("Expected Mach-O headers layout descriptor.");
        let load_commands_descriptor = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == "mac.macho.finder.load_commands")
            .expect("Expected Mach-O load commands layout descriptor.");
        let macho_header_field_names = macho_headers_descriptor
            .get_struct_layout_definition()
            .get_fields()
            .iter()
            .map(|field_definition| field_definition.get_field_name())
            .collect::<Vec<_>>();
        let load_command_field_names = load_commands_descriptor
            .get_struct_layout_definition()
            .get_fields()
            .iter()
            .map(|field_definition| field_definition.get_field_name())
            .collect::<Vec<_>>();

        assert_eq!(symbol_module.get_fields().len(), 1);
        assert_eq!(symbol_module.get_fields()[0].get_display_name(), "Mach-O Headers");
        assert_eq!(symbol_module.get_fields()[0].get_struct_layout_id(), "mac.macho.finder.headers");
        assert_eq!(macho_header_field_names, vec!["Header", "LoadCommands"]);
        assert_eq!(load_command_field_names, vec!["SegmentCommand00", "SymtabCommand01"]);
    }

    fn create_test_pe_memory_read_response(memory_read_request: &MemoryReadRequest) -> MemoryReadResponse {
        let header_bytes = build_test_pe_header_bytes();
        create_memory_read_response_from_bytes(memory_read_request, &header_bytes)
    }

    fn create_test_macho_memory_read_response(memory_read_request: &MemoryReadRequest) -> MemoryReadResponse {
        let header_bytes = build_test_macho_header_bytes();
        create_memory_read_response_from_bytes(memory_read_request, &header_bytes)
    }

    fn create_memory_read_response_from_bytes(
        memory_read_request: &MemoryReadRequest,
        header_bytes: &[u8],
    ) -> MemoryReadResponse {
        let read_start = memory_read_request.address as usize;
        let read_end = read_start.saturating_add(
            memory_read_request
                .symbolic_struct_definition
                .get_size_in_bytes(&SymbolRegistry::new()) as usize,
        );
        let mut valued_struct = memory_read_request
            .symbolic_struct_definition
            .get_default_valued_struct(&SymbolRegistry::new());
        let readable_end = read_end.min(header_bytes.len());

        let _ = valued_struct.copy_from_bytes(&header_bytes[read_start..readable_end]);

        MemoryReadResponse {
            valued_struct,
            address: memory_read_request.address,
            success: readable_end.saturating_sub(read_start) == read_end.saturating_sub(read_start),
        }
    }

    fn build_test_pe_header_bytes() -> Vec<u8> {
        let mut header_bytes = vec![0_u8; 0x1000];
        header_bytes[0..2].copy_from_slice(b"MZ");
        header_bytes[0x3C..0x40].copy_from_slice(&0x80_u32.to_le_bytes());
        header_bytes[0x80..0x84].copy_from_slice(b"PE\0\0");
        header_bytes[0x86..0x88].copy_from_slice(&3_u16.to_le_bytes());
        header_bytes[0x94..0x96].copy_from_slice(&0xE0_u16.to_le_bytes());
        header_bytes[0x98..0x9A].copy_from_slice(&0x10B_u16.to_le_bytes());

        header_bytes
    }

    fn build_test_macho_header_bytes() -> Vec<u8> {
        let mut header_bytes = vec![0_u8; 0x1000];
        header_bytes[0..4].copy_from_slice(&[0xCF, 0xFA, 0xED, 0xFE]);
        header_bytes[16..20].copy_from_slice(&2_u32.to_le_bytes());
        header_bytes[20..24].copy_from_slice(&0x60_u32.to_le_bytes());

        let first_command_offset = 32_usize;
        header_bytes[first_command_offset..first_command_offset + 4].copy_from_slice(&0x19_u32.to_le_bytes());
        header_bytes[first_command_offset + 4..first_command_offset + 8].copy_from_slice(&0x48_u32.to_le_bytes());
        header_bytes[first_command_offset + 64..first_command_offset + 68].copy_from_slice(&0x1_u32.to_le_bytes());
        header_bytes[first_command_offset + 72..first_command_offset + 88].copy_from_slice(b"__text\0\0\0\0\0\0\0\0\0\0");
        header_bytes[first_command_offset + 88..first_command_offset + 104].copy_from_slice(b"__TEXT\0\0\0\0\0\0\0\0\0\0");

        let second_command_offset = first_command_offset + 0x48;
        header_bytes[second_command_offset..second_command_offset + 4].copy_from_slice(&0x2_u32.to_le_bytes());
        header_bytes[second_command_offset + 4..second_command_offset + 8].copy_from_slice(&0x18_u32.to_le_bytes());
        header_bytes[second_command_offset + 8..second_command_offset + 12].copy_from_slice(&0x1000_u32.to_le_bytes());
        header_bytes[second_command_offset + 12..second_command_offset + 16].copy_from_slice(&0x10_u32.to_le_bytes());
        header_bytes[second_command_offset + 16..second_command_offset + 20].copy_from_slice(&0x2000_u32.to_le_bytes());
        header_bytes[second_command_offset + 20..second_command_offset + 24].copy_from_slice(&0x80_u32.to_le_bytes());

        header_bytes
    }
}
