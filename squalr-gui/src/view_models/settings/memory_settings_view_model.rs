use crate::MainWindowView;
use crate::MemorySettingsViewModelBindings;
use crate::view_models::dependency_container::DependencyContainer;
use slint::ComponentHandle;
use slint_mvvm::view_binding::ViewBinding;
use slint_mvvm_macros::create_view_bindings;
use squalr_engine::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use squalr_engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::commands::settings::memory::list::memory_settings_list_request::MemorySettingsListRequest;
use squalr_engine_api::commands::settings::memory::set::memory_settings_set_request::MemorySettingsSetRequest;
use std::sync::Arc;

pub struct MemorySettingsViewModel {
    view_binding: Arc<ViewBinding<MainWindowView>>,
    engine_execution_context: Arc<EngineExecutionContext>,
}

impl MemorySettingsViewModel {
    pub fn new(
        dependency_container: &DependencyContainer,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) -> anyhow::Result<Arc<Self>> {
        let view_binding = dependency_container.resolve::<ViewBinding<MainWindowView>>()?;
        let view = Arc::new(MemorySettingsViewModel {
            view_binding: view_binding.clone(),
            engine_execution_context: engine_execution_context.clone(),
        });

        create_view_bindings!(view_binding, {
            MemorySettingsViewModelBindings => {
                on_required_write_changed(value: bool) -> [engine_execution_context] -> Self::on_required_write_changed,
                on_required_execute_changed(value: bool) -> [engine_execution_context] -> Self::on_required_execute_changed,
                on_required_copy_on_write_changed(value: bool) -> [engine_execution_context] -> Self::on_required_copy_on_write_changed,
                on_excluded_write_changed(value: bool) -> [engine_execution_context] -> Self::on_excluded_write_changed,
                on_excluded_execute_changed(value: bool) -> [engine_execution_context] -> Self::on_excluded_execute_changed,
                on_excluded_copy_on_write_changed(value: bool) -> [engine_execution_context] -> Self::on_excluded_copy_on_write_changed,
                on_memory_type_none_changed(value: bool) -> [engine_execution_context] -> Self::on_memory_type_none_changed,
                on_memory_type_image_changed(value: bool) -> [engine_execution_context] -> Self::on_memory_type_image_changed,
                on_memory_type_private_changed(value: bool) -> [engine_execution_context] -> Self::on_memory_type_private_changed,
                on_memory_type_mapped_changed(value: bool) -> [engine_execution_context] -> Self::on_memory_type_mapped_changed,
            }
        });

        view.sync_ui_with_memory_settings();

        Ok(view)
    }

    fn on_required_write_changed(
        engine_execution_context: Arc<EngineExecutionContext>,
        value: bool,
    ) {
        let memory_settings_set_request = MemorySettingsSetRequest {
            required_write: Some(value),
            ..Default::default()
        };

        Self::update_config(&memory_settings_set_request, engine_execution_context);
    }

    fn on_required_execute_changed(
        engine_execution_context: Arc<EngineExecutionContext>,
        value: bool,
    ) {
        let memory_settings_set_request = MemorySettingsSetRequest {
            required_execute: Some(value),
            ..Default::default()
        };

        Self::update_config(&memory_settings_set_request, engine_execution_context);
    }

    fn on_required_copy_on_write_changed(
        engine_execution_context: Arc<EngineExecutionContext>,
        value: bool,
    ) {
        let memory_settings_set_request = MemorySettingsSetRequest {
            required_copy_on_write: Some(value),
            ..Default::default()
        };

        Self::update_config(&memory_settings_set_request, engine_execution_context);
    }

    fn on_excluded_write_changed(
        engine_execution_context: Arc<EngineExecutionContext>,
        value: bool,
    ) {
        let memory_settings_set_request = MemorySettingsSetRequest {
            excluded_write: Some(value),
            ..Default::default()
        };

        Self::update_config(&memory_settings_set_request, engine_execution_context);
    }

    fn on_excluded_execute_changed(
        engine_execution_context: Arc<EngineExecutionContext>,
        value: bool,
    ) {
        let memory_settings_set_request = MemorySettingsSetRequest {
            excluded_execute: Some(value),
            ..Default::default()
        };

        Self::update_config(&memory_settings_set_request, engine_execution_context);
    }

    fn on_excluded_copy_on_write_changed(
        engine_execution_context: Arc<EngineExecutionContext>,
        value: bool,
    ) {
        let memory_settings_set_request = MemorySettingsSetRequest {
            excluded_copy_on_write: Some(value),
            ..Default::default()
        };

        Self::update_config(&memory_settings_set_request, engine_execution_context);
    }

    fn on_memory_type_none_changed(
        engine_execution_context: Arc<EngineExecutionContext>,
        value: bool,
    ) {
        let memory_settings_set_request = MemorySettingsSetRequest {
            memory_type_none: Some(value),
            ..Default::default()
        };

        Self::update_config(&memory_settings_set_request, engine_execution_context);
    }

    fn on_memory_type_image_changed(
        engine_execution_context: Arc<EngineExecutionContext>,
        value: bool,
    ) {
        let memory_settings_set_request = MemorySettingsSetRequest {
            memory_type_image: Some(value),
            ..Default::default()
        };

        Self::update_config(&memory_settings_set_request, engine_execution_context);
    }

    fn on_memory_type_private_changed(
        engine_execution_context: Arc<EngineExecutionContext>,
        value: bool,
    ) {
        let memory_settings_set_request = MemorySettingsSetRequest {
            memory_type_private: Some(value),
            ..Default::default()
        };

        Self::update_config(&memory_settings_set_request, engine_execution_context);
    }

    fn on_memory_type_mapped_changed(
        engine_execution_context: Arc<EngineExecutionContext>,
        value: bool,
    ) {
        let memory_settings_set_request = MemorySettingsSetRequest {
            memory_type_mapped: Some(value),
            ..Default::default()
        };

        Self::update_config(&memory_settings_set_request, engine_execution_context);
    }

    fn update_config(
        memory_settings_set_request: &MemorySettingsSetRequest,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) {
        memory_settings_set_request.send(&engine_execution_context, |_memory_settings_set_response| {});
    }

    fn sync_ui_with_memory_settings(&self) {
        let memory_settings_list_request = MemorySettingsListRequest {};
        let view_binding = self.view_binding.clone();

        memory_settings_list_request.send(&self.engine_execution_context, move |scan_results_query_response| {
            view_binding.execute_on_ui_thread(|main_window_view, _| {
                let memory_settings_view = main_window_view.global::<MemorySettingsViewModelBindings>();
                if let Ok(memory_settings) = scan_results_query_response.memory_settings {
                    // Required
                    memory_settings_view.set_required_write(memory_settings.required_write);
                    memory_settings_view.set_required_execute(memory_settings.required_execute);
                    memory_settings_view.set_required_copy_on_write(memory_settings.required_copy_on_write);

                    // Excluded
                    memory_settings_view.set_excluded_write(memory_settings.excluded_write);
                    memory_settings_view.set_excluded_execute(memory_settings.excluded_execute);
                    memory_settings_view.set_excluded_copy_on_write(memory_settings.excluded_copy_on_write);

                    // Memory types
                    memory_settings_view.set_memory_type_none(memory_settings.memory_type_none);
                    memory_settings_view.set_memory_type_image(memory_settings.memory_type_image);
                    memory_settings_view.set_memory_type_private(memory_settings.memory_type_private);
                    memory_settings_view.set_memory_type_mapped(memory_settings.memory_type_mapped);
                }

                let implement_me_query_ranges = 5;
            });
        });
    }
}
