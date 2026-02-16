use crossbeam_channel::{Receiver, unbounded};
use squalr_engine_api::commands::privileged_command::PrivilegedCommand;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use squalr_engine_api::commands::project::list::project_list_response::ProjectListResponse;
use squalr_engine_api::commands::scan_results::add_to_project::scan_results_add_to_project_request::ScanResultsAddToProjectRequest;
use squalr_engine_api::commands::scan_results::add_to_project::scan_results_add_to_project_response::ScanResultsAddToProjectResponse;
use squalr_engine_api::commands::scan_results::delete::scan_results_delete_request::ScanResultsDeleteRequest;
use squalr_engine_api::commands::scan_results::delete::scan_results_delete_response::ScanResultsDeleteResponse;
use squalr_engine_api::commands::scan_results::freeze::scan_results_freeze_request::ScanResultsFreezeRequest;
use squalr_engine_api::commands::scan_results::freeze::scan_results_freeze_response::ScanResultsFreezeResponse;
use squalr_engine_api::commands::scan_results::list::scan_results_list_request::ScanResultsListRequest;
use squalr_engine_api::commands::scan_results::list::scan_results_list_response::ScanResultsListResponse;
use squalr_engine_api::commands::scan_results::query::scan_results_query_request::ScanResultsQueryRequest;
use squalr_engine_api::commands::scan_results::query::scan_results_query_response::ScanResultsQueryResponse;
use squalr_engine_api::commands::scan_results::refresh::scan_results_refresh_request::ScanResultsRefreshRequest;
use squalr_engine_api::commands::scan_results::refresh::scan_results_refresh_response::ScanResultsRefreshResponse;
use squalr_engine_api::commands::scan_results::scan_results_command::ScanResultsCommand;
use squalr_engine_api::commands::scan_results::set_property::scan_results_set_property_request::ScanResultsSetPropertyRequest;
use squalr_engine_api::commands::scan_results::set_property::scan_results_set_property_response::ScanResultsSetPropertyResponse;
use squalr_engine_api::commands::unprivileged_command_response::TypedUnprivilegedCommandResponse;
use squalr_engine_api::engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings;
use squalr_engine_api::engine::engine_binding_error::EngineBindingError;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::events::engine_event::EngineEvent;
use squalr_engine_api::structures::data_values::anonymous_value_string::AnonymousValueString;
use squalr_engine_api::structures::scan_results::scan_result_ref::ScanResultRef;
use squalr_engine_api::{commands::unprivileged_command::UnprivilegedCommand, commands::unprivileged_command_response::UnprivilegedCommandResponse};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use structopt::StructOpt;

use squalr_tests::mocks::mock_engine_bindings::MockEngineBindings;

struct FailingDispatchBindings;

impl EngineApiUnprivilegedBindings for FailingDispatchBindings {
    fn dispatch_privileged_command(
        &self,
        _engine_command: PrivilegedCommand,
        _callback: Box<dyn FnOnce(squalr_engine_api::commands::privileged_command_response::PrivilegedCommandResponse) + Send + Sync + 'static>,
    ) -> Result<(), EngineBindingError> {
        Err(EngineBindingError::unavailable("dispatching privileged command in test"))
    }

    fn dispatch_unprivileged_command(
        &self,
        _engine_command: UnprivilegedCommand,
        _engine_execution_context: &Arc<dyn EngineExecutionContext>,
        _callback: Box<dyn FnOnce(UnprivilegedCommandResponse) + Send + Sync + 'static>,
    ) -> Result<(), EngineBindingError> {
        Err(EngineBindingError::unavailable("dispatching unprivileged command in test"))
    }

    fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEvent>, EngineBindingError> {
        let (_event_sender, event_receiver) = unbounded();
        Ok(event_receiver)
    }
}

#[test]
fn scan_results_list_request_dispatches_list_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        ScanResultsListResponse {
            scan_results: vec![],
            page_index: 4,
            last_page_index: 12,
            page_size: 22,
            result_count: 264,
            total_size_in_bytes: 4096,
        }
        .to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();

    let scan_results_list_request = ScanResultsListRequest { page_index: 4 };
    let callback_page_index = Arc::new(RwLock::new(None::<u64>));
    let callback_page_index_clone = callback_page_index.clone();

    scan_results_list_request.send_unprivileged(&bindings, move |scan_results_list_response| {
        if let Ok(mut callback_page_index_guard) = callback_page_index_clone.write() {
            *callback_page_index_guard = Some(scan_results_list_response.page_index);
        }
    });

    let callback_page_index_guard = callback_page_index
        .read()
        .expect("callback capture lock should be available");
    assert_eq!(*callback_page_index_guard, Some(4));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Results(ScanResultsCommand::List {
            results_list_request: captured_scan_results_list_request,
        }) => {
            assert_eq!(captured_scan_results_list_request.page_index, 4);
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn scan_results_query_request_send_unprivileged_returns_false_when_dispatch_fails() {
    let bindings = FailingDispatchBindings;
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    let did_dispatch = ScanResultsQueryRequest { page_index: 0 }.send_unprivileged(&bindings, move |_scan_results_query_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(!did_dispatch);
    assert!(!callback_invoked.load(Ordering::SeqCst));
}

#[test]
fn scan_results_list_request_does_not_invoke_callback_when_response_variant_is_wrong() {
    let bindings = MockEngineBindings::new(
        ScanResultsQueryResponse::default().to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();

    let scan_results_list_request = ScanResultsListRequest { page_index: 9 };
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    scan_results_list_request.send_unprivileged(&bindings, move |_scan_results_list_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(!callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Results(ScanResultsCommand::List {
            results_list_request: captured_scan_results_list_request,
        }) => {
            assert_eq!(captured_scan_results_list_request.page_index, 9);
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn scan_results_query_request_dispatches_query_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        ScanResultsQueryResponse {
            scan_results: vec![],
            page_index: 3,
            last_page_index: 8,
            page_size: 20,
            result_count: 160,
            total_size_in_bytes: 2048,
        }
        .to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();

    let scan_results_query_request = ScanResultsQueryRequest { page_index: 3 };
    let callback_page_size = Arc::new(RwLock::new(None::<u64>));
    let callback_page_size_clone = callback_page_size.clone();

    scan_results_query_request.send_unprivileged(&bindings, move |scan_results_query_response| {
        if let Ok(mut callback_page_size_guard) = callback_page_size_clone.write() {
            *callback_page_size_guard = Some(scan_results_query_response.page_size);
        }
    });

    let callback_page_size_guard = callback_page_size
        .read()
        .expect("callback capture lock should be available");
    assert_eq!(*callback_page_size_guard, Some(20));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Results(ScanResultsCommand::Query {
            results_query_request: captured_scan_results_query_request,
        }) => {
            assert_eq!(captured_scan_results_query_request.page_index, 3);
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn scan_results_query_request_does_not_invoke_callback_when_response_variant_is_wrong() {
    let bindings = MockEngineBindings::new(
        ScanResultsDeleteResponse::default().to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();

    let scan_results_query_request = ScanResultsQueryRequest { page_index: 7 };
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    scan_results_query_request.send_unprivileged(&bindings, move |_scan_results_query_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(!callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Results(ScanResultsCommand::Query {
            results_query_request: captured_scan_results_query_request,
        }) => {
            assert_eq!(captured_scan_results_query_request.page_index, 7);
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn scan_results_freeze_request_dispatches_freeze_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        ScanResultsFreezeResponse {
            failed_freeze_toggle_scan_result_refs: vec![ScanResultRef::new(91)],
        }
        .to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();

    let scan_results_freeze_request = ScanResultsFreezeRequest {
        scan_result_refs: vec![ScanResultRef::new(8), ScanResultRef::new(13)],
        is_frozen: true,
    };
    let callback_failed_ref_count = Arc::new(RwLock::new(None::<usize>));
    let callback_failed_ref_count_clone = callback_failed_ref_count.clone();

    scan_results_freeze_request.send_unprivileged(&bindings, move |scan_results_freeze_response| {
        if let Ok(mut callback_failed_ref_count_guard) = callback_failed_ref_count_clone.write() {
            *callback_failed_ref_count_guard = Some(
                scan_results_freeze_response
                    .failed_freeze_toggle_scan_result_refs
                    .len(),
            );
        }
    });

    let callback_failed_ref_count_guard = callback_failed_ref_count
        .read()
        .expect("callback capture lock should be available");
    assert_eq!(*callback_failed_ref_count_guard, Some(1));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Results(ScanResultsCommand::Freeze {
            results_freeze_request: captured_scan_results_freeze_request,
        }) => {
            assert_eq!(captured_scan_results_freeze_request.scan_result_refs.len(), 2);
            assert!(captured_scan_results_freeze_request.is_frozen);
            assert_eq!(captured_scan_results_freeze_request.scan_result_refs[0].get_scan_result_global_index(), 8);
            assert_eq!(captured_scan_results_freeze_request.scan_result_refs[1].get_scan_result_global_index(), 13);
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn scan_results_freeze_request_does_not_invoke_callback_when_response_variant_is_wrong() {
    let bindings = MockEngineBindings::new(
        ScanResultsListResponse::default().to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();

    let scan_results_freeze_request = ScanResultsFreezeRequest {
        scan_result_refs: vec![ScanResultRef::new(77)],
        is_frozen: false,
    };
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    scan_results_freeze_request.send_unprivileged(&bindings, move |_scan_results_freeze_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(!callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Results(ScanResultsCommand::Freeze {
            results_freeze_request: captured_scan_results_freeze_request,
        }) => {
            assert_eq!(captured_scan_results_freeze_request.scan_result_refs.len(), 1);
            assert_eq!(captured_scan_results_freeze_request.scan_result_refs[0].get_scan_result_global_index(), 77);
            assert!(!captured_scan_results_freeze_request.is_frozen);
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn scan_results_delete_request_dispatches_delete_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        ScanResultsDeleteResponse::default().to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();

    let scan_results_delete_request = ScanResultsDeleteRequest {
        scan_result_refs: vec![ScanResultRef::new(17), ScanResultRef::new(29)],
    };
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    scan_results_delete_request.send_unprivileged(&bindings, move |_scan_results_delete_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Results(ScanResultsCommand::Delete {
            results_delete_request: captured_scan_results_delete_request,
        }) => {
            assert_eq!(captured_scan_results_delete_request.scan_result_refs.len(), 2);
            assert_eq!(captured_scan_results_delete_request.scan_result_refs[0].get_scan_result_global_index(), 17);
            assert_eq!(captured_scan_results_delete_request.scan_result_refs[1].get_scan_result_global_index(), 29);
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn scan_results_delete_request_does_not_invoke_callback_when_response_variant_is_wrong() {
    let bindings = MockEngineBindings::new(
        ScanResultsRefreshResponse::default().to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();

    let scan_results_delete_request = ScanResultsDeleteRequest {
        scan_result_refs: vec![ScanResultRef::new(45)],
    };
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    scan_results_delete_request.send_unprivileged(&bindings, move |_scan_results_delete_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(!callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Results(ScanResultsCommand::Delete {
            results_delete_request: captured_scan_results_delete_request,
        }) => {
            assert_eq!(captured_scan_results_delete_request.scan_result_refs.len(), 1);
            assert_eq!(captured_scan_results_delete_request.scan_result_refs[0].get_scan_result_global_index(), 45);
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn scan_results_refresh_request_dispatches_refresh_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        ScanResultsRefreshResponse::default().to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();

    let scan_results_refresh_request = ScanResultsRefreshRequest {
        scan_result_refs: vec![ScanResultRef::new(31), ScanResultRef::new(41)],
    };
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    scan_results_refresh_request.send_unprivileged(&bindings, move |_scan_results_refresh_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Results(ScanResultsCommand::Refresh {
            results_refresh_request: captured_scan_results_refresh_request,
        }) => {
            assert_eq!(captured_scan_results_refresh_request.scan_result_refs.len(), 2);
            assert_eq!(captured_scan_results_refresh_request.scan_result_refs[0].get_scan_result_global_index(), 31);
            assert_eq!(captured_scan_results_refresh_request.scan_result_refs[1].get_scan_result_global_index(), 41);
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn scan_results_refresh_request_does_not_invoke_callback_when_response_variant_is_wrong() {
    let bindings = MockEngineBindings::new(
        ScanResultsDeleteResponse::default().to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();

    let scan_results_refresh_request = ScanResultsRefreshRequest {
        scan_result_refs: vec![ScanResultRef::new(63)],
    };
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    scan_results_refresh_request.send_unprivileged(&bindings, move |_scan_results_refresh_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(!callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Results(ScanResultsCommand::Refresh {
            results_refresh_request: captured_scan_results_refresh_request,
        }) => {
            assert_eq!(captured_scan_results_refresh_request.scan_result_refs.len(), 1);
            assert_eq!(captured_scan_results_refresh_request.scan_result_refs[0].get_scan_result_global_index(), 63);
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn scan_results_add_to_project_request_dispatches_add_to_project_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        ScanResultsAddToProjectResponse::default().to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();

    let scan_results_add_to_project_request = ScanResultsAddToProjectRequest {
        scan_result_refs: vec![ScanResultRef::new(5), ScanResultRef::new(15)],
    };
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    scan_results_add_to_project_request.send_unprivileged(&bindings, move |_scan_results_add_to_project_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Results(ScanResultsCommand::AddToProject {
            results_add_to_project_request: captured_scan_results_add_to_project_request,
        }) => {
            assert_eq!(
                captured_scan_results_add_to_project_request
                    .scan_result_refs
                    .len(),
                2
            );
            assert_eq!(
                captured_scan_results_add_to_project_request.scan_result_refs[0].get_scan_result_global_index(),
                5
            );
            assert_eq!(
                captured_scan_results_add_to_project_request.scan_result_refs[1].get_scan_result_global_index(),
                15
            );
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn scan_results_add_to_project_request_does_not_invoke_callback_when_response_variant_is_wrong() {
    let bindings = MockEngineBindings::new(
        ScanResultsFreezeResponse::default().to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();

    let scan_results_add_to_project_request = ScanResultsAddToProjectRequest {
        scan_result_refs: vec![ScanResultRef::new(55)],
    };
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    scan_results_add_to_project_request.send_unprivileged(&bindings, move |_scan_results_add_to_project_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(!callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Results(ScanResultsCommand::AddToProject {
            results_add_to_project_request: captured_scan_results_add_to_project_request,
        }) => {
            assert_eq!(
                captured_scan_results_add_to_project_request
                    .scan_result_refs
                    .len(),
                1
            );
            assert_eq!(
                captured_scan_results_add_to_project_request.scan_result_refs[0].get_scan_result_global_index(),
                55
            );
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn scan_results_set_property_request_dispatches_set_property_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        ScanResultsSetPropertyResponse::default().to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();

    let anonymous_value_string = AnonymousValueString::from_str("255;dec;").expect("anonymous value string should parse");
    let scan_results_set_property_request = ScanResultsSetPropertyRequest {
        scan_result_refs: vec![ScanResultRef::new(7), ScanResultRef::new(11)],
        anonymous_value_string,
        field_namespace: "value".to_string(),
    };
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    scan_results_set_property_request.send_unprivileged(&bindings, move |_scan_results_set_property_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Results(ScanResultsCommand::SetProperty {
            results_set_property_request: captured_scan_results_set_property_request,
        }) => {
            assert_eq!(
                captured_scan_results_set_property_request
                    .scan_result_refs
                    .len(),
                2
            );
            assert_eq!(captured_scan_results_set_property_request.scan_result_refs[0].get_scan_result_global_index(), 7);
            assert_eq!(
                captured_scan_results_set_property_request.scan_result_refs[1].get_scan_result_global_index(),
                11
            );
            assert_eq!(
                captured_scan_results_set_property_request
                    .anonymous_value_string
                    .get_anonymous_value_string(),
                "255"
            );
            assert_eq!(captured_scan_results_set_property_request.field_namespace, "value");
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn scan_results_set_property_request_does_not_invoke_callback_when_response_variant_is_wrong() {
    let bindings = MockEngineBindings::new(
        ScanResultsQueryResponse::default().to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();

    let anonymous_value_string = AnonymousValueString::from_str("100;dec;").expect("anonymous value string should parse");
    let scan_results_set_property_request = ScanResultsSetPropertyRequest {
        scan_result_refs: vec![ScanResultRef::new(14)],
        anonymous_value_string,
        field_namespace: "health".to_string(),
    };
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    scan_results_set_property_request.send_unprivileged(&bindings, move |_scan_results_set_property_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(!callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Results(ScanResultsCommand::SetProperty {
            results_set_property_request: captured_scan_results_set_property_request,
        }) => {
            assert_eq!(
                captured_scan_results_set_property_request
                    .scan_result_refs
                    .len(),
                1
            );
            assert_eq!(
                captured_scan_results_set_property_request.scan_result_refs[0].get_scan_result_global_index(),
                14
            );
            assert_eq!(
                captured_scan_results_set_property_request
                    .anonymous_value_string
                    .get_anonymous_value_string(),
                "100"
            );
            assert_eq!(captured_scan_results_set_property_request.field_namespace, "health");
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_scan_results_list_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| PrivilegedCommand::from_iter_safe(["squalr-cli", "results", "list", "--page-index", "2"]));

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Results(ScanResultsCommand::List { results_list_request }) => {
            assert_eq!(results_list_request.page_index, 2);
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_scan_results_query_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| PrivilegedCommand::from_iter_safe(["squalr-cli", "results", "query", "--page-index", "5"]));

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Results(ScanResultsCommand::Query { results_query_request }) => {
            assert_eq!(results_query_request.page_index, 5);
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_scan_results_refresh_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "results",
            "refresh",
            "--scan-result-refs",
            "13",
            "--scan-result-refs",
            "21",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Results(ScanResultsCommand::Refresh { results_refresh_request }) => {
            assert_eq!(results_refresh_request.scan_result_refs.len(), 2);
            assert_eq!(results_refresh_request.scan_result_refs[0].get_scan_result_global_index(), 13);
            assert_eq!(results_refresh_request.scan_result_refs[1].get_scan_result_global_index(), 21);
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_scan_results_add_to_project_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "results",
            "add-to-project",
            "--scan-result-refs",
            "8",
            "--scan-result-refs",
            "34",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Results(ScanResultsCommand::AddToProject {
            results_add_to_project_request,
        }) => {
            assert_eq!(results_add_to_project_request.scan_result_refs.len(), 2);
            assert_eq!(results_add_to_project_request.scan_result_refs[0].get_scan_result_global_index(), 8);
            assert_eq!(results_add_to_project_request.scan_result_refs[1].get_scan_result_global_index(), 34);
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_scan_results_set_property_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "results",
            "set-property",
            "--scan-result-refs",
            "7",
            "--scan-result-refs",
            "11",
            "--anonymous-value-string",
            "255;dec;",
            "--field-namespace",
            "value",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Results(ScanResultsCommand::SetProperty { results_set_property_request }) => {
            assert_eq!(results_set_property_request.scan_result_refs.len(), 2);
            assert_eq!(results_set_property_request.scan_result_refs[0].get_scan_result_global_index(), 7);
            assert_eq!(results_set_property_request.scan_result_refs[1].get_scan_result_global_index(), 11);
            assert_eq!(
                results_set_property_request
                    .anonymous_value_string
                    .get_anonymous_value_string(),
                "255"
            );
            assert_eq!(results_set_property_request.field_namespace, "value".to_string());
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_scan_results_delete_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "results",
            "delete",
            "--scan-result-refs",
            "17",
            "--scan-result-refs",
            "29",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Results(ScanResultsCommand::Delete { results_delete_request }) => {
            assert_eq!(results_delete_request.scan_result_refs.len(), 2);
            assert_eq!(results_delete_request.scan_result_refs[0].get_scan_result_global_index(), 17);
            assert_eq!(results_delete_request.scan_result_refs[1].get_scan_result_global_index(), 29);
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_scan_results_freeze_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "results",
            "freeze",
            "--scan-result-refs",
            "3",
            "--scan-result-refs",
            "9",
            "--is-frozen",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Results(ScanResultsCommand::Freeze { results_freeze_request }) => {
            assert_eq!(results_freeze_request.scan_result_refs.len(), 2);
            assert_eq!(results_freeze_request.scan_result_refs[0].get_scan_result_global_index(), 3);
            assert_eq!(results_freeze_request.scan_result_refs[1].get_scan_result_global_index(), 9);
            assert!(results_freeze_request.is_frozen);
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_rejects_scan_results_set_property_with_invalid_anonymous_value_string() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "results",
            "set-property",
            "--scan-result-refs",
            "7",
            "--anonymous-value-string",
            "bad-format",
            "--field-namespace",
            "value",
        ])
    });

    assert!(parse_result.is_ok());
    assert!(parse_result.expect("parser should not panic").is_err());
}
