use crate::DataTypeRefViewData;
use crate::DisplayValueTypeView;
use crate::DisplayValueViewData;
use crate::MainWindowView;
use crate::ValidationViewModelBindings;
use crate::converters::data_type_ref_converter::DataTypeRefConverter;
use crate::converters::display_value_converter::DisplayValueConverter;
use crate::converters::display_value_type_converter::DisplayValueTypeConverter;
use olorin_engine::engine_execution_context::EngineExecutionContext;
use olorin_engine_api::dependency_injection::dependency_container::DependencyContainer;
use olorin_engine_api::registries::data_types::data_type_registry::DataTypeRegistry;
use olorin_engine_api::structures::data_values::anonymous_value::AnonymousValue;
use olorin_engine_api::structures::data_values::display_value_type::DisplayValueType;
use slint::ComponentHandle;
use slint::ModelRc;
use slint::SharedString;
use slint::VecModel;
use slint_mvvm::convert_from_view_data::ConvertFromViewData;
use slint_mvvm::convert_to_view_data::ConvertToViewData;
use slint_mvvm::view_binding::ViewBinding;
use slint_mvvm_macros::create_view_bindings;
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
                on_validate_data_value(value_string: SharedString, data_type_ref: DataTypeRefViewData, display_value: DisplayValueViewData) -> [] -> Self::on_validate_data_value,
                on_get_supported_display_types_for_data_type(data_type_ref: DataTypeRefViewData) -> [] -> Self::on_get_supported_display_types_for_data_type,
                on_get_default_display_type_for_data_type(data_type_ref: DataTypeRefViewData) -> [] -> Self::on_get_default_display_type_for_data_type,
                on_get_default_display_type_index_for_data_type(data_type_ref: DataTypeRefViewData) -> [] -> Self::on_get_default_display_type_index_for_data_type,
            }
        });

        dependency_container.register::<ValidationViewModel>(view_model);
    }

    fn on_validate_data_value(
        value_string: SharedString,
        data_type_ref: DataTypeRefViewData,
        display_value: DisplayValueViewData,
    ) -> bool {
        let display_value = DisplayValueConverter {}.convert_from_view_data(&display_value);
        let anonymous_value = AnonymousValue::new(&value_string, display_value);
        let data_type_ref = DataTypeRefConverter {}.convert_from_view_data(&data_type_ref);
        let DATA_TYPE_REGISTRY = DataTypeRegistry::new();

        DATA_TYPE_REGISTRY.validate_value(&data_type_ref, &anonymous_value)
    }

    fn on_get_supported_display_types_for_data_type(data_type_ref: DataTypeRefViewData) -> ModelRc<DisplayValueTypeView> {
        let data_type_ref = DataTypeRefConverter {}.convert_from_view_data(&data_type_ref);
        let DATA_TYPE_REGISTRY = DataTypeRegistry::new();
        let display_types = DATA_TYPE_REGISTRY.get_supported_display_types(&data_type_ref);

        ModelRc::new(VecModel::from(DisplayValueTypeConverter {}.convert_collection(&display_types)))
    }

    fn on_get_default_display_type_for_data_type(data_type_ref: DataTypeRefViewData) -> DisplayValueTypeView {
        /*
        let default_display_type = if let Some(data_type) = DataTypeRegistry::get_instance().get(&data_type_id.to_string()) {
            data_type.get_default_display_type()
        } else {
            DisplayValueType::Decimal
        };*/
        let default_display_type = DisplayValueType::Decimal;

        DisplayValueTypeConverter {}.convert_to_view_data(&default_display_type)
    }

    fn on_get_default_display_type_index_for_data_type(data_type_ref: DataTypeRefViewData) -> i32 {
        let data_type_ref = DataTypeRefConverter {}.convert_from_view_data(&data_type_ref);
        let DATA_TYPE_REGISTRY = DataTypeRegistry::new();
        let default_display_type = DATA_TYPE_REGISTRY.get_default_display_type(&data_type_ref);
        let display_types = DATA_TYPE_REGISTRY.get_supported_display_types(&data_type_ref);

        display_types
            .iter()
            .position(|next_display_type| next_display_type == &default_display_type)
            .map(|index| index as i32)
            .unwrap_or(-1)
    }
}
