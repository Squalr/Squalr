use crate::MainWindowView;
use crate::StructViewerViewModelBindings;
use crate::converters::valued_struct_converter::ValuedStructConverter;
use olorin_engine::engine_execution_context::EngineExecutionContext;
use olorin_engine_api::dependency_injection::dependency_container::DependencyContainer;
use olorin_engine_api::structures::structs::valued_struct::ValuedStruct;
use slint::ComponentHandle;
use slint::SharedString;
use slint_mvvm::convert_to_view_data::ConvertToViewData;
use slint_mvvm::view_binding::ViewBinding;
use slint_mvvm_macros::create_view_bindings;
use std::sync::Arc;

pub struct StructViewerViewModel {
    view_binding: Arc<ViewBinding<MainWindowView>>,
    engine_execution_context: Arc<EngineExecutionContext>,
}

impl StructViewerViewModel {
    pub fn register(dependency_container: &DependencyContainer) {
        dependency_container.resolve_all(Self::on_dependencies_resolved);
    }

    fn on_dependencies_resolved(
        dependency_container: DependencyContainer,
        (view_binding, engine_execution_context): (Arc<ViewBinding<MainWindowView>>, Arc<EngineExecutionContext>),
    ) {
        let view_model = Arc::new(StructViewerViewModel {
            view_binding: view_binding.clone(),
            engine_execution_context: engine_execution_context.clone(),
        });

        {
            let view_model = view_model.clone();

            // Route all view bindings to Rust.
            create_view_bindings!(view_binding, {
                StructViewerViewModelBindings => {
                    on_set_property_value(new_value: SharedString) -> [view_model] -> Self::on_set_property_value
                }
            });
        }

        dependency_container.register::<StructViewerViewModel>(view_model);
    }

    pub fn set_selected_structs(
        &self,
        selected_structs: Vec<ValuedStruct>,
    ) {
        let view_binding = &self.view_binding;
        let selected_struct = ValuedStruct::combine_exclusive(&selected_structs);

        view_binding.execute_on_ui_thread(move |main_window_view, _view_binding| {
            let struct_viewer_bindings = main_window_view.global::<StructViewerViewModelBindings>();

            struct_viewer_bindings.set_struct_under_view(ValuedStructConverter {}.convert_to_view_data(&selected_struct))
        });
    }

    fn on_set_property_value(
        view_model: Arc<StructViewerViewModel>,
        new_value: SharedString,
    ) {
    }
}
