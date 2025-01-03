use crate::models::docking::docking_layout::DockingLayout;
use crate::view_models::docking::docked_window_view_model::DockedWindowViewModel;
use crate::view_models::output::output_view_model::OutputViewModel;
use crate::view_models::scanners::manual_scan_view_model::ManualScanViewModel;
use crate::view_models::settings::memory_settings_view_model::MemorySettingsViewModel;
use crate::view_models::settings::scan_settings_view_model::ScanSettingsViewModel;
use crate::view_models::view_model_base::ViewModel;
use crate::DockedWindowViewModelBindings;
use crate::MainWindowView;
use crate::WindowViewModelBindings;
use slint::ComponentHandle;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use std::cell::RefCell;
use std::sync::Arc;

pub struct MainWindowViewModel {
    view_handle: Arc<MainWindowView>,
    docking_layout: Arc<RefCell<DockingLayout>>,
    docked_window_view_model: Arc<DockedWindowViewModel>,
    manual_scan_view_model: Arc<ManualScanViewModel>,
    memory_settings_view_model: Arc<MemorySettingsViewModel>,
    output_view_model: Arc<OutputViewModel>,
    scan_settings_view_model: Arc<ScanSettingsViewModel>,
}

/// Wraps the slint main window to internally manage and track the view handle for later use, as well as setting up
/// view code bindings to the corresponding slint UI.
impl MainWindowViewModel {
    pub fn new() -> Self {
        let view_handle = Arc::new(MainWindowView::new().unwrap());
        let docking_layout = Arc::new(RefCell::new(DockingLayout::default()));

        let view = MainWindowViewModel {
            view_handle: view_handle.clone(),
            docking_layout: docking_layout.clone(),
            docked_window_view_model: Arc::new(DockedWindowViewModel::new(view_handle.clone(), docking_layout.clone())),
            manual_scan_view_model: Arc::new(ManualScanViewModel::new(view_handle.clone())),
            memory_settings_view_model: Arc::new(MemorySettingsViewModel::new(view_handle.clone())),
            output_view_model: Arc::new(OutputViewModel::new(view_handle.clone())),
            scan_settings_view_model: Arc::new(ScanSettingsViewModel::new(view_handle.clone())),
        };

        view.create_view_bindings();
        Self::propagate_layout(&view_handle, &view.docking_layout);

        return view;
    }

    pub fn show(&self) {
        match self.view_handle.show() {
            Ok(_) => {}
            Err(e) => {
                Logger::get_instance().log(LogLevel::Error, "Error showing the main window.", Some(e.to_string().as_str()));
            }
        }
    }

    pub fn hide(&self) {
        match self.view_handle.hide() {
            Ok(_) => {}
            Err(e) => {
                Logger::get_instance().log(LogLevel::Error, "Error hiding the main window.", Some(e.to_string().as_str()));
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

    pub fn get_scan_settings_view_model(&self) -> &Arc<ScanSettingsViewModel> {
        return &self.scan_settings_view_model;
    }

    fn propagate_layout(
        view_handle: &Arc<MainWindowView>,
        docking_layout: &Arc<RefCell<DockingLayout>>,
    ) {
        let project_explorer_identifier = "project-explorer";
        let property_viewer_identifier = "property-viewer";
        let scan_results_identifier = "scan-results";
        let output_identifier = "output";
        let settings_identifier = "settings";

        let view = view_handle.global::<DockedWindowViewModelBindings>();
        let docking_layout = docking_layout.borrow();

        if let Some(docked_window_bounds) = docking_layout.calculate_window_rect(project_explorer_identifier) {
            view.set_project_explorer_panel(crate::DockedWindowData {
                identifier: project_explorer_identifier.into(),
                is_docked: true,
                position_x: docked_window_bounds.0,
                position_y: docked_window_bounds.1,
                width: docked_window_bounds.2,
                height: docked_window_bounds.3,
            });
        }

        if let Some(docked_window_bounds) = docking_layout.calculate_window_rect(property_viewer_identifier) {
            view.set_property_viewer_panel(crate::DockedWindowData {
                identifier: property_viewer_identifier.into(),
                is_docked: true,
                position_x: docked_window_bounds.0,
                position_y: docked_window_bounds.1,
                width: docked_window_bounds.2,
                height: docked_window_bounds.3,
            });
        }

        if let Some(docked_window_bounds) = docking_layout.calculate_window_rect(project_explorer_identifier) {
            view.set_project_explorer_panel(crate::DockedWindowData {
                identifier: project_explorer_identifier.into(),
                is_docked: true,
                position_x: docked_window_bounds.0,
                position_y: docked_window_bounds.1,
                width: docked_window_bounds.2,
                height: docked_window_bounds.3,
            });
        }

        if let Some(docked_window_bounds) = docking_layout.calculate_window_rect(scan_results_identifier) {
            view.set_scan_results_panel(crate::DockedWindowData {
                identifier: scan_results_identifier.into(),
                is_docked: true,
                position_x: docked_window_bounds.0,
                position_y: docked_window_bounds.1,
                width: docked_window_bounds.2,
                height: docked_window_bounds.3,
            });
        }

        if let Some(docked_window_bounds) = docking_layout.calculate_window_rect(output_identifier) {
            view.set_output_panel(crate::DockedWindowData {
                identifier: output_identifier.into(),
                is_docked: true,
                position_x: docked_window_bounds.0,
                position_y: docked_window_bounds.1,
                width: docked_window_bounds.2,
                height: docked_window_bounds.3,
            });
        }

        if let Some(docked_window_bounds) = docking_layout.calculate_window_rect(settings_identifier) {
            view.set_settings_panel(crate::DockedWindowData {
                identifier: settings_identifier.into(),
                is_docked: true,
                position_x: docked_window_bounds.0,
                position_y: docked_window_bounds.1,
                width: docked_window_bounds.2,
                height: docked_window_bounds.3,
            });
        }
    }
}

impl ViewModel for MainWindowViewModel {
    fn create_view_bindings(&self) {
        let main_window_view = self.view_handle.global::<WindowViewModelBindings>();
        let docked_window_view = self.view_handle.global::<DockedWindowViewModelBindings>();

        // Bind our output viewmodel to the logger.
        Logger::get_instance().subscribe(self.output_view_model.clone());

        let view_handle = self.view_handle.clone();
        let docking_layout = self.docking_layout.clone();
        docked_window_view.on_update_dock_root_size(move |width, height| {
            docking_layout.borrow_mut().set_available_width(width);
            docking_layout.borrow_mut().set_available_height(height);
            Self::propagate_layout(&view_handle, &docking_layout);
            return 0.0;
        });

        let view_handle = self.view_handle.clone();
        main_window_view.on_minimize(move || {
            view_handle.window().set_minimized(true);
        });

        let view_handle = self.view_handle.clone();
        main_window_view.on_maximize(move || {
            view_handle
                .window()
                .set_maximized(!view_handle.window().is_maximized());
        });

        main_window_view.on_close(move || {
            let _ = slint::quit_event_loop();
        });

        let view_handle = self.view_handle.clone();
        main_window_view.on_double_clicked(move || {
            view_handle
                .window()
                .set_maximized(!view_handle.window().is_maximized());
        });

        let view_handle = self.view_handle.clone();
        main_window_view.on_drag(move |delta_x, delta_y| {
            let mut position = view_handle.window().position();
            position.x = position.x + delta_x;
            position.y = position.y + delta_y;
            view_handle.window().set_position(position);
        });
    }
}
