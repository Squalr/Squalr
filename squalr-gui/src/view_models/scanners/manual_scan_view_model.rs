use crate::DataTypeView;
use crate::MainWindowView;
use crate::ManualScanViewModelBindings;
use crate::ScanConstraintTypeView;
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
use squalr_engine_api::commands::scan::hybrid::scan_hybrid_request::ScanHybridRequest;
use squalr_engine_api::commands::scan::new::scan_new_request::ScanNewRequest;
use squalr_engine_common::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_common::structures::data_values::anonymous_value::AnonymousValue;
use squalr_engine_common::structures::scanning::scan_parameters_local::ScanParametersLocal;
use std::sync::Arc;

pub struct ManualScanViewModel {
    _view_binding: ViewBinding<MainWindowView>,
    _engine_execution_context: Arc<EngineExecutionContext>,
}

impl ManualScanViewModel {
    pub fn new(
        view_binding: ViewBinding<MainWindowView>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) -> Self {
        let view: ManualScanViewModel = ManualScanViewModel {
            _view_binding: view_binding.clone(),
            _engine_execution_context: engine_execution_context.clone(),
        };

        create_view_bindings!(view_binding, {
            ManualScanViewModelBindings => {
                on_new_scan(data_type: DataTypeView) -> [engine_execution_context] -> Self::on_new_scan,
                on_start_scan(scan_constraint: ScanConstraintTypeView, scan_value: SharedString) -> [engine_execution_context] -> Self::on_start_scan,
            },
            ValueCollectorViewModelBindings => {
                on_collect_values() -> [engine_execution_context] -> Self::on_collect_values,
            },
        });

        view
    }

    fn on_new_scan(
        engine_execution_context: Arc<EngineExecutionContext>,
        data_type_view: DataTypeView,
    ) {
        match DataTypeRef::new(&data_type_view.data_type.to_string()) {
            Some(data_type) => {
                let memory_alignment = None; // JIRA: TODO
                let scan_parameters_local = vec![ScanParametersLocal::new(data_type, memory_alignment)];
                let scan_new_request = ScanNewRequest { scan_parameters_local };

                scan_new_request.send(&engine_execution_context, |_scan_new_response| {});
            }
            None => log::error!("Failed to create data type for new scan."),
        }
    }

    fn on_start_scan(
        engine_execution_context: Arc<EngineExecutionContext>,
        scan_constraint: ScanConstraintTypeView,
        scan_value: SharedString,
    ) {
        let scan_value = AnonymousValue::new(&scan_value.to_string());
        let scan_hybrid_request = ScanHybridRequest {
            scan_value: Some(scan_value),
            compare_type: ScanConstraintConverter::new().convert_from_view_data(&scan_constraint),
        };

        scan_hybrid_request.send(&engine_execution_context, |_scan_hybrid_response| {});
    }

    fn on_collect_values(engine_execution_context: Arc<EngineExecutionContext>) {
        let collect_values_request = ScanCollectValuesRequest {};

        collect_values_request.send(&engine_execution_context, |_scan_collect_values_response| {});
    }
}
