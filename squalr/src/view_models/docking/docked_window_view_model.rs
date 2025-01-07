use crate::DockedWindowViewModelBindings;
use crate::MainWindowView;
use crate::models::docking::docking_layout::DockingLayout;
use crate::mvvm::view_binding::ViewBinding;
use slint::ComponentHandle;
use slint::SharedString;
use slint_mvvm_macros::create_view_bindings;
use std::sync::Arc;
use std::sync::Mutex;

pub struct DockedWindowViewModel {
    view_binding: ViewBinding<MainWindowView>,
    docking_layout: Arc<Mutex<DockingLayout>>,
}

impl DockedWindowViewModel {
    pub fn new(
        view_binding: ViewBinding<MainWindowView>,
        docking_layout: Arc<Mutex<DockingLayout>>,
    ) -> Self {
        let view = DockedWindowViewModel {
            view_binding: view_binding,
            docking_layout: docking_layout.clone(),
        };

        create_view_bindings!(
            view.view_binding.clone(),
            {
                DockedWindowViewModelBindings => {
                    {
                        on_minimize() => Self::on_minimize
                    },
                    {
                        on_maximize() => Self::on_maximize
                    },
                    {
                        on_close() => Self::on_close
                    },
                    {
                        on_double_clicked() => Self::on_double_clicked
                    },
                    {
                        on_drag_left(dockable_window_id: SharedString, delta_x: i32, delta_y: i32)
                            => Self::on_drag_left
                    },
                    {
                        on_drag_right(dockable_window_id: SharedString, delta_x: i32, delta_y: i32)
                            => Self::on_drag_right
                    },
                    {
                        on_drag_top(dockable_window_id: SharedString, delta_x: i32, delta_y: i32)
                            => Self::on_drag_top
                    },
                    {
                        on_drag_bottom(dockable_window_id: SharedString, delta_x: i32, delta_y: i32)
                            => Self::on_drag_bottom
                    }
                }
            }
        );

        return view;
    }

    fn on_minimize() {
        // TODO: Implement as needed
    }

    fn on_maximize() {
        // TODO: Implement as needed
    }

    fn on_close() {
        // TODO: Implement as needed
    }

    fn on_double_clicked() {
        // TODO: Implement as needed
    }

    fn on_drag_left(
        _dockable_window_id: SharedString,
        _delta_x: i32,
        _delta_y: i32,
    ) {
        // TODO: Implement as needed
    }

    fn on_drag_right(
        _dockable_window_id: SharedString,
        _delta_x: i32,
        _delta_y: i32,
    ) {
        // TODO: Implement as needed
    }

    fn on_drag_top(
        _dockable_window_id: SharedString,
        _delta_x: i32,
        _delta_y: i32,
    ) {
        // TODO: Implement as needed
    }

    fn on_drag_bottom(
        _dockable_window_id: SharedString,
        _delta_x: i32,
        _delta_y: i32,
    ) {
        // TODO: Implement as needed
    }
}
