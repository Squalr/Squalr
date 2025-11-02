use crate::DataTypeRefViewData;
use crate::DisplayValueViewData;
use crate::MainWindowView;
use crate::StructViewerViewModelBindings;
use crate::converters::data_type_ref_converter::DataTypeRefConverter;
use crate::converters::display_value_converter::DisplayValueConverter;
use crate::converters::valued_struct_converter::ValuedStructConverter;
use crate::view_models::element_scanner::element_scan_results_view_model::ElementScanResultsViewModel;
use crate::view_models::struct_viewer::struct_viewer_domain::StructViewerDomain;
use slint::ComponentHandle;
use slint::SharedString;
use slint_mvvm::convert_from_view_data::ConvertFromViewData;
use slint_mvvm::convert_to_view_data::ConvertToViewData;
use slint_mvvm::view_binding::ViewBinding;
use slint_mvvm_macros::create_view_bindings;
use squalr_engine_api::dependency_injection::dependency_container::DependencyContainer;
use squalr_engine_api::dependency_injection::lazy::Lazy;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::data_values::anonymous_value::AnonymousValue;
use squalr_engine_api::structures::structs::valued_struct::ValuedStruct;
use std::sync::Arc;
use std::sync::RwLock;

pub struct StructViewerViewModel {
    view_binding: Arc<ViewBinding<MainWindowView>>,
    _engine_execution_context: Arc<EngineExecutionContext>,
    element_scan_results_view_model: Lazy<ElementScanResultsViewModel>,
    struct_viewer_domain: RwLock<StructViewerDomain>,
}

impl StructViewerViewModel {
    pub fn register(dependency_container: &DependencyContainer) {
        dependency_container.resolve_all(Self::on_dependencies_resolved);
    }

    fn on_dependencies_resolved(
        dependency_container: DependencyContainer,
        (view_binding, engine_execution_context): (Arc<ViewBinding<MainWindowView>>, Arc<EngineExecutionContext>),
    ) {
        let element_scan_results_view_model = dependency_container.get_dependency::<ElementScanResultsViewModel>();
        let view_model = Arc::new(StructViewerViewModel {
            view_binding: view_binding.clone(),
            _engine_execution_context: engine_execution_context.clone(),
            struct_viewer_domain: RwLock::new(StructViewerDomain::None),
            element_scan_results_view_model,
        });

        {
            let view_model = view_model.clone();

            // Route all view bindings to Rust.
            create_view_bindings!(view_binding, {
                StructViewerViewModelBindings => {
                    on_commit_field_change(field_namespace: SharedString, new_value: SharedString, display_value: DisplayValueViewData, data_type_ref: DataTypeRefViewData) -> [view_model] -> Self::on_commit_field_change
                }
            });
        }

        dependency_container.register::<StructViewerViewModel>(view_model);
    }

    pub fn set_selected_structs(
        &self,
        struct_viewer_domain: StructViewerDomain,
        selected_structs: Vec<ValuedStruct>,
    ) {
        let view_binding = &self.view_binding;
        let selected_struct = ValuedStruct::combine_exclusive(&selected_structs);

        if let Ok(mut struct_viewer_domain_lock) = self.struct_viewer_domain.write() {
            *struct_viewer_domain_lock = struct_viewer_domain
        }

        view_binding.execute_on_ui_thread(move |main_window_view, _view_binding| {
            let struct_viewer_bindings = main_window_view.global::<StructViewerViewModelBindings>();

            struct_viewer_bindings.set_struct_under_view(ValuedStructConverter {}.convert_to_view_data(&selected_struct))
        });
    }

    fn on_commit_field_change(
        view_model: Arc<StructViewerViewModel>,
        field_namespace: SharedString,
        new_value: SharedString,
        display_value: DisplayValueViewData,
        data_type_ref: DataTypeRefViewData,
    ) {
        let data_type_ref = DataTypeRefConverter {}.convert_from_view_data(&data_type_ref);
        let mut display_value = DisplayValueConverter {}.convert_from_view_data(&display_value);

        display_value.set_display_string(new_value.to_string());

        let anonymous_value = AnonymousValue::new(display_value);

        let Ok(struct_viewer_domain_lock) = view_model.struct_viewer_domain.read() else {
            log::error!("Failed to acquire read lock for struct viewer domain.");
            return;
        };

        match *struct_viewer_domain_lock {
            StructViewerDomain::None => {}
            StructViewerDomain::ScanResult => {
                let element_scan_results_view_model = match view_model.element_scan_results_view_model.get() {
                    Ok(element_scan_results_view_model) => element_scan_results_view_model,
                    Err(error) => {
                        log::error!("Error fetching scan results view model: {}", error);
                        return;
                    }
                };

                element_scan_results_view_model.set_selected_scan_results_value(field_namespace.to_string(), anonymous_value);
            }
        }
    }
}
