use crate::{app_context::AppContext, views::process_selector::view_data::process_selector_view_data::ProcessSelectorViewData};
use squalr_engine_api::{
    commands::{
        memory::read::{memory_read_request::MemoryReadRequest, memory_read_response::MemoryReadResponse},
        patches::apply::patch_apply_request::PatchApplyRequest,
        privileged_command_request::PrivilegedCommandRequest,
    },
    dependency_injection::dependency::Dependency,
    plugins::instruction_set::InstructionSet,
    structures::{
        data_types::{built_in_types::u8::data_type_u8::DataTypeU8, data_type_ref::DataTypeRef},
        data_values::container_type::ContainerType,
        patches::PatchKind,
        structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition},
    },
};
use std::sync::Arc;

pub struct InstructionPatchAction;

impl InstructionPatchAction {
    const INSTRUCTION_READ_BYTE_COUNT: u64 = 16;

    pub fn replace_known_instruction_with_no_operation(
        app_context: Arc<AppContext>,
        process_selector_view_data: Dependency<ProcessSelectorViewData>,
        address: u64,
        module_name: String,
        instruction_bytes: Vec<u8>,
        label: Option<String>,
    ) {
        let Some(instruction_set) = Self::get_instruction_set(&app_context, process_selector_view_data) else {
            log::warn!("Cannot replace instruction with no-operation bytes because no instruction set plugin is enabled for this target.");
            return;
        };
        let Some(byte_count) = Self::resolve_instruction_byte_count(&instruction_set, &instruction_bytes) else {
            log::warn!("Cannot replace instruction with no-operation bytes because the instruction length could not be decoded.");
            return;
        };

        Self::dispatch_no_operation_patch(app_context, instruction_set, address, module_name, byte_count, label);
    }

    pub fn replace_instruction_at_address_with_no_operation(
        app_context: Arc<AppContext>,
        process_selector_view_data: Dependency<ProcessSelectorViewData>,
        address: u64,
        module_name: String,
        label: Option<String>,
    ) {
        let Some(instruction_set) = Self::get_instruction_set(&app_context, process_selector_view_data) else {
            log::warn!("Cannot replace instruction with no-operation bytes because no instruction set plugin is enabled for this target.");
            return;
        };
        let memory_read_request = MemoryReadRequest {
            address,
            module_name: module_name.clone(),
            symbolic_struct_definition: Self::instruction_read_definition(),
            suppress_logging: true,
        };
        let app_context_for_patch = app_context.clone();

        memory_read_request.send(&app_context.engine_unprivileged_state, move |memory_read_response| {
            let Some(instruction_bytes) = Self::read_instruction_bytes(memory_read_response) else {
                log::warn!("Cannot replace instruction with no-operation bytes because the instruction bytes could not be read.");
                return;
            };
            let Some(byte_count) = Self::resolve_instruction_byte_count(&instruction_set, &instruction_bytes) else {
                log::warn!("Cannot replace instruction with no-operation bytes because the instruction length could not be decoded.");
                return;
            };

            Self::dispatch_no_operation_patch(app_context_for_patch, instruction_set, address, module_name, byte_count, label);
        });
    }

    fn get_instruction_set(
        app_context: &Arc<AppContext>,
        process_selector_view_data: Dependency<ProcessSelectorViewData>,
    ) -> Option<Arc<dyn InstructionSet>> {
        let target_architecture = process_selector_view_data
            .read("Instruction patch target architecture")
            .and_then(|process_selector_view_data| {
                process_selector_view_data
                    .opened_process
                    .as_ref()
                    .map(|opened_process_info| opened_process_info.get_target_architecture().clone())
            })?;

        app_context
            .engine_unprivileged_state
            .get_plugin_registry()
            .find_instruction_set(target_architecture.get_instruction_set_id())
    }

    fn dispatch_no_operation_patch(
        app_context: Arc<AppContext>,
        instruction_set: Arc<dyn InstructionSet>,
        address: u64,
        module_name: String,
        byte_count: usize,
        label: Option<String>,
    ) {
        let patched_bytes = match instruction_set.build_no_operation_fill(byte_count) {
            Ok(patched_bytes) if patched_bytes.len() == byte_count => patched_bytes,
            Ok(patched_bytes) => {
                log::warn!(
                    "Instruction set plugin produced {} no-operation bytes for a {} byte instruction.",
                    patched_bytes.len(),
                    byte_count
                );
                return;
            }
            Err(error) => {
                log::warn!("Cannot replace instruction with no-operation bytes: {}.", error);
                return;
            }
        };

        PatchApplyRequest {
            address,
            module_name,
            patched_bytes,
            kind: PatchKind::Code,
            label,
        }
        .send(&app_context.engine_unprivileged_state, |patch_apply_response| {
            if !patch_apply_response.status.get_success() {
                log::warn!(
                    "Replace with no-operation patch failed: {}.",
                    patch_apply_response
                        .status
                        .get_message()
                        .unwrap_or("unknown error")
                );
            }
        });
    }

    fn resolve_instruction_byte_count(
        instruction_set: &Arc<dyn InstructionSet>,
        instruction_bytes: &[u8],
    ) -> Option<usize> {
        instruction_set
            .disassemble_block(instruction_bytes, 0)
            .ok()
            .and_then(|instructions| instructions.into_iter().next())
            .map(|instruction| instruction.length)
            .filter(|byte_count| *byte_count > 0 && *byte_count <= instruction_bytes.len())
    }

    fn read_instruction_bytes(memory_read_response: MemoryReadResponse) -> Option<Vec<u8>> {
        if !memory_read_response.success {
            return None;
        }

        memory_read_response
            .valued_struct
            .get_fields()
            .first()
            .and_then(|valued_struct_field| valued_struct_field.get_data_value())
            .map(|data_value| data_value.get_value_bytes().clone())
            .filter(|instruction_bytes| !instruction_bytes.is_empty())
    }

    fn instruction_read_definition() -> SymbolicStructDefinition {
        SymbolicStructDefinition::new_anonymous(vec![SymbolicFieldDefinition::new(
            DataTypeRef::new(DataTypeU8::DATA_TYPE_ID),
            ContainerType::ArrayFixed(Self::INSTRUCTION_READ_BYTE_COUNT),
        )])
    }
}
