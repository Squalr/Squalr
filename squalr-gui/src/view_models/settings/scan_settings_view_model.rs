use crate::converters::floating_point_tolerance_converter::FloatingPointToleranceConverter;
use crate::converters::memory_alignment_converter::MemoryAlignmentConverter;
use crate::converters::memory_read_mode_converter::MemoryReadModeConverter;
use crate::{FloatingPointToleranceView, MemoryAlignmentView};
use crate::{MainWindowView, ScanSettingsViewModelBindings};
use slint::ComponentHandle;
use slint_mvvm::convert_from_view_data::ConvertFromViewData;
use slint_mvvm::{convert_to_view_data::ConvertToViewData, view_binding::ViewBinding};
use slint_mvvm_macros::create_view_bindings;
use squalr_engine::{command_executors::engine_request_executor::EngineCommandRequestExecutor, engine_execution_context::EngineExecutionContext};
use squalr_engine_api::commands::settings::scan::set::scan_settings_set_request::ScanSettingsSetRequest;
use squalr_engine_api::dependency_injection::dependency_container::DependencyContainer;
use squalr_engine_api::{
    commands::settings::scan::list::scan_settings_list_request::ScanSettingsListRequest, structures::memory::memory_alignment::MemoryAlignment,
};
use std::sync::Arc;

pub struct ScanSettingsViewModel {
    view_binding: Arc<ViewBinding<MainWindowView>>,
    engine_execution_context: Arc<EngineExecutionContext>,
}

impl ScanSettingsViewModel {
    pub fn register(dependency_container: &DependencyContainer) {
        dependency_container.resolve_all(Self::on_dependencies_resolved);
    }

    fn on_dependencies_resolved(
        dependency_container: DependencyContainer,
        (view_binding, engine_execution_context): (Arc<ViewBinding<MainWindowView>>, Arc<EngineExecutionContext>),
    ) {
        let view_model = Arc::new(ScanSettingsViewModel {
            view_binding: view_binding.clone(),
            engine_execution_context: engine_execution_context.clone(),
        });

        create_view_bindings!(view_binding, {
           ScanSettingsViewModelBindings => {
                on_results_page_size_changed(value: i32) -> [engine_execution_context] -> Self::on_results_page_size_changed,
                on_results_read_interval_changed(value: i32) -> [engine_execution_context] -> Self::on_results_read_interval_changed,
                on_project_read_interval_changed(value: i32) -> [engine_execution_context] -> Self::on_project_read_interval_changed,
                on_freeze_interval_changed(value: i32) -> [engine_execution_context] -> Self::on_freeze_interval_changed,
                on_memory_alignment_changed(value: MemoryAlignmentView) -> [engine_execution_context] -> Self::on_memory_alignment_changed,
                on_floating_point_tolerance_changed(value: FloatingPointToleranceView) -> [engine_execution_context] -> Self::on_floating_point_tolerance_changed,
                on_is_single_threaded_scan_changed(value: bool) -> [engine_execution_context] -> Self::on_is_single_threaded_scan_changed,
                on_debug_perform_validation_scan_changed(value: bool) -> [engine_execution_context] -> Self::on_debug_perform_validation_scan_changed,
            }
        });

        view_model.sync_ui_with_scan_settings();

        dependency_container.register::<ScanSettingsViewModel>(view_model);
    }

    fn on_results_page_size_changed(
        engine_execution_context: Arc<EngineExecutionContext>,
        value: i32,
    ) {
        let scan_settings_set_request = ScanSettingsSetRequest {
            results_page_size: Some(value as u32),
            ..Default::default()
        };

        Self::update_config(&scan_settings_set_request, engine_execution_context);
    }

    fn on_results_read_interval_changed(
        engine_execution_context: Arc<EngineExecutionContext>,
        value: i32,
    ) {
        let scan_settings_set_request = ScanSettingsSetRequest {
            results_read_interval: Some(value as u64),
            ..Default::default()
        };

        Self::update_config(&scan_settings_set_request, engine_execution_context);
    }

    fn on_project_read_interval_changed(
        engine_execution_context: Arc<EngineExecutionContext>,
        value: i32,
    ) {
        let scan_settings_set_request = ScanSettingsSetRequest {
            project_read_interval: Some(value as u64),
            ..Default::default()
        };

        Self::update_config(&scan_settings_set_request, engine_execution_context);
    }

    fn on_freeze_interval_changed(
        engine_execution_context: Arc<EngineExecutionContext>,
        value: i32,
    ) {
        let scan_settings_set_request = ScanSettingsSetRequest {
            freeze_interval: Some(value as u64),
            ..Default::default()
        };

        Self::update_config(&scan_settings_set_request, engine_execution_context);
    }

    fn on_memory_alignment_changed(
        engine_execution_context: Arc<EngineExecutionContext>,
        value: MemoryAlignmentView,
    ) {
        let scan_settings_set_request = ScanSettingsSetRequest {
            memory_alignment: Some(MemoryAlignmentConverter {}.convert_from_view_data(&value)),
            ..Default::default()
        };

        Self::update_config(&scan_settings_set_request, engine_execution_context);
    }

    fn on_floating_point_tolerance_changed(
        engine_execution_context: Arc<EngineExecutionContext>,
        value: FloatingPointToleranceView,
    ) {
        let scan_settings_set_request = ScanSettingsSetRequest {
            floating_point_tolerance: Some(FloatingPointToleranceConverter {}.convert_from_view_data(&value)),
            ..Default::default()
        };

        Self::update_config(&scan_settings_set_request, engine_execution_context);
    }

    fn on_is_single_threaded_scan_changed(
        engine_execution_context: Arc<EngineExecutionContext>,
        value: bool,
    ) {
        let scan_settings_set_request = ScanSettingsSetRequest {
            is_single_threaded_scan: value,
            ..Default::default()
        };

        Self::update_config(&scan_settings_set_request, engine_execution_context);
    }

    fn on_debug_perform_validation_scan_changed(
        engine_execution_context: Arc<EngineExecutionContext>,
        value: bool,
    ) {
        let scan_settings_set_request = ScanSettingsSetRequest {
            debug_perform_validation_scan: value,
            ..Default::default()
        };

        Self::update_config(&scan_settings_set_request, engine_execution_context);
    }

    fn update_config(
        scan_settings_set_request: &ScanSettingsSetRequest,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) {
        scan_settings_set_request.send(&engine_execution_context, |_memory_settings_set_response| {});
    }

    fn sync_ui_with_scan_settings(&self) {
        let scan_settings_list_request = ScanSettingsListRequest {};
        let view_binding = self.view_binding.clone();

        scan_settings_list_request.send(&self.engine_execution_context, move |scan_results_query_response| {
            view_binding.execute_on_ui_thread(|main_window_view, _| {
                let scan_settings_view = main_window_view.global::<ScanSettingsViewModelBindings>();
                if let Ok(scan_settings) = scan_results_query_response.scan_settings {
                    scan_settings_view.set_results_page_size(scan_settings.results_page_size as i32);
                    scan_settings_view.set_results_read_interval(scan_settings.results_read_interval as i32);
                    scan_settings_view.set_project_read_interval(scan_settings.project_read_interval as i32);
                    scan_settings_view.set_freeze_interval(scan_settings.freeze_interval as i32);
                    scan_settings_view
                        .set_floating_point_tolerance(FloatingPointToleranceConverter {}.convert_to_view_data(&scan_settings.floating_point_tolerance));
                    scan_settings_view.set_memory_alignment(
                        MemoryAlignmentConverter {}.convert_to_view_data(
                            &scan_settings
                                .memory_alignment
                                .unwrap_or(MemoryAlignment::Alignment1),
                        ),
                    );
                    scan_settings_view
                        .set_floating_point_tolerance(FloatingPointToleranceConverter {}.convert_to_view_data(&scan_settings.floating_point_tolerance));
                    scan_settings_view.set_memory_read_mode(MemoryReadModeConverter {}.convert_to_view_data(&scan_settings.memory_read_mode));
                    scan_settings_view.set_is_single_threaded_scan(scan_settings.is_single_threaded_scan);
                    scan_settings_view.set_debug_perform_validation_scan(scan_settings.debug_perform_validation_scan);
                }
            });
        });
    }
}
