use crossbeam_channel::{Receiver, Sender, unbounded};
use squalr_engine_api::commands::privileged_command::PrivilegedCommand;
use squalr_engine_api::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use squalr_engine_api::commands::project::list::project_list_response::ProjectListResponse;
use squalr_engine_api::commands::registry::get_metadata::registry_get_metadata_response::RegistryGetMetadataResponse;
use squalr_engine_api::commands::unprivileged_command::UnprivilegedCommand;
use squalr_engine_api::commands::unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse};
use squalr_engine_api::engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings;
use squalr_engine_api::engine::engine_binding_error::EngineBindingError;
use squalr_engine_api::engine::engine_event_envelope::EngineEventEnvelope;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::events::engine_event::EngineEventRequest;
use squalr_engine_api::events::registry::changed::registry_changed_event::RegistryChangedEvent;
use squalr_engine_api::registries::symbols::{
    data_type_descriptor::DataTypeDescriptor, privileged_registry_catalog::PrivilegedRegistryCatalog, struct_layout_descriptor::StructLayoutDescriptor,
    symbol_registry::SymbolRegistry,
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
    next_registry_catalog_response: Arc<RwLock<RegistryGetMetadataResponse>>,
    event_sender: Sender<EngineEventEnvelope>,
    event_receiver: Receiver<EngineEventEnvelope>,
}

impl TestEngineBindings {
    fn new(initial_privileged_registry_catalog: PrivilegedRegistryCatalog) -> Self {
        let (event_sender, event_receiver) = unbounded();

        Self {
            next_registry_catalog_response: Arc::new(RwLock::new(RegistryGetMetadataResponse {
                privileged_registry_catalog: initial_privileged_registry_catalog,
            })),
            event_sender,
            event_receiver,
        }
    }

    fn set_privileged_registry_catalog(
        &self,
        privileged_registry_catalog: PrivilegedRegistryCatalog,
    ) {
        if let Ok(mut next_registry_catalog_response) = self.next_registry_catalog_response.write() {
            *next_registry_catalog_response = RegistryGetMetadataResponse { privileged_registry_catalog };
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
        let registry_catalog_response = self
            .next_registry_catalog_response
            .read()
            .map(|registry_catalog_response| registry_catalog_response.clone())
            .map_err(|error| EngineBindingError::lock_failure("reading registry catalog response in test bindings", error.to_string()))?;

        callback(registry_catalog_response.to_engine_response());

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
fn privileged_registry_catalog_bootstraps_and_refreshes_through_engine_events() {
    let initial_privileged_registry_catalog = build_privileged_registry_catalog(1, 16);
    let refreshed_privileged_registry_catalog = build_privileged_registry_catalog(2, 32);
    let engine_bindings = Arc::new(RwLock::new(TestEngineBindings::new(initial_privileged_registry_catalog)));
    let engine_unprivileged_state = EngineUnprivilegedState::new(engine_bindings.clone());

    engine_unprivileged_state.initialize();

    assert!(wait_for_generation(&engine_unprivileged_state, 1));
    assert!(engine_unprivileged_state.is_registered_data_type_ref(&DataTypeRef::new("remote.test.type")));
    assert_eq!(
        get_privileged_registry_catalog_unit_size_in_bytes(&engine_unprivileged_state, "remote.test.type"),
        Some(16)
    );
    assert!(privileged_registry_catalog_contains_struct_layout(
        &engine_unprivileged_state,
        "remote.test.struct"
    ));

    if let Ok(engine_bindings_guard) = engine_bindings.read() {
        engine_bindings_guard.set_privileged_registry_catalog(refreshed_privileged_registry_catalog);
        engine_bindings_guard.emit_registry_changed(2);
    }

    assert!(wait_for_generation(&engine_unprivileged_state, 2));
    assert_eq!(
        get_privileged_registry_catalog_unit_size_in_bytes(&engine_unprivileged_state, "remote.test.type"),
        Some(32)
    );
}

fn wait_for_generation(
    engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
    expected_generation: u64,
) -> bool {
    for _ in 0..50 {
        let current_generation = engine_unprivileged_state.get_privileged_registry_generation();

        if current_generation >= expected_generation {
            return true;
        }

        thread::sleep(Duration::from_millis(10));
    }

    false
}

fn build_privileged_registry_catalog(
    generation: u64,
    remote_type_unit_size_in_bytes: u64,
) -> PrivilegedRegistryCatalog {
    let built_in_privileged_registry_catalog = SymbolRegistry::new().create_registry_catalog(generation);
    let mut data_type_descriptors = built_in_privileged_registry_catalog
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

    let mut struct_layout_descriptors = built_in_privileged_registry_catalog
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

    PrivilegedRegistryCatalog::new(generation, data_type_descriptors, struct_layout_descriptors)
}

fn get_privileged_registry_catalog_unit_size_in_bytes(
    engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
    data_type_id: &str,
) -> Option<u64> {
    engine_unprivileged_state
        .get_privileged_registry_catalog()
        .and_then(|privileged_registry_catalog| {
            privileged_registry_catalog
                .get_data_type_descriptors()
                .iter()
                .find(|data_type_descriptor| data_type_descriptor.get_data_type_id() == data_type_id)
                .map(|data_type_descriptor| data_type_descriptor.get_unit_size_in_bytes())
        })
}

fn privileged_registry_catalog_contains_struct_layout(
    engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
    symbolic_struct_id: &str,
) -> bool {
    engine_unprivileged_state
        .get_privileged_registry_catalog()
        .map(|privileged_registry_catalog| {
            privileged_registry_catalog
                .get_struct_layout_descriptors()
                .iter()
                .any(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == symbolic_struct_id)
        })
        .unwrap_or(false)
}
