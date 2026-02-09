use crate::views::element_scanner::scanner::{
    element_scanner_view_state::ElementScannerViewState, view_data::element_scanner_value_view_data::ElementScannerValueViewData,
};
use squalr_engine_api::{
    commands::{
        privileged_command_request::PrivilegedCommandRequest,
        scan::{
            collect_values::scan_collect_values_request::ScanCollectValuesRequest, element_scan::element_scan_request::ElementScanRequest,
            new::scan_new_request::ScanNewRequest, reset::scan_reset_request::ScanResetRequest,
        },
    },
    dependency_injection::dependency::Dependency,
    structures::{
        data_types::{built_in_types::i32::data_type_i32::DataTypeI32, data_type_ref::DataTypeRef},
        data_values::anonymous_value_string_format::AnonymousValueStringFormat,
        scanning::{
            comparisons::{scan_compare_type::ScanCompareType, scan_compare_type_immediate::ScanCompareTypeImmediate},
            constraints::anonymous_scan_constraint::AnonymousScanConstraint,
        },
    },
};
use squalr_engine_session::engine_unprivileged_state::EngineUnprivilegedState;
use std::sync::Arc;

#[derive(Clone)]
pub struct ElementScannerViewData {
    pub selected_data_type: DataTypeRef,
    pub active_display_format: AnonymousValueStringFormat,
    pub view_state: ElementScannerViewState,
    pub scan_values_and_constraints: Vec<ElementScannerValueViewData>,
}

impl ElementScannerViewData {
    const MAX_CONSTRAINTS: usize = 5;

    pub fn new() -> Self {
        Self {
            selected_data_type: DataTypeRef::new(DataTypeI32::get_data_type_id()),
            active_display_format: AnonymousValueStringFormat::Decimal,
            view_state: ElementScannerViewState::NoResults,
            scan_values_and_constraints: vec![ElementScannerValueViewData::new(Self::create_menu_id(0))],
        }
    }

    pub fn reset_scan(
        element_scanner_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    ) {
        let element_scanner_view_data_view_state = {
            match element_scanner_view_data.read("Element scanner view data reset scan") {
                Some(element_scanner_view_data) => element_scanner_view_data.view_state,
                None => return,
            }
        };

        match element_scanner_view_data_view_state {
            ElementScannerViewState::ScanInProgress => {
                return;
            }
            ElementScannerViewState::NoResults | ElementScannerViewState::HasResults => {}
        }

        let scan_reset_request = ScanResetRequest {};

        scan_reset_request.send(&engine_unprivileged_state, move |scan_reset_response| {
            if scan_reset_response.success {
                match element_scanner_view_data.write("Element scanner view data reset scan response") {
                    Some(mut element_scanner_view_data) => {
                        element_scanner_view_data.view_state = ElementScannerViewState::NoResults;
                    }
                    None => {}
                }
            }
        });
    }

    pub fn collect_values(engine_unprivileged_state: Arc<EngineUnprivilegedState>) {
        let collect_values_request = ScanCollectValuesRequest {};

        collect_values_request.send(&engine_unprivileged_state, |_scan_collect_values_response| {});
    }

    pub fn start_scan(
        element_scanner_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    ) {
        let element_scanner_view_data_view_state = {
            match element_scanner_view_data.read("Element scanner view data start scan") {
                Some(element_scanner_view_data) => element_scanner_view_data.view_state,
                None => return,
            }
        };

        match element_scanner_view_data_view_state {
            ElementScannerViewState::HasResults => {
                Self::start_next_scan(element_scanner_view_data, engine_unprivileged_state);
            }
            ElementScannerViewState::NoResults => {
                Self::new_scan(element_scanner_view_data, engine_unprivileged_state);
            }
            ElementScannerViewState::ScanInProgress => {
                log::error!("Cannot start a new scan while a scan is in progress.");
            }
        };
    }

    fn new_scan(
        element_scanner_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    ) {
        let engine_unprivileged_state_clone = engine_unprivileged_state.clone();
        let element_scanner_view_data = element_scanner_view_data.clone();
        let scan_new_request = ScanNewRequest {};

        // Start a new scan, and recurse to start the scan once the new scan is made.
        scan_new_request.send(&engine_unprivileged_state, move |_scan_new_response| {
            Self::start_next_scan(element_scanner_view_data, engine_unprivileged_state_clone);
        });
    }

    fn start_next_scan(
        element_scanner_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    ) {
        let element_scanner_view_data_clone = element_scanner_view_data.clone();
        let mut element_scanner_view_data = {
            match element_scanner_view_data.write("Element scanner view data start next scan") {
                Some(element_scanner_view_data) => element_scanner_view_data,
                None => return,
            }
        };
        let data_type_refs = vec![element_scanner_view_data.selected_data_type.clone()];
        let scan_constraints = element_scanner_view_data
            .scan_values_and_constraints
            .iter()
            .map(|scan_value_and_constraint| {
                AnonymousScanConstraint::new(
                    scan_value_and_constraint.selected_scan_compare_type,
                    Some(scan_value_and_constraint.current_scan_value.clone()),
                )
            })
            .collect();
        let element_scan_request = ElementScanRequest {
            scan_constraints,
            data_type_refs,
        };

        element_scanner_view_data.view_state = ElementScannerViewState::ScanInProgress;

        drop(element_scanner_view_data);

        element_scan_request.send(&engine_unprivileged_state, move |scan_execute_response| {
            // JIRA: We actually need to wait for the task to complete, which can be tricky with our request/response architecture.
            // For now we just set it immediately to avoid being stuck in in progress state.
            // JIRA: Use scan_execute_response.scan_results_metadata.
            match element_scanner_view_data_clone.write("Element scanner view data start next scan response") {
                Some(mut element_scanner_view_data) => {
                    element_scanner_view_data.view_state = ElementScannerViewState::HasResults;
                }
                None => {}
            }
        });
    }

    pub fn add_constraint(element_scanner_view_data: Dependency<Self>) {
        let mut element_scanner_view_data = match element_scanner_view_data.write("Element scanner view data add constraint") {
            Some(element_scanner_view_data) => element_scanner_view_data,
            None => return,
        };

        let next_index = element_scanner_view_data.scan_values_and_constraints.len();

        if next_index >= Self::MAX_CONSTRAINTS {
            return;
        }

        // If creating the 2nd constraint, <= is the most common constraint, so default to that for a better UX.
        if next_index == 1 {
            element_scanner_view_data
                .scan_values_and_constraints
                .push(ElementScannerValueViewData {
                    selected_scan_compare_type: ScanCompareType::Immediate(ScanCompareTypeImmediate::LessThanOrEqual),
                    ..ElementScannerValueViewData::new(Self::create_menu_id(next_index))
                });
        } else {
            element_scanner_view_data
                .scan_values_and_constraints
                .push(ElementScannerValueViewData::new(Self::create_menu_id(next_index)));
        }
    }

    pub fn remove_constraint(
        element_scanner_view_data: Dependency<Self>,
        index: usize,
    ) {
        let mut element_scanner_view_data = match element_scanner_view_data.write("Element scanner view data remove constraint") {
            Some(element_scanner_view_data) => element_scanner_view_data,
            None => return,
        };

        if index <= 0 {
            return;
        }

        element_scanner_view_data
            .scan_values_and_constraints
            .remove(index);
    }

    fn create_menu_id(index: usize) -> String {
        format!("element_scanner_data_type_selector_{}", index)
    }
}
