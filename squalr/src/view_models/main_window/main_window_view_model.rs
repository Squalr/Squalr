use crate::view_models::output::output_view_model::OutputViewModel;
use crate::view_models::scanners::manual_scan_view_model::ManualScanViewModel;
use crate::view_models::view_model::ViewModel;
use crate::MainWindowView;
use crate::WindowViewModelBindings;
use slint::ComponentHandle;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use std::sync::Arc;

pub struct MainWindowViewModel {
    view_handle: Arc<MainWindowView>,
    manual_scan_view_model: Arc<ManualScanViewModel>,
    output_view_model: Arc<OutputViewModel>,
}

/// Wraps the slint main window to internally manage and track the view handle for later use, as well as setting up
/// view code bindings to the corresponding slint UI.
impl MainWindowViewModel {
    pub fn new() -> Self {
        let view_handle = Arc::new(MainWindowView::new().unwrap());
        let view = MainWindowViewModel {
            view_handle: view_handle.clone(),
            manual_scan_view_model: Arc::new(ManualScanViewModel::new(view_handle.clone())),
            output_view_model: Arc::new(OutputViewModel::new(view_handle.clone())),
        };

        view.create_bindings();

        return view;
    }

    pub fn show(&self) {
        match self.view_handle.show() {
            Ok(_) => {}
            Err(e) => {
                Logger::get_instance().log(LogLevel::Error, "Fatal error showing the main window.", Some(e.to_string().as_str()));
            }
        }
    }

    pub fn get_manual_scan_view(&self) -> &Arc<ManualScanViewModel> {
        return &self.manual_scan_view_model;
    }
}

impl ViewModel for MainWindowViewModel {
    fn create_bindings(&self) {
        let view = self.view_handle.global::<WindowViewModelBindings>();

        // Bind our output viewmodel to the Squalr logger
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
