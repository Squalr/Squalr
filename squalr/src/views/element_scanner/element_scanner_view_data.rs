use crate::views::element_scanner::element_scanner_view_state::ElementScannerViewState;
use squalr_engine_api::{
    commands::{
        engine_command_request::EngineCommandRequest,
        scan::{
            collect_values::scan_collect_values_request::ScanCollectValuesRequest, new::scan_new_request::ScanNewRequest,
            reset::scan_reset_request::ScanResetRequest,
        },
    },
    dependency_injection::dependency::Dependency,
    engine::engine_execution_context::EngineExecutionContext,
    structures::{
        data_types::{built_in_types::i32::data_type_i32::DataTypeI32, data_type_ref::DataTypeRef},
        data_values::anonymous_value::AnonymousValue,
        scanning::comparisons::{scan_compare_type::ScanCompareType, scan_compare_type_immediate::ScanCompareTypeImmediate},
    },
};
use std::sync::Arc;

pub struct ElementScannerViewData {
    pub selected_data_type: DataTypeRef,
    pub selected_scan_compare_type: ScanCompareType,
    pub view_state: ElementScannerViewState,
}

impl ElementScannerViewData {
    pub fn new() -> Self {
        Self {
            selected_data_type: DataTypeRef::new(DataTypeI32::get_data_type_id()),
            selected_scan_compare_type: ScanCompareType::Immediate(ScanCompareTypeImmediate::Equal),
            view_state: ElementScannerViewState::NoResults,
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
        /*
        scan_value: &str,
        data_type_ids: Vec<String>,
        display_value: DisplayValue,
         */
        /*
        let scan_element_scanner_view_data_state = &element_scanner_view_data.scan_element_scanner_view_data_state;

        let scan_element_scanner_view_data_state_value = {
            *match scan_element_scanner_view_data_state.read() {
                Ok(guard) => guard,
                Err(error) => {
                    log::error!("Failed to acquire UI state lock to start scan: {}", error);
                    return;
                }
            }
        };

        let data_type_ids = data_type_ids
            .iter()
            .map(|data_type_id| data_type_id.to_string())
            .collect();
        let mut display_value = DisplayValueConverter {}.convert_from_view_data(&display_value);

        display_value.set_display_string(scan_value.to_string());

        let anonymous_value = AnonymousValue::new(display_value);

        match scan_element_scanner_view_data_state_value {
            ScanViewModelState::HasResults => {
                Self::start_next_scan(element_scanner_view_data, scan_compare_type, data_type_ids, anonymous_value);
            }
            ScanViewModelState::NoResults => {
                Self::new_scan(element_scanner_view_data, scan_compare_type, data_type_ids, anonymous_value);
            }
            ScanViewModelState::ScanInProgress => {
                log::error!("Cannot start a new scan while a scan is in progress.");
            }
        };*/
    }

    fn new_scan(
        element_scanner_view_data: Dependency<ElementScannerViewData>,
        engine_execution_context: Arc<EngineExecutionContext>,
        scan_compare_type: ScanCompareType,
        data_type_ids: Vec<String>,
        anonymous_value: AnonymousValue,
    ) {
        let engine_execution_context_clone = engine_execution_context.clone();
        let element_scanner_view_data = element_scanner_view_data.clone();
        let scan_new_request = ScanNewRequest {};

        // Start a new scan, and recurse to start the scan once the new scan is made.
        scan_new_request.send(&engine_execution_context, move |_scan_new_response| {
            Self::start_next_scan(
                element_scanner_view_data,
                engine_execution_context_clone,
                scan_compare_type,
                data_type_ids,
                anonymous_value,
            );
        });
    }

    fn start_next_scan(
        element_scanner_view_data: Dependency<ElementScannerViewData>,
        engine_execution_context: Arc<EngineExecutionContext>,
        scan_compare_type: ScanCompareType,
        data_type_ids: Vec<String>,
        anonymous_value: AnonymousValue,
    ) {
        /*
        let engine_execution_context = &element_scanner_view_data.engine_execution_context;
        let element_scanner_view_data = element_scanner_view_data.clone();
        let element_scan_request = ElementScanRequest {
            scan_value: Some(anonymous_value),
            data_type_ids: data_type_ids,
            compare_type: ScanConstraintConverter::new().convert_from_view_data(&scan_compare_type),
        };

        element_scan_request.send(&engine_execution_context, move |scan_execute_response| {
            let scan_element_scanner_view_data_state = &element_scanner_view_data.scan_element_scanner_view_data_state;

            if let Ok(mut scan_element_scanner_view_data_state) = scan_element_scanner_view_data_state.write() {
                *scan_element_scanner_view_data_state = ScanViewModelState::ScanInProgress;
            }
            // JIRA: We actually need to wait for the task to complete, which can be tricky with our request/response architecture.
            // For now we just set it immediately to avoid being stuck in in progress state.
            if let Ok(mut scan_element_scanner_view_data_state) = scan_element_scanner_view_data_state.write() {
                *scan_element_scanner_view_data_state = ScanViewModelState::HasResults;
            }
        }); */
    }
}
