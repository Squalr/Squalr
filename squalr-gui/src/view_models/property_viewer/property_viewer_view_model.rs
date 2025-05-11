use crate::MainWindowView;
use crate::PropertyEntryViewData;
use crate::PropertyViewerViewModelBindings;
use crate::view_models::property_viewer::property_comparer::PropertyComparer;
use crate::view_models::property_viewer::property_converter::PropertyConverter;
use slint::ComponentHandle;
use slint::SharedString;
use slint_mvvm::view_binding::ViewBinding;
use slint_mvvm::view_collection_binding::ViewCollectionBinding;
use slint_mvvm_macros::create_view_bindings;
use slint_mvvm_macros::create_view_model_collection;
use squalr_engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::properties::property::Property;
use std::sync::Arc;

pub struct PropertyViewerViewModel {
    view_binding: ViewBinding<MainWindowView>,
    selected_properties_collection: ViewCollectionBinding<PropertyEntryViewData, Property, MainWindowView>,
    engine_execution_context: Arc<EngineExecutionContext>,
}

impl PropertyViewerViewModel {
    pub fn new(
        view_binding: ViewBinding<MainWindowView>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) -> Arc<Self> {
        // Create a binding that allows us to easily update the view's selected properties.
        let selected_properties_collection = create_view_model_collection!(
            view_binding -> MainWindowView,
            PropertyViewerViewModelBindings -> { set_selected_properties, get_selected_properties },
            PropertyConverter -> [],
            PropertyComparer -> [],
        );

        let view = Arc::new(PropertyViewerViewModel {
            view_binding: view_binding.clone(),
            selected_properties_collection: selected_properties_collection.clone(),
            engine_execution_context: engine_execution_context.clone(),
        });

        // Route all view bindings to Rust.
        create_view_bindings!(view_binding, {
            PropertyViewerViewModelBindings => {
                on_set_property_value(new_value: SharedString) -> [selected_properties_collection, engine_execution_context] -> Self::on_set_property_value
            }
        });

        view
    }

    pub fn set_selected_properties(
        &self,
        selected_properties: Vec<Property>,
    ) {
        self.selected_properties_collection
            .update_from_source(selected_properties);
    }

    fn on_set_property_value(
        selected_properties_collection: ViewCollectionBinding<PropertyEntryViewData, Property, MainWindowView>,
        engine_execution_context: Arc<EngineExecutionContext>,
        new_value: SharedString,
    ) {
    }
}
