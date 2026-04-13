use crate::command_executors::pointer_scan::pointer_scan_target_resolver::PointerScanTargetResolver;
use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::command_executors::snapshot_region_builder::merge_memory_regions_into_snapshot_regions;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::pointer_scan::start::pointer_scan_start_request::PointerScanStartRequest;
use squalr_engine_api::commands::pointer_scan::start::pointer_scan_start_response::PointerScanStartResponse;
use squalr_engine_api::structures::data_values::anonymous_value_string::AnonymousValueString;
use squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
use squalr_engine_api::structures::memory::address_display::is_virtual_module_address;
use squalr_engine_api::structures::memory::memory_alignment::MemoryAlignment;
use squalr_engine_api::structures::pointer_scans::pointer_scan_address_space::PointerScanAddressSpace;
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use squalr_engine_api::structures::pointer_scans::pointer_scan_target_request::PointerScanTargetRequest;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::scanning::plans::pointer_scan::pointer_scan_parameters::PointerScanParameters;
use squalr_engine_api::structures::snapshots::snapshot::Snapshot;
use squalr_engine_scanning::pointer_scans::pointer_scan_executor_task::PointerScanExecutor;
use squalr_engine_scanning::pointer_scans::pointer_scan_materializer::PointerScanMaterializer;
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
        let effective_address_space = resolve_pointer_scan_address_space(self.address_space, &self.target);
        let effective_pointer_size = resolve_pointer_size_for_process_bitness(self.pointer_size, effective_address_space, &process_info);
        let modules = match effective_address_space {
            PointerScanAddressSpace::Auto => engine_privileged_state
                .get_os_providers()
                .memory_query_raw
                .get_modules(&process_info),
            PointerScanAddressSpace::GameMemory => engine_privileged_state
                .get_os_providers()
                .memory_query
                .get_modules(&process_info),
            PointerScanAddressSpace::EmulatorMemory => engine_privileged_state
                .get_os_providers()
                .memory_query_raw
                .get_modules(&process_info),
        };
        let parsed_target_address = self
            .target
            .target_address
            .as_ref()
            .and_then(parse_target_address);
        let pointer_scan_memory_regions = match effective_address_space {
            PointerScanAddressSpace::Auto => engine_privileged_state
                .get_os_providers()
                .memory_query_raw
                .get_memory_page_bounds(&process_info, PageRetrievalMode::FromUserMode),
            PointerScanAddressSpace::GameMemory => engine_privileged_state
                .get_os_providers()
                .memory_query
                .get_pointer_scan_memory_page_bounds(&process_info, PageRetrievalMode::FromVirtualModules, parsed_target_address),
            PointerScanAddressSpace::EmulatorMemory => engine_privileged_state
                .get_os_providers()
                .memory_query_raw
                .get_memory_page_bounds(&process_info, PageRetrievalMode::FromUserMode),
        };
        if pointer_scan_memory_regions.is_empty() {
            log::error!(
                "Pointer scan start aborted because no readable memory regions were returned for process {} (PID {}).",
                process_info.get_name(),
                process_info.get_process_id_raw()
            );

            return PointerScanStartResponse::default();
        }
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
        let pointer_scan_results_id = engine_privileged_state.allocate_pointer_scan_results_id();
        let pointer_scan_results = PointerScanExecutor::execute_scan_with_precollected_values(
            pointer_scan_snapshot.clone(),
            pointer_scan_snapshot,
            pointer_scan_results_id,
            pointer_scan_parameters,
            resolved_targets.target_descriptor,
            resolved_targets.target_addresses,
            effective_address_space,
            &modules,
            true,
        );
        let pointer_scan_summary = pointer_scan_results.summarize();

        match engine_privileged_state.get_pointer_scan_results().write() {
            Ok(mut pointer_scan_results_guard) => {
                *pointer_scan_results_guard = Some(pointer_scan_results);
            }
            Err(error) => {
                log::error!("Failed to acquire write lock on pointer scan results store: {}", error);
            }
        }
        match engine_privileged_state.get_pointer_scan_materializer().write() {
            Ok(mut pointer_scan_materializer_guard) => {
                *pointer_scan_materializer_guard = Some(PointerScanMaterializer::new());
            }
            Err(error) => {
                log::error!("Failed to acquire write lock on pointer scan materializer store: {}", error);
            }
        }

        PointerScanStartResponse {
            success: true,
            pointer_scan_summary: Some(pointer_scan_summary),
        }
    }
}

fn resolve_pointer_scan_address_space(
    requested_address_space: PointerScanAddressSpace,
    target_request: &PointerScanTargetRequest,
) -> PointerScanAddressSpace {
    match requested_address_space {
        PointerScanAddressSpace::Auto => target_request
            .target_address
            .as_ref()
            .and_then(parse_target_address)
            .filter(|target_address| is_virtual_module_address(*target_address))
            .map(|_| PointerScanAddressSpace::GameMemory)
            .unwrap_or(PointerScanAddressSpace::EmulatorMemory),
        PointerScanAddressSpace::GameMemory => PointerScanAddressSpace::GameMemory,
        PointerScanAddressSpace::EmulatorMemory => PointerScanAddressSpace::EmulatorMemory,
    }
}

fn resolve_pointer_size_for_process_bitness(
    requested_pointer_size: PointerScanPointerSize,
    address_space: PointerScanAddressSpace,
    process_info: &OpenedProcessInfo,
) -> PointerScanPointerSize {
    if matches!(address_space, PointerScanAddressSpace::GameMemory) {
        return requested_pointer_size;
    }

    let process_pointer_size = PointerScanPointerSize::from_process_bitness(process_info.get_bitness());

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

fn parse_target_address(target_address: &AnonymousValueString) -> Option<u64> {
    let trimmed_target_address = target_address.get_anonymous_value_string().trim();

    if trimmed_target_address.is_empty() {
        return None;
    }

    match target_address.get_anonymous_value_string_format() {
        AnonymousValueStringFormat::Address | AnonymousValueStringFormat::Hexadecimal => {
            let hexadecimal_input = trimmed_target_address
                .strip_prefix("0x")
                .or_else(|| trimmed_target_address.strip_prefix("0X"))
                .unwrap_or(trimmed_target_address);

            u64::from_str_radix(hexadecimal_input, 16).ok()
        }
        AnonymousValueStringFormat::Decimal => trimmed_target_address.parse::<u64>().ok(),
        AnonymousValueStringFormat::Binary => {
            let binary_input = trimmed_target_address
                .strip_prefix("0b")
                .or_else(|| trimmed_target_address.strip_prefix("0B"))
                .unwrap_or(trimmed_target_address);

            u64::from_str_radix(binary_input, 2).ok()
        }
        _ => trimmed_target_address.parse::<u64>().ok(),
    }
}

#[cfg(test)]
mod tests {
    use super::PointerScanStartRequest;
    use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
    use crate::engine_privileged_state::EnginePrivilegedState;
    use crossbeam_channel::{Receiver, unbounded};
    use squalr_engine_api::commands::privileged_command::PrivilegedCommand;
    use squalr_engine_api::commands::privileged_command_response::PrivilegedCommandResponse;
    use squalr_engine_api::engine::engine_api_priviliged_bindings::EngineApiPrivilegedBindings;
    use squalr_engine_api::engine::engine_binding_error::EngineBindingError;
    use squalr_engine_api::events::engine_event::EngineEvent;
    use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
    use squalr_engine_api::structures::data_values::anonymous_value_string::AnonymousValueString;
    use squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
    use squalr_engine_api::structures::data_values::container_type::ContainerType;
    use squalr_engine_api::structures::data_values::data_value::DataValue;
    use squalr_engine_api::structures::memory::bitness::Bitness;
    use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
    use squalr_engine_api::structures::memory::normalized_region::NormalizedRegion;
    use squalr_engine_api::structures::pointer_scans::pointer_scan_address_space::PointerScanAddressSpace;
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

        fn resolve_module_address(
            &self,
            modules: &Vec<NormalizedModule>,
            identifier: &str,
            offset: u64,
        ) -> Option<u64> {
            self.resolve_module(modules, identifier).checked_add(offset)
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
            address_space: PointerScanAddressSpace::EmulatorMemory,
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
            address_space: PointerScanAddressSpace::EmulatorMemory,
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
            address_space: PointerScanAddressSpace::EmulatorMemory,
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

    #[test]
    fn execute_returns_default_when_pointer_scan_memory_regions_are_unavailable() {
        let engine_bindings: Arc<RwLock<dyn EngineApiPrivilegedBindings>> = Arc::new(RwLock::new(TestEngineBindings));
        let os_providers = EngineOsProviders::new(
            Arc::new(TestProcessQueryProvider),
            Arc::new(TestMemoryQueryProvider {
                module_descriptors: vec![(String::from("game.exe"), 0x1000, 0x100)],
                usermode_memory_regions: Vec::new(),
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
                String::from("pointer-test-dead"),
                1,
                Bitness::Bit64,
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
            address_space: PointerScanAddressSpace::EmulatorMemory,
        }
        .execute(&engine_privileged_state);

        assert!(!pointer_scan_start_response.success);
        assert!(pointer_scan_start_response.pointer_scan_summary.is_none());
        assert!(
            engine_privileged_state
                .get_pointer_scan_results()
                .read()
                .expect("Expected the pointer scan results lock.")
                .is_none()
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
