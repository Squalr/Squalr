use crate::view_models::view_model_base::ViewModel;
use crate::view_models::view_model_base::ViewModelBase;
use crate::UndockedWindowView;
use crate::WindowViewModelBindings;
use slint::ComponentHandle;

pub struct UndockedWindowViewModel {
    view_model_base: ViewModelBase<UndockedWindowView>,
}

/// Wraps the slint main window to internally manage and track the view handle for later use, as well as setting up
/// view code bindings to the corresponding slint UI.
impl UndockedWindowViewModel {
    pub fn new() -> Self {
        let view = UndockedWindowView::new().unwrap();
        let view_model_base = ViewModelBase::new(ComponentHandle::as_weak(&view));
        let view = UndockedWindowViewModel {
            view_model_base: view_model_base.clone(),
        };

        return view;
    }

    pub fn show(&self) {
        self.view_model_base
            .execute_on_ui_thread(move |undocked_window_view, _view_model_base| {
                if let Err(err) = undocked_window_view.show() {
                    log::error!("Error showing an undocked window: {err}");
                }
            });
    }

    pub fn hide(&self) {
        self.view_model_base
            .execute_on_ui_thread(move |undocked_window_view, _view_model_base| {
                if let Err(err) = undocked_window_view.hide() {
                    log::error!("Error hiding an undocked window: {err}");
                }
            });
    }
}

impl ViewModel for UndockedWindowViewModel {
    fn create_view_bindings(&self) {
        self.view_model_base
            .execute_on_ui_thread(move |undocked_window_view, view_model_base| {
                let docked_window_bindings = undocked_window_view.global::<WindowViewModelBindings>();

                // Set up minimize handler
                let view_model = view_model_base.clone();
                docked_window_bindings.on_minimize(move || {
                    view_model.execute_on_ui_thread(move |undocked_window_view, _view_model_base| {
                        let window = undocked_window_view.window();
                        window.set_minimized(true);
                    });
                });

                // Set up maximize handler
                let view_model = view_model_base.clone();
                docked_window_bindings.on_maximize(move || {
                    view_model.execute_on_ui_thread(move |undocked_window_view, _view_model_base| {
                        let window = undocked_window_view.window();
                        window.set_maximized(!window.is_maximized());
                    });
                });

                // Set up close handler
                docked_window_bindings.on_close(move || {
                    if let Err(e) = slint::quit_event_loop() {
                        log::error!("Failed to quit event loop: {}", e);
                    }
                });

                // Set up double click handler
                let view_model = view_model_base.clone();
                docked_window_bindings.on_double_clicked(move || {
                    view_model.execute_on_ui_thread(move |undocked_window_view, _view_model_base| {
                        let window = undocked_window_view.window();
                        window.set_maximized(!window.is_maximized());
                    });
                });

                // Set up drag handler
                let view_model = view_model_base.clone();
                docked_window_bindings.on_drag(move |delta_x: i32, delta_y| {
                    view_model.execute_on_ui_thread(move |undocked_window_view, _view_model_base| {
                        let window = undocked_window_view.window();
                        let mut position = window.position();
                        position.x += delta_x;
                        position.y += delta_y;
                        window.set_position(position);
                    });
                });
            });
    }
}
