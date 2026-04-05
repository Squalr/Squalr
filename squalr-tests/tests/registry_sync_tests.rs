use crossbeam_channel::{Receiver, Sender, unbounded};
use squalr_engine_api::commands::privileged_command::PrivilegedCommand;
use squalr_engine_api::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use squalr_engine_api::commands::project::list::project_list_response::ProjectListResponse;
use squalr_engine_api::commands::registry::get_snapshot::registry_get_snapshot_response::RegistryGetSnapshotResponse;
use squalr_engine_api::commands::unprivileged_command::UnprivilegedCommand;
use squalr_engine_api::commands::unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse};
use squalr_engine_api::engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings;
use squalr_engine_api::engine::engine_binding_error::EngineBindingError;
use squalr_engine_api::engine::engine_event_envelope::EngineEventEnvelope;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::events::engine_event::EngineEventRequest;
use squalr_engine_api::events::registry::changed::registry_changed_event::RegistryChangedEvent;
use squalr_engine_api::registries::symbols::{
    data_type_descriptor::DataTypeDescriptor, symbol_registry::SymbolRegistry, symbol_registry_snapshot::RegistryMetadata,
    symbolic_struct_descriptor::StructLayoutDescriptor,
};
use squalr_engine_api::structures::{
    data_types::data_type_ref::DataTypeRef,
    data_values::anonymous_value_string_format::AnonymousValueStringFormat,
    data_values::container_type::ContainerType,
    memory::endian::Endian,
    structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition},
};
use squalr_engine_session::engine_unprivileged_state::EngineUnprivilegedState;
use std::{
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};

struct TestEngineBindings {
    next_registry_snapshot_response: Arc<RwLock<RegistryGetSnapshotResponse>>,
    event_sender: Sender<EngineEventEnvelope>,
    event_receiver: Receiver<EngineEventEnvelope>,
}

impl TestEngineBindings {
    fn new(initial_registry_snapshot: RegistryMetadata) -> Self {
        let (event_sender, event_receiver) = unbounded();

        Self {
            next_registry_snapshot_response: Arc::new(RwLock::new(RegistryGetSnapshotResponse {
                symbol_registry_snapshot: initial_registry_snapshot,
            })),
            event_sender,
            event_receiver,
        }
    }

    fn set_registry_snapshot(
        &self,
        symbol_registry_snapshot: RegistryMetadata,
    ) {
        if let Ok(mut next_registry_snapshot_response) = self.next_registry_snapshot_response.write() {
            *next_registry_snapshot_response = RegistryGetSnapshotResponse { symbol_registry_snapshot };
        }
    }

    fn emit_registry_changed(
        &self,
        generation: u64,
    ) {
        let _ = self
            .event_sender
            .send(EngineEventEnvelope::new(generation, RegistryChangedEvent { generation }.to_engine_event()));
    }
}

impl EngineApiUnprivilegedBindings for TestEngineBindings {
    fn dispatch_privileged_command(
        &self,
        _engine_command: PrivilegedCommand,
        callback: Box<dyn FnOnce(PrivilegedCommandResponse) + Send + Sync + 'static>,
    ) -> Result<(), EngineBindingError> {
        let registry_snapshot_response = self
            .next_registry_snapshot_response
            .read()
            .map(|registry_snapshot_response| registry_snapshot_response.clone())
            .map_err(|error| EngineBindingError::lock_failure("reading registry snapshot response in test bindings", error.to_string()))?;

        callback(registry_snapshot_response.to_engine_response());

        Ok(())
    }

    fn dispatch_unprivileged_command(
        &self,
        _engine_command: UnprivilegedCommand,
        _engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
        callback: Box<dyn FnOnce(UnprivilegedCommandResponse) + Send + Sync + 'static>,
    ) -> Result<(), EngineBindingError> {
        callback(ProjectListResponse::default().to_engine_response());

        Ok(())
    }

    fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEventEnvelope>, EngineBindingError> {
        Ok(self.event_receiver.clone())
    }
}

#[test]
fn symbol_registry_snapshot_bootstraps_and_refreshes_through_engine_events() {
    let initial_registry_snapshot = build_symbol_registry_snapshot(1, 16);
    let refreshed_registry_snapshot = build_symbol_registry_snapshot(2, 32);
    let engine_bindings = Arc::new(RwLock::new(TestEngineBindings::new(initial_registry_snapshot)));
    let engine_unprivileged_state = EngineUnprivilegedState::new(engine_bindings.clone());

    engine_unprivileged_state.initialize();

    assert!(wait_for_generation(&engine_unprivileged_state, 1));
    assert!(engine_unprivileged_state.is_registered_data_type_ref(&DataTypeRef::new("remote.test.type")));
    assert_eq!(get_snapshot_unit_size_in_bytes(&engine_unprivileged_state, "remote.test.type"), Some(16));
    assert!(snapshot_contains_symbolic_struct(&engine_unprivileged_state, "remote.test.struct"));

    if let Ok(engine_bindings_guard) = engine_bindings.read() {
        engine_bindings_guard.set_registry_snapshot(refreshed_registry_snapshot);
        engine_bindings_guard.emit_registry_changed(2);
    }

    assert!(wait_for_generation(&engine_unprivileged_state, 2));
    assert_eq!(get_snapshot_unit_size_in_bytes(&engine_unprivileged_state, "remote.test.type"), Some(32));
}

fn wait_for_generation(
    engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
    expected_generation: u64,
) -> bool {
    for _ in 0..50 {
        let current_generation = engine_unprivileged_state.get_privileged_symbol_generation();

        if current_generation >= expected_generation {
            return true;
        }

        thread::sleep(Duration::from_millis(10));
    }

    false
}

fn build_symbol_registry_snapshot(
    generation: u64,
    remote_type_unit_size_in_bytes: u64,
) -> RegistryMetadata {
    let built_in_symbol_registry_snapshot = SymbolRegistry::new().create_snapshot(generation);
    let mut data_type_descriptors = built_in_symbol_registry_snapshot
        .get_data_type_descriptors()
        .to_vec();
    data_type_descriptors.push(DataTypeDescriptor::new(
        "remote.test.type".to_string(),
        "remote-icon".to_string(),
        remote_type_unit_size_in_bytes,
        vec![AnonymousValueStringFormat::Hexadecimal],
        AnonymousValueStringFormat::Hexadecimal,
        Endian::Little,
        false,
        false,
    ));

    let mut struct_layout_descriptors = built_in_symbol_registry_snapshot
        .get_struct_layout_descriptors()
        .to_vec();
    struct_layout_descriptors.push(StructLayoutDescriptor::new(
        "remote.test.struct".to_string(),
        SymbolicStructDefinition::new(
            "remote.test.struct".to_string(),
            vec![SymbolicFieldDefinition::new(
                DataTypeRef::new("remote.test.type"),
                ContainerType::None,
            )],
        ),
    ));

    RegistryMetadata::new(generation, data_type_descriptors, struct_layout_descriptors)
}

fn get_snapshot_unit_size_in_bytes(
    engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
    data_type_id: &str,
) -> Option<u64> {
    engine_unprivileged_state
        .get_privileged_symbol_snapshot()
        .and_then(|symbol_registry_snapshot| {
            symbol_registry_snapshot
                .get_data_type_descriptors()
                .iter()
                .find(|data_type_descriptor| data_type_descriptor.get_data_type_id() == data_type_id)
                .map(|data_type_descriptor| data_type_descriptor.get_unit_size_in_bytes())
        })
}

fn snapshot_contains_symbolic_struct(
    engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
    symbolic_struct_id: &str,
) -> bool {
    engine_unprivileged_state
        .get_privileged_symbol_snapshot()
        .map(|symbol_registry_snapshot| {
            symbol_registry_snapshot
            .get_struct_layout_descriptors()
                .iter()
            .any(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == symbolic_struct_id)
        })
        .unwrap_or(false)
}
