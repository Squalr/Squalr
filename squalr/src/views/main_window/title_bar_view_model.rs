use crate::views::view_model::ViewModel;
use crate::MainWindowView;
use crate::TitleBarAdapter;
use slint::ComponentHandle;
use std::sync::Arc;

pub struct TitleBarViewModel {
    view_handle: Arc<MainWindowView>,
}

/// Wraps the slint main window to internally manage and track the view handle for later use, as well as setting up
/// view code bindings to the corresponding slint UI.
impl TitleBarViewModel {
    pub fn new(view_handle: Arc<MainWindowView>) -> Self {
        let view = TitleBarViewModel {
            view_handle: view_handle.clone(),
        };

        view.create_bindings();

        return view;
    }
}

impl ViewModel for TitleBarViewModel {
    fn create_bindings(&self) {
        let title_bar_adapter = self.view_handle.global::<TitleBarAdapter>();

        let view_handle = self.view_handle.clone();
        title_bar_adapter.on_minimize(move || {
            view_handle.window().set_minimized(true);
        });

        let view_handle = self.view_handle.clone();
        title_bar_adapter.on_maximize(move || {
            view_handle
                .window()
                .set_maximized(!view_handle.window().is_maximized());
        });

        title_bar_adapter.on_close(move || {
            /*
            let _ = slint::invoke_from_event_loop(|| {
                PanelWindowViewModel::new().show();
            }); */

            let _ = slint::quit_event_loop();
        });

        let view_handle = self.view_handle.clone();
        title_bar_adapter.on_drag(move |delta_x, delta_y| {
            let mut position = view_handle.window().position();
            position.x = position.x + delta_x;
            position.y = position.y + delta_y;
            view_handle.window().set_position(position);
        });
    }
}
