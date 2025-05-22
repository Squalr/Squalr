use crate::DataValueView;
use crate::MainWindowView;
use crate::MemoryAlignmentView;
use crate::ScanConstraintTypeView;
use crate::ScannerViewModelBindings;
use crate::ValueCollectorViewModelBindings;
use crate::view_models::scanners::scan_constraint_converter::ScanConstraintConverter;
use crate::view_models::settings::memory_alignment_converter::MemoryAlignmentConverter;
use slint::ComponentHandle;
use slint_mvvm::convert_from_view_data::ConvertFromViewData;
use slint_mvvm::view_binding::ViewBinding;
use slint_mvvm_macros::create_view_bindings;
use squalr_engine::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use squalr_engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::commands::scan::collect_values::scan_collect_values_request::ScanCollectValuesRequest;
use squalr_engine_api::commands::scan::execute::scan_execute_request::ScanExecuteRequest;
use squalr_engine_api::commands::scan::new::scan_new_request::ScanNewRequest;
use squalr_engine_api::commands::scan::reset::scan_reset_request::ScanResetRequest;
use squalr_engine_api::dependency_injection::dependency_container::DependencyContainer;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_values::anonymous_value::AnonymousValue;
use squalr_engine_api::structures::memory::memory_alignment::MemoryAlignment;
use squalr_engine_api::structures::scanning::data_type_and_alignment::DataTypeAndAlignment;
use squalr_engine_api::structures::scanning::memory_read_mode::MemoryReadMode;
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Copy, Clone, PartialEq)]
enum ScanViewModelState {
    NoResults,
    ScanInProgress,
    HasResults,
}

pub struct ScannerViewModel {
    view_binding: Arc<ViewBinding<MainWindowView>>,
    engine_execution_context: Arc<EngineExecutionContext>,
    scan_view_model_state: RwLock<ScanViewModelState>,
}

impl ScannerViewModel {
    pub fn register(dependency_container: &DependencyContainer) {
        dependency_container.resolve_all(Self::on_dependencies_resolved);
    }

    fn on_dependencies_resolved(
        dependency_container: DependencyContainer,
        (view_binding, engine_execution_context): (Arc<ViewBinding<MainWindowView>>, Arc<EngineExecutionContext>),
    ) {
        let view_model = Arc::new(ScannerViewModel {
            view_binding: view_binding.clone(),
            engine_execution_context: engine_execution_context.clone(),
            scan_view_model_state: RwLock::new(ScanViewModelState::NoResults),
        });

        {
            let view_model = view_model.clone();

            create_view_bindings!(view_model.view_binding, {
                ScannerViewModelBindings => {
                    on_reset_scan() -> [view_model] -> Self::on_reset_scan,
                    on_start_scan(data_value: DataValueView, memory_alignment: MemoryAlignmentView, scan_constraint: ScanConstraintTypeView) -> [view_model] -> Self::on_start_scan,
                },
                ValueCollectorViewModelBindings => {
                    on_collect_values() -> [view_model] -> Self::on_collect_values,
                },
            });
        }

        dependency_container.register::<ScannerViewModel>(view_model);
    }

    fn on_reset_scan(view_model: Arc<ScannerViewModel>) {
        let scan_reset_request = ScanResetRequest {};
        let engine_execution_context = &view_model.engine_execution_context;
        let view_model = view_model.clone();

        scan_reset_request.send(engine_execution_context, move |scan_reset_response| {
            let scan_view_model_state = &view_model.scan_view_model_state;

            if scan_reset_response.success {
                if let Ok(mut scan_view_model_state) = scan_view_model_state.write() {
                    *scan_view_model_state = ScanViewModelState::NoResults;
                }
            }
        });
    }

    fn on_start_scan(
        view_model: Arc<ScannerViewModel>,
        data_value: DataValueView,
        memory_alignment_view: MemoryAlignmentView,
        scan_constraint: ScanConstraintTypeView,
    ) {
        let scan_view_model_state = &view_model.scan_view_model_state;

        let scan_view_model_state_value = {
            *match scan_view_model_state.read() {
                Ok(guard) => guard,
                Err(err) => {
                    log::error!("Failed to acquire UI state lock to start scan: {}", err);
                    return;
                }
            }
        };

        let scan_value = data_value.display_value.to_string();
        let is_value_hex = data_value.is_value_hex;
        let data_type_id = data_value.data_type_ref.data_type_id.to_string();

        let scan_value = AnonymousValue::new_string(&scan_value, is_value_hex);
        let memory_alignment = MemoryAlignmentConverter {}.convert_from_view_data(&memory_alignment_view);
        let data_type_ref = DataTypeRef::new_from_anonymous_value(&data_type_id, &scan_value);

        match scan_view_model_state_value {
            ScanViewModelState::HasResults => {
                Self::start_scan(view_model, data_type_ref, memory_alignment, scan_constraint, scan_value);
            }
            ScanViewModelState::NoResults => {
                Self::new_scan(view_model, data_type_ref, memory_alignment, scan_constraint, scan_value);
            }
            ScanViewModelState::ScanInProgress => {
                log::error!("Cannot start a new scan while a scan is in progress.");
            }
        };
    }

    fn on_collect_values(view_model: Arc<ScannerViewModel>) {
        let collect_values_request = ScanCollectValuesRequest {};

        collect_values_request.send(&view_model.engine_execution_context, |_scan_collect_values_response| {});
    }

    fn new_scan(
        view_model: Arc<ScannerViewModel>,
        data_type_ref: DataTypeRef,
        memory_alignment: MemoryAlignment,
        scan_constraint: ScanConstraintTypeView,
        scan_value: AnonymousValue,
    ) {
        let engine_execution_context = &view_model.engine_execution_context;
        let view_model = view_model.clone();
        let scan_new_request = ScanNewRequest {};

        // Start a new scan, and recurse to start the scan once the new scan is made.
        scan_new_request.send(engine_execution_context, move |_scan_new_response| {
            Self::start_scan(view_model, data_type_ref, memory_alignment, scan_constraint, scan_value);
        });
    }

    fn start_scan(
        view_model: Arc<ScannerViewModel>,
        data_type_ref: DataTypeRef,
        memory_alignment: MemoryAlignment,
        scan_constraint: ScanConstraintTypeView,
        scan_value: AnonymousValue,
    ) {
        let engine_execution_context = &view_model.engine_execution_context;
        let view_model = view_model.clone();
        let scan_execute_request = ScanExecuteRequest {
            scan_value: Some(scan_value),
            data_types_and_alignments: vec![DataTypeAndAlignment::new(data_type_ref, Some(memory_alignment))],
            compare_type: ScanConstraintConverter::new().convert_from_view_data(&scan_constraint),
            memory_read_mode: MemoryReadMode::ReadBeforeScan, // JIRA: Setting for this
        };

        scan_execute_request.send(&engine_execution_context, move |scan_execute_response| {
            let scan_view_model_state = &view_model.scan_view_model_state;

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
