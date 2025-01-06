use crate::models::docking::docking_layout::DockingLayout;
use crate::view_models::view_model_base::ViewModel;
use crate::view_models::view_model_base::ViewModelBase;
use crate::DockedWindowViewModelBindings;
use crate::MainWindowView;
use slint::ComponentHandle;
use std::sync::Arc;
use std::sync::Mutex;

pub struct DockedWindowViewModel {
    view_model_base: ViewModelBase<MainWindowView>,
    docking_layout: Arc<Mutex<DockingLayout>>,
}

impl DockedWindowViewModel {
    pub fn new(
        view_model_base: ViewModelBase<MainWindowView>,
        docking_layout: Arc<Mutex<DockingLayout>>,
    ) -> Self {
        let view = DockedWindowViewModel {
            view_model_base: view_model_base,
            docking_layout: docking_layout.clone(),
        };

        view.create_view_bindings();

        return view;
    }
}

impl ViewModel for DockedWindowViewModel {
    fn create_view_bindings(&self) {
        self.view_model_base
            .execute_on_ui_thread(|main_window_view, view_model_base| {
                let docked_window_view = main_window_view.global::<DockedWindowViewModelBindings>();

                docked_window_view.on_minimize(move || {});

                docked_window_view.on_maximize(move || {});

                docked_window_view.on_close(move || {});

                docked_window_view.on_double_clicked(move || {});

                docked_window_view.on_drag_left(move |dockable_window_id, delta_x, delta_y| {});

                docked_window_view.on_drag_right(move |dockable_window_id, delta_x, delta_y| {});

                docked_window_view.on_drag_top(move |dockable_window_id, delta_x, delta_y| {});

                docked_window_view.on_drag_bottom(move |dockable_window_id, delta_x, delta_y| {});
            });
    }
}
