use crate::models::docking::docking_layout::DockingLayout;
use crate::view_models::output::output_view_model::OutputViewModel;
use crate::view_models::scanners::manual_scan_view_model::ManualScanViewModel;
use crate::view_models::settings::memory_settings_view_model::MemorySettingsViewModel;
use crate::view_models::settings::scan_settings_view_model::ScanSettingsViewModel;
use crate::view_models::view_model::ViewModel;
use crate::DockedWindowViewModelBindings;
use crate::MainWindowView;
use crate::WindowViewModelBindings;
use slint::ComponentHandle;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use std::sync::Arc;

pub struct MainWindowViewModel {
    view_handle: Arc<MainWindowView>,
    docking_layout: DockingLayout,
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
        let view = MainWindowViewModel {
            view_handle: view_handle.clone(),
            docking_layout: DockingLayout::default(),
            manual_scan_view_model: Arc::new(ManualScanViewModel::new(view_handle.clone())),
            memory_settings_view_model: Arc::new(MemorySettingsViewModel::new(view_handle.clone())),
            output_view_model: Arc::new(OutputViewModel::new(view_handle.clone())),
            scan_settings_view_model: Arc::new(ScanSettingsViewModel::new(view_handle.clone())),
        };

        view.create_bindings();
        view.propagate_layout();

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

    pub fn get_manual_scan_view_model(&self) -> &Arc<ManualScanViewModel> {
        return &self.manual_scan_view_model;
    }

    pub fn get_memory_settings_view_model(&self) -> &Arc<MemorySettingsViewModel> {
        return &self.memory_settings_view_model;
    }

    pub fn get_scan_settings_view_model(&self) -> &Arc<ScanSettingsViewModel> {
        return &self.scan_settings_view_model;
    }

    fn propagate_layout(&self) {
        let project_explorer_identifier = "project-explorer";
        let property_viewer_identifier = "property-viewer";
        let scan_results_identifier = "scan-results";
        let output_identifier = "output";
        let settings_identifier = "settings";

        let view = self.view_handle.global::<DockedWindowViewModelBindings>();

        let root_width = 1280.0; //view.get_dock_root_width();
        let root_height = 840.0; // view.get_dock_root_height();

        if let Some(docked_window_bounds) = self
            .docking_layout
            .calculate_window_rect(project_explorer_identifier, root_width, root_height)
        {
            view.set_project_explorer_panel(crate::DockedWindowData {
                identifier: project_explorer_identifier.into(),
                is_docked: true,
                position_x: docked_window_bounds.0,
                position_y: docked_window_bounds.1,
                width: docked_window_bounds.2,
                height: docked_window_bounds.3,
            });
        }

        if let Some(docked_window_bounds) = self
            .docking_layout
            .calculate_window_rect(property_viewer_identifier, root_width, root_height)
        {
            view.set_property_viewer_panel(crate::DockedWindowData {
                identifier: property_viewer_identifier.into(),
                is_docked: true,
                position_x: docked_window_bounds.0,
                position_y: docked_window_bounds.1,
                width: docked_window_bounds.2,
                height: docked_window_bounds.3,
            });
        }

        if let Some(docked_window_bounds) = self
            .docking_layout
            .calculate_window_rect(project_explorer_identifier, root_width, root_height)
        {
            view.set_project_explorer_panel(crate::DockedWindowData {
                identifier: project_explorer_identifier.into(),
                is_docked: true,
                position_x: docked_window_bounds.0,
                position_y: docked_window_bounds.1,
                width: docked_window_bounds.2,
                height: docked_window_bounds.3,
            });
        }

        if let Some(docked_window_bounds) = self
            .docking_layout
            .calculate_window_rect(scan_results_identifier, root_width, root_height)
        {
            view.set_scan_results_panel(crate::DockedWindowData {
                identifier: scan_results_identifier.into(),
                is_docked: true,
                position_x: docked_window_bounds.0,
                position_y: docked_window_bounds.1,
                width: docked_window_bounds.2,
                height: docked_window_bounds.3,
            });
        }

        if let Some(docked_window_bounds) = self
            .docking_layout
            .calculate_window_rect(output_identifier, root_width, root_height)
        {
            view.set_output_panel(crate::DockedWindowData {
                identifier: output_identifier.into(),
                is_docked: true,
                position_x: docked_window_bounds.0,
                position_y: docked_window_bounds.1,
                width: docked_window_bounds.2,
                height: docked_window_bounds.3,
            });
        }

        if let Some(docked_window_bounds) = self
            .docking_layout
            .calculate_window_rect(settings_identifier, root_width, root_height)
        {
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
    fn create_bindings(&self) {
        let view = self.view_handle.global::<WindowViewModelBindings>();

        // Bind our output viewmodel to the logger.
        Logger::get_instance().subscribe(self.output_view_model.clone());

        let view_handle = self.view_handle.clone();
        view.on_minimize(move || {
            view_handle.window().set_minimized(true);
        });

        let view_handle = self.view_handle.clone();
        view.on_maximize(move || {
            view_handle
                .window()
                .set_maximized(!view_handle.window().is_maximized());
        });

        view.on_close(move || {
            let _ = slint::quit_event_loop();
        });

        let view_handle = self.view_handle.clone();
        view.on_double_clicked(move || {
            view_handle
                .window()
                .set_maximized(!view_handle.window().is_maximized());
        });

        let view_handle = self.view_handle.clone();
        view.on_drag(move |delta_x, delta_y| {
            let mut position = view_handle.window().position();
            position.x = position.x + delta_x;
            position.y = position.y + delta_y;
            view_handle.window().set_position(position);
        });
    }
}
