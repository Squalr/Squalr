use crate::models::docking::docking_layout::DockingLayout;
use crate::view_models::view_model_base::ViewModel;
use crate::DockedWindowViewModelBindings;
use crate::MainWindowView;
use slint::ComponentHandle;
use std::cell::RefCell;
use std::sync::Arc;

pub struct DockedWindowViewModel {
    view_handle: Arc<MainWindowView>,
    docking_layout: Arc<RefCell<DockingLayout>>,
}

impl DockedWindowViewModel {
    pub fn new(
        view_handle: Arc<MainWindowView>,
        docking_layout: Arc<RefCell<DockingLayout>>,
    ) -> Self {
        let view = DockedWindowViewModel {
            view_handle: view_handle.clone(),
            docking_layout: docking_layout.clone(),
        };

        view.create_view_bindings();

        return view;
    }
}

impl ViewModel for DockedWindowViewModel {
    fn create_view_bindings(&self) {
        let docked_window_view = self.view_handle.global::<DockedWindowViewModelBindings>();

        let view_handle = self.view_handle.clone();
        docked_window_view.on_minimize(move || {});

        let view_handle = self.view_handle.clone();
        docked_window_view.on_maximize(move || {});

        docked_window_view.on_close(move || {});

        let view_handle = self.view_handle.clone();
        docked_window_view.on_double_clicked(move || {});

        let view_handle = self.view_handle.clone();
        docked_window_view.on_drag_left(move |dockable_window_id, delta_x, delta_y| {});

        let view_handle = self.view_handle.clone();
        docked_window_view.on_drag_right(move |dockable_window_id, delta_x, delta_y| {});

        let view_handle = self.view_handle.clone();
        docked_window_view.on_drag_top(move |dockable_window_id, delta_x, delta_y| {});

        let view_handle = self.view_handle.clone();
        docked_window_view.on_drag_bottom(move |dockable_window_id, delta_x, delta_y| {});
    }
}
