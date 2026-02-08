use squalr_engine::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use squalr_engine::engine_mode::EngineMode;
use squalr_engine::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::memory::read::memory_read_request::MemoryReadRequest;
use squalr_engine_api::commands::memory::write::memory_write_request::MemoryWriteRequest;
use squalr_engine_api::commands::process::close::process_close_request::ProcessCloseRequest;
use squalr_engine_api::commands::process::list::process_list_request::ProcessListRequest;
use squalr_engine_api::commands::process::open::process_open_request::ProcessOpenRequest;
use squalr_engine_api::commands::scan::new::scan_new_request::ScanNewRequest;
use squalr_engine_api::commands::scan_results::freeze::scan_results_freeze_request::ScanResultsFreezeRequest;
use squalr_engine_api::commands::scan_results::list::scan_results_list_request::ScanResultsListRequest;
use squalr_engine_api::commands::scan_results::query::scan_results_query_request::ScanResultsQueryRequest;
use squalr_engine_api::commands::scan_results::refresh::scan_results_refresh_request::ScanResultsRefreshRequest;
use squalr_engine_api::commands::scan_results::set_property::scan_results_set_property_request::ScanResultsSetPropertyRequest;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_values::anonymous_value_string::AnonymousValueString;
use squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
use squalr_engine_api::structures::data_values::container_type::ContainerType;
use squalr_engine_api::structures::memory::bitness::Bitness;
use squalr_engine_api::structures::memory::memory_alignment::MemoryAlignment;
use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
use squalr_engine_api::structures::memory::normalized_region::NormalizedRegion;
use squalr_engine_api::structures::memory::pointer::Pointer;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::processes::process_info::ProcessInfo;
use squalr_engine_api::structures::results::snapshot_region_scan_results::SnapshotRegionScanResults;
use squalr_engine_api::structures::scan_results::scan_result::ScanResult;
use squalr_engine_api::structures::scan_results::scan_result_ref::ScanResultRef;
use squalr_engine_api::structures::scanning::filters::snapshot_region_filter::SnapshotRegionFilter;
use squalr_engine_api::structures::scanning::filters::snapshot_region_filter_collection::SnapshotRegionFilterCollection;
use squalr_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_api::structures::structs::symbolic_struct_definition::SymbolicStructDefinition;
use squalr_tests::mocks::mock_os::MockEngineOs;

fn create_test_state() -> (MockEngineOs, std::sync::Arc<EnginePrivilegedState>) {
    let mock_engine_os = MockEngineOs::new();
    let engine_os_providers = mock_engine_os.create_providers();
    let engine_privileged_state = EnginePrivilegedState::new_with_os_providers(EngineMode::Standalone, engine_os_providers);

    (mock_engine_os, engine_privileged_state)
}

fn create_opened_process_info() -> OpenedProcessInfo {
    OpenedProcessInfo::new(std::process::id(), "test-process.exe".to_string(), 0xABC0, Bitness::Bit64, None)
}

fn seed_snapshot_with_single_scan_result(
    engine_privileged_state: &std::sync::Arc<EnginePrivilegedState>,
    result_address: u64,
) {
    let region_base = result_address & !0xFF;
    let mut snapshot_region = SnapshotRegion::new(NormalizedRegion::new(region_base, 0x100), Vec::new());
    snapshot_region.current_values = vec![0u8; 0x100];
    snapshot_region.previous_values = vec![0u8; 0x100];

    let snapshot_filter = SnapshotRegionFilter::new(result_address, 4);
    let snapshot_filter_collection = SnapshotRegionFilterCollection::new(vec![vec![snapshot_filter]], DataTypeRef::new("u32"), MemoryAlignment::Alignment1);
    snapshot_region.set_scan_results(SnapshotRegionScanResults::new(vec![snapshot_filter_collection]));

    let snapshot_ref = engine_privileged_state.get_snapshot();
    if let Ok(mut snapshot_guard) = snapshot_ref.write() {
        snapshot_guard.set_snapshot_regions(vec![snapshot_region]);
    }
}

#[test]
fn memory_write_executor_uses_injected_module_resolution_and_writer() {
    let (mock_engine_os, engine_privileged_state) = create_test_state();
    mock_engine_os.set_modules(vec![NormalizedModule::new("game.exe", 0x1000, 0x2000)]);
    engine_privileged_state
        .get_process_manager()
        .set_opened_process(create_opened_process_info());

    let memory_write_request = MemoryWriteRequest {
        address: 0x20,
        module_name: "game.exe".to_string(),
        value: vec![1, 2, 3, 4],
    };

    let memory_write_response = memory_write_request.execute(&engine_privileged_state);
    assert!(memory_write_response.success);

    let mock_os_state = mock_engine_os.get_state();
    let state_guard = match mock_os_state.lock() {
        Ok(state_guard) => state_guard,
        Err(error) => panic!("failed to lock mock state: {}", error),
    };
    assert_eq!(state_guard.memory_write_requests.len(), 1);
    assert_eq!(state_guard.memory_write_requests[0].0, 0x1020);
    assert_eq!(state_guard.memory_write_requests[0].1, vec![1, 2, 3, 4]);
}

#[test]
fn memory_read_executor_uses_injected_module_resolution_and_reader() {
    let (mock_engine_os, engine_privileged_state) = create_test_state();
    mock_engine_os.set_modules(vec![NormalizedModule::new("game.exe", 0x7000, 0x1000)]);
    engine_privileged_state
        .get_process_manager()
        .set_opened_process(create_opened_process_info());

    let memory_read_request = MemoryReadRequest {
        address: 0x10,
        module_name: "game.exe".to_string(),
        symbolic_struct_definition: SymbolicStructDefinition::new(String::new(), vec![]),
    };

    let memory_read_response = memory_read_request.execute(&engine_privileged_state);
    assert!(memory_read_response.success);

    let mock_os_state = mock_engine_os.get_state();
    let state_guard = match mock_os_state.lock() {
        Ok(state_guard) => state_guard,
        Err(error) => panic!("failed to lock mock state: {}", error),
    };
    assert_eq!(state_guard.memory_struct_read_addresses.len(), 1);
    assert_eq!(state_guard.memory_struct_read_addresses[0], 0x7010);
}

#[test]
fn memory_read_executor_returns_failure_without_mutating_write_state_when_reader_fails() {
    let (mock_engine_os, engine_privileged_state) = create_test_state();
    mock_engine_os.set_modules(vec![NormalizedModule::new("game.exe", 0x7000, 0x1000)]);
    mock_engine_os.set_read_success(false);
    engine_privileged_state
        .get_process_manager()
        .set_opened_process(create_opened_process_info());

    let memory_read_request = MemoryReadRequest {
        address: 0x44,
        module_name: "game.exe".to_string(),
        symbolic_struct_definition: SymbolicStructDefinition::new(String::new(), vec![]),
    };

    let memory_read_response = memory_read_request.execute(&engine_privileged_state);
    assert!(!memory_read_response.success);
    assert_eq!(memory_read_response.address, 0x44);

    let mock_os_state = mock_engine_os.get_state();
    let state_guard = match mock_os_state.lock() {
        Ok(state_guard) => state_guard,
        Err(error) => panic!("failed to lock mock state: {}", error),
    };
    assert_eq!(state_guard.memory_struct_read_addresses, vec![0x7044]);
    assert!(state_guard.memory_write_requests.is_empty());
}

#[test]
fn process_executors_use_injected_process_provider() {
    let (mock_engine_os, engine_privileged_state) = create_test_state();
    let process_identifier = std::process::id();
    let process_info = ProcessInfo::new(process_identifier, "calc.exe".to_string(), true, None);

    mock_engine_os.set_processes(vec![process_info.clone()]);
    mock_engine_os.set_opened_process_result(Some(OpenedProcessInfo::new(
        process_identifier,
        "calc.exe".to_string(),
        0xBEEF,
        Bitness::Bit64,
        None,
    )));

    let process_list_request = ProcessListRequest {
        require_windowed: true,
        search_name: Some("calc".to_string()),
        match_case: true,
        limit: Some(5),
        fetch_icons: false,
    };
    let process_list_response = process_list_request.execute(&engine_privileged_state);
    assert_eq!(process_list_response.processes.len(), 1);
    assert_eq!(process_list_response.processes[0].get_name(), "calc.exe");

    let process_open_request = ProcessOpenRequest {
        process_id: Some(process_identifier),
        search_name: Some("calc".to_string()),
        match_case: true,
    };
    let process_open_response = process_open_request.execute(&engine_privileged_state);
    assert!(process_open_response.opened_process_info.is_some());

    let process_close_response = ProcessCloseRequest {}.execute(&engine_privileged_state);
    assert!(process_close_response.process_info.is_some());

    let mock_os_state = mock_engine_os.get_state();
    let state_guard = match mock_os_state.lock() {
        Ok(state_guard) => state_guard,
        Err(error) => panic!("failed to lock mock state: {}", error),
    };
    assert_eq!(state_guard.process_query_requests.len(), 2);
    assert_eq!(state_guard.process_query_requests[0].search_name, Some("calc".to_string()));
    assert!(state_guard.process_query_requests[0].require_windowed);
    assert!(state_guard.process_query_requests[0].match_case);
    assert_eq!(state_guard.process_query_requests[0].limit, Some(5));
    assert!(!state_guard.process_query_requests[0].fetch_icons);
    assert_eq!(state_guard.process_query_requests[1].required_process_id, Some(process_identifier));
    assert_eq!(state_guard.open_process_requests, vec![process_identifier]);
    assert_eq!(state_guard.close_process_handles, vec![0xBEEF]);
}

#[test]
fn process_open_executor_returns_none_when_process_handle_resolution_fails() {
    let (mock_engine_os, engine_privileged_state) = create_test_state();
    let process_identifier = std::process::id();
    let process_info = ProcessInfo::new(process_identifier, "calc.exe".to_string(), true, None);

    mock_engine_os.set_processes(vec![process_info]);
    mock_engine_os.set_opened_process_result(None);

    let process_open_request = ProcessOpenRequest {
        process_id: Some(process_identifier),
        search_name: Some("calc".to_string()),
        match_case: true,
    };
    let process_open_response = process_open_request.execute(&engine_privileged_state);
    assert!(process_open_response.opened_process_info.is_none());
    assert!(
        engine_privileged_state
            .get_process_manager()
            .get_opened_process()
            .is_none()
    );

    let mock_os_state = mock_engine_os.get_state();
    let state_guard = match mock_os_state.lock() {
        Ok(state_guard) => state_guard,
        Err(error) => panic!("failed to lock mock state: {}", error),
    };
    assert_eq!(state_guard.open_process_requests, vec![process_identifier]);
}

#[test]
fn scan_new_executor_uses_injected_memory_page_bounds() {
    let (mock_engine_os, engine_privileged_state) = create_test_state();
    mock_engine_os.set_memory_pages(vec![
        NormalizedRegion::new(0x1000, 0x1000),
        NormalizedRegion::new(0x2000, 0x1000),
        NormalizedRegion::new(0x5000, 0x1000),
    ]);
    engine_privileged_state
        .get_process_manager()
        .set_opened_process(create_opened_process_info());

    let _scan_new_response = ScanNewRequest {}.execute(&engine_privileged_state);

    let snapshot_ref = engine_privileged_state.get_snapshot();
    let snapshot_guard = match snapshot_ref.read() {
        Ok(snapshot_guard) => snapshot_guard,
        Err(error) => panic!("failed to lock snapshot for read: {}", error),
    };
    let snapshot_regions = snapshot_guard.get_snapshot_regions();

    assert_eq!(snapshot_regions.len(), 2);
    assert_eq!(snapshot_regions[0].get_base_address(), 0x1000);
    assert_eq!(snapshot_regions[0].get_region_size(), 0x2000);
    assert_eq!(snapshot_regions[0].page_boundaries, vec![0x2000]);
    assert_eq!(snapshot_regions[1].get_base_address(), 0x5000);
    assert_eq!(snapshot_regions[1].get_region_size(), 0x1000);
}

#[test]
fn scan_results_list_executor_uses_injected_providers() {
    let (mock_engine_os, engine_privileged_state) = create_test_state();
    mock_engine_os.set_modules(vec![NormalizedModule::new("game.exe", 0x1000, 0x1000)]);
    engine_privileged_state
        .get_process_manager()
        .set_opened_process(create_opened_process_info());
    seed_snapshot_with_single_scan_result(&engine_privileged_state, 0x1010);

    let scan_results_list_response = ScanResultsListRequest { page_index: 0 }.execute(&engine_privileged_state);

    assert_eq!(scan_results_list_response.scan_results.len(), 1);
    assert_eq!(scan_results_list_response.scan_results[0].get_module(), "game.exe");
    assert_eq!(scan_results_list_response.scan_results[0].get_module_offset(), 0x10);

    let mock_os_state = mock_engine_os.get_state();
    let state_guard = match mock_os_state.lock() {
        Ok(state_guard) => state_guard,
        Err(error) => panic!("failed to lock mock state: {}", error),
    };
    assert_eq!(state_guard.memory_read_addresses, vec![0x1010]);
}

#[test]
fn scan_results_list_executor_handles_read_failure_without_incorrect_value_mutation() {
    let (mock_engine_os, engine_privileged_state) = create_test_state();
    mock_engine_os.set_modules(vec![NormalizedModule::new("game.exe", 0x1000, 0x1000)]);
    mock_engine_os.set_read_success(false);
    engine_privileged_state
        .get_process_manager()
        .set_opened_process(create_opened_process_info());
    seed_snapshot_with_single_scan_result(&engine_privileged_state, 0x1010);

    let scan_results_list_response = ScanResultsListRequest { page_index: 0 }.execute(&engine_privileged_state);

    assert_eq!(scan_results_list_response.scan_results.len(), 1);
    assert_eq!(scan_results_list_response.scan_results[0].get_module(), "game.exe");
    assert_eq!(scan_results_list_response.scan_results[0].get_module_offset(), 0x10);
    assert!(
        scan_results_list_response.scan_results[0]
            .get_recently_read_value()
            .is_none()
    );
    assert!(
        scan_results_list_response.scan_results[0]
            .get_recently_read_display_values()
            .is_empty()
    );

    let mock_os_state = mock_engine_os.get_state();
    let state_guard = match mock_os_state.lock() {
        Ok(state_guard) => state_guard,
        Err(error) => panic!("failed to lock mock state: {}", error),
    };
    assert_eq!(state_guard.memory_read_addresses, vec![0x1010]);
}

#[test]
fn scan_results_query_executor_uses_injected_providers() {
    let (mock_engine_os, engine_privileged_state) = create_test_state();
    mock_engine_os.set_modules(vec![NormalizedModule::new("engine.dll", 0x4000, 0x1000)]);
    engine_privileged_state
        .get_process_manager()
        .set_opened_process(create_opened_process_info());
    seed_snapshot_with_single_scan_result(&engine_privileged_state, 0x4014);

    let scan_results_query_response = ScanResultsQueryRequest { page_index: 0 }.execute(&engine_privileged_state);

    assert_eq!(scan_results_query_response.scan_results.len(), 1);
    assert_eq!(scan_results_query_response.scan_results[0].get_module(), "engine.dll");
    assert_eq!(scan_results_query_response.scan_results[0].get_module_offset(), 0x14);

    let mock_os_state = mock_engine_os.get_state();
    let state_guard = match mock_os_state.lock() {
        Ok(state_guard) => state_guard,
        Err(error) => panic!("failed to lock mock state: {}", error),
    };
    assert_eq!(state_guard.memory_read_addresses, vec![0x4014]);
}

#[test]
fn scan_results_query_executor_handles_read_failure_without_incorrect_value_mutation() {
    let (mock_engine_os, engine_privileged_state) = create_test_state();
    mock_engine_os.set_modules(vec![NormalizedModule::new("engine.dll", 0x4000, 0x1000)]);
    mock_engine_os.set_read_success(false);
    engine_privileged_state
        .get_process_manager()
        .set_opened_process(create_opened_process_info());
    seed_snapshot_with_single_scan_result(&engine_privileged_state, 0x4014);

    let scan_results_query_response = ScanResultsQueryRequest { page_index: 0 }.execute(&engine_privileged_state);

    assert_eq!(scan_results_query_response.scan_results.len(), 1);
    assert_eq!(scan_results_query_response.scan_results[0].get_module(), "engine.dll");
    assert_eq!(scan_results_query_response.scan_results[0].get_module_offset(), 0x14);
    assert!(
        scan_results_query_response.scan_results[0]
            .get_recently_read_value()
            .is_none()
    );
    assert!(
        scan_results_query_response.scan_results[0]
            .get_recently_read_display_values()
            .is_empty()
    );

    let mock_os_state = mock_engine_os.get_state();
    let state_guard = match mock_os_state.lock() {
        Ok(state_guard) => state_guard,
        Err(error) => panic!("failed to lock mock state: {}", error),
    };
    assert_eq!(state_guard.memory_read_addresses, vec![0x4014]);
}

#[test]
fn scan_results_refresh_executor_uses_injected_providers() {
    let (mock_engine_os, engine_privileged_state) = create_test_state();
    mock_engine_os.set_modules(vec![NormalizedModule::new("refresh.exe", 0x6000, 0x1000)]);
    engine_privileged_state
        .get_process_manager()
        .set_opened_process(create_opened_process_info());
    seed_snapshot_with_single_scan_result(&engine_privileged_state, 0x6020);

    let scan_results_refresh_response = ScanResultsRefreshRequest {
        scan_result_refs: vec![ScanResultRef::new(0)],
    }
    .execute(&engine_privileged_state);

    assert_eq!(scan_results_refresh_response.scan_results.len(), 1);
    assert_eq!(scan_results_refresh_response.scan_results[0].get_module(), "refresh.exe");
    assert_eq!(scan_results_refresh_response.scan_results[0].get_module_offset(), 0x20);

    let mock_os_state = mock_engine_os.get_state();
    let state_guard = match mock_os_state.lock() {
        Ok(state_guard) => state_guard,
        Err(error) => panic!("failed to lock mock state: {}", error),
    };
    assert_eq!(state_guard.memory_read_addresses, vec![0x6020]);
}

#[test]
fn scan_results_refresh_executor_handles_read_failure_without_incorrect_value_mutation() {
    let (mock_engine_os, engine_privileged_state) = create_test_state();
    mock_engine_os.set_modules(vec![NormalizedModule::new("refresh.exe", 0x6000, 0x1000)]);
    mock_engine_os.set_read_success(false);
    engine_privileged_state
        .get_process_manager()
        .set_opened_process(create_opened_process_info());
    seed_snapshot_with_single_scan_result(&engine_privileged_state, 0x6020);

    let scan_results_refresh_response = ScanResultsRefreshRequest {
        scan_result_refs: vec![ScanResultRef::new(0)],
    }
    .execute(&engine_privileged_state);

    assert_eq!(scan_results_refresh_response.scan_results.len(), 1);
    assert_eq!(scan_results_refresh_response.scan_results[0].get_module(), "refresh.exe");
    assert_eq!(scan_results_refresh_response.scan_results[0].get_module_offset(), 0x20);
    assert!(
        scan_results_refresh_response.scan_results[0]
            .get_recently_read_value()
            .is_none()
    );
    assert!(
        scan_results_refresh_response.scan_results[0]
            .get_recently_read_display_values()
            .is_empty()
    );

    let mock_os_state = mock_engine_os.get_state();
    let state_guard = match mock_os_state.lock() {
        Ok(state_guard) => state_guard,
        Err(error) => panic!("failed to lock mock state: {}", error),
    };
    assert_eq!(state_guard.memory_read_addresses, vec![0x6020]);
}

#[test]
fn scan_results_freeze_executor_uses_injected_providers() {
    let (mock_engine_os, engine_privileged_state) = create_test_state();
    mock_engine_os.set_modules(vec![NormalizedModule::new("freeze.exe", 0x8000, 0x1000)]);
    engine_privileged_state
        .get_process_manager()
        .set_opened_process(create_opened_process_info());
    seed_snapshot_with_single_scan_result(&engine_privileged_state, 0x8018);

    let scan_results_freeze_response = ScanResultsFreezeRequest {
        scan_result_refs: vec![ScanResultRef::new(0)],
        is_frozen: true,
    }
    .execute(&engine_privileged_state);

    assert!(
        scan_results_freeze_response
            .failed_freeze_toggle_scan_result_refs
            .is_empty()
    );

    let frozen_pointer = Pointer::new(0x18, Vec::new(), "freeze.exe".to_string());
    let freeze_list_registry = engine_privileged_state.get_freeze_list_registry();
    let freeze_list_registry_guard = match freeze_list_registry.read() {
        Ok(freeze_list_registry_guard) => freeze_list_registry_guard,
        Err(error) => panic!("failed to lock freeze list registry: {}", error),
    };
    assert!(freeze_list_registry_guard.is_address_frozen(&frozen_pointer));

    let mock_os_state = mock_engine_os.get_state();
    let state_guard = match mock_os_state.lock() {
        Ok(state_guard) => state_guard,
        Err(error) => panic!("failed to lock mock state: {}", error),
    };
    assert_eq!(state_guard.memory_read_addresses, vec![0x8018]);
}

#[test]
fn scan_results_freeze_executor_reports_failed_refs_when_memory_read_fails() {
    let (mock_engine_os, engine_privileged_state) = create_test_state();
    mock_engine_os.set_modules(vec![NormalizedModule::new("freeze.exe", 0x8000, 0x1000)]);
    mock_engine_os.set_read_success(false);
    engine_privileged_state
        .get_process_manager()
        .set_opened_process(create_opened_process_info());
    seed_snapshot_with_single_scan_result(&engine_privileged_state, 0x8018);

    let scan_result_ref = ScanResultRef::new(0);
    let scan_results_freeze_response = ScanResultsFreezeRequest {
        scan_result_refs: vec![scan_result_ref.clone()],
        is_frozen: true,
    }
    .execute(&engine_privileged_state);

    assert_eq!(
        scan_results_freeze_response
            .failed_freeze_toggle_scan_result_refs
            .len(),
        1
    );
    assert_eq!(
        scan_results_freeze_response.failed_freeze_toggle_scan_result_refs[0].get_scan_result_global_index(),
        scan_result_ref.get_scan_result_global_index()
    );

    let frozen_pointer = Pointer::new(0x18, Vec::new(), "freeze.exe".to_string());
    let freeze_list_registry = engine_privileged_state.get_freeze_list_registry();
    let freeze_list_registry_guard = match freeze_list_registry.read() {
        Ok(freeze_list_registry_guard) => freeze_list_registry_guard,
        Err(error) => panic!("failed to lock freeze list registry: {}", error),
    };
    assert!(!freeze_list_registry_guard.is_address_frozen(&frozen_pointer));
}

#[test]
fn scan_results_set_property_value_executor_uses_injected_memory_writer() {
    let (mock_engine_os, engine_privileged_state) = create_test_state();
    engine_privileged_state
        .get_process_manager()
        .set_opened_process(create_opened_process_info());
    seed_snapshot_with_single_scan_result(&engine_privileged_state, 0x901C);

    let scan_results_set_property_request = ScanResultsSetPropertyRequest {
        scan_result_refs: vec![ScanResultRef::new(0)],
        anonymous_value_string: AnonymousValueString::new("42".to_string(), AnonymousValueStringFormat::Decimal, ContainerType::None),
        field_namespace: ScanResult::PROPERTY_NAME_VALUE.to_string(),
    };
    let _scan_results_set_property_response = scan_results_set_property_request.execute(&engine_privileged_state);

    let mock_os_state = mock_engine_os.get_state();
    let state_guard = match mock_os_state.lock() {
        Ok(state_guard) => state_guard,
        Err(error) => panic!("failed to lock mock state: {}", error),
    };
    assert_eq!(state_guard.memory_write_requests.len(), 1);
    assert_eq!(state_guard.memory_write_requests[0].0, 0x901C);
    assert_eq!(state_guard.memory_write_requests[0].1, vec![42, 0, 0, 0]);
}

#[test]
fn memory_write_executor_returns_failure_without_freeze_mutation_when_writer_fails() {
    let (mock_engine_os, engine_privileged_state) = create_test_state();
    mock_engine_os.set_modules(vec![NormalizedModule::new("game.exe", 0x1000, 0x2000)]);
    mock_engine_os.set_write_success(false);
    engine_privileged_state
        .get_process_manager()
        .set_opened_process(create_opened_process_info());

    let memory_write_request = MemoryWriteRequest {
        address: 0x20,
        module_name: "game.exe".to_string(),
        value: vec![1, 2, 3, 4],
    };

    let memory_write_response = memory_write_request.execute(&engine_privileged_state);
    assert!(!memory_write_response.success);

    let frozen_pointer = Pointer::new(0x20, Vec::new(), "game.exe".to_string());
    let freeze_list_registry = engine_privileged_state.get_freeze_list_registry();
    let freeze_list_registry_guard = match freeze_list_registry.read() {
        Ok(freeze_list_registry_guard) => freeze_list_registry_guard,
        Err(error) => panic!("failed to lock freeze list registry: {}", error),
    };
    assert!(!freeze_list_registry_guard.is_address_frozen(&frozen_pointer));

    let mock_os_state = mock_engine_os.get_state();
    let state_guard = match mock_os_state.lock() {
        Ok(state_guard) => state_guard,
        Err(error) => panic!("failed to lock mock state: {}", error),
    };
    assert_eq!(state_guard.memory_write_requests.len(), 1);
    assert_eq!(state_guard.memory_write_requests[0].0, 0x1020);
    assert_eq!(state_guard.memory_write_requests[0].1, vec![1, 2, 3, 4]);
}

#[test]
fn scan_results_set_property_executor_handles_value_write_failures_without_freeze_mutation() {
    let (mock_engine_os, engine_privileged_state) = create_test_state();
    mock_engine_os.set_write_success(false);
    engine_privileged_state
        .get_process_manager()
        .set_opened_process(create_opened_process_info());
    seed_snapshot_with_single_scan_result(&engine_privileged_state, 0x901C);

    let scan_results_set_property_request = ScanResultsSetPropertyRequest {
        scan_result_refs: vec![ScanResultRef::new(0)],
        anonymous_value_string: AnonymousValueString::new("42".to_string(), AnonymousValueStringFormat::Decimal, ContainerType::None),
        field_namespace: ScanResult::PROPERTY_NAME_VALUE.to_string(),
    };
    let _scan_results_set_property_response = scan_results_set_property_request.execute(&engine_privileged_state);

    let frozen_pointer = Pointer::new(0x901C, Vec::new(), String::new());
    let freeze_list_registry = engine_privileged_state.get_freeze_list_registry();
    let freeze_list_registry_guard = match freeze_list_registry.read() {
        Ok(freeze_list_registry_guard) => freeze_list_registry_guard,
        Err(error) => panic!("failed to lock freeze list registry: {}", error),
    };
    assert!(!freeze_list_registry_guard.is_address_frozen(&frozen_pointer));

    let mock_os_state = mock_engine_os.get_state();
    let state_guard = match mock_os_state.lock() {
        Ok(state_guard) => state_guard,
        Err(error) => panic!("failed to lock mock state: {}", error),
    };
    assert_eq!(state_guard.memory_write_requests.len(), 1);
    assert_eq!(state_guard.memory_write_requests[0].0, 0x901C);
    assert_eq!(state_guard.memory_write_requests[0].1, vec![42, 0, 0, 0]);
}

#[test]
fn scan_results_set_property_executor_handles_freeze_toggle_failures_without_freeze_mutation() {
    let (mock_engine_os, engine_privileged_state) = create_test_state();
    mock_engine_os.set_modules(vec![NormalizedModule::new("toggle.exe", 0xA000, 0x1000)]);
    mock_engine_os.set_read_success(false);
    engine_privileged_state
        .get_process_manager()
        .set_opened_process(create_opened_process_info());
    seed_snapshot_with_single_scan_result(&engine_privileged_state, 0xA024);

    let freeze_scan_results_set_property_request = ScanResultsSetPropertyRequest {
        scan_result_refs: vec![ScanResultRef::new(0)],
        anonymous_value_string: AnonymousValueString::new("true".to_string(), AnonymousValueStringFormat::Bool, ContainerType::None),
        field_namespace: ScanResult::PROPERTY_NAME_IS_FROZEN.to_string(),
    };
    let _freeze_scan_results_set_property_response = freeze_scan_results_set_property_request.execute(&engine_privileged_state);

    let frozen_pointer = Pointer::new(0x24, Vec::new(), "toggle.exe".to_string());
    let freeze_list_registry = engine_privileged_state.get_freeze_list_registry();
    let freeze_list_registry_guard = match freeze_list_registry.read() {
        Ok(freeze_list_registry_guard) => freeze_list_registry_guard,
        Err(error) => panic!("failed to lock freeze list registry: {}", error),
    };
    assert!(!freeze_list_registry_guard.is_address_frozen(&frozen_pointer));
}

#[test]
fn scan_results_set_property_freeze_toggle_executor_uses_injected_providers() {
    let (mock_engine_os, engine_privileged_state) = create_test_state();
    mock_engine_os.set_modules(vec![NormalizedModule::new("toggle.exe", 0xA000, 0x1000)]);
    engine_privileged_state
        .get_process_manager()
        .set_opened_process(create_opened_process_info());
    seed_snapshot_with_single_scan_result(&engine_privileged_state, 0xA024);

    let freeze_scan_results_set_property_request = ScanResultsSetPropertyRequest {
        scan_result_refs: vec![ScanResultRef::new(0)],
        anonymous_value_string: AnonymousValueString::new("true".to_string(), AnonymousValueStringFormat::Bool, ContainerType::None),
        field_namespace: ScanResult::PROPERTY_NAME_IS_FROZEN.to_string(),
    };
    let _freeze_scan_results_set_property_response = freeze_scan_results_set_property_request.execute(&engine_privileged_state);

    let frozen_pointer = Pointer::new(0x24, Vec::new(), "toggle.exe".to_string());
    let freeze_list_registry = engine_privileged_state.get_freeze_list_registry();
    let freeze_list_registry_guard = match freeze_list_registry.read() {
        Ok(freeze_list_registry_guard) => freeze_list_registry_guard,
        Err(error) => panic!("failed to lock freeze list registry: {}", error),
    };
    assert!(freeze_list_registry_guard.is_address_frozen(&frozen_pointer));
    drop(freeze_list_registry_guard);

    let unfreeze_scan_results_set_property_request = ScanResultsSetPropertyRequest {
        scan_result_refs: vec![ScanResultRef::new(0)],
        anonymous_value_string: AnonymousValueString::new("false".to_string(), AnonymousValueStringFormat::Bool, ContainerType::None),
        field_namespace: ScanResult::PROPERTY_NAME_IS_FROZEN.to_string(),
    };
    let _unfreeze_scan_results_set_property_response = unfreeze_scan_results_set_property_request.execute(&engine_privileged_state);

    let freeze_list_registry_guard = match freeze_list_registry.read() {
        Ok(freeze_list_registry_guard) => freeze_list_registry_guard,
        Err(error) => panic!("failed to lock freeze list registry: {}", error),
    };
    assert!(!freeze_list_registry_guard.is_address_frozen(&frozen_pointer));

    let mock_os_state = mock_engine_os.get_state();
    let state_guard = match mock_os_state.lock() {
        Ok(state_guard) => state_guard,
        Err(error) => panic!("failed to lock mock state: {}", error),
    };
    assert_eq!(state_guard.memory_read_addresses, vec![0xA024]);
}
