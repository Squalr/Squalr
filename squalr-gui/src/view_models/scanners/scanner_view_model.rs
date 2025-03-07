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
use squalr_engine::command_executors::engine_request_executor::EngineRequestExecutor;
use squalr_engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::commands::scan::collect_values::scan_collect_values_request::ScanCollectValuesRequest;
use squalr_engine_api::commands::scan::execute::scan_execute_request::ScanExecuteRequest;
use squalr_engine_api::commands::scan::new::scan_new_request::ScanNewRequest;
use squalr_engine_api::commands::scan::reset::scan_reset_request::ScanResetRequest;
use squalr_engine_common::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_common::structures::data_values::anonymous_value::AnonymousValue;
use squalr_engine_common::structures::scanning::memory_read_mode::MemoryReadMode;
use squalr_engine_common::structures::scanning::scan_parameters_local::ScanParametersLocal;
use std::sync::Arc;

pub struct ScannerViewModel {
    _view_binding: ViewBinding<MainWindowView>,
    _engine_execution_context: Arc<EngineExecutionContext>,
}

impl ScannerViewModel {
    pub fn new(
        view_binding: ViewBinding<MainWindowView>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) -> Self {
        let view: ScannerViewModel = ScannerViewModel {
            _view_binding: view_binding.clone(),
            _engine_execution_context: engine_execution_context.clone(),
        };

        create_view_bindings!(view_binding, {
            ScannerViewModelBindings => {
                on_reset_scan() -> [engine_execution_context] -> Self::on_reset_scan,
                on_start_scan(data_type: DataTypeView, scan_constraint: ScanConstraintTypeView, scan_value: SharedString) -> [engine_execution_context] -> Self::on_start_scan,
            },
            ValueCollectorViewModelBindings => {
                on_collect_values() -> [engine_execution_context] -> Self::on_collect_values,
            },
        });

        view
    }

    fn on_reset_scan(engine_execution_context: Arc<EngineExecutionContext>) {
        let scan_reset_request = ScanResetRequest {};

        scan_reset_request.send(&engine_execution_context, |scan_reset_response| {
            if scan_reset_response.success {
                //
            }
        });
    }

    fn on_start_scan(
        engine_execution_context: Arc<EngineExecutionContext>,
        data_type_view: DataTypeView,
        scan_constraint: ScanConstraintTypeView,
        scan_value: SharedString,
    ) {
        let scan_value = scan_value.to_string();

        match DataTypeRef::new(&data_type_view.data_type.to_string()) {
            Some(data_type) => {
                let memory_alignment = None; // JIRA: Pull from settings
                let scan_parameters_local = vec![ScanParametersLocal::new(data_type, memory_alignment)];
                let scan_new_request = ScanNewRequest { scan_parameters_local };

                let engine_execution_context_clone = engine_execution_context.clone();

                scan_new_request.send(&engine_execution_context, move |_scan_new_response| {
                    let scan_value = AnonymousValue::new(&scan_value);
                    let scan_execute_request = ScanExecuteRequest {
                        scan_value: Some(scan_value),
                        compare_type: ScanConstraintConverter::new().convert_from_view_data(&scan_constraint),
                        memory_read_mode: MemoryReadMode::ReadBeforeScan, // JIRA: Setting for this
                    };

                    scan_execute_request.send(&engine_execution_context_clone, |_scan_execute_response| {});
                });
            }
            None => log::error!("Failed to create data type for new scan."),
        }
    }

    fn on_collect_values(engine_execution_context: Arc<EngineExecutionContext>) {
        let collect_values_request = ScanCollectValuesRequest {};

        collect_values_request.send(&engine_execution_context, |_scan_collect_values_response| {});
    }
}
