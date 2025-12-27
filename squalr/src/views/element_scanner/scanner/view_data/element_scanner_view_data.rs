use crate::views::element_scanner::scanner::{
    element_scanner_view_state::ElementScannerViewState, view_data::element_scanner_value_view_data::ElementScannerValueViewData,
};
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
        data_values::{anonymous_value::AnonymousValue, display_value::DisplayValue},
        scanning::{
            comparisons::{scan_compare_type::ScanCompareType, scan_compare_type_immediate::ScanCompareTypeImmediate},
            constraints::anonymous_scan_constraint::AnonymousScanConstraint,
        },
    },
};
use std::sync::Arc;

pub struct ElementScannerViewData {
    pub selected_data_type: DataTypeRef,
    pub view_state: ElementScannerViewState,
    pub scan_values_and_constraints: Vec<ElementScannerValueViewData>,
}

impl ElementScannerViewData {
    const MAX_CONSTRAINTS: usize = 5;

    pub fn new() -> Self {
        Self {
            selected_data_type: DataTypeRef::new(DataTypeI32::get_data_type_id()),
            view_state: ElementScannerViewState::NoResults,
            scan_values_and_constraints: vec![ElementScannerValueViewData::new()],
        }
    }

    pub fn reset_scan(
        element_scanner_view_data: Dependency<Self>,
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
        element_scanner_view_data: Dependency<Self>,
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
        element_scanner_view_data: Dependency<Self>,
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
        element_scanner_view_data: Dependency<Self>,
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
        let data_type_refs = vec![element_scanner_view_data.selected_data_type.clone()];
        let scan_constraints = element_scanner_view_data
            .scan_values_and_constraints
            .iter()
            .map(|scan_value_and_constraint| {
                AnonymousScanConstraint::new(
                    scan_value_and_constraint.selected_scan_compare_type,
                    Some(AnonymousValue::new(&scan_value_and_constraint.current_scan_value)),
                )
            })
            .collect();
        let element_scan_request = ElementScanRequest {
            scan_constraints,
            data_type_refs,
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

    pub fn add_constraint(element_scanner_view_data: Dependency<Self>) {
        let mut element_scanner_view_data = match element_scanner_view_data.write() {
            Ok(element_scanner_view_data) => element_scanner_view_data,
            Err(error) => {
                log::error!("Failed to write element scanner view state: {}", error);
                return;
            }
        };

        if element_scanner_view_data.scan_values_and_constraints.len() >= Self::MAX_CONSTRAINTS {
            return;
        }

        // If creating the 2nd constraint, <= is the most common constraint, so default to that for a better UX.
        if element_scanner_view_data.scan_values_and_constraints.len() == 1 {
            element_scanner_view_data
                .scan_values_and_constraints
                .push(ElementScannerValueViewData {
                    selected_scan_compare_type: ScanCompareType::Immediate(ScanCompareTypeImmediate::LessThanOrEqual),
                    current_scan_value: DisplayValue::default(),
                });
        } else {
            element_scanner_view_data
                .scan_values_and_constraints
                .push(ElementScannerValueViewData::new());
        }
    }

    pub fn remove_constraint(
        element_scanner_view_data: Dependency<Self>,
        index: usize,
    ) {
        let mut element_scanner_view_data = match element_scanner_view_data.write() {
            Ok(element_scanner_view_data) => element_scanner_view_data,
            Err(error) => {
                log::error!("Failed to write element scanner view state: {}", error);
                return;
            }
        };

        if index <= 0 {
            return;
        }

        element_scanner_view_data
            .scan_values_and_constraints
            .remove(index);
    }
}
