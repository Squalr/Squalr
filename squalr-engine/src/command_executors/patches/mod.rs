use crate::command_executors::privileged_command_executor::PrivilegedCommandExecutor;
use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::patches::{
    apply::{patch_apply_request::PatchApplyRequest, patch_apply_response::PatchApplyResponse},
    list::{patch_list_request::PatchListRequest, patch_list_response::PatchListResponse},
    no_operation::{patch_no_operation_request::PatchNoOperationRequest, patch_no_operation_response::PatchNoOperationResponse},
    patches_command::PatchesCommand,
    restore::{patch_restore_request::PatchRestoreRequest, patch_restore_response::PatchRestoreResponse},
    restore_address::{patch_restore_address_request::PatchRestoreAddressRequest, patch_restore_address_response::PatchRestoreAddressResponse},
};
use squalr_engine_api::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use squalr_engine_api::structures::debugger::DebuggerSessionState;
use squalr_engine_api::structures::patches::PatchCommandStatus;
use squalr_engine_api::structures::patches::PatchKind;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_session::os::PageRetrievalMode;
use std::sync::Arc;

fn failure_status(error_message: impl Into<String>) -> PatchCommandStatus {
    PatchCommandStatus::failure(error_message)
}

fn no_opened_process_status() -> PatchCommandStatus {
    failure_status("No opened process to patch.")
}

fn run_patch_operation_with_debugger_paused<T>(
    engine_privileged_state: &Arc<EnginePrivilegedState>,
    opened_process_info: &OpenedProcessInfo,
    operation_name: &str,
    patch_operation: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    let debugger_service = engine_privileged_state.get_debugger_service();
    let should_resume_after_patch = debugger_service.get_active_session_state_for_process(opened_process_info) == Some(DebuggerSessionState::Running);

    if should_resume_after_patch {
        debugger_service
            .pause()
            .map_err(|error_message| format!("Failed to pause debugger before {}: {}", operation_name, error_message))?;
    }

    let patch_result = patch_operation();
    let resume_result = if should_resume_after_patch {
        debugger_service
            .resume()
            .map(|_| ())
            .map_err(|error_message| format!("Failed to resume debugger after {}: {}", operation_name, error_message))
    } else {
        Ok(())
    };

    match (patch_result, resume_result) {
        (Ok(patch_result), Ok(())) => Ok(patch_result),
        (Err(error_message), _) => Err(error_message),
        (Ok(_), Err(error_message)) => Err(error_message),
    }
}

impl PrivilegedCommandExecutor for PatchesCommand {
    type ResponseType = PrivilegedCommandResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandExecutor>::ResponseType {
        match self {
            PatchesCommand::Apply { patch_apply_request } => patch_apply_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            PatchesCommand::NoOperation { patch_no_operation_request } => patch_no_operation_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            PatchesCommand::Restore { patch_restore_request } => patch_restore_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            PatchesCommand::RestoreAddress { patch_restore_address_request } => patch_restore_address_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            PatchesCommand::List { patch_list_request } => patch_list_request
                .execute(engine_privileged_state)
                .to_engine_response(),
        }
    }
}

impl PrivilegedCommandRequestExecutor for PatchNoOperationRequest {
    type ResponseType = PatchNoOperationResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        let Some(opened_process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        else {
            return PatchNoOperationResponse {
                status: no_opened_process_status(),
                patch: None,
            };
        };

        let (instruction_target_architecture, patch_address) = opened_process_info
            .get_target_architecture()
            .normalize_instruction_address(self.address);
        let instruction_set_id = instruction_target_architecture
            .get_instruction_set_id()
            .to_string();
        let Some(instruction_set) = engine_privileged_state
            .get_plugin_registry()
            .find_instruction_set(&instruction_set_id)
        else {
            return PatchNoOperationResponse {
                status: failure_status(format!(
                    "No instruction set plugin is enabled for target architecture '{}'.",
                    instruction_set_id
                )),
                patch: None,
            };
        };
        let instruction_bytes = match resolve_no_operation_instruction_bytes(
            patch_address,
            &self.module_name,
            engine_privileged_state,
            &opened_process_info,
            instruction_set.get_max_instruction_size(),
        ) {
            Ok(instruction_bytes) => instruction_bytes,
            Err(error_message) => {
                return PatchNoOperationResponse {
                    status: failure_status(error_message),
                    patch: None,
                };
            }
        };
        let instruction_byte_count = match instruction_set.get_first_instruction_length(&instruction_bytes) {
            Ok(instruction_byte_count) => instruction_byte_count,
            Err(error_message) => {
                return PatchNoOperationResponse {
                    status: failure_status(error_message),
                    patch: None,
                };
            }
        };
        let patched_bytes = match instruction_set.build_no_operation_fill(instruction_byte_count) {
            Ok(patched_bytes) if patched_bytes.len() == instruction_byte_count => patched_bytes,
            Ok(patched_bytes) => {
                return PatchNoOperationResponse {
                    status: failure_status(format!(
                        "Instruction set plugin produced {} no-operation bytes for a {} byte instruction.",
                        patched_bytes.len(),
                        instruction_byte_count
                    )),
                    patch: None,
                };
            }
            Err(error_message) => {
                return PatchNoOperationResponse {
                    status: failure_status(error_message),
                    patch: None,
                };
            }
        };

        match run_patch_operation_with_debugger_paused(engine_privileged_state, &opened_process_info, "no-operation patch", || {
            engine_privileged_state.get_patch_service().apply_patch(
                &opened_process_info,
                engine_privileged_state.get_os_providers(),
                patch_address,
                &self.module_name,
                &patched_bytes,
                PatchKind::NoOperation,
                self.label.clone(),
            )
        }) {
            Ok(patch) => PatchNoOperationResponse {
                status: PatchCommandStatus::success(),
                patch: Some(patch),
            },
            Err(error_message) => PatchNoOperationResponse {
                status: failure_status(error_message),
                patch: None,
            },
        }
    }
}

impl PrivilegedCommandRequestExecutor for PatchApplyRequest {
    type ResponseType = PatchApplyResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        let Some(opened_process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        else {
            return PatchApplyResponse {
                status: no_opened_process_status(),
                patch: None,
            };
        };

        match run_patch_operation_with_debugger_paused(engine_privileged_state, &opened_process_info, "patch apply", || {
            engine_privileged_state.get_patch_service().apply_patch(
                &opened_process_info,
                engine_privileged_state.get_os_providers(),
                self.address,
                &self.module_name,
                &self.patched_bytes,
                self.kind,
                self.label.clone(),
            )
        }) {
            Ok(patch) => PatchApplyResponse {
                status: PatchCommandStatus::success(),
                patch: Some(patch),
            },
            Err(error_message) => PatchApplyResponse {
                status: failure_status(error_message),
                patch: None,
            },
        }
    }
}

impl PrivilegedCommandRequestExecutor for PatchRestoreRequest {
    type ResponseType = PatchRestoreResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        let Some(opened_process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        else {
            return PatchRestoreResponse {
                status: no_opened_process_status(),
                patch: None,
            };
        };

        match run_patch_operation_with_debugger_paused(engine_privileged_state, &opened_process_info, "patch restore", || {
            engine_privileged_state
                .get_patch_service()
                .restore_patch(&opened_process_info, engine_privileged_state.get_os_providers(), &self.patch_id)
        }) {
            Ok(patch) => PatchRestoreResponse {
                status: PatchCommandStatus::success(),
                patch: Some(patch),
            },
            Err(error_message) => PatchRestoreResponse {
                status: failure_status(error_message),
                patch: None,
            },
        }
    }
}

impl PrivilegedCommandRequestExecutor for PatchRestoreAddressRequest {
    type ResponseType = PatchRestoreAddressResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        let Some(opened_process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        else {
            return PatchRestoreAddressResponse {
                status: no_opened_process_status(),
                patch: None,
            };
        };

        match run_patch_operation_with_debugger_paused(engine_privileged_state, &opened_process_info, "patch restore by address", || {
            engine_privileged_state
                .get_patch_service()
                .restore_patch_at_address(
                    &opened_process_info,
                    engine_privileged_state.get_os_providers(),
                    self.address,
                    &self.module_name,
                    self.expected_kind,
                )
        }) {
            Ok(patch) => PatchRestoreAddressResponse {
                status: PatchCommandStatus::success(),
                patch: Some(patch),
            },
            Err(error_message) => PatchRestoreAddressResponse {
                status: failure_status(error_message),
                patch: None,
            },
        }
    }
}

impl PrivilegedCommandRequestExecutor for PatchListRequest {
    type ResponseType = PatchListResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        let Some(opened_process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        else {
            return PatchListResponse {
                status: no_opened_process_status(),
                patches: Vec::new(),
            };
        };

        match engine_privileged_state
            .get_patch_service()
            .list_patches(&opened_process_info)
        {
            Ok(patches) => PatchListResponse {
                status: PatchCommandStatus::success(),
                patches,
            },
            Err(error_message) => PatchListResponse {
                status: failure_status(error_message),
                patches: Vec::new(),
            },
        }
    }
}

fn resolve_no_operation_instruction_bytes(
    address: u64,
    module_name: &str,
    engine_privileged_state: &Arc<EnginePrivilegedState>,
    opened_process_info: &OpenedProcessInfo,
    max_instruction_size: usize,
) -> Result<Vec<u8>, String> {
    let absolute_address = resolve_absolute_address(engine_privileged_state, opened_process_info, address, module_name)?;
    let instruction_read_byte_count =
        resolve_instruction_read_byte_count(engine_privileged_state, opened_process_info, absolute_address, max_instruction_size)?;
    let mut instruction_bytes = vec![0_u8; instruction_read_byte_count];
    if !engine_privileged_state
        .get_os_providers()
        .memory_read
        .read_bytes(opened_process_info, absolute_address, &mut instruction_bytes)
    {
        return Err(format!("Failed to read instruction bytes at 0x{:X}.", absolute_address));
    }

    Ok(instruction_bytes)
}

fn resolve_instruction_read_byte_count(
    engine_privileged_state: &Arc<EnginePrivilegedState>,
    opened_process_info: &OpenedProcessInfo,
    absolute_address: u64,
    max_instruction_size: usize,
) -> Result<usize, String> {
    if max_instruction_size == 0 {
        return Err(String::from("Instruction set plugin reported a zero-byte maximum instruction size."));
    }

    let containing_region = engine_privileged_state
        .get_os_providers()
        .memory_query
        .get_memory_page_bounds(opened_process_info, PageRetrievalMode::FromUserMode)
        .into_iter()
        .find(|normalized_region| absolute_address >= normalized_region.get_base_address() && absolute_address < normalized_region.get_end_address());
    let Some(containing_region) = containing_region else {
        return Err(format!("Address 0x{:X} is not inside a readable memory region.", absolute_address));
    };
    let remaining_region_size = containing_region
        .get_end_address()
        .saturating_sub(absolute_address) as usize;
    let instruction_read_byte_count = max_instruction_size.min(remaining_region_size);

    if instruction_read_byte_count == 0 {
        return Err(format!(
            "Address 0x{:X} has no readable instruction bytes remaining in its region.",
            absolute_address
        ));
    }

    Ok(instruction_read_byte_count)
}

fn resolve_absolute_address(
    engine_privileged_state: &Arc<EnginePrivilegedState>,
    opened_process_info: &OpenedProcessInfo,
    address: u64,
    module_name: &str,
) -> Result<u64, String> {
    if module_name.trim().is_empty() {
        return Ok(address);
    }

    let modules = engine_privileged_state
        .get_os_providers()
        .memory_query
        .get_modules(opened_process_info);
    let Some(module) = modules
        .iter()
        .find(|module| module.get_module_name().eq_ignore_ascii_case(module_name))
    else {
        return Err(format!("Module '{}' is not loaded in the opened process.", module_name));
    };

    module
        .get_base_address()
        .checked_add(address)
        .ok_or_else(|| format!("Module-relative address {}+0x{:X} overflowed.", module_name, address))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
    use crossbeam_channel::{Receiver, unbounded};
    use squalr_engine_api::{
        commands::{privileged_command::PrivilegedCommand, privileged_command_response::PrivilegedCommandResponse},
        engine::{
            engine_api_priviliged_bindings::EngineApiPrivilegedBindings, engine_binding_error::EngineBindingError, engine_event_envelope::EngineEventEnvelope,
        },
        events::engine_event::EngineEvent,
        plugins::memory_view::PageRetrievalMode,
        structures::{
            data_values::data_value::DataValue,
            memory::{bitness::Bitness, normalized_module::NormalizedModule, normalized_region::NormalizedRegion},
            processes::{opened_process_info::OpenedProcessInfo, process_info::ProcessInfo, target_architecture::TargetArchitecture},
            structs::valued_struct::ValuedStruct,
        },
    };
    use squalr_engine_session::{
        engine_privileged_state::EnginePrivilegedState,
        os::{
            ProcessQueryError, ProcessQueryOptions,
            engine_os_provider::{EngineOsProviders, MemoryQueryProvider, MemoryReadProvider, MemoryWriteProvider, ProcessQueryProvider},
        },
    };
    use std::sync::{Arc, Mutex, RwLock};

    struct NoOpPrivilegedBindings;

    impl EngineApiPrivilegedBindings for NoOpPrivilegedBindings {
        fn emit_event(
            &self,
            _event: EngineEvent,
        ) -> Result<(), EngineBindingError> {
            Ok(())
        }

        fn dispatch_internal_command(
            &self,
            _engine_command: PrivilegedCommand,
            _callback: Box<dyn FnOnce(PrivilegedCommandResponse) + Send + Sync + 'static>,
        ) -> Result<(), EngineBindingError> {
            Err(EngineBindingError::unavailable("dispatching internal commands in no-operation patch tests"))
        }

        fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEventEnvelope>, EngineBindingError> {
            let (_event_sender, event_receiver) = unbounded();

            Ok(event_receiver)
        }
    }

    struct NoOpProcessQueryProvider;

    impl ProcessQueryProvider for NoOpProcessQueryProvider {
        fn start_monitoring(&self) -> Result<(), ProcessQueryError> {
            Ok(())
        }

        fn get_processes(
            &self,
            _process_query_options: ProcessQueryOptions,
        ) -> Vec<ProcessInfo> {
            Vec::new()
        }

        fn open_process(
            &self,
            _process_info: &ProcessInfo,
        ) -> Result<OpenedProcessInfo, ProcessQueryError> {
            Err(ProcessQueryError::internal("open_process", "not implemented in no-operation patch tests"))
        }

        fn close_process(
            &self,
            _handle: u64,
        ) -> Result<(), ProcessQueryError> {
            Ok(())
        }
    }

    #[derive(Clone)]
    struct TestMemoryQueryProvider {
        regions: Vec<NormalizedRegion>,
    }

    impl MemoryQueryProvider for TestMemoryQueryProvider {
        fn get_modules(
            &self,
            _process_info: &OpenedProcessInfo,
        ) -> Vec<NormalizedModule> {
            Vec::new()
        }

        fn address_to_module(
            &self,
            _address: u64,
            _modules: &Vec<NormalizedModule>,
        ) -> Option<(String, u64)> {
            None
        }

        fn resolve_module(
            &self,
            _modules: &Vec<NormalizedModule>,
            _identifier: &str,
        ) -> u64 {
            0
        }

        fn get_memory_page_bounds(
            &self,
            _process_info: &OpenedProcessInfo,
            _page_retrieval_mode: PageRetrievalMode,
        ) -> Vec<NormalizedRegion> {
            self.regions.clone()
        }
    }

    #[derive(Clone)]
    struct TestMemory {
        base_address: u64,
        bytes: Arc<Mutex<Vec<u8>>>,
        read_lengths: Arc<Mutex<Vec<usize>>>,
        written_bytes: Arc<Mutex<Vec<Vec<u8>>>>,
    }

    impl TestMemory {
        fn new(
            base_address: u64,
            bytes: Vec<u8>,
        ) -> Self {
            Self {
                base_address,
                bytes: Arc::new(Mutex::new(bytes)),
                read_lengths: Arc::new(Mutex::new(Vec::new())),
                written_bytes: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn read_lengths(&self) -> Vec<usize> {
            self.read_lengths
                .lock()
                .map(|read_lengths| read_lengths.clone())
                .unwrap_or_default()
        }

        fn written_bytes(&self) -> Vec<Vec<u8>> {
            self.written_bytes
                .lock()
                .map(|written_bytes| written_bytes.clone())
                .unwrap_or_default()
        }
    }

    impl MemoryReadProvider for TestMemory {
        fn read(
            &self,
            _process_info: &OpenedProcessInfo,
            _address: u64,
            _data_value: &mut DataValue,
        ) -> bool {
            false
        }

        fn read_struct(
            &self,
            _process_info: &OpenedProcessInfo,
            _address: u64,
            _valued_struct: &mut ValuedStruct,
        ) -> bool {
            false
        }

        fn read_bytes(
            &self,
            _process_info: &OpenedProcessInfo,
            address: u64,
            values: &mut [u8],
        ) -> bool {
            if self
                .read_lengths
                .lock()
                .map(|mut read_lengths| read_lengths.push(values.len()))
                .is_err()
            {
                return false;
            }

            let Some(read_start_offset) = address.checked_sub(self.base_address) else {
                return false;
            };
            let read_start_offset = read_start_offset as usize;
            let Ok(bytes) = self.bytes.lock() else {
                return false;
            };
            let Some(read_end_offset) = read_start_offset.checked_add(values.len()) else {
                return false;
            };

            if read_end_offset > bytes.len() {
                return false;
            }

            values.copy_from_slice(&bytes[read_start_offset..read_end_offset]);

            true
        }
    }

    impl MemoryWriteProvider for TestMemory {
        fn write_bytes(
            &self,
            _process_info: &OpenedProcessInfo,
            address: u64,
            values: &[u8],
        ) -> bool {
            if self
                .written_bytes
                .lock()
                .map(|mut written_bytes| written_bytes.push(values.to_vec()))
                .is_err()
            {
                return false;
            }

            let Some(write_start_offset) = address.checked_sub(self.base_address) else {
                return false;
            };
            let write_start_offset = write_start_offset as usize;
            let Ok(mut bytes) = self.bytes.lock() else {
                return false;
            };
            let Some(write_end_offset) = write_start_offset.checked_add(values.len()) else {
                return false;
            };

            if write_end_offset > bytes.len() {
                return false;
            }

            bytes[write_start_offset..write_end_offset].copy_from_slice(values);

            true
        }
    }

    fn create_test_engine_privileged_state(
        opened_process_info: OpenedProcessInfo,
        memory_regions: Vec<NormalizedRegion>,
        test_memory: TestMemory,
    ) -> Arc<EnginePrivilegedState> {
        let engine_bindings: Arc<RwLock<dyn EngineApiPrivilegedBindings>> = Arc::new(RwLock::new(NoOpPrivilegedBindings));
        let os_providers = EngineOsProviders::new(
            Arc::new(NoOpProcessQueryProvider),
            Arc::new(TestMemoryQueryProvider { regions: memory_regions }),
            Arc::new(test_memory.clone()),
            Arc::new(test_memory),
        );
        let engine_privileged_state =
            EnginePrivilegedState::new(engine_bindings, os_providers).expect("Expected no-operation patch test engine state to initialize.");
        engine_privileged_state
            .get_process_manager()
            .set_opened_process(opened_process_info);

        engine_privileged_state
    }

    #[test]
    fn no_operation_patch_reads_only_target_instruction_set_max_size() {
        let test_memory = TestMemory::new(0x1000, vec![0x1F, 0x20, 0x03, 0xD5, 0xAA, 0xBB, 0xCC, 0xDD]);
        let opened_process_info =
            OpenedProcessInfo::new(1, String::from("arm64-target"), 1, Bitness::Bit64, None).with_target_architecture(TargetArchitecture::arm64());
        let engine_privileged_state = create_test_engine_privileged_state(opened_process_info, vec![NormalizedRegion::new(0x1000, 0x100)], test_memory.clone());
        let patch_no_operation_request = PatchNoOperationRequest {
            address: 0x1000,
            module_name: String::new(),
            label: None,
        };

        let patch_no_operation_response = patch_no_operation_request.execute(&engine_privileged_state);

        assert!(
            patch_no_operation_response.status.get_success(),
            "Expected no-operation patch to succeed: {:?}.",
            patch_no_operation_response.status.get_message()
        );
        assert_eq!(test_memory.read_lengths(), vec![4, 4]);
        assert_eq!(test_memory.written_bytes(), vec![vec![0x1F, 0x20, 0x03, 0xD5]]);
    }

    #[test]
    fn no_operation_patch_replaces_fixed_width_instruction_without_full_disassembly() {
        let test_memory = TestMemory::new(0x1000, vec![0xAA, 0xBB, 0xCC, 0xDD]);
        let opened_process_info =
            OpenedProcessInfo::new(1, String::from("arm64-target"), 1, Bitness::Bit64, None).with_target_architecture(TargetArchitecture::arm64());
        let engine_privileged_state = create_test_engine_privileged_state(opened_process_info, vec![NormalizedRegion::new(0x1000, 0x4)], test_memory.clone());
        let patch_no_operation_request = PatchNoOperationRequest {
            address: 0x1000,
            module_name: String::new(),
            label: None,
        };

        let patch_no_operation_response = patch_no_operation_request.execute(&engine_privileged_state);

        assert!(
            patch_no_operation_response.status.get_success(),
            "Expected fixed-width no-operation patch to succeed: {:?}.",
            patch_no_operation_response.status.get_message()
        );
        assert_eq!(test_memory.read_lengths(), vec![4, 4]);
        assert_eq!(test_memory.written_bytes(), vec![vec![0x1F, 0x20, 0x03, 0xD5]]);
    }

    #[test]
    fn no_operation_patch_clamps_instruction_read_to_remaining_region_size() {
        let test_memory = TestMemory::new(0x1000, vec![0xAA, 0xBB, 0x90]);
        let opened_process_info =
            OpenedProcessInfo::new(1, String::from("x64-target"), 1, Bitness::Bit64, None).with_target_architecture(TargetArchitecture::x64());
        let engine_privileged_state = create_test_engine_privileged_state(opened_process_info, vec![NormalizedRegion::new(0x1000, 0x3)], test_memory.clone());
        let patch_no_operation_request = PatchNoOperationRequest {
            address: 0x1002,
            module_name: String::new(),
            label: None,
        };

        let patch_no_operation_response = patch_no_operation_request.execute(&engine_privileged_state);

        assert!(
            patch_no_operation_response.status.get_success(),
            "Expected no-operation patch to succeed: {:?}.",
            patch_no_operation_response.status.get_message()
        );
        assert_eq!(test_memory.read_lengths(), vec![1, 1]);
        assert_eq!(test_memory.written_bytes(), vec![vec![0x90]]);
    }

    #[test]
    fn no_operation_patch_replaces_thumb16_instruction_with_two_byte_nop() {
        let test_memory = TestMemory::new(0x1000, vec![0x70, 0x47, 0xAA, 0xBB]);
        let opened_process_info =
            OpenedProcessInfo::new(1, String::from("thumb-target"), 1, Bitness::Bit32, None).with_target_architecture(TargetArchitecture::thumb());
        let engine_privileged_state = create_test_engine_privileged_state(opened_process_info, vec![NormalizedRegion::new(0x1000, 0x4)], test_memory.clone());
        let patch_no_operation_request = PatchNoOperationRequest {
            address: 0x1000,
            module_name: String::new(),
            label: None,
        };

        let patch_no_operation_response = patch_no_operation_request.execute(&engine_privileged_state);

        assert!(
            patch_no_operation_response.status.get_success(),
            "Expected Thumb16 no-operation patch to succeed: {:?}.",
            patch_no_operation_response.status.get_message()
        );
        assert_eq!(test_memory.read_lengths(), vec![4, 2]);
        assert_eq!(test_memory.written_bytes(), vec![vec![0x00, 0xBF]]);
    }

    #[test]
    fn no_operation_patch_selects_thumb_for_arm_interworking_address() {
        let test_memory = TestMemory::new(0x1000, vec![0x70, 0x47, 0xAA, 0xBB]);
        let opened_process_info =
            OpenedProcessInfo::new(1, String::from("arm-target"), 1, Bitness::Bit32, None).with_target_architecture(TargetArchitecture::arm());
        let engine_privileged_state = create_test_engine_privileged_state(opened_process_info, vec![NormalizedRegion::new(0x1000, 0x4)], test_memory.clone());
        let patch_no_operation_request = PatchNoOperationRequest {
            address: 0x1001,
            module_name: String::new(),
            label: None,
        };

        let patch_no_operation_response = patch_no_operation_request.execute(&engine_privileged_state);

        assert!(
            patch_no_operation_response.status.get_success(),
            "Expected ARM interworking no-operation patch to succeed: {:?}.",
            patch_no_operation_response.status.get_message()
        );
        assert_eq!(test_memory.read_lengths(), vec![4, 2]);
        assert_eq!(test_memory.written_bytes(), vec![vec![0x00, 0xBF]]);
        assert_eq!(
            patch_no_operation_response
                .patch
                .as_ref()
                .map(|patch| patch.get_region().get_base_address()),
            Some(0x1000)
        );
    }

    #[test]
    fn no_operation_patch_replaces_thumb32_instruction_with_four_byte_nop_fill() {
        let test_memory = TestMemory::new(0x1000, vec![0x00, 0xF0, 0x00, 0x80]);
        let opened_process_info =
            OpenedProcessInfo::new(1, String::from("thumb-target"), 1, Bitness::Bit32, None).with_target_architecture(TargetArchitecture::thumb());
        let engine_privileged_state = create_test_engine_privileged_state(opened_process_info, vec![NormalizedRegion::new(0x1000, 0x4)], test_memory.clone());
        let patch_no_operation_request = PatchNoOperationRequest {
            address: 0x1000,
            module_name: String::new(),
            label: None,
        };

        let patch_no_operation_response = patch_no_operation_request.execute(&engine_privileged_state);

        assert!(
            patch_no_operation_response.status.get_success(),
            "Expected Thumb32 no-operation patch to succeed: {:?}.",
            patch_no_operation_response.status.get_message()
        );
        assert_eq!(test_memory.read_lengths(), vec![4, 4]);
        assert_eq!(test_memory.written_bytes(), vec![vec![0x00, 0xBF, 0x00, 0xBF]]);
    }
}
