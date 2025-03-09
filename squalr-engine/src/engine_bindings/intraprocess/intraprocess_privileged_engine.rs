use crate::engine_bindings::engine_priviliged_bindings::EnginePrivilegedBindings;
use crate::engine_execution_context::EngineExecutionContext;
use crate::engine_privileged_state::EnginePrivilegedState;
use std::sync::Arc;

pub struct IntraProcessPrivilegedEngine {
    engine_execution_context: Option<Arc<EngineExecutionContext>>,
}

impl EnginePrivilegedBindings for IntraProcessPrivilegedEngine {
    fn initialize(
        &mut self,
        _engine_privileged_state: &Option<Arc<EnginePrivilegedState>>,
        engine_execution_context: &Option<Arc<EngineExecutionContext>>,
    ) -> Result<(), String> {
        if let Some(engine_execution_context) = engine_execution_context {
            self.engine_execution_context = Some(engine_execution_context.clone());
            Ok(())
        } else {
            Err("No engine execution context provided! Engine event dispatching will be non-functional without this.".to_string())
        }
    }
}

impl IntraProcessPrivilegedEngine {
    pub fn new() -> IntraProcessPrivilegedEngine {
        let instance = IntraProcessPrivilegedEngine {
            engine_execution_context: None,
        };

        instance
    }
}
