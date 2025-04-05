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
use squalr_engine_api::structures::data_types::data_type_meta_data::DataTypeMetaData;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_values::anonymous_value::AnonymousValue;
use squalr_engine_api::structures::memory::memory_alignment::MemoryAlignment;
use squalr_engine_api::structures::scanning::parameters::user_scan_parameters_local::UserScanParametersLocal;
use squalr_engine_api::structures::scanning::scan_memory_read_mode::ScanMemoryReadMode;
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Copy, Clone, PartialEq)]
enum ScanViewModelState {
    NoResults,
    ScanInProgress,
    HasResults,
}

pub struct ScannerViewModel {
    view_binding: ViewBinding<MainWindowView>,
    engine_execution_context: Arc<EngineExecutionContext>,
    scan_view_model_state: RwLock<ScanViewModelState>,
}

impl ScannerViewModel {
    pub fn new(
        view_binding: ViewBinding<MainWindowView>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) -> Arc<Self> {
        let view = Arc::new(ScannerViewModel {
            view_binding: view_binding.clone(),
            engine_execution_context: engine_execution_context.clone(),
            scan_view_model_state: RwLock::new(ScanViewModelState::NoResults),
        });

        {
            let view = view.clone();

            create_view_bindings!(view.view_binding, {
                ScannerViewModelBindings => {
                    on_reset_scan() -> [view] -> Self::on_reset_scan,
                    on_start_scan(data_type: DataTypeView, scan_constraint: ScanConstraintTypeView, scan_value: SharedString, is_value_hex: bool) -> [view] -> Self::on_start_scan,
                },
                ValueCollectorViewModelBindings => {
                    on_collect_values() -> [view] -> Self::on_collect_values,
                },
            });
        }

        view
    }

    fn on_reset_scan(scanner_view_model: Arc<ScannerViewModel>) {
        let scan_reset_request = ScanResetRequest {};
        let engine_execution_context = &scanner_view_model.engine_execution_context;
        let scanner_view_model = scanner_view_model.clone();

        scan_reset_request.send(engine_execution_context, move |scan_reset_response| {
            let scan_view_model_state = &scanner_view_model.scan_view_model_state;

            if scan_reset_response.success {
                if let Ok(mut scan_view_model_state) = scan_view_model_state.write() {
                    *scan_view_model_state = ScanViewModelState::NoResults;
                }
            }
        });
    }

    fn on_start_scan(
        scanner_view_model: Arc<ScannerViewModel>,
        data_type_view: DataTypeView,
        scan_constraint: ScanConstraintTypeView,
        scan_value: SharedString,
        is_value_hex: bool,
    ) {
        let scan_view_model_state = &scanner_view_model.scan_view_model_state;

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
                Self::start_scan(scanner_view_model, scan_constraint, AnonymousValue::new_string(&scan_value, is_value_hex));
            }
            ScanViewModelState::NoResults => {
                let data_type_id = data_type_view.data_type.to_string();
                let scan_value = AnonymousValue::new_string(&scan_value, is_value_hex);
                let data_type_meta_data = match scan_value.deanonymize_value(&data_type_id) {
                    Ok(value) => DataTypeMetaData::SizedContainer(value.get_size_in_bytes()),
                    Err(_) => DataTypeMetaData::None,
                };
                let data_type = DataTypeRef::new(&data_type_id, data_type_meta_data);

                Self::new_scan(scanner_view_model, data_type, scan_constraint, scan_value);
            }
            ScanViewModelState::ScanInProgress => {
                log::error!("Cannot start a new scan while a scan is in progress.");
            }
        };
    }

    fn on_collect_values(scanner_view_model: Arc<ScannerViewModel>) {
        let collect_values_request = ScanCollectValuesRequest {};

        collect_values_request.send(&scanner_view_model.engine_execution_context, |_scan_collect_values_response| {});
    }

    fn new_scan(
        scanner_view_model: Arc<ScannerViewModel>,
        data_type: DataTypeRef,
        scan_constraint: ScanConstraintTypeView,
        scan_value: AnonymousValue,
    ) {
        let engine_execution_context = &scanner_view_model.engine_execution_context;
        let scanner_view_model = scanner_view_model.clone();
        let memory_alignment = Some(MemoryAlignment::Alignment1); // JIRA: Pull from settings
        let user_scan_parameters_local = vec![UserScanParametersLocal::new(data_type, memory_alignment)];
        let scan_new_request = ScanNewRequest { user_scan_parameters_local };

        // Start a new scan, and recurse to start the scan once the new scan is made.
        scan_new_request.send(engine_execution_context, move |_scan_new_response| {
            Self::start_scan(scanner_view_model, scan_constraint, scan_value);
        });
    }

    fn start_scan(
        scanner_view_model: Arc<ScannerViewModel>,
        scan_constraint: ScanConstraintTypeView,
        scan_value: AnonymousValue,
    ) {
        let engine_execution_context = &scanner_view_model.engine_execution_context;
        let scanner_view_model = scanner_view_model.clone();
        let scan_execute_request = ScanExecuteRequest {
            scan_value: Some(scan_value),
            compare_type: ScanConstraintConverter::new().convert_from_view_data(&scan_constraint),
            memory_read_mode: ScanMemoryReadMode::ReadBeforeScan, // JIRA: Setting for this
        };

        scan_execute_request.send(&engine_execution_context, move |scan_execute_response| {
            let scan_view_model_state = &scanner_view_model.scan_view_model_state;

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
