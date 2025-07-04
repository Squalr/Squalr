use crate::DataTypeRefViewData;
use crate::DisplayValueTypeView;
use crate::MainWindowView;
use crate::ValidationViewModelBindings;
use crate::converters::data_type_ref_converter::DataTypeRefConverter;
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
use squalr_engine_api::structures::data_values::display_value_type::DisplayValueType;
use squalr_engine_api::structures::structs::container_type::ContainerType;
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
                on_validate_data_value(value_string: SharedString, data_type_ref: DataTypeRefViewData, display_value_type: DisplayValueTypeView) -> [] -> Self::on_validate_data_value,
                on_validate_anonymous_value(anonymous_value: SharedString, data_type_id: SharedString, display_value_type: DisplayValueTypeView) -> [] -> Self::on_validate_anonymous_value,
                on_get_supported_display_types_for_data_type(data_type_id: SharedString) -> [] -> Self::on_get_supported_display_types_for_data_type,
                on_get_default_display_type_for_data_type(data_type_id: SharedString) -> [] -> Self::on_get_default_display_type_for_data_type,
                on_get_default_display_type_index_for_data_type(data_type_id: SharedString) -> [] -> Self::on_get_default_display_type_index_for_data_type,
            }
        });

        dependency_container.register::<ValidationViewModel>(view_model);
    }

    fn on_validate_data_value(
        value_string: SharedString,
        data_type_ref: DataTypeRefViewData,
        display_value_type: DisplayValueTypeView,
    ) -> bool {
        let display_value_type = DisplayValueTypeConverter {}.convert_from_view_data(&display_value_type);
        let anonymous_value = AnonymousValue::new(&value_string, display_value_type);
        let data_type_ref = DataTypeRefConverter {}.convert_from_view_data(&data_type_ref);

        // Since we have a data type ref with metadata, we can validate more explicitly (ie number of elements in an array).
        data_type_ref.validate_value(&anonymous_value)
    }

    fn on_validate_anonymous_value(
        anonymous_value: SharedString,
        data_type_id: SharedString,
        display_value_type: DisplayValueTypeView,
    ) -> bool {
        let display_value_type = DisplayValueTypeConverter {}.convert_from_view_data(&display_value_type);
        let anonymous_value = AnonymousValue::new(&anonymous_value, display_value_type);

        // For anonymous values, we do not have any data type with meta data that we are validating against, so we just validate each part.
        if let Some(data_type) = DataTypeRegistry::get_instance().get(&data_type_id.to_string()) {
            if !data_type.validate_value(anonymous_value.get_value()) {
                return false;
            }
        } else {
            return false;
        }

        true
    }

    fn on_get_supported_display_types_for_data_type(data_type_id: SharedString) -> ModelRc<DisplayValueTypeView> {
        let display_types = if let Some(data_type) = DataTypeRegistry::get_instance().get(&data_type_id.to_string()) {
            data_type.get_supported_display_types()
        } else {
            vec![]
        };

        ModelRc::new(VecModel::from(DisplayValueTypeConverter {}.convert_collection(&display_types)))
    }

    fn on_get_default_display_type_for_data_type(data_type_id: SharedString) -> DisplayValueTypeView {
        let default_display_type = if let Some(data_type) = DataTypeRegistry::get_instance().get(&data_type_id.to_string()) {
            data_type.get_default_display_type()
        } else {
            DisplayValueType::Decimal(ContainerType::None)
        };

        DisplayValueTypeConverter {}.convert_to_view_data(&default_display_type)
    }

    fn on_get_default_display_type_index_for_data_type(data_type_id: SharedString) -> i32 {
        let (default_display_type, display_types) = if let Some(data_type) = DataTypeRegistry::get_instance().get(&data_type_id.to_string()) {
            (data_type.get_default_display_type(), data_type.get_supported_display_types())
        } else {
            (DisplayValueType::Decimal(ContainerType::None), vec![])
        };

        display_types
            .iter()
            .position(|next_display_type| next_display_type == &default_display_type)
            .map(|index| index as i32)
            .unwrap_or(-1)
    }
}
