use crate::view_models::view_model::ViewModel;
use crate::DockedWindowViewModelBindings;
use crate::MainWindowView;
use slint::ComponentHandle;
use std::sync::Arc;

pub struct DockedWindowViewModel {
    view_handle: Arc<MainWindowView>,
}

impl DockedWindowViewModel {
    pub fn new(view_handle: Arc<MainWindowView>) -> Self {
        let view = DockedWindowViewModel {
            view_handle: view_handle.clone(),
        };

        view.create_bindings();

        return view;
    }
}

impl ViewModel for DockedWindowViewModel {
    fn create_bindings(&self) {
        let view = self.view_handle.global::<DockedWindowViewModelBindings>();

        let view_handle = self.view_handle.clone();
        view.on_minimize(move || {});

        let view_handle = self.view_handle.clone();
        view.on_maximize(move || {});

        view.on_close(move || {});

        let view_handle = self.view_handle.clone();
        view.on_double_clicked(move || {});

        let view_handle = self.view_handle.clone();
        view.on_drag_left(move |delta_x, delta_y| {});

        let view_handle = self.view_handle.clone();
        view.on_drag_right(move |delta_x, delta_y| {});

        let view_handle = self.view_handle.clone();
        view.on_drag_top(move |delta_x, delta_y| {});

        let view_handle = self.view_handle.clone();
        view.on_drag_bottom(move |delta_x, delta_y| {});
    }
}
