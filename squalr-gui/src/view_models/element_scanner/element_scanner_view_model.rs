use crate::DisplayValueViewData;
use crate::ElementScannerViewModelBindings;
use crate::MainWindowView;
use crate::ScanConstraintTypeView;
use crate::ValueCollectorViewModelBindings;
use crate::converters::data_value_converter::DataValueConverter;
use crate::converters::display_value_converter::DisplayValueConverter;
use crate::converters::scan_constraint_converter::ScanConstraintConverter;
use squalr_engine_api::commands::engine_command_request::EngineCommandRequest;
use squalr_engine_api::commands::scan::collect_values::scan_collect_values_request::ScanCollectValuesRequest;
use squalr_engine_api::commands::scan::element_scan::element_scan_request::ElementScanRequest;
use squalr_engine_api::commands::scan::new::scan_new_request::ScanNewRequest;
use squalr_engine_api::commands::scan::reset::scan_reset_request::ScanResetRequest;
use squalr_engine_api::dependency_injection::dependency_container::DependencyContainer;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::data_types::built_in_types::i32::data_type_i32::DataTypeI32;
use squalr_engine_api::structures::data_values::anonymous_value::AnonymousValue;
use slint::ComponentHandle;
use slint::Model;
use slint::ModelRc;
use slint::SharedString;
use slint_mvvm::convert_from_view_data::ConvertFromViewData;
use slint_mvvm::convert_to_view_data::ConvertToViewData;
use slint_mvvm::view_binding::ViewBinding;
use slint_mvvm_macros::create_view_bindings;
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Copy, Clone, PartialEq)]
enum ScanViewModelState {
    NoResults,
    ScanInProgress,
    HasResults,
}

pub struct ElementScannerViewModel {
    view_binding: Arc<ViewBinding<MainWindowView>>,
    engine_execution_context: Arc<EngineExecutionContext>,
    scan_view_model_state: RwLock<ScanViewModelState>,
}

impl ElementScannerViewModel {
    pub fn register(dependency_container: &DependencyContainer) {
        dependency_container.resolve_all(Self::on_dependencies_resolved);
    }

    fn on_dependencies_resolved(
        dependency_container: DependencyContainer,
        (view_binding, engine_execution_context): (Arc<ViewBinding<MainWindowView>>, Arc<EngineExecutionContext>),
    ) {
        let view_model = Arc::new(ElementScannerViewModel {
            view_binding: view_binding.clone(),
            engine_execution_context: engine_execution_context.clone(),
            scan_view_model_state: RwLock::new(ScanViewModelState::NoResults),
        });

        {
            let view_model = view_model.clone();

            create_view_bindings!(view_model.view_binding, {
                ElementScannerViewModelBindings => {
                    on_reset_scan() -> [view_model] -> Self::on_reset_scan,
                    on_start_scan(scan_value: SharedString, data_type_ids: ModelRc<SharedString>, display_value: DisplayValueViewData, scan_constraint: ScanConstraintTypeView) -> [view_model] -> Self::on_start_scan,
                },
                ValueCollectorViewModelBindings => {
                    on_collect_values() -> [view_model] -> Self::on_collect_values,
                },
            });
        }

        Self::set_default_selection(view_model.clone());

        dependency_container.register::<ElementScannerViewModel>(view_model);
    }

    fn set_default_selection(view_model: Arc<ElementScannerViewModel>) {
        view_model
            .view_binding
            .execute_on_ui_thread(move |main_window_view, _view_binding| {
                let scanner_view_model_bindings = main_window_view.global::<ElementScannerViewModelBindings>();
                let data_value = DataTypeI32::get_value_from_primitive(0);

                scanner_view_model_bindings.set_active_data_value(DataValueConverter {}.convert_to_view_data(&data_value));
                scanner_view_model_bindings.set_active_icon_id(DataTypeI32::get_icon_id().into());
            });
    }

    fn on_reset_scan(view_model: Arc<ElementScannerViewModel>) {
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
        view_model: Arc<ElementScannerViewModel>,
        scan_value: SharedString,
        data_type_ids: ModelRc<SharedString>,
        display_value: DisplayValueViewData,
        scan_constraint: ScanConstraintTypeView,
    ) {
        let scan_view_model_state = &view_model.scan_view_model_state;

        let scan_view_model_state_value = {
            *match scan_view_model_state.read() {
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

        match scan_view_model_state_value {
            ScanViewModelState::HasResults => {
                Self::start_scan(view_model, scan_constraint, data_type_ids, anonymous_value);
            }
            ScanViewModelState::NoResults => {
                Self::new_scan(view_model, scan_constraint, data_type_ids, anonymous_value);
            }
            ScanViewModelState::ScanInProgress => {
                log::error!("Cannot start a new scan while a scan is in progress.");
            }
        };
    }

    fn on_collect_values(view_model: Arc<ElementScannerViewModel>) {
        let collect_values_request = ScanCollectValuesRequest {};

        collect_values_request.send(&view_model.engine_execution_context, |_scan_collect_values_response| {});
    }

    fn new_scan(
        view_model: Arc<ElementScannerViewModel>,
        scan_constraint: ScanConstraintTypeView,
        data_type_ids: Vec<String>,
        anonymous_value: AnonymousValue,
    ) {
        let engine_execution_context = &view_model.engine_execution_context;
        let view_model = view_model.clone();
        let scan_new_request = ScanNewRequest {};

        // Start a new scan, and recurse to start the scan once the new scan is made.
        scan_new_request.send(engine_execution_context, move |_scan_new_response| {
            Self::start_scan(view_model, scan_constraint, data_type_ids, anonymous_value);
        });
    }

    fn start_scan(
        view_model: Arc<ElementScannerViewModel>,
        scan_constraint: ScanConstraintTypeView,
        data_type_ids: Vec<String>,
        anonymous_value: AnonymousValue,
    ) {
        let engine_execution_context = &view_model.engine_execution_context;
        let view_model = view_model.clone();
        let element_scan_request = ElementScanRequest {
            scan_value: Some(anonymous_value),
            data_type_ids: data_type_ids,
            compare_type: ScanConstraintConverter::new().convert_from_view_data(&scan_constraint),
        };

        element_scan_request.send(&engine_execution_context, move |scan_execute_response| {
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
