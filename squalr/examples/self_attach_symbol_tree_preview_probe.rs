#![cfg(target_os = "macos")]

use eframe::egui::Context;
use squalr::app_context::AppContext;
use squalr::models::docking::{docking_manager::DockingManager, hierarchy::dock_node::DockNode};
use squalr::ui::theme::Theme;
use squalr::views::symbol_tree::symbol_tree_runtime_data_controller::SymbolTreeRuntimeDataController;
use squalr_engine::engine_bindings::standalone::standalone_engine_api_unprivileged_bindings::StandaloneEngineApiUnprivilegedBindings;
use squalr_engine::engine_mode::EngineMode;
use squalr_engine::engine_privileged_state::create_engine_privileged_state;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::process::close::process_close_request::ProcessCloseRequest;
use squalr_engine_api::commands::process::open::process_open_request::ProcessOpenRequest;
use squalr_engine_api::events::process::changed::process_changed_event::ProcessChangedEvent;
use squalr_engine_api::plugins::symbol_tree::symbol_tree_action::{
    DataTypeRegistryStore, ProcessMemoryStore, ProjectSymbolStore, SymbolTreeActionContext, SymbolTreeActionSelection, SymbolTreeActionServices,
    SymbolTreeWindowStore,
};
use squalr_engine_api::plugins::symbol_tree::symbol_tree_plugin::SymbolTreePlugin;
use squalr_engine_api::structures::projects::{
    project_symbol_catalog::ProjectSymbolCatalog, project_symbol_module::ProjectSymbolModule,
};
use squalr_engine_api::structures::structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition};
use squalr_engine_session::engine_unprivileged_state::{EngineUnprivilegedState, EngineUnprivilegedStateOptions};
use squalr_plugin_binary_symbols::BinarySymbolsPlugin;
use std::collections::HashSet;
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock, mpsc};
use std::thread;
use std::time::{Duration, Instant};

struct ProbeProjectSymbolStore {
    project_symbol_catalog: Arc<Mutex<ProjectSymbolCatalog>>,
}

impl ProjectSymbolStore for ProbeProjectSymbolStore {
    fn read_catalog(&self) -> Result<ProjectSymbolCatalog, String> {
        self.project_symbol_catalog
            .lock()
            .map(|project_symbol_catalog| project_symbol_catalog.clone())
            .map_err(|error| format!("Failed to read probe project symbol catalog: {error}"))
    }

    fn write_catalog(
        &self,
        _reason: &str,
        update_catalog: Box<dyn FnOnce(&mut ProjectSymbolCatalog) -> Result<(), String> + Send>,
    ) -> Result<(), String> {
        let mut project_symbol_catalog = self
            .project_symbol_catalog
            .lock()
            .map_err(|error| format!("Failed to write probe project symbol catalog: {error}"))?;

        update_catalog(&mut project_symbol_catalog)
    }
}

struct ProbeProcessMemoryStore {
    engine_unprivileged_state: Arc<EngineUnprivilegedState>,
}

impl ProcessMemoryStore for ProbeProcessMemoryStore {
    fn read_module_bytes(
        &self,
        module_name: &str,
        offset: u64,
        length: u64,
    ) -> Result<Vec<u8>, String> {
        let symbolic_struct_definition = SymbolicStructDefinition::new_anonymous(vec![SymbolicFieldDefinition::new(
            squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef::new("u8"),
            squalr_engine_api::structures::data_values::container_type::ContainerType::ArrayFixed(length),
        )]);
        let memory_read_request = squalr_engine_api::commands::memory::read::memory_read_request::MemoryReadRequest {
            address: offset,
            module_name: module_name.to_string(),
            symbolic_struct_definition,
            suppress_logging: true,
        };
        let (response_sender, response_receiver) = mpsc::channel();

        if !memory_read_request.send(&self.engine_unprivileged_state, move |memory_read_response| {
            let _ = response_sender.send(memory_read_response);
        }) {
            return Err(format!("Failed to dispatch memory read for {module_name}+0x{offset:X}."));
        }

        let memory_read_response = response_receiver
            .recv_timeout(Duration::from_secs(2))
            .map_err(|error| format!("Timed out waiting for memory read response: {error}"))?;

        if !memory_read_response.success {
            return Err(format!("Memory read failed for {module_name}+0x{offset:X}."));
        }

        Ok(memory_read_response.valued_struct.get_bytes())
    }
}

struct ProbeDataTypeRegistryStore {
    engine_unprivileged_state: Arc<EngineUnprivilegedState>,
}

impl DataTypeRegistryStore for ProbeDataTypeRegistryStore {
    fn get_registered_data_type_refs(&self) -> Vec<squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef> {
        self.engine_unprivileged_state.get_registered_data_type_refs()
    }

    fn get_unit_size_in_bytes(
        &self,
        data_type_ref: &squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef,
    ) -> u64 {
        self.engine_unprivileged_state.get_unit_size_in_bytes(data_type_ref)
    }
}

struct ProbeSymbolTreeWindowStore;

impl SymbolTreeWindowStore for ProbeSymbolTreeWindowStore {
    fn request_refresh(&self) {
    }

    fn focus_tree_node(
        &self,
        _tree_node_key: &str,
    ) {
    }
}

struct ProbeSymbolTreeActionServices {
    project_symbol_store: ProbeProjectSymbolStore,
    process_memory_store: ProbeProcessMemoryStore,
    data_type_registry_store: ProbeDataTypeRegistryStore,
    symbol_tree_window_store: ProbeSymbolTreeWindowStore,
}

impl SymbolTreeActionServices for ProbeSymbolTreeActionServices {
    fn symbol_store(&self) -> &dyn ProjectSymbolStore {
        &self.project_symbol_store
    }

    fn process_memory(&self) -> &dyn ProcessMemoryStore {
        &self.process_memory_store
    }

    fn data_type_registry(&self) -> &dyn DataTypeRegistryStore {
        &self.data_type_registry_store
    }

    fn symbol_tree_window(&self) -> &dyn SymbolTreeWindowStore {
        &self.symbol_tree_window_store
    }
}

fn main() -> Result<(), String> {
    env_logger::builder().is_test(false).try_init().ok();

    let (app_context, module_name) = create_app_context_without_process()?;
    wait_for_registry(&app_context.engine_unprivileged_state);

    let process_changed_events = Arc::new(Mutex::new(Vec::new()));
    let process_changed_events_for_listener = process_changed_events.clone();
    app_context
        .engine_unprivileged_state
        .listen_for_engine_event::<ProcessChangedEvent>(move |process_changed_event| {
            if let Ok(mut process_changed_events) = process_changed_events_for_listener.lock() {
                process_changed_events.push(process_changed_event.process_info.clone());
            }
        });

    println!("opening self process {}", std::process::id());
    open_self_process(&app_context.engine_unprivileged_state)?;
    thread::sleep(Duration::from_millis(250));

    let project_symbol_catalog = populate_macho_symbol_catalog(&app_context, &module_name)?;
    println!(
        "populated Mach-O catalog with {} module(s)",
        project_symbol_catalog.get_symbol_modules().len()
    );

    close_process(&app_context.engine_unprivileged_state)?;
    thread::sleep(Duration::from_millis(250));

    let symbol_tree_runtime_data_controller = SymbolTreeRuntimeDataController::new(app_context.clone());
    let mut expanded_tree_node_keys = HashSet::from([format!("module:{module_name}")]);

    let first_runtime_data = symbol_tree_runtime_data_controller.build_runtime_data(&project_symbol_catalog, &expanded_tree_node_keys);
    let mach_headers_node_key = first_runtime_data
        .symbol_tree_entries
        .iter()
        .find(|symbol_tree_entry| symbol_tree_entry.get_display_name() == "Mach-O Headers")
        .map(|symbol_tree_entry| symbol_tree_entry.get_node_key().to_string())
        .ok_or_else(|| String::from("Failed to find Mach-O Headers node while detached."))?;
    expanded_tree_node_keys.insert(mach_headers_node_key);

    let second_runtime_data = symbol_tree_runtime_data_controller.build_runtime_data(&project_symbol_catalog, &expanded_tree_node_keys);
    let header_node_key = second_runtime_data
        .symbol_tree_entries
        .iter()
        .find(|symbol_tree_entry| symbol_tree_entry.get_display_name() == "Header")
        .map(|symbol_tree_entry| symbol_tree_entry.get_node_key().to_string())
        .ok_or_else(|| String::from("Failed to find Header node while detached."))?;
    expanded_tree_node_keys.insert(header_node_key);

    let detached_runtime_data = symbol_tree_runtime_data_controller.build_runtime_data(&project_symbol_catalog, &expanded_tree_node_keys);
    let magic_node_key = detached_runtime_data
        .symbol_tree_entries
        .iter()
        .find(|symbol_tree_entry| symbol_tree_entry.get_display_name() == "magic")
        .map(|symbol_tree_entry| symbol_tree_entry.get_node_key().to_string())
        .ok_or_else(|| String::from("Failed to find magic node while detached."))?;
    let detached_preview = detached_runtime_data
        .preview_values_by_node_key
        .get(&magic_node_key)
        .cloned()
        .unwrap_or_default();
    println!("detached preview = {detached_preview:?}");

    println!("reopening self process {}", std::process::id());
    open_self_process(&app_context.engine_unprivileged_state)?;
    thread::sleep(Duration::from_millis(250));

    let preview_deadline = Instant::now() + Duration::from_secs(5);

    while Instant::now() < preview_deadline {
        let runtime_data = symbol_tree_runtime_data_controller.build_runtime_data(&project_symbol_catalog, &expanded_tree_node_keys);
        let preview_value = runtime_data
            .preview_values_by_node_key
            .get(&magic_node_key)
            .cloned()
            .unwrap_or_default();
        let preview_snapshot_state = app_context
            .engine_unprivileged_state
            .get_virtual_snapshot("symbol_tree_preview_values");
        let preview_generation = preview_snapshot_state
            .as_ref()
            .map(|preview_snapshot| preview_snapshot.get_generation())
            .unwrap_or_default();
        let preview_in_flight = preview_snapshot_state
            .as_ref()
            .map(|preview_snapshot| preview_snapshot.get_is_refresh_in_progress())
            .unwrap_or(false);
        println!(
            "preview={preview_value:?} generation={preview_generation} in_flight={preview_in_flight}"
        );

        if !preview_value.is_empty() {
            break;
        }

        thread::sleep(Duration::from_millis(250));
    }

    let final_preview = symbol_tree_runtime_data_controller
        .build_runtime_data(&project_symbol_catalog, &expanded_tree_node_keys)
        .preview_values_by_node_key
        .get(&magic_node_key)
        .cloned()
        .unwrap_or_default();
    println!("final preview = {final_preview:?}");

    let process_changed_events = process_changed_events
        .lock()
        .map_err(|error| format!("Failed to read process changed events: {error}"))?;
    println!("process changed events observed = {}", process_changed_events.len());
    for (event_index, process_info) in process_changed_events.iter().enumerate() {
        let process_name = process_info
            .as_ref()
            .map(|process_info| format!("{} ({})", process_info.get_name(), process_info.get_process_id()))
            .unwrap_or_else(|| String::from("<detached>"));
        println!("  event[{event_index}] = {process_name}");
    }

    Ok(())
}

fn create_app_context_without_process() -> Result<(Arc<AppContext>, String), String> {
    let engine_privileged_state = create_engine_privileged_state(EngineMode::Standalone)
        .map_err(|error| format!("Failed to create privileged engine state: {error}"))?;
    let current_process_name = std::env::current_exe()
        .ok()
        .and_then(|current_executable_path| {
            current_executable_path
                .file_name()
                .map(|file_name| file_name.to_string_lossy().to_string())
        })
        .unwrap_or_else(|| String::from("cargo"));
    let module_name = Path::new(&current_process_name)
        .file_name()
        .map(|file_name| file_name.to_string_lossy().to_string())
        .unwrap_or(current_process_name);
    let engine_bindings = Arc::new(RwLock::new(StandaloneEngineApiUnprivilegedBindings::new(&engine_privileged_state)));
    let engine_unprivileged_state =
        EngineUnprivilegedState::new_with_options(engine_bindings, EngineUnprivilegedStateOptions { enable_console_logging: true });
    engine_unprivileged_state.initialize();

    let egui_context = Context::default();
    let app_context = Arc::new(AppContext::new(
        egui_context.clone(),
        Arc::new(Theme::new(&egui_context)),
        Arc::new(RwLock::new(DockingManager::new(DockNode::default()))),
        engine_unprivileged_state,
    ));

    Ok((app_context, module_name))
}

fn wait_for_registry(engine_unprivileged_state: &Arc<EngineUnprivilegedState>) {
    let deadline = Instant::now() + Duration::from_secs(2);

    while engine_unprivileged_state.get_registered_data_type_refs().is_empty() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
}

fn open_self_process(engine_unprivileged_state: &Arc<EngineUnprivilegedState>) -> Result<(), String> {
    let process_open_request = ProcessOpenRequest {
        process_id: Some(std::process::id()),
        search_name: None,
        match_case: false,
    };
    let (response_sender, response_receiver) = mpsc::channel();

    if !process_open_request.send(engine_unprivileged_state, move |process_open_response| {
        let _ = response_sender.send(process_open_response);
    }) {
        return Err(String::from("Failed to dispatch process open request."));
    }

    let process_open_response = response_receiver
        .recv_timeout(Duration::from_secs(2))
        .map_err(|error| format!("Timed out waiting for process open response: {error}"))?;

    if process_open_response.opened_process_info.is_none() {
        return Err(String::from("Process open response did not contain an opened process."));
    }

    Ok(())
}

fn close_process(engine_unprivileged_state: &Arc<EngineUnprivilegedState>) -> Result<(), String> {
    let process_close_request = ProcessCloseRequest {};
    let (response_sender, response_receiver) = mpsc::channel();

    if !process_close_request.send(engine_unprivileged_state, move |process_close_response| {
        let _ = response_sender.send(process_close_response);
    }) {
        return Err(String::from("Failed to dispatch process close request."));
    }

    let _process_close_response = response_receiver
        .recv_timeout(Duration::from_secs(2))
        .map_err(|error| format!("Timed out waiting for process close response: {error}"))?;

    Ok(())
}

fn populate_macho_symbol_catalog(
    app_context: &Arc<AppContext>,
    module_name: &str,
) -> Result<ProjectSymbolCatalog, String> {
    let project_symbol_catalog = Arc::new(Mutex::new(ProjectSymbolCatalog::new_with_modules_and_symbol_claims(
        vec![ProjectSymbolModule::new(module_name.to_string(), 0x1000)],
        Vec::new(),
        Vec::new(),
    )));
    let services = ProbeSymbolTreeActionServices {
        project_symbol_store: ProbeProjectSymbolStore {
            project_symbol_catalog: project_symbol_catalog.clone(),
        },
        process_memory_store: ProbeProcessMemoryStore {
            engine_unprivileged_state: app_context.engine_unprivileged_state.clone(),
        },
        data_type_registry_store: ProbeDataTypeRegistryStore {
            engine_unprivileged_state: app_context.engine_unprivileged_state.clone(),
        },
        symbol_tree_window_store: ProbeSymbolTreeWindowStore,
    };
    let plugin = BinarySymbolsPlugin::new();
    let action = plugin
        .symbol_tree_actions()
        .into_iter()
        .find(|symbol_tree_action| symbol_tree_action.action_id() == "builtin.symbols.binary.populate-binary-symbols")
        .ok_or_else(|| String::from("Failed to find populate binary symbols action."))?;

    action.execute(
        &SymbolTreeActionContext::new(SymbolTreeActionSelection::ModuleRoot {
            module_name: module_name.to_string(),
        }),
        &services,
    )?;

    project_symbol_catalog
        .lock()
        .map(|project_symbol_catalog| project_symbol_catalog.clone())
        .map_err(|error| format!("Failed to clone populated project symbol catalog: {error}"))
}
