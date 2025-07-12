use crate::MainWindowView;
use crate::StructViewerViewModelBindings;
use crate::ValuedStructViewData;
use crate::comparers::valued_struct_comparer::ValuedStructComparer;
use crate::converters::valued_struct_converter::ValuedStructConverter;
use slint::ComponentHandle;
use slint::SharedString;
use slint_mvvm::view_binding::ViewBinding;
use slint_mvvm::view_collection_binding::ViewCollectionBinding;
use slint_mvvm_macros::create_view_bindings;
use slint_mvvm_macros::create_view_model_collection;
use squalr_engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::dependency_injection::dependency_container::DependencyContainer;
use squalr_engine_api::structures::structs::valued_struct::ValuedStruct;
use std::sync::Arc;

pub struct StructViewerViewModel {
    view_binding: Arc<ViewBinding<MainWindowView>>,
    structs_under_view_collection: ViewCollectionBinding<ValuedStructViewData, ValuedStruct, MainWindowView>,
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
        // Create a binding that allows us to easily update the view's selected properties.
        let structs_under_view_collection = create_view_model_collection!(
            view_binding -> MainWindowView,
            StructViewerViewModelBindings -> { set_structs_under_view, get_structs_under_view },
            ValuedStructConverter -> [],
            ValuedStructComparer -> [],
        );

        let view_model = Arc::new(StructViewerViewModel {
            view_binding: view_binding.clone(),
            structs_under_view_collection: structs_under_view_collection.clone(),
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
        // JIRA: This is a hack, figure out why it is needed and fix it.
        self.structs_under_view_collection.update_from_source(vec![]);

        // JIRA: FIXME
        // self.structs_under_view_collection
        // .update_from_source(ValuedStruct::combine_property_collections(&selected_properties));
    }

    fn on_set_property_value(
        view_model: Arc<StructViewerViewModel>,
        new_value: SharedString,
    ) {
    }
}
