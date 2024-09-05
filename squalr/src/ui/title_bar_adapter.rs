use slint::*;

use crate::{mvc::TitleBarController, ui};

// a helper function to make adapter and controller connection a little bit easier
fn connect_with_controller(
    view_handle: &ui::MainWindow,
    controller: &TitleBarController,
    connect_adapter_controller: impl FnOnce(ui::TitleBarAdapter, TitleBarController) + 'static,
) {
    connect_adapter_controller(view_handle.global::<ui::TitleBarAdapter>(), controller.clone());
}

// one place to implement connection between adapter (view) and controller
pub fn connect(
    view_handle: &ui::MainWindow,
    controller: TitleBarController,
) {
    connect_with_controller(view_handle, &controller, {
        move |adapter, controller| {
            adapter.on_minimize(move || {
                controller.minimize();
            })
        }
    });

    connect_with_controller(view_handle, &controller, {
        move |adapter, controller| {
            adapter.on_maximize(move || {
                controller.maximize();
            })
        }
    });

    connect_with_controller(view_handle, &controller, {
        move |adapter, controller| {
            adapter.on_close(move || {
                controller.close();
            })
        }
    });
}
