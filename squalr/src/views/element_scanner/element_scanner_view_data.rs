use crate::views::element_scanner::element_scanner_view_state::ElementScannerViewState;
use squalr_engine_api::{
    commands::{
        engine_command_request::EngineCommandRequest,
        scan::{
            collect_values::scan_collect_values_request::ScanCollectValuesRequest, element_scan::element_scan_request::ElementScanRequest,
            new::scan_new_request::ScanNewRequest, reset::scan_reset_request::ScanResetRequest,
        },
    },
    dependency_injection::dependency::Dependency,
    engine::engine_execution_context::EngineExecutionContext,
    structures::{
        data_types::{built_in_types::i32::data_type_i32::DataTypeI32, data_type_ref::DataTypeRef},
        data_values::{anonymous_value::AnonymousValue, display_value::DisplayValue, display_value_type::DisplayValueType},
        scanning::comparisons::{scan_compare_type::ScanCompareType, scan_compare_type_immediate::ScanCompareTypeImmediate},
        structs::container_type::ContainerType,
    },
};
use std::sync::Arc;

pub struct ElementScannerViewData {
    pub selected_data_type: DataTypeRef,
    pub selected_scan_compare_type: ScanCompareType,
    pub view_state: ElementScannerViewState,
    pub current_scan_value: DisplayValue,
}

impl ElementScannerViewData {
    pub fn new() -> Self {
        Self {
            selected_data_type: DataTypeRef::new(DataTypeI32::get_data_type_id()),
            selected_scan_compare_type: ScanCompareType::Immediate(ScanCompareTypeImmediate::Equal),
            view_state: ElementScannerViewState::NoResults,
            current_scan_value: DisplayValue::new(String::new(), DisplayValueType::Decimal, ContainerType::None),
        }
    }

    pub fn reset_scan(
        element_scanner_view_data: Dependency<ElementScannerViewData>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) {
        let scan_reset_request = ScanResetRequest {};

        scan_reset_request.send(&engine_execution_context, move |scan_reset_response| {
            if scan_reset_response.success {
                match element_scanner_view_data.write() {
                    Ok(mut element_scanner_view_data) => {
                        element_scanner_view_data.view_state = ElementScannerViewState::NoResults;
                    }
                    Err(error) => {
                        log::error!("Failed to write element scanner view state: {}", error);
                    }
                }
            }
        });
    }

    pub fn collect_values(engine_execution_context: Arc<EngineExecutionContext>) {
        let collect_values_request = ScanCollectValuesRequest {};

        collect_values_request.send(&engine_execution_context, |_scan_collect_values_response| {});
    }

    pub fn start_scan(
        element_scanner_view_data: Dependency<ElementScannerViewData>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) {
        let element_scanner_view_data_view_state = {
            match element_scanner_view_data.read() {
                Ok(element_scanner_view_data) => element_scanner_view_data.view_state,
                Err(error) => {
                    log::error!("Failed to acquire UI state lock to start scan: {}", error);
                    return;
                }
            }
        };

        match element_scanner_view_data_view_state {
            ElementScannerViewState::HasResults => {
                Self::start_next_scan(element_scanner_view_data, engine_execution_context);
            }
            ElementScannerViewState::NoResults => {
                Self::new_scan(element_scanner_view_data, engine_execution_context);
            }
            ElementScannerViewState::ScanInProgress => {
                log::error!("Cannot start a new scan while a scan is in progress.");
            }
        };
    }

    fn new_scan(
        element_scanner_view_data: Dependency<ElementScannerViewData>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) {
        let engine_execution_context_clone = engine_execution_context.clone();
        let element_scanner_view_data = element_scanner_view_data.clone();
        let scan_new_request = ScanNewRequest {};

        // Start a new scan, and recurse to start the scan once the new scan is made.
        scan_new_request.send(&engine_execution_context, move |_scan_new_response| {
            Self::start_next_scan(element_scanner_view_data, engine_execution_context_clone);
        });
    }

    fn start_next_scan(
        element_scanner_view_data: Dependency<ElementScannerViewData>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) {
        let element_scanner_view_data_clone = element_scanner_view_data.clone();
        let element_scanner_view_data = {
            match element_scanner_view_data.read() {
                Ok(element_scanner_view_data) => element_scanner_view_data,
                Err(error) => {
                    log::error!("Failed to acquire UI state lock to start scan: {}", error);
                    return;
                }
            }
        };
        let data_type_ids = vec![
            element_scanner_view_data
                .selected_data_type
                .get_data_type_id()
                .to_string(),
        ];
        let anonymous_value = AnonymousValue::new(&element_scanner_view_data.current_scan_value);
        let element_scan_request = ElementScanRequest {
            scan_value: Some(anonymous_value),
            data_type_ids: data_type_ids,
            compare_type: element_scanner_view_data.selected_scan_compare_type.clone(),
        };

        drop(element_scanner_view_data);

        element_scan_request.send(&engine_execution_context, move |scan_execute_response| {
            match element_scanner_view_data_clone.write() {
                Ok(mut element_scanner_view_data) => {
                    element_scanner_view_data.view_state = ElementScannerViewState::ScanInProgress;
                }
                Err(error) => {
                    log::error!("Failed to write element scanner view state: {}", error);
                }
            }
            // JIRA: We actually need to wait for the task to complete, which can be tricky with our request/response architecture.
            // For now we just set it immediately to avoid being stuck in in progress state.
            // JIRA: Use scan_execute_response.trackable_task_handle;
            match element_scanner_view_data_clone.write() {
                Ok(mut element_scanner_view_data) => {
                    element_scanner_view_data.view_state = ElementScannerViewState::HasResults;
                }
                Err(error) => {
                    log::error!("Failed to write element scanner view state: {}", error);
                }
            }
        });
    }
}
