use crate::command_executors::project::project_symbol_sync::sync_project_symbol_catalog;
use crate::command_executors::project_items::project_item_symbol_resolution::{
    is_promotable_project_item, resolve_project_item_locator, resolve_project_item_struct_layout_id, resolve_project_item_type_id,
};
use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project_items::promote_symbol::project_items_promote_symbol_request::ProjectItemsPromoteSymbolRequest;
use squalr_engine_api::commands::project_items::promote_symbol::project_items_promote_symbol_response::{
    ProjectItemsPromoteSymbolConflict, ProjectItemsPromoteSymbolResponse,
};
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::memory::pointer::Pointer;
use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_symbol_ref::ProjectItemTypeSymbolRef;
use squalr_engine_api::structures::projects::project_items::built_in_types::{
    project_item_type_address::ProjectItemTypeAddress, project_item_type_pointer::ProjectItemTypePointer,
};
use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
use squalr_engine_api::structures::projects::project_symbol_claim::ProjectSymbolClaim;
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

        let mut existing_symbol_claims = opened_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_symbol_claims()
            .to_vec();
        let mut project_item_replacements = Vec::new();
        let mut promoted_symbol_locator_keys = Vec::new();
        let mut promoted_symbol_count = 0_u64;
        let mut reused_symbol_count = 0_u64;
        let mut conflicts = Vec::new();
        let mut did_mutate_symbol_catalog = false;

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

            let Some(promoted_symbol_candidate) = build_promoted_symbol(
                engine_unprivileged_state,
                opened_project.get_project_info().get_project_symbol_catalog(),
                &project_item_path,
                &project_item,
            ) else {
                log::warn!("Skipping non-promotable project item during promote-symbol request: {:?}", project_item_path);
                continue;
            };

            if let Some(existing_exact_symbol) = find_exact_symbol_claim(&existing_symbol_claims, &promoted_symbol_candidate).cloned() {
                project_item_replacements.push((project_item_ref, build_symbol_ref_project_item(&project_item, &existing_exact_symbol)));
                reused_symbol_count = reused_symbol_count.saturating_add(1);
                continue;
            }

            if let Some(conflicting_symbol_index) =
                find_symbol_claim_index_by_locator_key(&existing_symbol_claims, &promoted_symbol_candidate.get_symbol_locator_key())
            {
                if !self.overwrite_conflicting_symbols {
                    conflicts.push(ProjectItemsPromoteSymbolConflict {
                        project_item_path: project_item_path.clone(),
                        symbol_locator_key: promoted_symbol_candidate.get_symbol_locator_key().to_string(),
                        existing_display_name: existing_symbol_claims[conflicting_symbol_index]
                            .get_display_name()
                            .to_string(),
                        existing_locator_display: existing_symbol_claims[conflicting_symbol_index]
                            .get_locator()
                            .to_string(),
                        requested_display_name: promoted_symbol_candidate.get_display_name().to_string(),
                    });
                    continue;
                }

                existing_symbol_claims[conflicting_symbol_index] = promoted_symbol_candidate.clone();
            } else {
                existing_symbol_claims.push(promoted_symbol_candidate.clone());
            }

            did_mutate_symbol_catalog = true;
            promoted_symbol_count = promoted_symbol_count.saturating_add(1);
            promoted_symbol_locator_keys.push(promoted_symbol_candidate.get_symbol_locator_key().to_string());
            project_item_replacements.push((project_item_ref, build_symbol_ref_project_item(&project_item, &promoted_symbol_candidate)));
        }

        if project_item_replacements.is_empty() && !did_mutate_symbol_catalog {
            return ProjectItemsPromoteSymbolResponse {
                success: true,
                promoted_symbol_count,
                reused_symbol_count,
                promoted_symbol_locator_keys,
                conflicts,
            };
        }

        for (project_item_ref, replacement_project_item) in &project_item_replacements {
            if let Some(project_item) = opened_project.get_project_items_mut().get_mut(project_item_ref) {
                *project_item = replacement_project_item.clone();
            }
        }

        let updated_project_symbol_catalog = if did_mutate_symbol_catalog {
            let project_info = opened_project.get_project_info_mut();
            let updated_project_symbol_catalog = {
                let project_symbol_catalog = project_info.get_project_symbol_catalog_mut();
                project_symbol_catalog.set_symbol_claims(existing_symbol_claims.clone());
                project_symbol_catalog.clone()
            };
            project_info.set_has_unsaved_changes(true);

            Some(updated_project_symbol_catalog)
        } else {
            opened_project
                .get_project_info_mut()
                .set_has_unsaved_changes(true);
            None
        };

        if let Err(error) = opened_project.save_to_path(&project_directory_path, false) {
            log::error!("Failed to save project after promote-symbol operation: {}", error);

            return ProjectItemsPromoteSymbolResponse::default();
        }

        drop(opened_project_guard);

        if let Some(updated_project_symbol_catalog) = updated_project_symbol_catalog {
            if !sync_project_symbol_catalog(engine_unprivileged_state, updated_project_symbol_catalog) {
                log::error!("Failed to sync project symbol catalog after promote-symbol operation.");

                return ProjectItemsPromoteSymbolResponse {
                    success: false,
                    promoted_symbol_count,
                    reused_symbol_count,
                    promoted_symbol_locator_keys,
                    conflicts,
                };
            }
        }

        ProjectItemsPromoteSymbolResponse {
            success: true,
            promoted_symbol_count,
            reused_symbol_count,
            promoted_symbol_locator_keys,
            conflicts,
        }
    }
}

fn build_promoted_symbol(
    engine_execution_context: &Arc<dyn EngineExecutionContext>,
    project_symbol_catalog: &squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog,
    project_item_path: &Path,
    project_item: &ProjectItem,
) -> Option<ProjectSymbolClaim> {
    if !is_promotable_project_item(project_item) {
        return None;
    }

    let display_name = build_display_name(project_item, project_item_path);
    let struct_layout_id = resolve_project_item_struct_layout_id(project_symbol_catalog, project_item)?;
    let locator = resolve_project_item_locator(engine_execution_context, project_symbol_catalog, project_item)?;
    let mut promoted_symbol = ProjectSymbolClaim::new(display_name, locator, struct_layout_id);

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

fn build_symbol_ref_project_item(
    source_project_item: &ProjectItem,
    promoted_symbol: &ProjectSymbolClaim,
) -> ProjectItem {
    let mut symbol_ref_project_item = ProjectItemTypeSymbolRef::new_project_item(
        source_project_item.get_field_name().as_str(),
        &promoted_symbol.get_symbol_locator_key(),
        &source_project_item.get_field_description(),
    );

    if source_project_item.get_is_activated() {
        symbol_ref_project_item.toggle_activated();
    }

    let freeze_display_value = if source_project_item.get_item_type().get_project_item_type_id() == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
        ProjectItemTypePointer::get_field_freeze_data_value_interpreter(source_project_item)
    } else {
        let mut address_project_item = source_project_item.clone();

        ProjectItemTypeAddress::get_field_freeze_data_value_interpreter(&mut address_project_item)
    };

    ProjectItemTypeSymbolRef::set_field_freeze_data_value_interpreter(&mut symbol_ref_project_item, &freeze_display_value);
    ProjectItemTypeSymbolRef::set_field_symbol_locator_display(&mut symbol_ref_project_item, &promoted_symbol.get_locator().to_string());

    symbol_ref_project_item
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

fn append_pointer_metadata(
    metadata: &mut std::collections::BTreeMap<String, String>,
    pointer: &Pointer,
    project_item: &ProjectItem,
) {
    metadata.insert(String::from("source.pointer_root"), pointer.get_root_display_text());
    metadata.insert(String::from("source.pointer_root_module"), pointer.get_module_name().to_string());
    metadata.insert(String::from("source.pointer_root_offset"), format!("0x{:X}", pointer.get_address()));
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

fn find_exact_symbol_claim<'a>(
    existing_symbol_claims: &'a [ProjectSymbolClaim],
    promoted_symbol: &ProjectSymbolClaim,
) -> Option<&'a ProjectSymbolClaim> {
    existing_symbol_claims.iter().find(|existing_symbol_claim| {
        existing_symbol_claim.get_display_name() == promoted_symbol.get_display_name()
            && existing_symbol_claim.get_struct_layout_id() == promoted_symbol.get_struct_layout_id()
            && existing_symbol_claim.get_locator() == promoted_symbol.get_locator()
    })
}

fn find_symbol_claim_index_by_locator_key(
    existing_symbol_claims: &[ProjectSymbolClaim],
    symbol_locator_key: &str,
) -> Option<usize> {
    existing_symbol_claims
        .iter()
        .position(|existing_symbol_claim| existing_symbol_claim.get_symbol_locator_key() == symbol_locator_key)
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
            project_items::built_in_types::project_item_type_pointer::ProjectItemTypePointer,
            project_items::built_in_types::project_item_type_symbol_ref::ProjectItemTypeSymbolRef, project_items::project_item::ProjectItem,
            project_items::project_item_ref::ProjectItemRef, project_manifest::ProjectManifest, project_symbol_catalog::ProjectSymbolCatalog,
            project_symbol_claim::ProjectSymbolClaim, project_symbol_locator::ProjectSymbolLocator,
        },
        structs::valued_struct::ValuedStruct,
    };
    use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
    use squalr_engine_session::engine_unprivileged_state::{EngineUnprivilegedState, EngineUnprivilegedStateOptions};
    use std::{
        collections::HashMap,
        fs::File,
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
        std::fs::create_dir_all(&root_directory_path).expect("Expected test project root directory to be created.");
        File::create(&project_item_path).expect("Expected test project item placeholder file to be created.");

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
    fn promote_symbol_request_persists_address_item_as_symbol_claim() {
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
            overwrite_conflicting_symbols: false,
        }
        .execute(&engine_execution_context);

        assert!(promote_symbol_response.success);
        assert_eq!(promote_symbol_response.promoted_symbol_count, 1);
        assert_eq!(promote_symbol_response.reused_symbol_count, 0);
        assert_eq!(promote_symbol_response.promoted_symbol_locator_keys, vec![String::from("module:game.exe:1234")]);
        assert!(promote_symbol_response.conflicts.is_empty());

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected promoted project to load from disk.");
        let symbol_claims = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_symbol_claims();

        assert_eq!(symbol_claims.len(), 1);
        assert_eq!(symbol_claims[0].get_symbol_locator_key(), "module:game.exe:1234");
        assert_eq!(symbol_claims[0].get_display_name(), "Health");
        assert_eq!(
            symbol_claims[0].get_locator(),
            &ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0x1234)
        );
        assert_eq!(symbol_claims[0].get_struct_layout_id(), "u8");
        assert_eq!(symbol_claims[0].get_metadata().get("source.project_item_type"), Some(&String::from("address")));
        assert_eq!(
            symbol_claims[0].get_metadata().get("source.project_item_path"),
            Some(&project_item_path.to_string_lossy().into_owned())
        );
        let promoted_project_item = loaded_project
            .get_project_items()
            .get(&ProjectItemRef::new(project_item_path.clone()))
            .expect("Expected promoted project item to remain in the project.");

        assert_eq!(
            promoted_project_item.get_item_type().get_project_item_type_id(),
            ProjectItemTypeSymbolRef::PROJECT_ITEM_TYPE_ID
        );
        assert_eq!(
            ProjectItemTypeSymbolRef::get_field_symbol_locator_key(promoted_project_item),
            "module:game.exe:1234"
        );

        let captured_project_symbol_catalogs = captured_project_symbol_catalogs
            .lock()
            .expect("Expected captured symbol catalog lock in test.");
        assert_eq!(captured_project_symbol_catalogs.len(), 1);
        assert_eq!(captured_project_symbol_catalogs[0].get_symbol_claims(), symbol_claims);
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
            overwrite_conflicting_symbols: false,
        }
        .execute(&engine_execution_context);

        assert!(promote_symbol_response.success);
        assert_eq!(promote_symbol_response.promoted_symbol_count, 1);
        assert_eq!(promote_symbol_response.reused_symbol_count, 0);
        assert_eq!(promote_symbol_response.promoted_symbol_locator_keys, vec![String::from("absolute:2020")]);
        assert!(promote_symbol_response.conflicts.is_empty());

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected promoted project to load from disk.");
        let symbol_claims = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_symbol_claims();

        assert_eq!(symbol_claims.len(), 1);
        assert_eq!(symbol_claims[0].get_locator(), &ProjectSymbolLocator::new_absolute_address(0x2020));
        assert_eq!(symbol_claims[0].get_struct_layout_id(), "u32");
        assert_eq!(symbol_claims[0].get_metadata().get("source.project_item_type"), Some(&String::from("pointer")));
        assert_eq!(
            symbol_claims[0].get_metadata().get("source.pointer_root"),
            Some(&String::from("game.exe+0x1000"))
        );
        assert_eq!(symbol_claims[0].get_metadata().get("source.pointer_offsets"), Some(&String::from("[32]")));
        assert_eq!(
            symbol_claims[0].get_metadata().get("source.pointer_size"),
            Some(&PointerScanPointerSize::Pointer64.to_string())
        );
        assert_eq!(
            symbol_claims[0]
                .get_metadata()
                .get("source.evaluated_pointer_path"),
            Some(&String::from("game.exe+0x1000 -> 0x2020"))
        );
        let promoted_project_item = loaded_project
            .get_project_items()
            .values()
            .find(|project_item| project_item.get_item_type().get_project_item_type_id() == ProjectItemTypeSymbolRef::PROJECT_ITEM_TYPE_ID)
            .expect("Expected pointer promotion to replace the project item with a symbol ref.");

        assert_eq!(ProjectItemTypeSymbolRef::get_field_symbol_locator_key(promoted_project_item), "absolute:2020");
        assert_eq!(
            symbol_claims[0]
                .get_metadata()
                .get("source.pointer_root_module"),
            Some(&String::from("game.exe"))
        );
        assert_eq!(
            symbol_claims[0]
                .get_metadata()
                .get("source.pointer_root_offset"),
            Some(&String::from("0x1000"))
        );
    }

    #[test]
    fn promote_symbol_request_reuses_exact_existing_symbol_and_replaces_project_item() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let address_project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU8::get_value_from_primitive(0));
        let (mut project, project_item_path) = create_project_with_item(temp_directory.path(), "health.json", address_project_item);
        project
            .get_project_info_mut()
            .get_project_symbol_catalog_mut()
            .set_symbol_claims(vec![ProjectSymbolClaim::new_module_offset(
                String::from("Health"),
                String::from("game.exe"),
                0x1234,
                String::from("u8"),
            )]);
        project
            .save_to_path(temp_directory.path(), true)
            .expect("Expected test project to save after seeding symbol catalog.");
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
            overwrite_conflicting_symbols: false,
        }
        .execute(&engine_execution_context);

        assert!(promote_symbol_response.success);
        assert_eq!(promote_symbol_response.promoted_symbol_count, 0);
        assert_eq!(promote_symbol_response.reused_symbol_count, 1);
        assert!(promote_symbol_response.promoted_symbol_locator_keys.is_empty());
        assert!(promote_symbol_response.conflicts.is_empty());

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected promoted project to load from disk.");
        let symbol_claims = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_symbol_claims();
        let promoted_project_item = loaded_project
            .get_project_items()
            .get(&ProjectItemRef::new(project_item_path))
            .expect("Expected project item to remain in project after reuse.");

        assert_eq!(symbol_claims.len(), 1);
        assert_eq!(
            ProjectItemTypeSymbolRef::get_field_symbol_locator_key(promoted_project_item),
            "module:game.exe:1234"
        );
        assert!(
            captured_project_symbol_catalogs
                .lock()
                .expect("Expected captured symbol catalog lock in test.")
                .is_empty()
        );
    }

    #[test]
    fn promote_symbol_request_reports_conflicts_without_overwriting_existing_symbol() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let address_project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU8::get_value_from_primitive(0));
        let (mut project, project_item_path) = create_project_with_item(temp_directory.path(), "health.json", address_project_item);
        project
            .get_project_info_mut()
            .get_project_symbol_catalog_mut()
            .set_symbol_claims(vec![ProjectSymbolClaim::new_module_offset(
                String::from("Other Health"),
                String::from("game.exe"),
                0x1234,
                String::from("u8"),
            )]);
        project
            .save_to_path(temp_directory.path(), true)
            .expect("Expected test project to save after seeding conflicting symbol.");
        let mock_promote_bindings = MockPromoteBindings::new(|_memory_read_request| MemoryReadResponse::default());
        let engine_unprivileged_state = create_engine_unprivileged_state(mock_promote_bindings);

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let promote_symbol_response = ProjectItemsPromoteSymbolRequest {
            project_item_paths: vec![project_item_path.clone()],
            overwrite_conflicting_symbols: false,
        }
        .execute(&engine_execution_context);

        assert!(promote_symbol_response.success);
        assert_eq!(promote_symbol_response.promoted_symbol_count, 0);
        assert_eq!(promote_symbol_response.reused_symbol_count, 0);
        assert!(promote_symbol_response.promoted_symbol_locator_keys.is_empty());
        assert_eq!(promote_symbol_response.conflicts.len(), 1);
        assert_eq!(promote_symbol_response.conflicts[0].project_item_path, project_item_path);
        assert_eq!(promote_symbol_response.conflicts[0].symbol_locator_key, "module:game.exe:1234");
    }

    #[test]
    fn promote_symbol_request_overwrites_conflicting_symbol_when_requested() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let address_project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU8::get_value_from_primitive(0));
        let (mut project, project_item_path) = create_project_with_item(temp_directory.path(), "health.json", address_project_item);
        project
            .get_project_info_mut()
            .get_project_symbol_catalog_mut()
            .set_symbol_claims(vec![ProjectSymbolClaim::new_module_offset(
                String::from("Other Health"),
                String::from("game.exe"),
                0x1234,
                String::from("u8"),
            )]);
        project
            .save_to_path(temp_directory.path(), true)
            .expect("Expected test project to save after seeding conflicting symbol.");
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
            overwrite_conflicting_symbols: true,
        }
        .execute(&engine_execution_context);

        assert!(promote_symbol_response.success);
        assert_eq!(promote_symbol_response.promoted_symbol_count, 1);
        assert_eq!(promote_symbol_response.reused_symbol_count, 0);
        assert_eq!(promote_symbol_response.promoted_symbol_locator_keys, vec![String::from("module:game.exe:1234")]);
        assert!(promote_symbol_response.conflicts.is_empty());

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected promoted project to load from disk.");
        let symbol_claims = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_symbol_claims();

        assert_eq!(symbol_claims.len(), 1);
        assert_eq!(
            symbol_claims[0].get_locator(),
            &ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0x1234)
        );
        assert_eq!(
            loaded_project
                .get_project_items()
                .get(&ProjectItemRef::new(project_item_path))
                .map(ProjectItemTypeSymbolRef::get_field_symbol_locator_key),
            Some(String::from("module:game.exe:1234"))
        );
        assert_eq!(
            captured_project_symbol_catalogs
                .lock()
                .expect("Expected captured symbol catalog lock in test.")
                .len(),
            1
        );
    }
}
