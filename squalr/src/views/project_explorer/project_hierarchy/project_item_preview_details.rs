use squalr_engine::services::projects::project_item_symbol_resolution::{
    resolve_address_target_runtime_pointer_with_optional_catalog, resolve_project_item_struct_layout_id,
};
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::plugins::instruction_set::{instruction_set_id_from_instruction_data_type_id, normalize_instruction_data_type_id};
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_values::{
    anonymous_value_string_format::AnonymousValueStringFormat,
    container_type::ContainerType,
    data_value_preview_formatter::{DataValuePreviewFormatOptions, DataValuePreviewFormatter},
};
use squalr_engine_api::structures::projects::project_info::ProjectInfo;
use squalr_engine_api::structures::projects::project_items::built_in_types::{
    project_item_type_address::ProjectItemTypeAddress, project_item_type_pointer::ProjectItemTypePointer,
};
use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
use squalr_engine_api::structures::structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition};
use squalr_engine_session::{
    engine_unprivileged_state::EngineUnprivilegedState,
    virtual_snapshots::{virtual_snapshot_query::VirtualSnapshotQuery, virtual_snapshot_query_result::VirtualSnapshotQueryResult},
};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

pub struct ProjectItemPreviewDetails;

impl ProjectItemPreviewDetails {
    const PROJECT_ITEM_PREVIEW_FORMAT_OPTIONS: DataValuePreviewFormatOptions = DataValuePreviewFormatOptions::new(4, 96, 96);
    const INSTRUCTION_PREVIEW_BYTE_COUNT: u64 = 16;

    pub fn copy_project_item_preview_fields(
        source_project_item: &ProjectItem,
        target_project_item: &mut ProjectItem,
    ) {
        let preview_value = Self::read_project_item_preview_value(source_project_item);
        let preview_display_format = Self::read_project_item_preview_display_format(source_project_item);
        let project_item_type_id = target_project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            ProjectItemTypeAddress::set_field_freeze_data_value_interpreter(target_project_item, &preview_value);
            if let Some(preview_display_format) = preview_display_format {
                ProjectItemTypeAddress::set_field_freeze_display_format(target_project_item, preview_display_format);
            }
        } else if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            ProjectItemTypePointer::set_field_freeze_data_value_interpreter(target_project_item, &preview_value);
            if let Some(preview_display_format) = preview_display_format {
                ProjectItemTypePointer::set_field_freeze_display_format(target_project_item, preview_display_format);
            }
        }
    }

    pub fn build_project_item_virtual_snapshot_query(
        opened_project_info: Option<&ProjectInfo>,
        project_item_path: &Path,
        project_item: &ProjectItem,
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
    ) -> Option<VirtualSnapshotQuery> {
        let query_id = project_item_path.to_string_lossy().to_string();
        let symbolic_struct_namespace = resolve_project_item_struct_layout_id(&ProjectSymbolCatalog::default(), project_item)?;
        let symbolic_struct_definition = Self::build_project_item_preview_symbolic_struct_definition(engine_unprivileged_state, &symbolic_struct_namespace)?;
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            let mut project_item = project_item.clone();
            let address_target = ProjectItemTypeAddress::get_address_target(&mut project_item);

            let project_symbol_catalog = opened_project_info.map(|opened_project_info| opened_project_info.get_project_symbol_catalog());
            let runtime_pointer = resolve_address_target_runtime_pointer_with_optional_catalog(project_symbol_catalog, &address_target)?;

            return if runtime_pointer.get_offset_segments().is_empty() {
                Some(VirtualSnapshotQuery::Address {
                    query_id,
                    address: runtime_pointer.get_address(),
                    module_name: runtime_pointer.get_module_name().to_string(),
                    symbolic_struct_definition,
                })
            } else {
                Some(VirtualSnapshotQuery::Pointer {
                    query_id,
                    pointer: runtime_pointer,
                    symbolic_struct_definition,
                })
            };
        }

        if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            return Some(VirtualSnapshotQuery::Pointer {
                query_id,
                pointer: ProjectItemTypePointer::get_field_pointer(project_item),
                symbolic_struct_definition,
            });
        }

        None
    }

    pub fn build_project_item_preview_value_from_virtual_snapshot_result(
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
        _opened_project_info: Option<&ProjectInfo>,
        project_item: &ProjectItem,
        virtual_snapshot_query_result: &VirtualSnapshotQueryResult,
    ) -> String {
        let Some(memory_read_response) = virtual_snapshot_query_result.memory_read_response.as_ref() else {
            return String::new();
        };

        if !memory_read_response.success {
            return String::new();
        }

        let first_read_field_data_value = memory_read_response
            .valued_struct
            .get_fields()
            .first()
            .and_then(|valued_struct_field| valued_struct_field.get_data_value());
        let Some(first_read_field_data_value) = first_read_field_data_value else {
            return String::new();
        };

        if let Some(instruction_set_id) = Self::resolve_project_item_instruction_set_id(project_item) {
            return Self::format_instruction_preview_from_bytes(engine_unprivileged_state, &instruction_set_id, first_read_field_data_value.get_value_bytes());
        }

        let default_anonymous_value_string_format = Self::read_project_item_preview_display_format(project_item)
            .unwrap_or_else(|| engine_unprivileged_state.get_default_anonymous_value_string_format(first_read_field_data_value.get_data_type_ref()));
        let symbolic_field_container_type = Self::resolve_project_item_symbolic_container_type(project_item);
        let preview_was_truncated = Self::project_item_preview_was_truncated(project_item);

        engine_unprivileged_state
            .anonymize_value(first_read_field_data_value, default_anonymous_value_string_format)
            .map(|anonymous_value_string| {
                let preview_value = DataValuePreviewFormatter::format_anonymous_value_preview(
                    &anonymous_value_string,
                    symbolic_field_container_type,
                    preview_was_truncated,
                    Self::PROJECT_ITEM_PREVIEW_FORMAT_OPTIONS,
                );

                preview_value
            })
            .unwrap_or_default()
    }

    fn read_project_item_preview_value(project_item: &ProjectItem) -> String {
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            let mut project_item = project_item.clone();

            ProjectItemTypeAddress::get_field_freeze_data_value_interpreter(&mut project_item)
        } else if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            ProjectItemTypePointer::get_field_freeze_data_value_interpreter(project_item)
        } else {
            String::new()
        }
    }

    fn read_project_item_preview_display_format(project_item: &ProjectItem) -> Option<AnonymousValueStringFormat> {
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            ProjectItemTypeAddress::get_field_freeze_display_format(project_item)
        } else if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            ProjectItemTypePointer::get_field_freeze_display_format(project_item)
        } else {
            None
        }
    }

    fn build_project_item_preview_symbolic_struct_definition(
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
        symbolic_struct_namespace: &str,
    ) -> Option<SymbolicStructDefinition> {
        if let Some(instruction_data_type_id) = Self::normalize_instruction_data_type_id(symbolic_struct_namespace) {
            return Some(Self::build_instruction_preview_symbolic_struct_definition(&instruction_data_type_id));
        }

        let symbolic_struct_definition = engine_unprivileged_state.resolve_struct_layout_definition(symbolic_struct_namespace)?;
        let preview_field_definition = SymbolicFieldDefinition::from_str(symbolic_struct_namespace).ok();

        let Some(preview_field_definition) = preview_field_definition else {
            return Some(symbolic_struct_definition);
        };

        let preview_container_type = DataValuePreviewFormatter::limit_array_container_type(preview_field_definition.get_container_type());

        if preview_container_type == preview_field_definition.get_container_type() {
            Some(symbolic_struct_definition)
        } else {
            Some(SymbolicStructDefinition::new_anonymous(vec![SymbolicFieldDefinition::new(
                preview_field_definition.get_data_type_ref().clone(),
                preview_container_type,
            )]))
        }
    }

    fn resolve_project_item_symbolic_container_type(project_item: &ProjectItem) -> ContainerType {
        let Some(symbolic_struct_namespace) = resolve_project_item_struct_layout_id(&ProjectSymbolCatalog::default(), project_item) else {
            return ContainerType::None;
        };

        if Self::normalize_instruction_data_type_id(&symbolic_struct_namespace).is_some() {
            return ContainerType::None;
        }

        SymbolicFieldDefinition::from_str(&symbolic_struct_namespace)
            .map(|symbolic_field_definition| symbolic_field_definition.get_container_type())
            .unwrap_or(ContainerType::None)
    }

    fn project_item_preview_was_truncated(project_item: &ProjectItem) -> bool {
        let Some(symbolic_struct_namespace) = resolve_project_item_struct_layout_id(&ProjectSymbolCatalog::default(), project_item) else {
            return false;
        };
        let Some(symbolic_field_definition) = SymbolicFieldDefinition::from_str(&symbolic_struct_namespace).ok() else {
            return false;
        };

        DataValuePreviewFormatter::array_preview_was_truncated(symbolic_field_definition.get_container_type())
    }

    fn normalize_instruction_data_type_id(symbolic_struct_namespace: &str) -> Option<String> {
        normalize_instruction_data_type_id(symbolic_struct_namespace)
    }

    fn resolve_project_item_instruction_set_id(project_item: &ProjectItem) -> Option<String> {
        let symbolic_struct_namespace = resolve_project_item_struct_layout_id(&ProjectSymbolCatalog::default(), project_item)?;
        let instruction_data_type_id = Self::normalize_instruction_data_type_id(&symbolic_struct_namespace)?;

        instruction_set_id_from_instruction_data_type_id(&instruction_data_type_id)
    }

    fn format_instruction_preview_from_bytes(
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
        instruction_set_id: &str,
        instruction_bytes: &[u8],
    ) -> String {
        engine_unprivileged_state
            .get_plugin_registry()
            .find_instruction_set(instruction_set_id)
            .and_then(|instruction_set| instruction_set.disassemble_block(instruction_bytes, 0).ok())
            .and_then(|instructions| instructions.into_iter().next())
            .map(|instruction| instruction.text)
            .filter(|instruction_text| !instruction_text.trim().is_empty())
            .unwrap_or_default()
    }

    fn build_instruction_preview_symbolic_struct_definition(data_type_id: &str) -> SymbolicStructDefinition {
        SymbolicStructDefinition::new_anonymous(vec![SymbolicFieldDefinition::new(
            DataTypeRef::new(data_type_id),
            ContainerType::ArrayFixed(Self::INSTRUCTION_PREVIEW_BYTE_COUNT),
        )])
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectItemPreviewDetails;
    use crossbeam_channel::{Receiver, unbounded};
    use squalr_engine_api::commands::memory::read::memory_read_response::MemoryReadResponse;
    use squalr_engine_api::commands::privileged_command::PrivilegedCommand;
    use squalr_engine_api::commands::privileged_command_response::PrivilegedCommandResponse;
    use squalr_engine_api::commands::unprivileged_command::UnprivilegedCommand;
    use squalr_engine_api::commands::unprivileged_command_response::UnprivilegedCommandResponse;
    use squalr_engine_api::engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings;
    use squalr_engine_api::engine::engine_binding_error::EngineBindingError;
    use squalr_engine_api::engine::engine_event_envelope::EngineEventEnvelope;
    use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
    use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
    use squalr_engine_api::structures::data_values::{container_type::ContainerType, data_value::DataValue};
    use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress;
    use squalr_engine_api::structures::structs::valued_struct::ValuedStruct;
    use squalr_engine_session::engine_unprivileged_state::{EngineUnprivilegedState, EngineUnprivilegedStateOptions};
    use squalr_engine_session::virtual_snapshots::virtual_snapshot_query_result::VirtualSnapshotQueryResult;
    use std::sync::{Arc, RwLock};

    #[test]
    fn instruction_project_item_preview_reads_instruction_byte_window() {
        let symbolic_struct_definition = ProjectItemPreviewDetails::build_instruction_preview_symbolic_struct_definition("i_x64");
        let preview_field = symbolic_struct_definition
            .get_fields()
            .first()
            .expect("Expected instruction preview definition to contain one field.");

        assert_eq!(preview_field.get_data_type_ref().get_data_type_id(), "i_x64");
        assert_eq!(preview_field.get_container_type(), ContainerType::ArrayFixed(16));
    }

    #[test]
    fn instruction_project_item_preview_normalizes_decorated_instruction_type() {
        assert_eq!(
            ProjectItemPreviewDetails::normalize_instruction_data_type_id("i_x64[3]").as_deref(),
            Some("i_x64")
        );
    }

    #[test]
    fn instruction_project_item_preview_uses_first_disassembled_instruction() {
        let engine_unprivileged_state = create_engine_unprivileged_state();
        let project_item =
            ProjectItemTypeAddress::new_project_item("Instruction", 0x1234, "game.exe", "", DataValue::new(DataTypeRef::new("i_x64"), vec![0x90]));
        let virtual_snapshot_query_result =
            create_value_virtual_snapshot_query_result(DataValue::new(DataTypeRef::new("i_x64"), vec![0x90, 0x90, 0x90, 0x90]), true);

        let preview_value = ProjectItemPreviewDetails::build_project_item_preview_value_from_virtual_snapshot_result(
            &engine_unprivileged_state,
            None,
            &project_item,
            &virtual_snapshot_query_result,
        );

        assert_eq!(preview_value, "nop");
    }

    #[test]
    fn instruction_project_item_preview_clears_on_failed_read() {
        let engine_unprivileged_state = create_engine_unprivileged_state();
        let mut project_item =
            ProjectItemTypeAddress::new_project_item("Instruction", 0x1234, "game.exe", "", DataValue::new(DataTypeRef::new("i_x64"), vec![0x90]));
        ProjectItemTypeAddress::set_field_freeze_data_value_interpreter(&mut project_item, "inc dword ptr [eax]");
        let virtual_snapshot_query_result = create_value_virtual_snapshot_query_result(DataValue::new(DataTypeRef::new("i_x64"), Vec::new()), false);

        let preview_value = ProjectItemPreviewDetails::build_project_item_preview_value_from_virtual_snapshot_result(
            &engine_unprivileged_state,
            None,
            &project_item,
            &virtual_snapshot_query_result,
        );

        assert_eq!(preview_value, "");
    }

    #[test]
    fn instruction_project_item_preview_disassembles_byte_window_as_block() {
        let engine_unprivileged_state = create_engine_unprivileged_state();
        let project_item =
            ProjectItemTypeAddress::new_project_item("Instruction", 0x1234, "game.exe", "", DataValue::new(DataTypeRef::new("i_x64"), vec![0x90]));
        let virtual_snapshot_query_result =
            create_value_virtual_snapshot_query_result(DataValue::new(DataTypeRef::new("i_x64"), vec![0xFF, 0x00, 0x90, 0x90]), true);

        let preview_value = ProjectItemPreviewDetails::build_project_item_preview_value_from_virtual_snapshot_result(
            &engine_unprivileged_state,
            None,
            &project_item,
            &virtual_snapshot_query_result,
        );

        assert_eq!(preview_value, "inc dword [rax]");
    }

    fn create_engine_unprivileged_state() -> Arc<EngineUnprivilegedState> {
        EngineUnprivilegedState::new_with_options(
            Arc::new(RwLock::new(NoOpEngineBindings)),
            EngineUnprivilegedStateOptions { enable_console_logging: false },
        )
    }

    fn create_value_virtual_snapshot_query_result(
        data_value: DataValue,
        success: bool,
    ) -> VirtualSnapshotQueryResult {
        let valued_struct = if success {
            ValuedStruct::new_anonymous(vec![data_value.to_named_valued_struct_field("value".to_string(), true)])
        } else {
            ValuedStruct::default()
        };

        VirtualSnapshotQueryResult {
            memory_read_response: Some(MemoryReadResponse {
                valued_struct,
                address: 0,
                success,
            }),
            resolved_address: Some(0x1234),
            resolved_module_name: "game.exe".to_string(),
            evaluated_pointer_path: String::new(),
        }
    }

    struct NoOpEngineBindings;

    impl EngineApiUnprivilegedBindings for NoOpEngineBindings {
        fn dispatch_privileged_command(
            &self,
            _engine_command: PrivilegedCommand,
            _callback: Box<dyn FnOnce(PrivilegedCommandResponse) + Send + Sync + 'static>,
        ) -> Result<(), EngineBindingError> {
            Err(EngineBindingError::unavailable("dispatching privileged commands in project item preview tests"))
        }

        fn dispatch_unprivileged_command(
            &self,
            _engine_command: UnprivilegedCommand,
            _engine_execution_context: &Arc<dyn EngineExecutionContext>,
            _callback: Box<dyn FnOnce(UnprivilegedCommandResponse) + Send + Sync + 'static>,
        ) -> Result<(), EngineBindingError> {
            Err(EngineBindingError::unavailable(
                "dispatching unprivileged commands in project item preview tests",
            ))
        }

        fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEventEnvelope>, EngineBindingError> {
            let (_event_sender, event_receiver) = unbounded();

            Ok(event_receiver)
        }
    }
}
