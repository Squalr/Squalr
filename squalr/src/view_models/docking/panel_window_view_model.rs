use crate::view_models::view_model::ViewModel;
use crate::PanelWindowView;
use slint::*;
use std::sync::Arc;

pub struct PanelWindowViewModel {
    view_handle: Arc<PanelWindowView>,
}

/// Wraps the slint main window to internally manage and track the view handle for later use, as well as setting up
/// view code bindings to the corresponding slint UI.
impl PanelWindowViewModel {
    pub fn new() -> Self {
        let view_handle = Arc::new(PanelWindowView::new().unwrap());
        let view = PanelWindowViewModel {
            view_handle: view_handle.clone(),
        };

        return view;
    }

    pub fn show(&self) {
        return self.view_handle.show().unwrap();
    }
}

impl ViewModel for PanelWindowViewModel {
    fn create_bindings(&self) {
        let view_handle = self.view_handle.clone();
        self.view_handle.on_minimize(move || {
            view_handle.window().set_minimized(true);
        });

        let view_handle = self.view_handle.clone();
        self.view_handle.on_maximize(move || {
            view_handle
                .window()
                .set_maximized(!view_handle.window().is_maximized());
        });

        self.view_handle.on_close(move || {
            let _ = slint::quit_event_loop();
        });

        let view_handle = self.view_handle.clone();
        self.view_handle.on_double_clicked(move || {
            view_handle
                .window()
                .set_maximized(!view_handle.window().is_maximized());
        });

        let view_handle = self.view_handle.clone();
        self.view_handle.on_drag(move |delta_x, delta_y| {
            let mut position = view_handle.window().position();
            position.x = position.x + delta_x;
            position.y = position.y + delta_y;
            view_handle.window().set_position(position);
        });
    }
}
