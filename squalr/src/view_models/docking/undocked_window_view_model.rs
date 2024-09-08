use crate::view_models::view_model::ViewModel;
use crate::UndockedWindowView;
use crate::WindowViewModelBindings;
use slint::*;
use std::sync::Arc;

pub struct UndockedWindowViewModel {
    view_handle: Arc<UndockedWindowView>,
}

/// Wraps the slint main window to internally manage and track the view handle for later use, as well as setting up
/// view code bindings to the corresponding slint UI.
impl UndockedWindowViewModel {
    pub fn new() -> Self {
        let view_handle = Arc::new(UndockedWindowView::new().unwrap());
        let view = UndockedWindowViewModel {
            view_handle: view_handle.clone(),
        };

        return view;
    }

    pub fn show(&self) {
        return self.view_handle.show().unwrap();
    }
}

impl ViewModel for UndockedWindowViewModel {
    fn create_bindings(&self) {
        let view = self.view_handle.global::<WindowViewModelBindings>();

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
