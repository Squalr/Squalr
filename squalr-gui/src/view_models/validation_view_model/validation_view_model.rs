use crate::DataTypeView;
use crate::MainWindowView;
use crate::ValidationViewModelBindings;
use slint::ComponentHandle;
use slint::SharedString;
use slint_mvvm::view_binding::ViewBinding;
use slint_mvvm_macros::create_view_bindings;
use squalr_engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::registries::data_types::data_type_registry::DataTypeRegistry;
use squalr_engine_api::structures::data_values::anonymous_value::AnonymousValue;
use std::sync::Arc;

pub struct ValidationViewModel {
    _view_binding: ViewBinding<MainWindowView>,
    _engine_execution_context: Arc<EngineExecutionContext>,
}

impl ValidationViewModel {
    pub fn new(
        view_binding: ViewBinding<MainWindowView>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) -> Arc<Self> {
        let view = Arc::new(ValidationViewModel {
            _view_binding: view_binding.clone(),
            _engine_execution_context: engine_execution_context.clone(),
        });

        create_view_bindings!(view_binding, {
            ValidationViewModelBindings => {
                on_validate_data_value(data_type_view: DataTypeView, value: SharedString, is_value_hex: bool) -> [] -> Self::on_validate_data_value,
            }
        });

        view
    }

    fn on_validate_data_value(
        data_value_view: DataTypeView,
        value: SharedString,
        is_value_hex: bool,
    ) -> bool {
        let anonymous_value = AnonymousValue::new(&value, is_value_hex);
        let registry = DataTypeRegistry::get_instance().get_registry();
        let data_type = data_value_view.data_type.to_string();

        if let Some(data_type) = registry.get(&data_type) {
            if let Ok(_) = data_type.deanonymize_value(&anonymous_value) {
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}
