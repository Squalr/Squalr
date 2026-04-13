use crate::command_executors::project::project_symbol_sync::sync_project_symbol_catalog;
use crate::command_executors::project_items::project_item_symbol_resolution::{
    is_promotable_project_item, resolve_project_item_root_locator, resolve_project_item_struct_layout_id, resolve_project_item_type_id,
};
use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project_items::promote_symbol::project_items_promote_symbol_request::ProjectItemsPromoteSymbolRequest;
use squalr_engine_api::commands::project_items::promote_symbol::project_items_promote_symbol_response::ProjectItemsPromoteSymbolResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::memory::pointer::Pointer;
use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_pointer::ProjectItemTypePointer;
use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
use squalr_engine_api::structures::projects::project_root_symbol::ProjectRootSymbol;
use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
use std::path::{Path, PathBuf};
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
                promoted_symbol_count: 0,
                promoted_symbol_keys: Vec::new(),
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

        let mut existing_rooted_symbols = opened_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_rooted_symbols()
            .to_vec();
        let mut promoted_symbols = Vec::new();

        for requested_project_item_path in &self.project_item_paths {
            let project_item_path = resolve_project_item_path(&project_directory_path, requested_project_item_path);
            let project_item_ref = ProjectItemRef::new(project_item_path.clone());
            let Some(project_item) = opened_project
                .get_project_items()
                .get(&project_item_ref)
                .cloned()
            else {
                log::warn!("Skipping promote-symbol request for missing project item: {:?}", project_item_path);
                continue;
            };

            let Some(promoted_symbol) = build_promoted_symbol(engine_unprivileged_state, &project_item_path, &project_item, &existing_rooted_symbols) else {
                log::warn!("Skipping non-promotable project item during promote-symbol request: {:?}", project_item_path);
                continue;
            };

            if existing_rooted_symbols.iter().any(|existing_rooted_symbol| {
                existing_rooted_symbol.get_display_name() == promoted_symbol.get_display_name()
                    && existing_rooted_symbol.get_struct_layout_id() == promoted_symbol.get_struct_layout_id()
                    && existing_rooted_symbol.get_root_locator() == promoted_symbol.get_root_locator()
            }) {
                continue;
            }

            existing_rooted_symbols.push(promoted_symbol.clone());
            promoted_symbols.push(promoted_symbol);
        }

        if promoted_symbols.is_empty() {
            return ProjectItemsPromoteSymbolResponse {
                success: true,
                promoted_symbol_count: 0,
                promoted_symbol_keys: Vec::new(),
            };
        }

        let updated_project_symbol_catalog = {
            let project_info = opened_project.get_project_info_mut();
            let updated_project_symbol_catalog = {
                let project_symbol_catalog = project_info.get_project_symbol_catalog_mut();

                project_symbol_catalog
                    .get_rooted_symbols_mut()
                    .extend(promoted_symbols.iter().cloned());

                project_symbol_catalog.clone()
            };
            project_info.set_has_unsaved_changes(true);

            updated_project_symbol_catalog
        };

        if let Err(error) = opened_project.save_to_path(&project_directory_path, false) {
            log::error!("Failed to save project after promote-symbol operation: {}", error);

            return ProjectItemsPromoteSymbolResponse::default();
        }

        drop(opened_project_guard);

        if !sync_project_symbol_catalog(engine_unprivileged_state, updated_project_symbol_catalog) {
            log::error!("Failed to sync project symbol catalog after promote-symbol operation.");

            return ProjectItemsPromoteSymbolResponse {
                success: false,
                promoted_symbol_count: promoted_symbols.len() as u64,
                promoted_symbol_keys: promoted_symbols
                    .iter()
                    .map(|promoted_symbol| promoted_symbol.get_symbol_key().to_string())
                    .collect(),
            };
        }

        ProjectItemsPromoteSymbolResponse {
            success: true,
            promoted_symbol_count: promoted_symbols.len() as u64,
            promoted_symbol_keys: promoted_symbols
                .iter()
                .map(|promoted_symbol| promoted_symbol.get_symbol_key().to_string())
                .collect(),
        }
    }
}

fn build_promoted_symbol(
    engine_execution_context: &Arc<dyn EngineExecutionContext>,
    project_item_path: &Path,
    project_item: &ProjectItem,
    existing_rooted_symbols: &[ProjectRootSymbol],
) -> Option<ProjectRootSymbol> {
    if !is_promotable_project_item(project_item) {
        return None;
    }

    let display_name = build_display_name(project_item, project_item_path);
    let struct_layout_id = resolve_project_item_struct_layout_id(project_item)?;
    let root_locator = resolve_project_item_root_locator(engine_execution_context, project_item)?;
    let symbol_key = build_unique_symbol_key(&display_name, existing_rooted_symbols);
    let mut promoted_symbol = ProjectRootSymbol::new(symbol_key, display_name, root_locator, struct_layout_id);

    promoted_symbol
        .get_metadata_mut()
        .insert(String::from("source.project_item_path"), project_item_path.to_string_lossy().into_owned());

    if let Some(project_item_type_id) = resolve_project_item_type_id(project_item) {
        promoted_symbol
            .get_metadata_mut()
            .insert(String::from("source.project_item_type"), project_item_type_id.to_string());
    }

    if project_item.get_item_type().get_project_item_type_id() == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
        let pointer = ProjectItemTypePointer::get_field_pointer(project_item);
        append_pointer_metadata(promoted_symbol.get_metadata_mut(), &pointer, project_item);
    }

    Some(promoted_symbol)
}

fn build_display_name(
    project_item: &ProjectItem,
    project_item_path: &Path,
) -> String {
    let project_item_name = project_item.get_field_name();
    let trimmed_project_item_name = project_item_name.trim();

    if !trimmed_project_item_name.is_empty() {
        return trimmed_project_item_name.to_string();
    }

    project_item_path
        .file_stem()
        .and_then(|file_stem| file_stem.to_str())
        .map(str::trim)
        .filter(|file_stem| !file_stem.is_empty())
        .unwrap_or("Symbol")
        .to_string()
}

fn build_unique_symbol_key(
    display_name: &str,
    existing_rooted_symbols: &[ProjectRootSymbol],
) -> String {
    let sanitized_component = sanitize_symbol_key_component(display_name);
    let base_symbol_key = format!("sym.{}", sanitized_component);
    let mut duplicate_sequence_number = 1_u64;
    let mut candidate_symbol_key = base_symbol_key.clone();

    while existing_rooted_symbols
        .iter()
        .any(|existing_rooted_symbol| existing_rooted_symbol.get_symbol_key() == candidate_symbol_key)
    {
        duplicate_sequence_number = duplicate_sequence_number.saturating_add(1);
        candidate_symbol_key = format!("{}.{}", base_symbol_key, duplicate_sequence_number);
    }

    candidate_symbol_key
}

fn sanitize_symbol_key_component(display_name: &str) -> String {
    let mut sanitized_component = String::with_capacity(display_name.len());
    let mut previous_character_was_separator = false;

    for display_name_character in display_name.chars() {
        let mapped_character = if display_name_character.is_ascii_alphanumeric() {
            display_name_character.to_ascii_lowercase()
        } else {
            '.'
        };

        if mapped_character == '.' {
            if previous_character_was_separator {
                continue;
            }

            previous_character_was_separator = true;
        } else {
            previous_character_was_separator = false;
        }

        sanitized_component.push(mapped_character);
    }

    let trimmed_component = sanitized_component.trim_matches('.');

    if trimmed_component.is_empty() {
        String::from("symbol")
    } else {
        trimmed_component.to_string()
    }
}

fn append_pointer_metadata(
    metadata: &mut std::collections::BTreeMap<String, String>,
    pointer: &Pointer,
    project_item: &ProjectItem,
) {
    metadata.insert(String::from("source.pointer_root"), pointer.get_root_display_text());
    metadata.insert(
        String::from("source.pointer_offsets"),
        serde_json::to_string(pointer.get_offsets()).unwrap_or_else(|_| String::from("[]")),
    );
    metadata.insert(String::from("source.pointer_size"), pointer.get_pointer_size().to_string());

    let evaluated_pointer_path = ProjectItemTypePointer::get_field_evaluated_pointer_path(project_item);

    if !evaluated_pointer_path.trim().is_empty() {
        metadata.insert(String::from("source.evaluated_pointer_path"), evaluated_pointer_path);
    }
}

fn resolve_project_item_path(
    project_directory_path: &Path,
    project_item_path: &Path,
) -> PathBuf {
    if project_item_path.is_absolute() {
        project_item_path.to_path_buf()
    } else {
        project_directory_path.join(project_item_path)
    }
}

#[cfg(test)]
mod tests {
    use super::{build_unique_symbol_key, sanitize_symbol_key_component};
    use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
    use crossbeam_channel::{Receiver, unbounded};
    use squalr_engine_api::commands::{
        memory::{memory_command::MemoryCommand, read::memory_read_request::MemoryReadRequest, read::memory_read_response::MemoryReadResponse},
        privileged_command::PrivilegedCommand,
        privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse},
        project_items::promote_symbol::project_items_promote_symbol_request::ProjectItemsPromoteSymbolRequest,
        registry::{registry_command::RegistryCommand, set_project_symbols::registry_set_project_symbols_response::RegistrySetProjectSymbolsResponse},
        unprivileged_command::UnprivilegedCommand,
        unprivileged_command_response::UnprivilegedCommandResponse,
    };
    use squalr_engine_api::engine::{
        engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings, engine_binding_error::EngineBindingError, engine_event_envelope::EngineEventEnvelope,
        engine_execution_context::EngineExecutionContext,
    };
    use squalr_engine_api::structures::{
        data_types::built_in_types::{u8::data_type_u8::DataTypeU8, u64::data_type_u64::DataTypeU64},
        memory::pointer::Pointer,
        pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
        projects::{
            project::Project, project_info::ProjectInfo, project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress,
            project_items::built_in_types::project_item_type_directory::ProjectItemTypeDirectory,
            project_items::built_in_types::project_item_type_pointer::ProjectItemTypePointer, project_items::project_item::ProjectItem,
            project_items::project_item_ref::ProjectItemRef, project_manifest::ProjectManifest, project_root_symbol::ProjectRootSymbol,
            project_root_symbol_locator::ProjectRootSymbolLocator, project_symbol_catalog::ProjectSymbolCatalog,
        },
        structs::valued_struct::ValuedStruct,
    };
    use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
    use squalr_engine_session::engine_unprivileged_state::{EngineUnprivilegedState, EngineUnprivilegedStateOptions};
    use std::{
        collections::HashMap,
        path::{Path, PathBuf},
        sync::{Arc, Mutex, RwLock},
    };

    struct MockPromoteBindings {
        captured_project_symbol_catalogs: Arc<Mutex<Vec<ProjectSymbolCatalog>>>,
        memory_read_response_factory: Arc<dyn Fn(&MemoryReadRequest) -> MemoryReadResponse + Send + Sync>,
    }

    impl MockPromoteBindings {
        fn new(memory_read_response_factory: impl Fn(&MemoryReadRequest) -> MemoryReadResponse + Send + Sync + 'static) -> Self {
            Self {
                captured_project_symbol_catalogs: Arc::new(Mutex::new(Vec::new())),
                memory_read_response_factory: Arc::new(memory_read_response_factory),
            }
        }

        fn captured_project_symbol_catalogs(&self) -> Arc<Mutex<Vec<ProjectSymbolCatalog>>> {
            self.captured_project_symbol_catalogs.clone()
        }
    }

    impl EngineApiUnprivilegedBindings for MockPromoteBindings {
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

                    captured_project_symbol_catalogs.push(
                        registry_set_project_symbols_request
                            .project_symbol_catalog
                            .clone(),
                    );
                    drop(captured_project_symbol_catalogs);

                    callback(RegistrySetProjectSymbolsResponse { success: true }.to_engine_response());

                    Ok(())
                }
                PrivilegedCommand::Memory(MemoryCommand::Read { memory_read_request }) => {
                    callback((self.memory_read_response_factory)(&memory_read_request).to_engine_response());

                    Ok(())
                }
                _ => Err(EngineBindingError::unavailable(
                    "dispatching unsupported privileged command in promote-symbol tests",
                )),
            }
        }

        fn dispatch_unprivileged_command(
            &self,
            _engine_command: UnprivilegedCommand,
            _engine_execution_context: &Arc<dyn EngineExecutionContext>,
            _callback: Box<dyn FnOnce(UnprivilegedCommandResponse) + Send + Sync + 'static>,
        ) -> Result<(), EngineBindingError> {
            Err(EngineBindingError::unavailable("dispatching unprivileged commands in promote-symbol tests"))
        }

        fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEventEnvelope>, EngineBindingError> {
            let (_event_sender, event_receiver) = unbounded();

            Ok(event_receiver)
        }
    }

    fn create_engine_unprivileged_state(mock_promote_bindings: MockPromoteBindings) -> Arc<EngineUnprivilegedState> {
        let engine_bindings: Arc<RwLock<dyn EngineApiUnprivilegedBindings>> = Arc::new(RwLock::new(mock_promote_bindings));

        EngineUnprivilegedState::new_with_options(engine_bindings, EngineUnprivilegedStateOptions { enable_console_logging: false })
    }

    fn create_project_with_item(
        project_directory_path: &Path,
        project_item_file_name: &str,
        project_item: ProjectItem,
    ) -> (Project, PathBuf) {
        let project_file_path = project_directory_path.join(Project::PROJECT_FILE);
        let root_directory_path = project_directory_path.join(Project::PROJECT_DIR);
        let project_root_ref = ProjectItemRef::new(root_directory_path.clone());
        let project_item_path = root_directory_path.join(project_item_file_name);
        let project_item_ref = ProjectItemRef::new(project_item_path.clone());
        let project_info = ProjectInfo::new(project_file_path, None, ProjectManifest::default());
        let mut project_items = HashMap::new();

        project_items.insert(project_root_ref.clone(), ProjectItemTypeDirectory::new_project_item(&project_root_ref));
        project_items.insert(project_item_ref, project_item);

        let mut project = Project::new(project_info, project_items, project_root_ref);
        project
            .save_to_path(project_directory_path, true)
            .expect("Expected test project to save.");

        (project, project_item_path)
    }

    fn create_pointer_memory_read_response(pointer_value: u64) -> MemoryReadResponse {
        MemoryReadResponse {
            valued_struct: ValuedStruct::new_anonymous(vec![
                DataTypeU64::get_value_from_primitive(pointer_value).to_named_valued_struct_field(String::from("value"), true),
            ]),
            address: pointer_value,
            success: true,
        }
    }

    #[test]
    fn sanitize_symbol_key_component_collapses_non_identifier_characters() {
        assert_eq!(sanitize_symbol_key_component("Player Stats (Main)"), "player.stats.main");
    }

    #[test]
    fn build_unique_symbol_key_appends_suffix_for_duplicate_symbol_keys() {
        let existing_rooted_symbols = vec![ProjectRootSymbol::new_absolute_address(
            String::from("sym.player.stats"),
            String::from("Player Stats"),
            0x1234,
            String::from("player.stats"),
        )];

        assert_eq!(
            build_unique_symbol_key("Player Stats", &existing_rooted_symbols),
            String::from("sym.player.stats.2")
        );
    }

    #[test]
    fn promote_symbol_request_persists_address_item_as_rooted_symbol() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let address_project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU8::get_value_from_primitive(0));
        let (project, project_item_path) = create_project_with_item(temp_directory.path(), "health.json", address_project_item);
        let mock_promote_bindings = MockPromoteBindings::new(|_memory_read_request| MemoryReadResponse::default());
        let captured_project_symbol_catalogs = mock_promote_bindings.captured_project_symbol_catalogs();
        let engine_unprivileged_state = create_engine_unprivileged_state(mock_promote_bindings);

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let promote_symbol_response = ProjectItemsPromoteSymbolRequest {
            project_item_paths: vec![project_item_path.clone()],
        }
        .execute(&engine_execution_context);

        assert!(promote_symbol_response.success);
        assert_eq!(promote_symbol_response.promoted_symbol_count, 1);
        assert_eq!(promote_symbol_response.promoted_symbol_keys, vec![String::from("sym.health")]);

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected promoted project to load from disk.");
        let rooted_symbols = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_rooted_symbols();

        assert_eq!(rooted_symbols.len(), 1);
        assert_eq!(rooted_symbols[0].get_symbol_key(), "sym.health");
        assert_eq!(rooted_symbols[0].get_display_name(), "Health");
        assert_eq!(
            rooted_symbols[0].get_root_locator(),
            &ProjectRootSymbolLocator::new_module_offset(String::from("game.exe"), 0x1234)
        );
        assert_eq!(rooted_symbols[0].get_struct_layout_id(), "u8");
        assert_eq!(rooted_symbols[0].get_metadata().get("source.project_item_type"), Some(&String::from("address")));
        assert_eq!(
            rooted_symbols[0].get_metadata().get("source.project_item_path"),
            Some(&project_item_path.to_string_lossy().into_owned())
        );

        let captured_project_symbol_catalogs = captured_project_symbol_catalogs
            .lock()
            .expect("Expected captured symbol catalog lock in test.");
        assert_eq!(captured_project_symbol_catalogs.len(), 1);
        assert_eq!(captured_project_symbol_catalogs[0].get_rooted_symbols(), rooted_symbols);
    }

    #[test]
    fn promote_symbol_request_resolves_pointer_item_tail_and_preserves_pointer_metadata() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let pointer = Pointer::new_with_size(0x1000, vec![0x20], String::from("game.exe"), PointerScanPointerSize::Pointer64);
        let mut pointer_project_item = ProjectItemTypePointer::new_project_item("Player Gold", &pointer, "", "u32");
        ProjectItemTypePointer::set_field_evaluated_pointer_path(&mut pointer_project_item, "game.exe+0x1000 -> 0x2020");
        let (project, project_item_path) = create_project_with_item(temp_directory.path(), "player_gold.json", pointer_project_item);
        let mock_promote_bindings = MockPromoteBindings::new(|memory_read_request| {
            assert_eq!(memory_read_request.address, 0x1000);
            assert_eq!(memory_read_request.module_name, "game.exe");

            create_pointer_memory_read_response(0x2000)
        });
        let engine_unprivileged_state = create_engine_unprivileged_state(mock_promote_bindings);

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let promote_symbol_response = ProjectItemsPromoteSymbolRequest {
            project_item_paths: vec![project_item_path],
        }
        .execute(&engine_execution_context);

        assert!(promote_symbol_response.success);
        assert_eq!(promote_symbol_response.promoted_symbol_count, 1);
        assert_eq!(promote_symbol_response.promoted_symbol_keys, vec![String::from("sym.player.gold")]);

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected promoted project to load from disk.");
        let rooted_symbols = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_rooted_symbols();

        assert_eq!(rooted_symbols.len(), 1);
        assert_eq!(rooted_symbols[0].get_root_locator(), &ProjectRootSymbolLocator::new_absolute_address(0x2020));
        assert_eq!(rooted_symbols[0].get_struct_layout_id(), "u32");
        assert_eq!(rooted_symbols[0].get_metadata().get("source.project_item_type"), Some(&String::from("pointer")));
        assert_eq!(
            rooted_symbols[0].get_metadata().get("source.pointer_root"),
            Some(&String::from("game.exe+0x1000"))
        );
        assert_eq!(rooted_symbols[0].get_metadata().get("source.pointer_offsets"), Some(&String::from("[32]")));
        assert_eq!(
            rooted_symbols[0].get_metadata().get("source.pointer_size"),
            Some(&PointerScanPointerSize::Pointer64.to_string())
        );
        assert_eq!(
            rooted_symbols[0]
                .get_metadata()
                .get("source.evaluated_pointer_path"),
            Some(&String::from("game.exe+0x1000 -> 0x2020"))
        );
    }
}
