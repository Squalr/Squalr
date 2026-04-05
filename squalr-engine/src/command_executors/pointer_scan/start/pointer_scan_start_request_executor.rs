use crate::command_executors::pointer_scan::pointer_scan_target_resolver::PointerScanTargetResolver;
use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::command_executors::snapshot_region_builder::merge_memory_regions_into_snapshot_regions;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::pointer_scan::start::pointer_scan_start_request::PointerScanStartRequest;
use squalr_engine_api::commands::pointer_scan::start::pointer_scan_start_response::PointerScanStartResponse;
use squalr_engine_api::structures::memory::bitness::Bitness;
use squalr_engine_api::structures::memory::memory_alignment::MemoryAlignment;
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::scanning::plans::pointer_scan::pointer_scan_parameters::PointerScanParameters;
use squalr_engine_api::structures::snapshots::snapshot::Snapshot;
use squalr_engine_scanning::pointer_scans::pointer_scan_executor_task::PointerScanExecutor;
use squalr_engine_scanning::scan_settings_config::ScanSettingsConfig;
use squalr_engine_scanning::scanners::scan_execution_context::ScanExecutionContext;
use squalr_engine_session::os::PageRetrievalMode;
use std::sync::{Arc, RwLock};

impl PrivilegedCommandRequestExecutor for PointerScanStartRequest {
    type ResponseType = PointerScanStartResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        let Some(process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        else {
            log::error!("No opened process.");

            return PointerScanStartResponse::default();
        };
        let effective_pointer_size = resolve_pointer_size_for_process_bitness(self.pointer_size, &process_info);
        let modules = engine_privileged_state
            .get_os_providers()
            .memory_query
            .get_modules(&process_info);
        let pointer_scan_memory_regions = engine_privileged_state
            .get_os_providers()
            .memory_query
            .get_memory_page_bounds(&process_info, PageRetrievalMode::FromUserMode);
        let mut pointer_scan_snapshot = Snapshot::new();
        pointer_scan_snapshot.set_snapshot_regions(merge_memory_regions_into_snapshot_regions(pointer_scan_memory_regions));
        let pointer_scan_snapshot = Arc::new(RwLock::new(pointer_scan_snapshot));
        let memory_read_provider = engine_privileged_state.get_os_providers().memory_read.clone();
        let scan_execution_context = ScanExecutionContext::new(
            None,
            None,
            Some(Arc::new(move |opened_process_info, address, values| {
                memory_read_provider.read_bytes(opened_process_info, address, values)
            })),
        );
        let pointer_scan_parameters = PointerScanParameters::new(
            effective_pointer_size,
            self.offset_radius,
            self.max_depth,
            ScanSettingsConfig::get_is_single_threaded_scan(),
            ScanSettingsConfig::get_debug_perform_validation_scan(),
        );
        PointerScanExecutor::collect_pointer_scan_values(
            process_info.clone(),
            pointer_scan_snapshot.clone(),
            pointer_scan_snapshot.clone(),
            true,
            &scan_execution_context,
        );
        let resolved_targets = match engine_privileged_state.read_symbol_registry(|symbol_registry| {
            PointerScanTargetResolver::resolve_targets(
                &self.target,
                symbol_registry,
                effective_pointer_size,
                pointer_scan_snapshot.clone(),
                process_info.clone(),
                ScanSettingsConfig::get_memory_alignment().unwrap_or(MemoryAlignment::Alignment1),
                ScanSettingsConfig::get_floating_point_tolerance(),
                ScanSettingsConfig::get_is_single_threaded_scan(),
                ScanSettingsConfig::get_debug_perform_validation_scan(),
                &scan_execution_context,
            )
        }) {
            Ok(resolved_targets) => resolved_targets,
            Err(error) => {
                log::error!("{}", error);
                return PointerScanStartResponse::default();
            }
        };
        let pointer_scan_session_id = engine_privileged_state.allocate_pointer_scan_session_id();
        let pointer_scan_session = PointerScanExecutor::execute_scan_with_precollected_values(
            pointer_scan_snapshot.clone(),
            pointer_scan_snapshot,
            pointer_scan_session_id,
            pointer_scan_parameters,
            resolved_targets.target_descriptor,
            resolved_targets.target_addresses,
            &modules,
            true,
        );
        let pointer_scan_summary = pointer_scan_session.summarize();

        match engine_privileged_state.get_pointer_scan_session().write() {
            Ok(mut pointer_scan_session_guard) => {
                *pointer_scan_session_guard = Some(pointer_scan_session);
            }
            Err(error) => {
                log::error!("Failed to acquire write lock on pointer scan session store: {}", error);
            }
        }

        PointerScanStartResponse {
            pointer_scan_summary: Some(pointer_scan_summary),
        }
    }
}

fn resolve_pointer_size_for_process_bitness(
    requested_pointer_size: PointerScanPointerSize,
    process_info: &OpenedProcessInfo,
) -> PointerScanPointerSize {
    let process_pointer_size = match process_info.get_bitness() {
        Bitness::Bit32 => PointerScanPointerSize::Pointer32,
        Bitness::Bit64 => PointerScanPointerSize::Pointer64,
    };

    if requested_pointer_size != process_pointer_size {
        log::warn!(
            "Pointer scan requested {} for process {} (PID {}) but the process is {:?}; using {} instead.",
            requested_pointer_size,
            process_info.get_name(),
            process_info.get_process_id_raw(),
            process_info.get_bitness(),
            process_pointer_size,
        );
    }

    process_pointer_size
}

#[cfg(test)]
mod tests {
    use super::PointerScanStartRequest;
    use squalr_engine_api::structures::memory::bitness::Bitness;
    use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
    use squalr_engine_api::structures::memory::normalized_region::NormalizedRegion;
    use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
    use squalr_engine_api::structures::pointer_scans::pointer_scan_target_request::PointerScanTargetRequest;
    use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
    use squalr_engine_api::structures::processes::process_info::ProcessInfo;
    use squalr_engine_api::structures::structs::valued_struct::ValuedStruct;
    use squalr_engine_session::os::engine_os_provider::{
        EngineOsProviders, MemoryQueryProvider, MemoryReadProvider, MemoryWriteProvider, ProcessQueryProvider,
    };
    use squalr_engine_session::os::{PageRetrievalMode, ProcessQueryError, ProcessQueryOptions};
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};

    #[derive(Default)]
    struct TestEngineBindings;

    impl EngineApiPrivilegedBindings for TestEngineBindings {
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
            Ok(())
        }

        fn subscribe_to_engine_events(&self) -> Result<Receiver<squalr_engine_api::engine::engine_event_envelope::EngineEventEnvelope>, EngineBindingError> {
            let (_sender, receiver) = unbounded();

            Ok(receiver)
        }
    }

    struct TestProcessQueryProvider;

    impl ProcessQueryProvider for TestProcessQueryProvider {
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
            process_info: &ProcessInfo,
        ) -> Result<OpenedProcessInfo, ProcessQueryError> {
            Ok(OpenedProcessInfo::new(
                process_info.get_process_id(),
                process_info.get_name().to_string(),
                1,
                Bitness::Bit64,
                None,
            ))
        }

        fn close_process(
            &self,
            _handle: u64,
        ) -> Result<(), ProcessQueryError> {
            Ok(())
        }
    }

    struct TestMemoryQueryProvider {
        module_descriptors: Vec<(String, u64, u64)>,
        usermode_memory_regions: Vec<NormalizedRegion>,
    }

    impl MemoryQueryProvider for TestMemoryQueryProvider {
        fn get_modules(
            &self,
            _process_info: &OpenedProcessInfo,
        ) -> Vec<NormalizedModule> {
            self.module_descriptors
                .iter()
                .map(|(module_name, base_address, region_size)| NormalizedModule::new(module_name, *base_address, *region_size))
                .collect()
        }

        fn address_to_module(
            &self,
            address: u64,
            modules: &Vec<NormalizedModule>,
        ) -> Option<(String, u64)> {
            modules.iter().find_map(|module| {
                if module.contains_address(address) {
                    Some((module.get_module_name().to_string(), address.saturating_sub(module.get_base_address())))
                } else {
                    None
                }
            })
        }

        fn resolve_module(
            &self,
            modules: &Vec<NormalizedModule>,
            identifier: &str,
        ) -> u64 {
            modules
                .iter()
                .find(|module| module.get_module_name() == identifier)
                .map(NormalizedModule::get_base_address)
                .unwrap_or_default()
        }

        fn get_memory_page_bounds(
            &self,
            _process_info: &OpenedProcessInfo,
            page_retrieval_mode: PageRetrievalMode,
        ) -> Vec<NormalizedRegion> {
            if page_retrieval_mode == PageRetrievalMode::FromUserMode {
                self.usermode_memory_regions.clone()
            } else {
                Vec::new()
            }
        }
    }

    struct TestMemoryReadProvider {
        memory_bytes_by_address: HashMap<u64, u8>,
    }

    impl MemoryReadProvider for TestMemoryReadProvider {
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
            for (byte_index, value) in values.iter_mut().enumerate() {
                *value = *self
                    .memory_bytes_by_address
                    .get(&address.saturating_add(byte_index as u64))
                    .unwrap_or(&0);
            }

            true
        }
    }

    struct TestMemoryWriteProvider;

    impl MemoryWriteProvider for TestMemoryWriteProvider {
        fn write_bytes(
            &self,
            _process_info: &OpenedProcessInfo,
            _address: u64,
            _values: &[u8],
        ) -> bool {
            false
        }
    }

    #[test]
    fn execute_uses_fresh_usermode_snapshot_when_global_snapshot_is_empty() {
        let engine_bindings: Arc<RwLock<dyn EngineApiPrivilegedBindings>> = Arc::new(RwLock::new(TestEngineBindings));
        let os_providers = EngineOsProviders::new(
            Arc::new(TestProcessQueryProvider),
            Arc::new(TestMemoryQueryProvider {
                module_descriptors: vec![(String::from("game.exe"), 0x1000, 0x100)],
                usermode_memory_regions: vec![
                    NormalizedRegion::new(0x1000, 0x40),
                    NormalizedRegion::new(0x2000, 0x40),
                    NormalizedRegion::new(0x3000, 0x40),
                ],
            }),
            Arc::new(TestMemoryReadProvider {
                memory_bytes_by_address: build_pointer_scan_memory_map(),
            }),
            Arc::new(TestMemoryWriteProvider),
        );
        let engine_privileged_state = EnginePrivilegedState::new(engine_bindings, os_providers).expect("Expected the test engine state to initialize.");

        engine_privileged_state
            .get_process_manager()
            .set_opened_process(OpenedProcessInfo::new(
                std::process::id(),
                String::from("pointer-test"),
                1,
                Bitness::Bit64,
                None,
            ));

        assert_eq!(
            engine_privileged_state
                .get_snapshot()
                .read()
                .expect("Expected the shared snapshot lock.")
                .get_region_count(),
            0
        );

        let pointer_scan_start_response = PointerScanStartRequest {
            target: PointerScanTargetRequest::address(AnonymousValueString::new(
                String::from("0x3010"),
                AnonymousValueStringFormat::Hexadecimal,
                ContainerType::None,
            )),
            pointer_size: PointerScanPointerSize::Pointer64,
            max_depth: 3,
            offset_radius: 0x20,
        }
        .execute(&engine_privileged_state);
        let pointer_scan_summary = pointer_scan_start_response
            .pointer_scan_summary
            .expect("Expected the pointer scan summary.");

        assert_eq!(pointer_scan_summary.get_root_node_count(), 2);
        assert_eq!(pointer_scan_summary.get_total_node_count(), 3);
        assert_eq!(pointer_scan_summary.get_total_static_node_count(), 2);
        assert_eq!(pointer_scan_summary.get_total_heap_node_count(), 1);
        assert_eq!(
            engine_privileged_state
                .get_snapshot()
                .read()
                .expect("Expected the shared snapshot lock.")
                .get_region_count(),
            0
        );
    }

    #[test]
    fn execute_uses_opened_process_bitness_for_pointer_scans() {
        let engine_bindings: Arc<RwLock<dyn EngineApiPrivilegedBindings>> = Arc::new(RwLock::new(TestEngineBindings));
        let os_providers = EngineOsProviders::new(
            Arc::new(TestProcessQueryProvider),
            Arc::new(TestMemoryQueryProvider {
                module_descriptors: vec![(String::from("game.exe"), 0x1000, 0x100)],
                usermode_memory_regions: vec![
                    NormalizedRegion::new(0x1000, 0x40),
                    NormalizedRegion::new(0x2000, 0x40),
                    NormalizedRegion::new(0x3000, 0x40),
                ],
            }),
            Arc::new(TestMemoryReadProvider {
                memory_bytes_by_address: build_pointer_scan_memory_map_32(),
            }),
            Arc::new(TestMemoryWriteProvider),
        );
        let engine_privileged_state = EnginePrivilegedState::new(engine_bindings, os_providers).expect("Expected the test engine state to initialize.");

        engine_privileged_state
            .get_process_manager()
            .set_opened_process(OpenedProcessInfo::new(
                std::process::id(),
                String::from("pointer-test-32"),
                1,
                Bitness::Bit32,
                None,
            ));

        let pointer_scan_start_response = PointerScanStartRequest {
            target: PointerScanTargetRequest::address(AnonymousValueString::new(
                String::from("0x3010"),
                AnonymousValueStringFormat::Hexadecimal,
                ContainerType::None,
            )),
            pointer_size: PointerScanPointerSize::Pointer64,
            max_depth: 3,
            offset_radius: 0x20,
        }
        .execute(&engine_privileged_state);
        let pointer_scan_summary = pointer_scan_start_response
            .pointer_scan_summary
            .expect("Expected the pointer scan summary.");

        assert_eq!(pointer_scan_summary.get_pointer_size(), PointerScanPointerSize::Pointer32);
        assert_eq!(pointer_scan_summary.get_root_node_count(), 2);
        assert_eq!(pointer_scan_summary.get_total_node_count(), 3);
        assert_eq!(pointer_scan_summary.get_total_static_node_count(), 2);
        assert_eq!(pointer_scan_summary.get_total_heap_node_count(), 1);
    }

    #[test]
    fn execute_supports_value_seeded_pointer_scans() {
        let engine_bindings: Arc<RwLock<dyn EngineApiPrivilegedBindings>> = Arc::new(RwLock::new(TestEngineBindings));
        let os_providers = EngineOsProviders::new(
            Arc::new(TestProcessQueryProvider),
            Arc::new(TestMemoryQueryProvider {
                module_descriptors: vec![(String::from("game.exe"), 0x1000, 0x100)],
                usermode_memory_regions: vec![
                    NormalizedRegion::new(0x1000, 0x40),
                    NormalizedRegion::new(0x2000, 0x40),
                    NormalizedRegion::new(0x3000, 0x40),
                ],
            }),
            Arc::new(TestMemoryReadProvider {
                memory_bytes_by_address: build_pointer_scan_memory_map(),
            }),
            Arc::new(TestMemoryWriteProvider),
        );
        let engine_privileged_state = EnginePrivilegedState::new(engine_bindings, os_providers).expect("Expected the test engine state to initialize.");

        engine_privileged_state
            .get_process_manager()
            .set_opened_process(OpenedProcessInfo::new(
                std::process::id(),
                String::from("pointer-test-value"),
                1,
                Bitness::Bit64,
                None,
            ));

        let pointer_scan_start_response = PointerScanStartRequest {
            target: PointerScanTargetRequest::value(
                AnonymousValueString::new(String::from("0x3000"), AnonymousValueStringFormat::Hexadecimal, ContainerType::None),
                DataTypeRef::new("u64"),
            ),
            pointer_size: PointerScanPointerSize::Pointer64,
            max_depth: 2,
            offset_radius: 0x20,
        }
        .execute(&engine_privileged_state);
        let pointer_scan_summary = pointer_scan_start_response
            .pointer_scan_summary
            .expect("Expected the pointer scan summary.");

        assert_eq!(pointer_scan_summary.get_root_node_count(), 1);
        assert_eq!(pointer_scan_summary.get_total_static_node_count(), 1);
        assert_eq!(pointer_scan_summary.get_total_heap_node_count(), 0);
        assert_eq!(
            pointer_scan_summary
                .get_target_descriptor()
                .get_target_address_count(),
            1
        );
        assert_eq!(
            pointer_scan_summary
                .get_target_descriptor()
                .get_data_type_ref()
                .map(DataTypeRef::get_data_type_id),
            Some("u64")
        );
    }

    fn build_pointer_scan_memory_map() -> HashMap<u64, u8> {
        let mut memory_bytes_by_address = HashMap::new();

        write_pointer_bytes(&mut memory_bytes_by_address, 0x1010, 0x1FF0_u64);
        write_pointer_bytes(&mut memory_bytes_by_address, 0x1030, 0x3020_u64);
        write_pointer_bytes(&mut memory_bytes_by_address, 0x2000, 0x3000_u64);

        memory_bytes_by_address
    }

    fn build_pointer_scan_memory_map_32() -> HashMap<u64, u8> {
        let mut memory_bytes_by_address = HashMap::new();

        write_pointer_bytes_32(&mut memory_bytes_by_address, 0x1010, 0x1FF0_u32);
        write_pointer_bytes_32(&mut memory_bytes_by_address, 0x1030, 0x3020_u32);
        write_pointer_bytes_32(&mut memory_bytes_by_address, 0x2000, 0x3000_u32);

        memory_bytes_by_address
    }

    fn write_pointer_bytes(
        memory_bytes_by_address: &mut HashMap<u64, u8>,
        address: u64,
        value: u64,
    ) {
        for (byte_index, byte_value) in value.to_le_bytes().iter().enumerate() {
            memory_bytes_by_address.insert(address.saturating_add(byte_index as u64), *byte_value);
        }
    }

    fn write_pointer_bytes_32(
        memory_bytes_by_address: &mut HashMap<u64, u8>,
        address: u64,
        value: u32,
    ) {
        for (byte_index, byte_value) in value.to_le_bytes().iter().enumerate() {
            memory_bytes_by_address.insert(address.saturating_add(byte_index as u64), *byte_value);
        }
    }
}
