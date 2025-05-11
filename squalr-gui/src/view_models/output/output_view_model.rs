use crate::MainWindowView;
use crate::OutputViewModelBindings;
use crate::view_models::dependency_container::DependencyContainer;
use slint::ComponentHandle;
use slint_mvvm::view_binding::ViewBinding;
use squalr_engine::engine_execution_context::EngineExecutionContext;
use std::sync::Arc;
use std::thread;

pub struct OutputViewModel {
    _view_binding: Arc<ViewBinding<MainWindowView>>,
    _engine_execution_context: Arc<EngineExecutionContext>,
}

impl OutputViewModel {
    pub fn new(
        dependency_container: &DependencyContainer,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) -> anyhow::Result<Arc<Self>> {
        let view_binding = dependency_container.resolve::<ViewBinding<MainWindowView>>()?;
        let view = Arc::new(OutputViewModel {
            _view_binding: view_binding.clone(),
            _engine_execution_context: engine_execution_context.clone(),
        });

        match engine_execution_context.get_logger().subscribe_to_logs() {
            Ok(receiver) => {
                thread::spawn(move || {
                    while let Ok(log_message) = receiver.recv() {
                        Self::on_log_event(view_binding.clone(), log_message);
                    }
                });
            }
            Err(err) => {
                log::error!("Error subscribing to engine logs: {}", err);
            }
        }

        Ok(view)
    }

    fn on_log_event(
        view_binding: Arc<ViewBinding<MainWindowView>>,
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
