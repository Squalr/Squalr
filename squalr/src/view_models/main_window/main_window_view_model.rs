use crate::DockedWindowData;
use crate::DockedWindowViewModelBindings;
use crate::MainWindowView;
use crate::WindowViewModelBindings;
use crate::models::docking::docking_layout::DockingLayout;
use crate::mvvm::view_binding::ViewBinding;
use crate::view_models::docking::docked_window_view_model::DockedWindowViewModel;
use crate::view_models::output::output_view_model::OutputViewModel;
use crate::view_models::process_selector::process_selector_view_model::ProcessSelectorViewModel;
use crate::view_models::scanners::manual_scan_view_model::ManualScanViewModel;
use crate::view_models::settings::memory_settings_view_model::MemorySettingsViewModel;
use crate::view_models::settings::scan_settings_view_model::ScanSettingsViewModel;
use slint::ComponentHandle;
use slint_mvvm_macros::create_view_bindings;
use squalr_engine_common::logging::logger::Logger;
use std::borrow::BorrowMut;
use std::sync::Arc;
use std::sync::Mutex;

pub struct MainWindowViewModel {
    _view: MainWindowView,
    view_binding: ViewBinding<MainWindowView>,
    docking_layout: Arc<Mutex<DockingLayout>>,
    docked_window_view_model: Arc<DockedWindowViewModel>,
    manual_scan_view_model: Arc<ManualScanViewModel>,
    memory_settings_view_model: Arc<MemorySettingsViewModel>,
    output_view_model: Arc<OutputViewModel>,
    process_selector_view_model: Arc<ProcessSelectorViewModel>,
    scan_settings_view_model: Arc<ScanSettingsViewModel>,
}

impl MainWindowViewModel {
    pub fn new() -> Self {
        let view = MainWindowView::new().unwrap();
        let view_binding = ViewBinding::new(ComponentHandle::as_weak(&view));
        let docking_layout = Arc::new(Mutex::new(DockingLayout::default()));

        let view: MainWindowViewModel = MainWindowViewModel {
            _view: view,
            view_binding: view_binding.clone(),
            docking_layout: docking_layout.clone(),
            docked_window_view_model: Arc::new(DockedWindowViewModel::new(view_binding.clone(), docking_layout.clone())),
            manual_scan_view_model: Arc::new(ManualScanViewModel::new(view_binding.clone())),
            memory_settings_view_model: Arc::new(MemorySettingsViewModel::new(view_binding.clone())),
            output_view_model: Arc::new(OutputViewModel::new(view_binding.clone())),
            process_selector_view_model: Arc::new(ProcessSelectorViewModel::new(view_binding.clone())),
            scan_settings_view_model: Arc::new(ScanSettingsViewModel::new(view_binding.clone())),
        };

        Logger::get_instance().subscribe(view.output_view_model.clone());

        create_view_bindings!(
            view_binding,
            {
                WindowViewModelBindings => {
                    on_minimize() -> Self::on_minimize [view_binding],
                    on_maximize() -> Self::on_maximize [view_binding],
                    on_close() -> Self::on_close [],
                    on_double_clicked() -> Self::on_double_clicked [view_binding],
                    on_drag(delta_x: i32, delta_y: i32) -> Self::on_drag [view_binding]
                },
                DockedWindowViewModelBindings => {
                    on_update_dock_root_size(width: f32, height: f32) -> Self::on_update_dock_root_size [view_binding, docking_layout],
                    on_update_dock_root_width(width: f32) -> Self::on_update_dock_root_width [view_binding, docking_layout],
                    on_update_dock_root_height(height: f32) -> Self::on_update_dock_root_height [view_binding, docking_layout]
                }
            }
        );

        Self::propagate_layout(&view.view_binding, &view.docking_layout);

        return view;
    }

    pub fn initialize(&self) {
        self.show();
    }

    pub fn show(&self) {
        if let Ok(handle) = self.view_binding.get_view_handle().lock() {
            if let Some(view) = handle.upgrade() {
                if let Err(err) = view.show() {
                    log::error!("Error showing the main window: {err}");
                }
            }
        }
    }

    pub fn hide(&self) {
        if let Ok(handle) = self.view_binding.get_view_handle().lock() {
            if let Some(view) = handle.upgrade() {
                if let Err(err) = view.hide() {
                    log::error!("Error hiding the main window: {err}");
                }
            }
        }
    }

    pub fn get_docked_window_view_model(&self) -> &Arc<DockedWindowViewModel> {
        &self.docked_window_view_model
    }

    pub fn get_manual_scan_view_model(&self) -> &Arc<ManualScanViewModel> {
        &self.manual_scan_view_model
    }

    pub fn get_memory_settings_view_model(&self) -> &Arc<MemorySettingsViewModel> {
        &self.memory_settings_view_model
    }

    pub fn get_output_view_model(&self) -> &Arc<OutputViewModel> {
        &self.output_view_model
    }

    pub fn get_process_selector_view_model(&self) -> &Arc<ProcessSelectorViewModel> {
        &self.process_selector_view_model
    }

    pub fn get_scan_settings_view_model(&self) -> &Arc<ScanSettingsViewModel> {
        &self.scan_settings_view_model
    }

    fn on_minimize(view_binding: ViewBinding<MainWindowView>) {
        view_binding.execute_on_ui_thread(move |main_window_view, _view_binding| {
            let window = main_window_view.window();
            window.set_minimized(true);
        });
    }

    fn on_maximize(view_binding: ViewBinding<MainWindowView>) {
        view_binding.execute_on_ui_thread(move |main_window_view, _view_binding| {
            let window = main_window_view.window();
            window.set_maximized(!window.is_maximized());
        });
    }

    fn on_close() {
        if let Err(e) = slint::quit_event_loop() {
            log::error!("Failed to quit event loop: {}", e);
        }
    }

    fn on_double_clicked(view_binding: ViewBinding<MainWindowView>) {
        view_binding.execute_on_ui_thread(move |main_window_view, _view_binding| {
            let window = main_window_view.window();
            window.set_maximized(!window.is_maximized());
        });
    }

    fn on_drag(
        view_binding: ViewBinding<MainWindowView>,
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

    fn on_update_dock_root_size(
        view_binding: ViewBinding<MainWindowView>,
        docking_layout: Arc<Mutex<DockingLayout>>,
        width: f32,
        height: f32,
    ) -> f32 {
        docking_layout
            .lock()
            .unwrap()
            .borrow_mut()
            .set_available_size(width, height);
        Self::propagate_layout(&view_binding, &docking_layout);
        0.0
    }

    fn on_update_dock_root_width(
        view_binding: ViewBinding<MainWindowView>,
        docking_layout: Arc<Mutex<DockingLayout>>,
        width: f32,
    ) {
        docking_layout
            .lock()
            .unwrap()
            .borrow_mut()
            .set_available_width(width);
        Self::propagate_layout(&view_binding, &docking_layout);
    }

    fn on_update_dock_root_height(
        view_binding: ViewBinding<MainWindowView>,
        docking_layout: Arc<Mutex<DockingLayout>>,
        height: f32,
    ) {
        docking_layout
            .lock()
            .unwrap()
            .borrow_mut()
            .set_available_height(height);
        Self::propagate_layout(&view_binding, &docking_layout);
    }

    fn create_docked_window_data(
        identifier: &str,
        docked_window_bounds: (f32, f32, f32, f32),
    ) -> DockedWindowData {
        DockedWindowData {
            identifier: identifier.into(),
            is_docked: true,
            position_x: docked_window_bounds.0,
            position_y: docked_window_bounds.1,
            width: docked_window_bounds.2,
            height: docked_window_bounds.3,
        }
    }

    fn propagate_layout(
        view_binding: &ViewBinding<MainWindowView>,
        docking_layout: &Arc<Mutex<DockingLayout>>,
    ) {
        let docking_layout = docking_layout.clone();

        view_binding.execute_on_ui_thread(move |main_window_view, _view_binding| {
            let docked_window_bindings = main_window_view.global::<DockedWindowViewModelBindings>();
            let docking_layout = match docking_layout.lock() {
                Ok(guard) => guard,
                Err(err) => {
                    log::error!("Failed to acquire docking layout lock: {}", err);
                    return;
                }
            };

            let window_identifier = "process-selector";
            if let Some(docked_window_bounds) = docking_layout.calculate_window_rect(window_identifier) {
                docked_window_bindings.set_process_selector_panel(Self::create_docked_window_data(window_identifier, docked_window_bounds));
            }

            let window_identifier = "project-explorer";
            if let Some(docked_window_bounds) = docking_layout.calculate_window_rect(window_identifier) {
                docked_window_bindings.set_project_explorer_panel(Self::create_docked_window_data(window_identifier, docked_window_bounds));
            }

            let window_identifier = "property-viewer";
            if let Some(docked_window_bounds) = docking_layout.calculate_window_rect(window_identifier) {
                docked_window_bindings.set_property_viewer_panel(Self::create_docked_window_data(window_identifier, docked_window_bounds));
            }

            let window_identifier = "scan-results";
            if let Some(docked_window_bounds) = docking_layout.calculate_window_rect(window_identifier) {
                docked_window_bindings.set_scan_results_panel(Self::create_docked_window_data(window_identifier, docked_window_bounds));
            }

            let window_identifier = "output";
            if let Some(docked_window_bounds) = docking_layout.calculate_window_rect(window_identifier) {
                docked_window_bindings.set_output_panel(Self::create_docked_window_data(window_identifier, docked_window_bounds));
            }

            let window_identifier = "settings";
            if let Some(docked_window_bounds) = docking_layout.calculate_window_rect(window_identifier) {
                docked_window_bindings.set_settings_panel(Self::create_docked_window_data(window_identifier, docked_window_bounds));
            }
        });
    }
}
