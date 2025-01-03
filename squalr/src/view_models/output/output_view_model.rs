use crate::view_models::view_model_base::ViewModel;
use crate::MainWindowView;
use crate::OutputViewModelBindings;
use slint::ComponentHandle;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger_observer::LoggerObserver;
use std::sync::Arc;

pub struct OutputViewModel {
    view_handle: Arc<MainWindowView>,
}

impl OutputViewModel {
    pub fn new(view_handle: Arc<MainWindowView>) -> Self {
        let view = OutputViewModel {
            view_handle: view_handle.clone(),
        };

        view.create_view_bindings();

        return view;
    }
}

impl LoggerObserver for OutputViewModel {
    fn on_log_event(
        &self,
        log_level: LogLevel,
        message: &str,
        inner_message: Option<&str>,
    ) {
        let log_message = match inner_message {
            Some(inner) => format!("[{:?}] {} - {}\n", log_level, message, inner),
            None => format!("[{:?}] {}\n", log_level, message),
        };

        let view = self.view_handle.global::<OutputViewModelBindings>();
        let mut shared_string = view.get_output_text();
        shared_string.push_str(log_message.as_str());
        view.set_output_text(shared_string);
    }
}

impl ViewModel for OutputViewModel {
    fn create_view_bindings(&self) {
        let _ = self.view_handle.global::<OutputViewModelBindings>();
    }
}
