use crate::MainWindowView;
use crate::OutputViewModelBindings;
use crate::mvvm::view_binding::ViewModel;
use crate::mvvm::view_binding::ViewBinding;
use slint::ComponentHandle;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger_observer::LoggerObserver;

pub struct OutputViewModel {
    view_binding: ViewBinding<MainWindowView>,
}

impl OutputViewModel {
    pub fn new(view_binding: ViewBinding<MainWindowView>) -> Self {
        let view = OutputViewModel {
            view_binding: view_binding,
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

        self.view_binding
            .execute_on_ui_thread(move |main_window_view, _view_binding| {
                let view = main_window_view.global::<OutputViewModelBindings>();
                let mut shared_string = view.get_output_text();
                shared_string.push_str(log_message.as_str());
                view.set_output_text(shared_string);
            });
    }
}

impl ViewModel for OutputViewModel {
    fn create_view_bindings(&self) {
        self.view_binding
            .execute_on_ui_thread(move |_main_window_view, _view_binding| {
                // TODO
            });
    }
}
