use crate::MainWindowView;
use crate::OutputViewModelBindings;
use slint::ComponentHandle;
use slint_mvvm::view_binding::ViewBinding;
use squalr_engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_common::logging::file_system_logger::FileSystemLogger;
use std::sync::Arc;
use std::thread;

pub struct OutputViewModel {
    _view_binding: ViewBinding<MainWindowView>,
    _engine_execution_context: Arc<EngineExecutionContext>,
}

impl OutputViewModel {
    pub fn new(
        view_binding: ViewBinding<MainWindowView>,
        engine_execution_context: Arc<EngineExecutionContext>,
        file_system_logger: Arc<FileSystemLogger>,
    ) -> Self {
        let view = OutputViewModel {
            _view_binding: view_binding.clone(),
            _engine_execution_context: engine_execution_context,
        };

        let receiver = file_system_logger.subscribe_to_logs();

        thread::spawn(move || {
            while let Ok(log_message) = receiver.recv() {
                Self::on_log_event(view_binding.clone(), log_message);
            }
        });

        view
    }

    fn on_log_event(
        view_binding: ViewBinding<MainWindowView>,
        log_message: String,
    ) {
        view_binding.execute_on_ui_thread(move |main_window_view, _view_binding| {
            let view = main_window_view.global::<OutputViewModelBindings>();
            let mut shared_string = view.get_output_text();
            shared_string.push_str(&log_message);
            view.set_output_text(shared_string);
        });
    }
}
