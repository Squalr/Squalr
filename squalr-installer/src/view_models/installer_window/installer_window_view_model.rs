use crate::InstallerViewModelBindings;
use crate::InstallerWindowView;
use crate::WindowViewModelBindings;
use crate::view_models::installer_window::logging::install_logger::InstallLogger;
use slint::ComponentHandle;
use slint_mvvm::view_binding::ViewBinding;
use slint_mvvm_macros::create_view_bindings;
use squalr_engine::app_provisioner::installer::app_installer::AppInstaller;
use squalr_engine::app_provisioner::installer::install_phase::InstallPhase;
use squalr_engine::app_provisioner::{app_provisioner_config::AppProvisionerConfig, progress_tracker::ProgressTracker};

pub struct InstallerWindowViewModel {
    _view: InstallerWindowView,
    view_binding: ViewBinding<InstallerWindowView>,
}

impl InstallerWindowViewModel {
    pub fn new() -> Self {
        let view = InstallerWindowView::new().unwrap();
        let view_binding = ViewBinding::new(ComponentHandle::as_weak(&view));

        // Initialize the logger such that we can bind logs to the view.
        if let Err(err) = InstallLogger::init(view_binding.clone()) {
            eprintln!("Failed to initialize UI logger: {}", err);
        }

        let view = InstallerWindowViewModel {
            _view: view,
            view_binding: view_binding.clone(),
        };

        create_view_bindings!(view_binding, {
            WindowViewModelBindings => {
                on_minimize() -> [view_binding] -> Self::on_minimize,
                on_maximize() -> [view_binding] -> Self::on_maximize,
                on_close() -> [] -> Self::on_close,
                on_double_clicked() -> [view_binding] -> Self::on_double_clicked,
                on_drag(delta_x: i32, delta_y: i32) -> [view_binding] -> Self::on_drag
            },
            InstallerViewModelBindings => {
                on_launch_app() -> [] -> Self::on_launch_app
            }
        });

        view.start_installer_with_progress_tracking();

        return view;
    }

    pub fn initialize(&self) {
        self.show();
    }

    pub fn show(&self) {
        self.view_binding
            .execute_on_ui_thread(move |installer_window_view, _view_binding| {
                if let Err(err) = installer_window_view.show() {
                    log::error!("Error showing the installer window: {err}");
                }
            });
    }

    pub fn hide(&self) {
        self.view_binding
            .execute_on_ui_thread(move |installer_window_view, _view_binding| {
                if let Err(err) = installer_window_view.hide() {
                    log::error!("Error hiding the installer window: {err}");
                }
            });
    }

    fn start_installer_with_progress_tracking(&self) {
        let view_binding = self.view_binding.clone();

        match AppProvisionerConfig::get_default_install_dir() {
            Ok(install_dir) => {
                let progress_tracker = ProgressTracker::new();
                let receiver = progress_tracker.subscribe();

                std::thread::spawn(move || {
                    for progress in receiver {
                        view_binding.execute_on_ui_thread(move |installer_window_view, _view_binding| {
                            let installer_view = installer_window_view.global::<InstallerViewModelBindings>();
                            installer_view.set_installer_progress(progress.progress_percent as f32);
                            installer_view.set_installer_progress_string(format!("{:.0}%", progress.progress_percent as f32 * 100.0).into());

                            if progress.phase == InstallPhase::Complete {
                                installer_view.set_install_complete(true);
                            }
                        });
                    }
                });

                AppInstaller::run_installation(install_dir, progress_tracker)
            }
            Err(err) => log::error!("Failed to get default install directory: {}", err),
        }
    }
    fn on_minimize(view_binding: ViewBinding<InstallerWindowView>) {
        view_binding.execute_on_ui_thread(|installer_window_view, _| {
            installer_window_view.window().set_minimized(true);
        });
    }

    fn on_maximize(view_binding: ViewBinding<InstallerWindowView>) {
        view_binding.execute_on_ui_thread(|installer_window_view, _| {
            let window = installer_window_view.window();
            window.set_maximized(!window.is_maximized());
        });
    }

    fn on_close() {
        if let Err(e) = slint::quit_event_loop() {
            log::error!("Failed to quit event loop: {}", e);
        }
    }

    fn on_double_clicked(view_binding: ViewBinding<InstallerWindowView>) {
        view_binding.execute_on_ui_thread(|installer_window_view, _| {
            let window = installer_window_view.window();
            window.set_maximized(!window.is_maximized());
        });
    }

    fn on_drag(
        view_binding: ViewBinding<InstallerWindowView>,
        delta_x: i32,
        delta_y: i32,
    ) {
        view_binding.execute_on_ui_thread(move |installer_window_view, _| {
            let window = installer_window_view.window();
            let mut position = window.position();
            position.x += delta_x;
            position.y += delta_y;
            window.set_position(position);
        });
    }

    fn on_launch_app() {
        /*
        match AppInstaller::get_instance().read() {
            Ok(app_installer) => {
                app_installer.launch_app();
            }
            Err(err) => {
                log::error!("Failed to acquire lock for launching app: {err}");
            }
        }*/
    }
}
