use crate::{app_context::AppContext, views::process_selector::view_data::process_selector_view_data::ProcessSelectorViewData};
use squalr_engine_api::{
    commands::{
        patches::{
            no_operation::patch_no_operation_request::PatchNoOperationRequest, restore_address::patch_restore_address_request::PatchRestoreAddressRequest,
        },
        privileged_command_request::PrivilegedCommandRequest,
    },
    dependency_injection::dependency::Dependency,
    structures::patches::PatchKind,
};
use std::sync::Arc;

pub struct InstructionPatchAction;

impl InstructionPatchAction {
    pub fn replace_known_instruction_with_no_operation(
        app_context: Arc<AppContext>,
        _process_selector_view_data: Dependency<ProcessSelectorViewData>,
        address: u64,
        module_name: String,
        instruction_bytes: Vec<u8>,
        label: Option<String>,
    ) {
        Self::dispatch_no_operation_patch(app_context, address, module_name, Some(instruction_bytes), label);
    }

    pub fn replace_instruction_at_address_with_no_operation(
        app_context: Arc<AppContext>,
        _process_selector_view_data: Dependency<ProcessSelectorViewData>,
        address: u64,
        module_name: String,
        label: Option<String>,
    ) {
        Self::dispatch_no_operation_patch(app_context, address, module_name, None, label);
    }

    pub fn restore_no_operation_patch_at_address(
        app_context: Arc<AppContext>,
        address: u64,
        module_name: String,
    ) {
        PatchRestoreAddressRequest {
            address,
            module_name,
            expected_kind: Some(PatchKind::NoOperation),
        }
        .send(&app_context.engine_unprivileged_state, |patch_restore_address_response| {
            if !patch_restore_address_response.status.get_success() {
                log::warn!(
                    "Restore original code failed: {}.",
                    patch_restore_address_response
                        .status
                        .get_message()
                        .unwrap_or("unknown error")
                );
            }
        });
    }

    fn dispatch_no_operation_patch(
        app_context: Arc<AppContext>,
        address: u64,
        module_name: String,
        instruction_bytes_hint: Option<Vec<u8>>,
        label: Option<String>,
    ) {
        PatchNoOperationRequest {
            address,
            module_name,
            instruction_bytes_hint,
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
}
