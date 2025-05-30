use crate::DisplayValueTypeView;
use crate::MainWindowView;
use crate::ValidationViewModelBindings;
use crate::converters::display_value_type_converter::DisplayValueTypeConverter;
use slint::ComponentHandle;
use slint::ModelRc;
use slint::SharedString;
use slint::VecModel;
use slint_mvvm::convert_from_view_data::ConvertFromViewData;
use slint_mvvm::convert_to_view_data::ConvertToViewData;
use slint_mvvm::view_binding::ViewBinding;
use slint_mvvm_macros::create_view_bindings;
use squalr_engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::dependency_injection::dependency_container::DependencyContainer;
use squalr_engine_api::registries::data_types::data_type_registry::DataTypeRegistry;
use squalr_engine_api::structures::data_values::anonymous_value::AnonymousValue;
use std::sync::Arc;

pub struct ValidationViewModel {
    _view_binding: Arc<ViewBinding<MainWindowView>>,
    _engine_execution_context: Arc<EngineExecutionContext>,
}

impl ValidationViewModel {
    pub fn register(dependency_container: &DependencyContainer) {
        dependency_container.resolve_all(Self::on_dependencies_resolved);
    }

    fn on_dependencies_resolved(
        dependency_container: DependencyContainer,
        (view_binding, engine_execution_context): (Arc<ViewBinding<MainWindowView>>, Arc<EngineExecutionContext>),
    ) {
        let view_model = Arc::new(ValidationViewModel {
            _view_binding: view_binding.clone(),
            _engine_execution_context: engine_execution_context.clone(),
        });

        create_view_bindings!(view_binding, {
            ValidationViewModelBindings => {
                on_validate_data_value(data_value: SharedString, data_type_id: SharedString, display_value_type: DisplayValueTypeView) -> [] -> Self::on_validate_data_value,
                on_get_supported_display_types_for_data_type(data_type_id: SharedString) -> [] -> Self::on_get_supported_display_types_for_data_type,
            }
        });

        dependency_container.register::<ValidationViewModel>(view_model);
    }

    fn on_validate_data_value(
        data_value: SharedString,
        data_type_id: SharedString,
        display_value_type: DisplayValueTypeView,
    ) -> bool {
        let display_value_type = DisplayValueTypeConverter {}.convert_from_view_data(&display_value_type);
        let anonymous_value = AnonymousValue::new(&data_value, display_value_type);
        let registry = DataTypeRegistry::get_instance().get_registry();

        if let Some(data_type) = registry.get(&data_type_id.to_string()) {
            data_type.validate_value(&anonymous_value)
        } else {
            false
        }
    }

    fn on_get_supported_display_types_for_data_type(data_type_id: SharedString) -> ModelRc<DisplayValueTypeView> {
        let registry = DataTypeRegistry::get_instance().get_registry();

        let display_types = if let Some(data_type) = registry.get(&data_type_id.to_string()) {
            data_type.get_supported_display_types()
        } else {
            vec![]
        };

        ModelRc::new(VecModel::from(DisplayValueTypeConverter {}.convert_collection(&display_types)))
    }
}
