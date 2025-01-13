use crate::DockedWindowViewModelBindings;
use crate::MainWindowView;
use crate::models::docking::docking_layout::DockingLayout;
use slint::ComponentHandle;
use slint::SharedString;
use slint_mvvm::view_binding::ViewBinding;
use slint_mvvm_macros::create_view_bindings;
use std::sync::Arc;
use std::sync::RwLock;

pub struct DockedWindowViewModel {
    _view_binding: ViewBinding<MainWindowView>,
    _docking_layout: Arc<RwLock<DockingLayout>>,
}

impl DockedWindowViewModel {
    pub fn new(
        view_binding: ViewBinding<MainWindowView>,
        docking_layout: Arc<RwLock<DockingLayout>>,
    ) -> Self {
        let view = DockedWindowViewModel {
            _view_binding: view_binding.clone(),
            _docking_layout: docking_layout.clone(),
        };

        create_view_bindings!(view_binding, {
            DockedWindowViewModelBindings => {
                    on_minimize() -> [] -> Self::on_minimize,
                    on_maximize() -> [] -> Self::on_maximize,
                    on_close() -> [view_binding] -> Self::on_close,
                    on_double_clicked() -> [] -> Self::on_double_clicked,
                    on_drag_left(dockable_window_id: SharedString, delta_x: i32, delta_y: i32) -> [] -> Self::on_drag_left,
                    on_drag_right(dockable_window_id: SharedString, delta_x: i32, delta_y: i32) -> [] -> Self::on_drag_right,
                    on_drag_top(dockable_window_id: SharedString, delta_x: i32, delta_y: i32) -> [] -> Self::on_drag_top,
                    on_drag_bottom(dockable_window_id: SharedString, delta_x: i32, delta_y: i32) -> [] -> Self::on_drag_bottom,
            }
        });

        view
    }

    fn on_minimize() {
        // TODO: Implement me.
    }

    fn on_maximize() {
        // TODO: Implement me.
    }

    fn on_close(view_binding: ViewBinding<MainWindowView>) {
        view_binding.execute_on_ui_thread(|_view_binding, _main_window_view_model| {
            //
        });
    }

    fn on_double_clicked() {
        // TODO: Implement me.
    }

    fn on_drag_left(
        _dockable_window_id: SharedString,
        _delta_x: i32,
        _delta_y: i32,
    ) {
        // TODO: Implement me.
    }

    fn on_drag_right(
        _dockable_window_id: SharedString,
        _delta_x: i32,
        _delta_y: i32,
    ) {
        // TODO: Implement me.
    }

    fn on_drag_top(
        _dockable_window_id: SharedString,
        _delta_x: i32,
        _delta_y: i32,
    ) {
        // TODO: Implement me.
    }

    fn on_drag_bottom(
        _dockable_window_id: SharedString,
        _delta_x: i32,
        _delta_y: i32,
    ) {
        // TODO: Implement me.
    }
}
