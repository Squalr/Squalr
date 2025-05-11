use crate::MainWindowView;
use crate::WindowViewModelBindings;
use crate::models::audio::audio_player::AudioPlayer;
use crate::view_models::conversions_view_model::conversions_view_model::ConversionsViewModel;
use crate::view_models::docking::dock_root_view_model::DockRootViewModel;
use crate::view_models::output::output_view_model::OutputViewModel;
use crate::view_models::process_selector::process_selector_view_model::ProcessSelectorViewModel;
use crate::view_models::project_explorer::project_explorer_view_model::ProjectExplorerViewModel;
use crate::view_models::property_viewer::property_viewer_view_model::PropertyViewerViewModel;
use crate::view_models::scan_results::scan_results_view_model::ScanResultsViewModel;
use crate::view_models::scanners::scanner_view_model::ScannerViewModel;
use crate::view_models::settings::memory_settings_view_model::MemorySettingsViewModel;
use crate::view_models::settings::scan_settings_view_model::ScanSettingsViewModel;
use crate::view_models::validation_view_model::validation_view_model::ValidationViewModel;
use slint::ComponentHandle;
use slint_mvvm::view_binding::ViewBinding;
use slint_mvvm_macros::create_view_bindings;
use squalr_engine_api::dependency_injection::dependency_container::DependencyContainer;
use std::sync::Arc;

pub struct MainWindowViewModel {}

impl MainWindowViewModel {
    pub fn new(dependency_container: &mut DependencyContainer) -> anyhow::Result<Arc<Self>> {
        let view = MainWindowView::new().unwrap();
        let view_binding = Arc::new(ViewBinding::new(ComponentHandle::as_weak(&view)));

        dependency_container.register(|_dependency_container| Ok(Arc::new(AudioPlayer::new())));

        {
            let view_binding = view_binding.clone();

            dependency_container.register(move |_dependency_container| Ok(view_binding.clone()));
        }

        dependency_container.register(move |dependency_container| DockRootViewModel::new(dependency_container));
        dependency_container.register(move |dependency_container| ScannerViewModel::new(dependency_container));
        dependency_container.register(move |dependency_container| MemorySettingsViewModel::new(dependency_container));
        dependency_container.register(move |dependency_container| OutputViewModel::new(dependency_container));
        dependency_container.register(move |dependency_container| ProcessSelectorViewModel::new(dependency_container));
        dependency_container.register(move |dependency_container| ProjectExplorerViewModel::new(dependency_container));
        dependency_container.register(move |dependency_container| PropertyViewerViewModel::new(dependency_container));
        dependency_container.register(move |dependency_container| ScanSettingsViewModel::new(dependency_container));
        dependency_container.register(move |dependency_container| ScanResultsViewModel::new(dependency_container));
        dependency_container.register(move |dependency_container| ConversionsViewModel::new(dependency_container));
        dependency_container.register(move |dependency_container| ValidationViewModel::new(dependency_container));

        let view = Arc::new(MainWindowViewModel {});

        create_view_bindings!(view_binding, {
            WindowViewModelBindings => {
                on_minimize() -> [view_binding] -> Self::on_minimize,
                on_maximize() -> [view_binding] -> Self::on_maximize,
                on_close() -> [] -> Self::on_close,
                on_double_clicked() -> [view_binding] -> Self::on_double_clicked,
                on_drag(delta_x: i32, delta_y: i32) -> [view_binding] -> Self::on_drag
            }
        });

        Self::show(view_binding);

        Ok(view)
    }

    pub fn show(view_binding: Arc<ViewBinding<MainWindowView>>) {
        if let Ok(handle) = view_binding.get_view_handle().lock() {
            if let Some(view) = handle.upgrade() {
                if let Err(err) = view.show() {
                    log::error!("Error showing the main window: {err}");
                }
            }
        }
    }

    pub fn hide(view_binding: Arc<ViewBinding<MainWindowView>>) {
        if let Ok(handle) = view_binding.get_view_handle().lock() {
            if let Some(view) = handle.upgrade() {
                if let Err(err) = view.hide() {
                    log::error!("Error hiding the main window: {err}");
                }
            }
        }
    }

    fn on_minimize(view_binding: Arc<ViewBinding<MainWindowView>>) {
        view_binding.execute_on_ui_thread(move |main_window_view, _view_binding| {
            let window = main_window_view.window();
            window.set_minimized(true);
        });
    }

    fn on_maximize(view_binding: Arc<ViewBinding<MainWindowView>>) {
        view_binding.execute_on_ui_thread(move |main_window_view, _view_binding| {
            let window = main_window_view.window();
            window.set_maximized(!window.is_maximized());
        });
    }

    fn on_close() {
        if let Err(err) = slint::quit_event_loop() {
            log::error!("Failed to quit event loop: {}", err);
        }
    }

    fn on_double_clicked(view_binding: Arc<ViewBinding<MainWindowView>>) {
        view_binding.execute_on_ui_thread(move |main_window_view, _view_binding| {
            let window = main_window_view.window();
            window.set_maximized(!window.is_maximized());
        });
    }

    fn on_drag(
        view_binding: Arc<ViewBinding<MainWindowView>>,
        delta_x: i32,
        delta_y: i32,
    ) {
        view_binding.execute_on_ui_thread(move |main_window_view, _view_binding| {
            let window = main_window_view.window();
            let mut position = window.position();
            position.x += delta_x;
            position.y += delta_y;
            window.set_position(position);
        });
    }
}
