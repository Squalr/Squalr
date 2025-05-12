use crate::DataTypeView;
use crate::MainWindowView;
use crate::ValidationViewModelBindings;
use slint::ComponentHandle;
use slint::SharedString;
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
                on_validate_data_value(data_type_view: DataTypeView, value: SharedString, is_value_hex: bool) -> [] -> Self::on_validate_data_value,
            }
        });

        dependency_container.register::<ValidationViewModel>(view_model);
    }

    fn on_validate_data_value(
        data_value_view: DataTypeView,
        value: SharedString,
        is_value_hex: bool,
    ) -> bool {
        let anonymous_value = AnonymousValue::new_string(&value, is_value_hex);
        let registry = DataTypeRegistry::get_instance().get_registry();
        let data_type = data_value_view.data_type.to_string();

        if let Some(data_type) = registry.get(&data_type) {
            data_type.validate_value(&anonymous_value)
        } else {
            false
        }
    }
}
