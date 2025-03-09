use crate::DataTypeView;
use crate::MainWindowView;
use crate::ScanConstraintTypeView;
use crate::ScannerViewModelBindings;
use crate::ValueCollectorViewModelBindings;
use crate::view_models::scanners::scan_constraint_converter::ScanConstraintConverter;
use slint::ComponentHandle;
use slint::SharedString;
use slint_mvvm::view_binding::ViewBinding;
use slint_mvvm::view_data_converter::ViewDataConverter;
use slint_mvvm_macros::create_view_bindings;
use squalr_engine::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use squalr_engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::commands::scan::collect_values::scan_collect_values_request::ScanCollectValuesRequest;
use squalr_engine_api::commands::scan::execute::scan_execute_request::ScanExecuteRequest;
use squalr_engine_api::commands::scan::new::scan_new_request::ScanNewRequest;
use squalr_engine_api::commands::scan::reset::scan_reset_request::ScanResetRequest;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_values::anonymous_value::AnonymousValue;
use squalr_engine_api::structures::memory::memory_alignment::MemoryAlignment;
use squalr_engine_api::structures::scanning::scan_memory_read_mode::ScanMemoryReadMode;
use squalr_engine_api::structures::scanning::scan_parameters_local::ScanParametersLocal;
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Copy, Clone, PartialEq)]
enum ScanViewModelState {
    NoResults,
    ScanInProgress,
    HasResults,
}

pub struct ScannerViewModel {
    _view_binding: ViewBinding<MainWindowView>,
    _engine_execution_context: Arc<EngineExecutionContext>,
    scan_view_model_state: Arc<RwLock<ScanViewModelState>>,
}

impl ScannerViewModel {
    pub fn new(
        view_binding: ViewBinding<MainWindowView>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) -> Self {
        let view: ScannerViewModel = ScannerViewModel {
            _view_binding: view_binding.clone(),
            _engine_execution_context: engine_execution_context.clone(),
            scan_view_model_state: Arc::new(RwLock::new(ScanViewModelState::NoResults)),
        };

        let scan_view_model_state = view.scan_view_model_state.clone();

        create_view_bindings!(view_binding, {
            ScannerViewModelBindings => {
                on_reset_scan() -> [engine_execution_context, scan_view_model_state] -> Self::on_reset_scan,
                on_start_scan(data_type: DataTypeView, scan_constraint: ScanConstraintTypeView, scan_value: SharedString) -> [engine_execution_context, scan_view_model_state] -> Self::on_start_scan,
            },
            ValueCollectorViewModelBindings => {
                on_collect_values() -> [engine_execution_context] -> Self::on_collect_values,
            },
        });

        view
    }

    fn on_reset_scan(
        engine_execution_context: Arc<EngineExecutionContext>,
        scan_view_model_state: Arc<RwLock<ScanViewModelState>>,
    ) {
        let scan_reset_request = ScanResetRequest {};

        scan_reset_request.send(&engine_execution_context, move |scan_reset_response| {
            if scan_reset_response.success {
                if let Ok(mut scan_view_model_state) = scan_view_model_state.write() {
                    *scan_view_model_state = ScanViewModelState::NoResults;
                }
            }
        });
    }

    fn on_start_scan(
        engine_execution_context: Arc<EngineExecutionContext>,
        scan_view_model_state: Arc<RwLock<ScanViewModelState>>,
        data_type_view: DataTypeView,
        scan_constraint: ScanConstraintTypeView,
        scan_value: SharedString,
    ) {
        let scan_view_model_state_value = {
            *match scan_view_model_state.read() {
                Ok(guard) => guard,
                Err(err) => {
                    log::error!("Failed to acquire UI state lock to start scan: {}", err);
                    return;
                }
            }
        };

        match scan_view_model_state_value {
            ScanViewModelState::HasResults => {
                Self::start_scan(engine_execution_context, scan_view_model_state, scan_constraint, scan_value.into());
            }
            ScanViewModelState::NoResults => match DataTypeRef::new(&data_type_view.data_type.to_string()) {
                Some(data_type) => Self::new_scan(engine_execution_context, scan_view_model_state, data_type, scan_constraint, scan_value.into()),
                None => log::error!("Failed to create data type for new scan."),
            },
            ScanViewModelState::ScanInProgress => {
                log::error!("Cannot start a new scan while a scan is in progress.");
            }
        };
    }

    fn on_collect_values(engine_execution_context: Arc<EngineExecutionContext>) {
        let collect_values_request = ScanCollectValuesRequest {};

        collect_values_request.send(&engine_execution_context, |_scan_collect_values_response| {});
    }

    fn new_scan(
        engine_execution_context: Arc<EngineExecutionContext>,
        scan_view_model_state: Arc<RwLock<ScanViewModelState>>,
        data_type: DataTypeRef,
        scan_constraint: ScanConstraintTypeView,
        scan_value: String,
    ) {
        let memory_alignment = Some(MemoryAlignment::Alignment4); // JIRA: Pull from settings
        let scan_parameters_local = vec![ScanParametersLocal::new(data_type, memory_alignment)];
        let scan_new_request = ScanNewRequest { scan_parameters_local };

        // Captured variables for scan once we create it.
        let scan_value = scan_value.into();
        let engine_execution_context_clone = engine_execution_context.clone();
        let scan_view_model_state = scan_view_model_state.clone();

        // Start a new scan, and recurse to start the scan once the new scan is made.
        scan_new_request.send(&engine_execution_context, move |_scan_new_response| {
            Self::start_scan(engine_execution_context_clone, scan_view_model_state, scan_constraint, scan_value);
        });
    }

    fn start_scan(
        engine_execution_context: Arc<EngineExecutionContext>,
        scan_view_model_state: Arc<RwLock<ScanViewModelState>>,
        scan_constraint: ScanConstraintTypeView,
        scan_value: String,
    ) {
        let scan_value = AnonymousValue::new(&scan_value);
        let scan_execute_request = ScanExecuteRequest {
            scan_value: Some(scan_value),
            compare_type: ScanConstraintConverter::new().convert_from_view_data(&scan_constraint),
            memory_read_mode: ScanMemoryReadMode::ReadBeforeScan, // JIRA: Setting for this
        };

        scan_execute_request.send(&engine_execution_context, move |scan_execute_response| {
            if let Ok(mut scan_view_model_state) = scan_view_model_state.write() {
                *scan_view_model_state = ScanViewModelState::ScanInProgress;
            }
            // JIRA: We actually need to wait for the task to complete, which can be tricky with our request/response architecture.
            // For now we just set it immediately to avoid being stuck in in progress state.
            if let Ok(mut scan_view_model_state) = scan_view_model_state.write() {
                *scan_view_model_state = ScanViewModelState::HasResults;
            }
        });
    }
}
