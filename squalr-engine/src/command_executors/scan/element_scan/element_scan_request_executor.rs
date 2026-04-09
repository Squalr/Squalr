use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::command_executors::scan::scan_results_metadata_collector::collect_scan_results_metadata;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::scan::element_scan::element_scan_request::ElementScanRequest;
use squalr_engine_api::commands::scan::element_scan::element_scan_response::ElementScanResponse;
use squalr_engine_api::events::scan_results::updated::scan_results_updated_event::ScanResultsUpdatedEvent;
use squalr_engine_api::registries::scan_rules::element_scan_rule_registry::ElementScanRuleRegistry;
use squalr_engine_api::structures::memory::memory_alignment::MemoryAlignment;
use squalr_engine_api::structures::scanning::constraints::scan_constraint_builder::ScanConstraintBuilder;
use squalr_engine_api::structures::scanning::constraints::scan_constraint_finalized::ScanConstraintFinalized;
use squalr_engine_api::structures::scanning::plans::element_scan::element_scan_plan::ElementScanPlan;
use squalr_engine_scanning::scan_settings_config::ScanSettingsConfig;
use squalr_engine_scanning::scanners::element_scan_executor_task::ElementScanExecutor;
use squalr_engine_scanning::scanners::scan_execution_context::ScanExecutionContext;
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for ElementScanRequest {
    type ResponseType = ElementScanResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        if let Some(process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        {
            let snapshot = engine_privileged_state.get_snapshot();
            let alignment = ScanSettingsConfig::get_memory_alignment().unwrap_or(MemoryAlignment::Alignment1);
            let floating_point_tolerance = ScanSettingsConfig::get_floating_point_tolerance();
            let memory_read_mode = ScanSettingsConfig::get_memory_read_mode();
            let is_single_thread_scan = ScanSettingsConfig::get_is_single_threaded_scan();
            let debug_perform_validation_scan = ScanSettingsConfig::get_debug_perform_validation_scan();

            if debug_perform_validation_scan {
                log::warn!(
                    "Element scan debug validation is enabled; specialized scanners will be verified against scalar iterative scans and runtime cost will increase."
                );
            }

            // Deanonymize all scan constraints against all data types.
            // For example, an immediate comparison of >= 23 could end up being a byte, float, etc.
            let scan_constraints_by_data_type = engine_privileged_state.read_symbol_registry(|symbol_registry| {
                let scan_constraint_builder = ScanConstraintBuilder::new(symbol_registry, floating_point_tolerance);

                self.data_type_refs
                    .iter()
                    .map(|data_type_ref| {
                        // Deanonymize the initial anonymous scan constraints against the current data type.
                        let scan_constraints = self
                            .scan_constraints
                            .iter()
                            .filter_map(
                                |anonymous_scan_constraint| match scan_constraint_builder.build(anonymous_scan_constraint, data_type_ref) {
                                    Ok(scan_constraint) => scan_constraint,
                                    Err(error) => {
                                        log::error!("Unable to create scan constraint: {}", error);
                                        None
                                    }
                                },
                            )
                            .collect();

                        // Optimize the scan constraints by running them through each parameter rule sequentially.
                        let scan_constraints_finalized = ElementScanRuleRegistry::get_instance()
                            .get_scan_parameters_rule_registry()
                            .iter()
                            .fold(scan_constraints, |mut scan_constraint, (_id, scan_parameter_rule)| {
                                scan_parameter_rule.map_parameters(symbol_registry, &mut scan_constraint);
                                scan_constraint
                            })
                            .into_iter()
                            .map(|scan_constraint| ScanConstraintFinalized::new(symbol_registry, scan_constraint))
                            .collect();

                        (data_type_ref.clone(), scan_constraints_finalized)
                    })
                    .collect()
            });

            let element_scan_plan = ElementScanPlan::new(
                scan_constraints_by_data_type,
                alignment,
                floating_point_tolerance,
                memory_read_mode,
                is_single_thread_scan,
                debug_perform_validation_scan,
            );
            let memory_read_provider = engine_privileged_state.get_os_providers().memory_read.clone();
            let scan_execution_context = ScanExecutionContext::new(
                None,
                None,
                Some(Arc::new(move |opened_process_info, address, values| {
                    memory_read_provider.read_bytes(opened_process_info, address, values)
                })),
            );
            engine_privileged_state.read_symbol_registry(|symbol_registry| {
                ElementScanExecutor::execute_scan(
                    process_info,
                    snapshot.clone(),
                    symbol_registry,
                    element_scan_plan,
                    true,
                    &scan_execution_context,
                );
            });

            match snapshot.write() {
                Ok(mut snapshot_guard) => {
                    snapshot_guard.clear_deleted_scan_result_indices();
                }
                Err(error) => {
                    log::error!("Failed to acquire write lock on snapshot to clear deleted scan result indices: {}", error);
                }
            }

            engine_privileged_state.emit_event(ScanResultsUpdatedEvent { is_new_scan: false });

            ElementScanResponse {
                scan_results_metadata: collect_scan_results_metadata(engine_privileged_state),
            }
        } else {
            log::error!("No opened process");
            ElementScanResponse::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ElementScanExecutor;
    use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
    use crate::engine_privileged_state::EnginePrivilegedState;
    use squalr_engine_api::commands::scan::element_scan::element_scan_request::ElementScanRequest;
    use squalr_engine_api::commands::scan::new::scan_new_request::ScanNewRequest;
    use squalr_engine_api::engine::{
        engine_api_priviliged_bindings::EngineApiPrivilegedBindings, engine_binding_error::EngineBindingError, engine_event_envelope::EngineEventEnvelope,
    };
    use squalr_engine_api::events::engine_event::EngineEvent;
    use squalr_engine_api::registries::symbols::symbol_registry::SymbolRegistry;
    use squalr_engine_api::structures::{
        data_types::{data_type_ref::DataTypeRef, floating_point_tolerance::FloatingPointTolerance},
        data_values::{anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType},
        memory::{bitness::Bitness, memory_alignment::MemoryAlignment, normalized_region::NormalizedRegion},
        processes::{opened_process_info::OpenedProcessInfo, process_info::ProcessInfo},
        scanning::{
            comparisons::{
                scan_compare_type::ScanCompareType, scan_compare_type_immediate::ScanCompareTypeImmediate, scan_compare_type_relative::ScanCompareTypeRelative,
            },
            constraints::{anonymous_scan_constraint::AnonymousScanConstraint, scan_constraint_finalized::ScanConstraintFinalized},
            memory_read_mode::MemoryReadMode,
            plans::element_scan::element_scan_plan::ElementScanPlan,
        },
    };
    use squalr_engine_session::os::engine_os_provider::{
        EngineOsProviders, MemoryQueryProvider, MemoryReadProvider, MemoryWriteProvider, ProcessQueryProvider,
    };
    use squalr_engine_session::os::{PageRetrievalMode, ProcessQueryError, ProcessQueryOptions};
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};

    const TEST_REGION_BASE_ADDRESS: u64 = 0x5000;
    const TEST_REGION_SIZE: u64 = 0x10;
    const TEST_MATCH_ADDRESS: u64 = TEST_REGION_BASE_ADDRESS + 0x4;

    struct NoOpEngineBindings;

    impl EngineApiPrivilegedBindings for NoOpEngineBindings {
        fn emit_event(
            &self,
            _event: EngineEvent,
        ) -> Result<(), EngineBindingError> {
            Ok(())
        }

        fn dispatch_internal_command(
            &self,
            _engine_command: squalr_engine_api::commands::privileged_command::PrivilegedCommand,
            _callback: Box<dyn FnOnce(squalr_engine_api::commands::privileged_command_response::PrivilegedCommandResponse) + Send + Sync + 'static>,
        ) -> Result<(), EngineBindingError> {
            Err(EngineBindingError::unavailable("dispatching internal commands in element scan tests"))
        }

        fn subscribe_to_engine_events(&self) -> Result<crossbeam_channel::Receiver<EngineEventEnvelope>, EngineBindingError> {
            let (_sender, receiver) = crossbeam_channel::unbounded();

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
            vec![]
        }

        fn open_process(
            &self,
            _process_info: &ProcessInfo,
        ) -> Result<OpenedProcessInfo, ProcessQueryError> {
            Err(ProcessQueryError::internal("open_process", "not used in element scan tests"))
        }

        fn close_process(
            &self,
            _handle: u64,
        ) -> Result<(), ProcessQueryError> {
            Ok(())
        }
    }

    struct TestMemoryQueryProvider;

    impl MemoryQueryProvider for TestMemoryQueryProvider {
        fn get_modules(
            &self,
            _process_info: &OpenedProcessInfo,
        ) -> Vec<squalr_engine_api::structures::memory::normalized_module::NormalizedModule> {
            vec![]
        }

        fn address_to_module(
            &self,
            _address: u64,
            _modules: &Vec<squalr_engine_api::structures::memory::normalized_module::NormalizedModule>,
        ) -> Option<(String, u64)> {
            None
        }

        fn resolve_module(
            &self,
            _modules: &Vec<squalr_engine_api::structures::memory::normalized_module::NormalizedModule>,
            _identifier: &str,
        ) -> u64 {
            0
        }

        fn get_memory_page_bounds(
            &self,
            _process_info: &OpenedProcessInfo,
            _page_retrieval_mode: PageRetrievalMode,
        ) -> Vec<NormalizedRegion> {
            vec![NormalizedRegion::new(
                TEST_REGION_BASE_ADDRESS,
                TEST_REGION_SIZE,
            )]
        }
    }

    struct TestMemoryReadProvider {
        bytes: Arc<RwLock<Vec<u8>>>,
    }

    impl MemoryReadProvider for TestMemoryReadProvider {
        fn read(
            &self,
            _process_info: &OpenedProcessInfo,
            _address: u64,
            _data_value: &mut squalr_engine_api::structures::data_values::data_value::DataValue,
        ) -> bool {
            false
        }

        fn read_struct(
            &self,
            _process_info: &OpenedProcessInfo,
            _address: u64,
            _valued_struct: &mut squalr_engine_api::structures::structs::valued_struct::ValuedStruct,
        ) -> bool {
            false
        }

        fn read_bytes(
            &self,
            _process_info: &OpenedProcessInfo,
            address: u64,
            values: &mut [u8],
        ) -> bool {
            let Ok(bytes) = self.bytes.read() else {
                return false;
            };

            let Some(start_offset) = address.checked_sub(TEST_REGION_BASE_ADDRESS) else {
                return false;
            };
            let start_offset = start_offset as usize;
            let end_offset = start_offset.saturating_add(values.len());

            if end_offset > bytes.len() {
                return false;
            }

            values.copy_from_slice(&bytes[start_offset..end_offset]);
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
            true
        }
    }

    fn create_test_engine_privileged_state(memory_bytes: Arc<RwLock<Vec<u8>>>) -> Arc<EnginePrivilegedState> {
        let engine_os_providers = EngineOsProviders::new(
            Arc::new(TestProcessQueryProvider),
            Arc::new(TestMemoryQueryProvider),
            Arc::new(TestMemoryReadProvider { bytes: memory_bytes }),
            Arc::new(TestMemoryWriteProvider),
        );
        let engine_bindings = Arc::new(RwLock::new(NoOpEngineBindings));
        let engine_privileged_state = squalr_engine_session::engine_privileged_state::EnginePrivilegedState::new(engine_bindings, engine_os_providers)
            .expect("Expected test engine privileged state to initialize.");

        engine_privileged_state
            .get_process_manager()
            .set_opened_process(OpenedProcessInfo::new(1, String::from("scan-target.exe"), 1, Bitness::Bit64, None));

        let _ = ScanNewRequest {}.execute(&engine_privileged_state);

        engine_privileged_state
    }

    fn build_i24_exact_scan_plan(
        symbol_registry: &SymbolRegistry,
        value: i32,
        memory_alignment: MemoryAlignment,
    ) -> ElementScanPlan {
        let data_type_ref = DataTypeRef::new("i24");
        let anonymous_scan_constraint = AnonymousScanConstraint::new(
            ScanCompareType::Immediate(ScanCompareTypeImmediate::Equal),
            Some(AnonymousValueString::new(
                value.to_string(),
                AnonymousValueStringFormat::Decimal,
                ContainerType::None,
            )),
        );
        let scan_constraint = anonymous_scan_constraint
            .deanonymize_constraint(symbol_registry, &data_type_ref, FloatingPointTolerance::default())
            .expect("Expected i24 scan constraint to deanonymize.");
        let scan_constraint_finalized = ScanConstraintFinalized::new(symbol_registry, scan_constraint);
        let mut scan_constraints_by_data_type = HashMap::new();
        scan_constraints_by_data_type.insert(data_type_ref, vec![scan_constraint_finalized]);

        ElementScanPlan::new(
            scan_constraints_by_data_type,
            memory_alignment,
            FloatingPointTolerance::default(),
            MemoryReadMode::ReadBeforeScan,
            true,
            false,
        )
    }

    fn build_i24_relative_scan_plan(
        symbol_registry: &SymbolRegistry,
        scan_compare_type_relative: ScanCompareTypeRelative,
        memory_alignment: MemoryAlignment,
    ) -> ElementScanPlan {
        let data_type_ref = DataTypeRef::new("i24");
        let anonymous_scan_constraint = AnonymousScanConstraint::new(
            ScanCompareType::Relative(scan_compare_type_relative),
            Some(AnonymousValueString::new(
                String::from("0"),
                AnonymousValueStringFormat::Decimal,
                ContainerType::None,
            )),
        );
        let scan_constraint = anonymous_scan_constraint
            .deanonymize_constraint(symbol_registry, &data_type_ref, FloatingPointTolerance::default())
            .expect("Expected i24 relative scan constraint to deanonymize.");
        let scan_constraint_finalized = ScanConstraintFinalized::new(symbol_registry, scan_constraint);
        let mut scan_constraints_by_data_type = HashMap::new();
        scan_constraints_by_data_type.insert(data_type_ref, vec![scan_constraint_finalized]);

        ElementScanPlan::new(
            scan_constraints_by_data_type,
            memory_alignment,
            FloatingPointTolerance::default(),
            MemoryReadMode::ReadBeforeScan,
            true,
            false,
        )
    }

    fn execute_scan_plan(
        engine_privileged_state: &Arc<EnginePrivilegedState>,
        element_scan_plan: ElementScanPlan,
    ) {
        let process_info = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
            .expect("Expected opened process in scan test.");
        let snapshot = engine_privileged_state.get_snapshot();
        let memory_read_provider = engine_privileged_state.get_os_providers().memory_read.clone();
        let scan_execution_context = squalr_engine_scanning::scanners::scan_execution_context::ScanExecutionContext::new(
            None,
            None,
            Some(Arc::new(move |opened_process_info, address, values| {
                memory_read_provider.read_bytes(opened_process_info, address, values)
            })),
        );

        engine_privileged_state.read_symbol_registry(|symbol_registry| {
            ElementScanExecutor::execute_scan(
                process_info,
                snapshot.clone(),
                symbol_registry,
                element_scan_plan,
                false,
                &scan_execution_context,
            );
        });
    }

    fn get_first_result_address(engine_privileged_state: &Arc<EnginePrivilegedState>) -> Option<u64> {
        let snapshot = engine_privileged_state.get_snapshot();
        let Ok(snapshot) = snapshot.read() else {
            return None;
        };

        let result_address: Option<u64> = engine_privileged_state.read_symbol_registry(|symbol_registry| {
            snapshot
                .get_scan_result(symbol_registry, 0)
                .map(|scan_result| scan_result.get_address())
        });

        result_address
    }

    fn write_i32_value(
        memory_bytes: &Arc<RwLock<Vec<u8>>>,
        value: i32,
    ) {
        let mut bytes = memory_bytes.write().expect("Expected memory bytes write lock.");
        let start_offset = (TEST_MATCH_ADDRESS - TEST_REGION_BASE_ADDRESS) as usize;
        let encoded_value = value.to_le_bytes();

        bytes[start_offset..start_offset + encoded_value.len()].copy_from_slice(&encoded_value);
    }

    fn write_i32_array_value(
        memory_bytes: &Arc<RwLock<Vec<u8>>>,
        start_address: u64,
        values: &[i32],
    ) {
        let mut bytes = memory_bytes.write().expect("Expected memory bytes write lock.");
        let start_offset = (start_address - TEST_REGION_BASE_ADDRESS) as usize;
        let encoded_values: Vec<u8> = values.iter().flat_map(|value| value.to_le_bytes()).collect();
        let end_offset = start_offset + encoded_values.len();

        bytes[start_offset..end_offset].copy_from_slice(&encoded_values);
    }

    fn write_region_bytes(
        memory_bytes: &Arc<RwLock<Vec<u8>>>,
        region_bytes: &[u8],
    ) {
        let mut bytes = memory_bytes.write().expect("Expected memory bytes write lock.");

        bytes[..region_bytes.len()].copy_from_slice(region_bytes);
    }

    #[test]
    fn i24_exact_rescan_preserves_match_with_alignment_1() {
        let memory_bytes = Arc::new(RwLock::new(vec![0u8; TEST_REGION_SIZE as usize]));
        let engine_privileged_state = create_test_engine_privileged_state(memory_bytes.clone());

        write_i32_value(&memory_bytes, 3);
        engine_privileged_state.read_symbol_registry(|symbol_registry| {
            execute_scan_plan(
                &engine_privileged_state,
                build_i24_exact_scan_plan(symbol_registry, 3, MemoryAlignment::Alignment1),
            );
        });

        assert_eq!(
            engine_privileged_state
                .get_snapshot()
                .read()
                .expect("Expected snapshot read lock.")
                .get_number_of_results(),
            1
        );
        assert_eq!(get_first_result_address(&engine_privileged_state), Some(TEST_MATCH_ADDRESS));

        write_i32_value(&memory_bytes, 2);
        engine_privileged_state.read_symbol_registry(|symbol_registry| {
            execute_scan_plan(
                &engine_privileged_state,
                build_i24_exact_scan_plan(symbol_registry, 2, MemoryAlignment::Alignment1),
            );
        });

        assert_eq!(
            engine_privileged_state
                .get_snapshot()
                .read()
                .expect("Expected snapshot read lock.")
                .get_number_of_results(),
            1
        );
        assert_eq!(get_first_result_address(&engine_privileged_state), Some(TEST_MATCH_ADDRESS));
    }

    #[test]
    fn i24_exact_rescan_preserves_match_with_alignment_4() {
        let memory_bytes = Arc::new(RwLock::new(vec![0u8; TEST_REGION_SIZE as usize]));
        let engine_privileged_state = create_test_engine_privileged_state(memory_bytes.clone());

        write_i32_value(&memory_bytes, 3);
        engine_privileged_state.read_symbol_registry(|symbol_registry| {
            execute_scan_plan(
                &engine_privileged_state,
                build_i24_exact_scan_plan(symbol_registry, 3, MemoryAlignment::Alignment4),
            );
        });

        assert_eq!(
            engine_privileged_state
                .get_snapshot()
                .read()
                .expect("Expected snapshot read lock.")
                .get_number_of_results(),
            1
        );
        assert_eq!(get_first_result_address(&engine_privileged_state), Some(TEST_MATCH_ADDRESS));

        write_i32_value(&memory_bytes, 2);
        engine_privileged_state.read_symbol_registry(|symbol_registry| {
            execute_scan_plan(
                &engine_privileged_state,
                build_i24_exact_scan_plan(symbol_registry, 2, MemoryAlignment::Alignment4),
            );
        });

        assert_eq!(
            engine_privileged_state
                .get_snapshot()
                .read()
                .expect("Expected snapshot read lock.")
                .get_number_of_results(),
            1
        );
        assert_eq!(get_first_result_address(&engine_privileged_state), Some(TEST_MATCH_ADDRESS));
    }

    #[test]
    fn i24_exact_scan_finds_match_after_partial_suffix_mismatch() {
        let memory_bytes = Arc::new(RwLock::new(vec![0u8; TEST_REGION_SIZE as usize]));
        let engine_privileged_state = create_test_engine_privileged_state(memory_bytes.clone());

        write_region_bytes(&memory_bytes, &[0u8, 3u8, 0u8, 0u8, 9u8, 9u8]);

        engine_privileged_state.read_symbol_registry(|symbol_registry| {
            execute_scan_plan(
                &engine_privileged_state,
                build_i24_exact_scan_plan(symbol_registry, 3, MemoryAlignment::Alignment1),
            );
        });

        assert_eq!(
            engine_privileged_state
                .get_snapshot()
                .read()
                .expect("Expected snapshot read lock.")
                .get_number_of_results(),
            1
        );
        assert_eq!(get_first_result_address(&engine_privileged_state), Some(TEST_REGION_BASE_ADDRESS + 1));
    }

    #[test]
    fn i24_relative_decreased_preserves_match_with_alignment_1() {
        let memory_bytes = Arc::new(RwLock::new(vec![0u8; TEST_REGION_SIZE as usize]));
        let engine_privileged_state = create_test_engine_privileged_state(memory_bytes.clone());

        write_i32_value(&memory_bytes, 3);
        engine_privileged_state.read_symbol_registry(|symbol_registry| {
            execute_scan_plan(
                &engine_privileged_state,
                build_i24_exact_scan_plan(symbol_registry, 3, MemoryAlignment::Alignment1),
            );
        });

        write_i32_value(&memory_bytes, 2);
        engine_privileged_state.read_symbol_registry(|symbol_registry| {
            execute_scan_plan(
                &engine_privileged_state,
                build_i24_relative_scan_plan(symbol_registry, ScanCompareTypeRelative::Decreased, MemoryAlignment::Alignment1),
            );
        });

        assert_eq!(
            engine_privileged_state
                .get_snapshot()
                .read()
                .expect("Expected snapshot read lock.")
                .get_number_of_results(),
            1
        );
        assert_eq!(get_first_result_address(&engine_privileged_state), Some(TEST_MATCH_ADDRESS));
    }

    #[test]
    fn i24_relative_increased_preserves_match_with_alignment_1() {
        let memory_bytes = Arc::new(RwLock::new(vec![0u8; TEST_REGION_SIZE as usize]));
        let engine_privileged_state = create_test_engine_privileged_state(memory_bytes.clone());

        write_i32_value(&memory_bytes, 2);
        engine_privileged_state.read_symbol_registry(|symbol_registry| {
            execute_scan_plan(
                &engine_privileged_state,
                build_i24_exact_scan_plan(symbol_registry, 2, MemoryAlignment::Alignment1),
            );
        });

        write_i32_value(&memory_bytes, 3);
        engine_privileged_state.read_symbol_registry(|symbol_registry| {
            execute_scan_plan(
                &engine_privileged_state,
                build_i24_relative_scan_plan(symbol_registry, ScanCompareTypeRelative::Increased, MemoryAlignment::Alignment1),
            );
        });

        assert_eq!(
            engine_privileged_state
                .get_snapshot()
                .read()
                .expect("Expected snapshot read lock.")
                .get_number_of_results(),
            1
        );
        assert_eq!(get_first_result_address(&engine_privileged_state), Some(TEST_MATCH_ADDRESS));
    }

    #[test]
    fn element_scan_request_finds_i32_array_matches() {
        let memory_bytes = Arc::new(RwLock::new(vec![0u8; TEST_REGION_SIZE as usize]));
        let engine_privileged_state = create_test_engine_privileged_state(memory_bytes.clone());
        let match_address = TEST_REGION_BASE_ADDRESS;
        let element_scan_request = ElementScanRequest {
            scan_constraints: vec![AnonymousScanConstraint::new(
                ScanCompareType::Immediate(ScanCompareTypeImmediate::Equal),
                Some(AnonymousValueString::new(
                    String::from("1, 2"),
                    AnonymousValueStringFormat::Decimal,
                    ContainerType::Array,
                )),
            )],
            data_type_refs: vec![DataTypeRef::new("i32")],
        };

        write_i32_array_value(&memory_bytes, match_address, &[1, 2]);

        let _ = element_scan_request.execute(&engine_privileged_state);

        let snapshot = engine_privileged_state.get_snapshot();
        let snapshot_guard = snapshot.read().expect("Expected snapshot read lock.");

        assert_eq!(snapshot_guard.get_number_of_results(), 1);
        engine_privileged_state.read_symbol_registry(|symbol_registry| {
            let scan_result = snapshot_guard
                .get_scan_result(symbol_registry, 0)
                .expect("Expected an i32 array scan result.");
            let decimal_display_value = scan_result
                .get_current_display_value(AnonymousValueStringFormat::Decimal)
                .expect("Expected decimal display value for i32 array scan result.");

            assert_eq!(scan_result.get_address(), match_address);
            assert_eq!(decimal_display_value.get_anonymous_value_string(), "1, 2");
            assert_eq!(decimal_display_value.get_container_type(), ContainerType::ArrayFixed(2));
        });
    }
}
