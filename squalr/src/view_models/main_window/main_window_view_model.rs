use crate::models::docking::docking_layout::DockingLayout;
use crate::view_models::docking::docked_window_view_model::DockedWindowViewModel;
use crate::view_models::output::output_view_model::OutputViewModel;
use crate::view_models::process_selector::process_selector_view_model::ProcessSelectorViewModel;
use crate::view_models::scanners::manual_scan_view_model::ManualScanViewModel;
use crate::view_models::settings::memory_settings_view_model::MemorySettingsViewModel;
use crate::view_models::settings::scan_settings_view_model::ScanSettingsViewModel;
use crate::view_models::view_model_base::ViewModel;
use crate::view_models::view_model_base::ViewModelBase;
use crate::DockedWindowViewModelBindings;
use crate::MainWindowView;
use crate::WindowViewModelBindings;
use slint::ComponentHandle;
use squalr_engine_common::logging::logger::Logger;
use std::borrow::BorrowMut;
use std::sync::Arc;
use std::sync::Mutex;

pub struct MainWindowViewModel {
    _view: MainWindowView,
    view_model_base: ViewModelBase<MainWindowView>,
    docking_layout: Arc<Mutex<DockingLayout>>,
    docked_window_view_model: Arc<DockedWindowViewModel>,
    manual_scan_view_model: Arc<ManualScanViewModel>,
    memory_settings_view_model: Arc<MemorySettingsViewModel>,
    output_view_model: Arc<OutputViewModel>,
    process_selector_view_model: Arc<ProcessSelectorViewModel>,
    scan_settings_view_model: Arc<ScanSettingsViewModel>,
}

/// Wraps the slint main window to internally manage and track the view handle for later use, as well as setting up
/// view code bindings to the corresponding slint UI.
impl MainWindowViewModel {
    pub fn new() -> Self {
        let view = MainWindowView::new().unwrap();
        let view_model_base = ViewModelBase::new(ComponentHandle::as_weak(&view));
        let docking_layout = Arc::new(Mutex::new(DockingLayout::default()));

        let view = MainWindowViewModel {
            _view: view,
            view_model_base: view_model_base.clone(),
            docking_layout: docking_layout.clone(),
            docked_window_view_model: Arc::new(DockedWindowViewModel::new(view_model_base.clone(), docking_layout.clone())),
            manual_scan_view_model: Arc::new(ManualScanViewModel::new(view_model_base.clone())),
            memory_settings_view_model: Arc::new(MemorySettingsViewModel::new(view_model_base.clone())),
            output_view_model: Arc::new(OutputViewModel::new(view_model_base.clone())),
            process_selector_view_model: Arc::new(ProcessSelectorViewModel::new(view_model_base.clone())),
            scan_settings_view_model: Arc::new(ScanSettingsViewModel::new(view_model_base.clone())),
        };

        view.create_view_bindings();
        Self::propagate_layout(&view_model_base, &view.docking_layout);

        return view;
    }

    pub fn initialize(&self) {
        self.show();
    }

    pub fn show(&self) {
        if let Ok(handle) = self.view_model_base.get_view_handle().lock() {
            if let Some(view) = handle.upgrade() {
                if let Err(err) = view.show() {
                    log::error!("Error showing the main window: {err}");
                }
            }
        }
    }

    pub fn hide(&self) {
        if let Ok(handle) = self.view_model_base.get_view_handle().lock() {
            if let Some(view) = handle.upgrade() {
                if let Err(err) = view.hide() {
                    log::error!("Error hiding the main window: {err}");
                }
            }
        }
    }

    pub fn get_docked_window_view_model(&self) -> &Arc<DockedWindowViewModel> {
        return &self.docked_window_view_model;
    }

    pub fn get_manual_scan_view_model(&self) -> &Arc<ManualScanViewModel> {
        return &self.manual_scan_view_model;
    }

    pub fn get_memory_settings_view_model(&self) -> &Arc<MemorySettingsViewModel> {
        return &self.memory_settings_view_model;
    }

    pub fn get_output_view_model(&self) -> &Arc<OutputViewModel> {
        return &self.output_view_model;
    }

    pub fn get_process_selector_view_model(&self) -> &Arc<ProcessSelectorViewModel> {
        return &self.process_selector_view_model;
    }

    pub fn get_scan_settings_view_model(&self) -> &Arc<ScanSettingsViewModel> {
        return &self.scan_settings_view_model;
    }

    fn propagate_layout(
        view_model_base: &ViewModelBase<MainWindowView>,
        docking_layout: &Arc<Mutex<DockingLayout>>,
    ) {
        let docking_layout = docking_layout.clone();

        view_model_base.execute_on_ui_thread(move |main_window_view, _view_model_base| {
            let docked_window_bindings = main_window_view.global::<DockedWindowViewModelBindings>();

            let process_selector_identifier = "process-selector";
            let project_explorer_identifier = "project-explorer";
            let property_viewer_identifier = "property-viewer";
            let scan_results_identifier = "scan-results";
            let output_identifier = "output";
            let settings_identifier = "settings";

            let docking_layout = match docking_layout.lock() {
                Ok(guard) => guard,
                Err(err) => {
                    log::error!("Failed to acquire docking layout lock: {}", err);
                    return;
                }
            };

            if let Some(docked_window_bounds) = docking_layout.calculate_window_rect(process_selector_identifier) {
                docked_window_bindings.set_process_selector_panel(crate::DockedWindowData {
                    identifier: process_selector_identifier.into(),
                    is_docked: true,
                    position_x: docked_window_bounds.0,
                    position_y: docked_window_bounds.1,
                    width: docked_window_bounds.2,
                    height: docked_window_bounds.3,
                });
            }

            if let Some(docked_window_bounds) = docking_layout.calculate_window_rect(project_explorer_identifier) {
                docked_window_bindings.set_project_explorer_panel(crate::DockedWindowData {
                    identifier: project_explorer_identifier.into(),
                    is_docked: true,
                    position_x: docked_window_bounds.0,
                    position_y: docked_window_bounds.1,
                    width: docked_window_bounds.2,
                    height: docked_window_bounds.3,
                });
            }

            if let Some(docked_window_bounds) = docking_layout.calculate_window_rect(property_viewer_identifier) {
                docked_window_bindings.set_property_viewer_panel(crate::DockedWindowData {
                    identifier: property_viewer_identifier.into(),
                    is_docked: true,
                    position_x: docked_window_bounds.0,
                    position_y: docked_window_bounds.1,
                    width: docked_window_bounds.2,
                    height: docked_window_bounds.3,
                });
            }

            if let Some(docked_window_bounds) = docking_layout.calculate_window_rect(project_explorer_identifier) {
                docked_window_bindings.set_project_explorer_panel(crate::DockedWindowData {
                    identifier: project_explorer_identifier.into(),
                    is_docked: true,
                    position_x: docked_window_bounds.0,
                    position_y: docked_window_bounds.1,
                    width: docked_window_bounds.2,
                    height: docked_window_bounds.3,
                });
            }

            if let Some(docked_window_bounds) = docking_layout.calculate_window_rect(scan_results_identifier) {
                docked_window_bindings.set_scan_results_panel(crate::DockedWindowData {
                    identifier: scan_results_identifier.into(),
                    is_docked: true,
                    position_x: docked_window_bounds.0,
                    position_y: docked_window_bounds.1,
                    width: docked_window_bounds.2,
                    height: docked_window_bounds.3,
                });
            }

            if let Some(docked_window_bounds) = docking_layout.calculate_window_rect(output_identifier) {
                docked_window_bindings.set_output_panel(crate::DockedWindowData {
                    identifier: output_identifier.into(),
                    is_docked: true,
                    position_x: docked_window_bounds.0,
                    position_y: docked_window_bounds.1,
                    width: docked_window_bounds.2,
                    height: docked_window_bounds.3,
                });
            }

            if let Some(docked_window_bounds) = docking_layout.calculate_window_rect(settings_identifier) {
                docked_window_bindings.set_settings_panel(crate::DockedWindowData {
                    identifier: settings_identifier.into(),
                    is_docked: true,
                    position_x: docked_window_bounds.0,
                    position_y: docked_window_bounds.1,
                    width: docked_window_bounds.2,
                    height: docked_window_bounds.3,
                });
            }
        });
    }
}

impl ViewModel for MainWindowViewModel {
    fn create_view_bindings(&self) {
        // Bind our output viewmodel to the logger.
        Logger::get_instance().subscribe(self.output_view_model.clone());

        let docking_layout = self.docking_layout.clone();

        self.view_model_base
            .execute_on_ui_thread(move |main_window_view, view_model_base| {
                let docked_window_view = main_window_view.global::<DockedWindowViewModelBindings>();

                let view_model = view_model_base.clone();
                let mut docking_layout_mut = docking_layout.clone();
                docked_window_view.on_update_dock_root_size(move |width, height| {
                    docking_layout_mut
                        .borrow_mut()
                        .lock()
                        .unwrap()
                        .set_available_size(width, height);
                    Self::propagate_layout(&view_model, &docking_layout_mut);
                    return 0.0;
                });

                let view_model = view_model_base.clone();
                let mut docking_layout_mut = docking_layout.clone();
                docked_window_view.on_update_dock_root_width(move |width| {
                    docking_layout_mut
                        .borrow_mut()
                        .lock()
                        .unwrap()
                        .set_available_width(width);
                    Self::propagate_layout(&view_model, &docking_layout_mut);
                });

                let view_model = view_model_base.clone();
                let mut docking_layout_mut = docking_layout.clone();
                docked_window_view.on_update_dock_root_height(move |height| {
                    docking_layout_mut
                        .borrow_mut()
                        .lock()
                        .unwrap()
                        .set_available_height(height);
                    Self::propagate_layout(&view_model, &docking_layout_mut);
                });
            });

        self.view_model_base
            .execute_on_ui_thread(move |main_window_view, view_model_base| {
                let main_window_bindings = main_window_view.global::<WindowViewModelBindings>();

                // Set up minimize handler
                let view_model = view_model_base.clone();
                main_window_bindings.on_minimize(move || {
                    view_model.execute_on_ui_thread(move |main_window_view, _view_model_base| {
                        let window = main_window_view.window();
                        window.set_minimized(true);
                    });
                });

                // Set up maximize handler
                let view_model = view_model_base.clone();
                main_window_bindings.on_maximize(move || {
                    view_model.execute_on_ui_thread(move |main_window_view, _view_model_base| {
                        let window = main_window_view.window();
                        window.set_maximized(!window.is_maximized());
                    });
                });

                // Set up close handler
                main_window_bindings.on_close(move || {
                    if let Err(e) = slint::quit_event_loop() {
                        log::error!("Failed to quit event loop: {}", e);
                    }
                });

                // Set up double click handler
                let view_model = view_model_base.clone();
                main_window_bindings.on_double_clicked(move || {
                    view_model.execute_on_ui_thread(move |main_window_view, _view_model_base| {
                        let window = main_window_view.window();
                        window.set_maximized(!window.is_maximized());
                    });
                });

                // Set up drag handler
                let view_model = view_model_base.clone();
                main_window_bindings.on_drag(move |delta_x: i32, delta_y| {
                    view_model.execute_on_ui_thread(move |main_window_view, _view_model_base| {
                        let window = main_window_view.window();
                        let mut position = window.position();
                        position.x += delta_x;
                        position.y += delta_y;
                        window.set_position(position);
                    });
                });
            });
    }
}
