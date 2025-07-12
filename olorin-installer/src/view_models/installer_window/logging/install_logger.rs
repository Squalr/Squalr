use crate::InstallerViewModelBindings;
use crate::InstallerWindowView;
use log::{Level, Log, Metadata, Record, SetLoggerError};
use slint::ComponentHandle;
use slint_mvvm::view_binding::ViewBinding;
use std::sync::{Arc, Mutex};

pub struct InstallLogger {
    view_binding: ViewBinding<InstallerWindowView>,
    current_logs: Arc<Mutex<String>>,
}

impl InstallLogger {
    pub fn new(view_binding: ViewBinding<InstallerWindowView>) -> Self {
        InstallLogger {
            view_binding,
            current_logs: Arc::new(Mutex::new(String::new())),
        }
    }

    pub fn init(view_binding: ViewBinding<InstallerWindowView>) -> Result<(), SetLoggerError> {
        let logger = InstallLogger::new(view_binding);
        log::set_boxed_logger(Box::new(logger))?;
        log::set_max_level(log::LevelFilter::Info);
        Ok(())
    }
}

impl Log for InstallLogger {
    fn enabled(
        &self,
        metadata: &Metadata,
    ) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(
        &self,
        record: &Record,
    ) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let log_message = format!("[{}] {}\n", record.level(), record.args());

        // Update our internal log buffer
        if let Ok(mut logs) = self.current_logs.lock() {
            logs.push_str(&log_message);
        }

        // Update the UI
        let current_logs = self.current_logs.clone();
        self.view_binding
            .execute_on_ui_thread(move |installer_window_view, _view_binding| {
                if let Ok(logs) = current_logs.lock() {
                    let installer_view = installer_window_view.global::<InstallerViewModelBindings>();
                    installer_view.set_installer_logs(logs.clone().into());
                }
            });
    }

    fn flush(&self) {}
}
