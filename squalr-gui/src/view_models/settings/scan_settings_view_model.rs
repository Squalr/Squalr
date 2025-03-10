use crate::MainWindowView;
use slint_mvvm::view_binding::ViewBinding;
use squalr_engine::engine_execution_context::EngineExecutionContext;
use std::sync::Arc;

pub struct ScanSettingsViewModel {
    _view_binding: ViewBinding<MainWindowView>,
    _engine_execution_context: Arc<EngineExecutionContext>,
}

impl ScanSettingsViewModel {
    pub fn new(
        view_binding: ViewBinding<MainWindowView>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) -> Arc<Self> {
        let view = Arc::new(ScanSettingsViewModel {
            _view_binding: view_binding,
            _engine_execution_context: engine_execution_context,
        });

        view
    }
}
