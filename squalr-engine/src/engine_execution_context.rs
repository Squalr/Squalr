use crate::engine_bindings::engine_unprivileged_bindings::EngineUnprivilegedBindings;
use crate::engine_bindings::interprocess::interprocess_unprivileged_host::InterprocessUnprivilegedHost;
use crate::engine_bindings::standalone::standalone_unprivileged_interface::StandaloneUnprivilegedInterface;
use crate::engine_mode::EngineMode;
use crate::engine_privileged_state::EnginePrivilegedState;
use crossbeam_channel::Receiver;
use squalr_engine_api::commands::engine_response::EngineResponse;
use squalr_engine_api::{commands::engine_command::EngineCommand, events::engine_event::EngineEvent};
use std::sync::{Arc, RwLock};

/// Exposes the ability to send commands to the engine, and handle events from the engine.
pub struct EngineExecutionContext {
    /// The bindings that allow sending commands to the engine.
    engine_bindings: Arc<RwLock<dyn EngineUnprivilegedBindings>>,
}

impl EngineExecutionContext {
    pub fn new(engine_mode: EngineMode) -> Arc<Self> {
        let engine_bindings: Arc<RwLock<dyn EngineUnprivilegedBindings>> = match engine_mode {
            EngineMode::Standalone => Arc::new(RwLock::new(StandaloneUnprivilegedInterface::new())),
            EngineMode::PrivilegedShell => unreachable!("Unprivileged execution context should never be created from a privileged shell."),
            EngineMode::UnprivilegedHost => Arc::new(RwLock::new(InterprocessUnprivilegedHost::new())),
        };

        let execution_context = Arc::new(EngineExecutionContext { engine_bindings });

        execution_context
    }

    pub fn initialize(
        &self,
        engine_privileged_state: &Option<Arc<EnginePrivilegedState>>,
    ) {
        match self.engine_bindings.write() {
            Ok(mut engine_bindings) => {
                if let Err(err) = engine_bindings.initialize(engine_privileged_state) {
                    log::error!("Error initializing unprivileged engine bindings: {}", err);
                }
            }
            Err(err) => {
                log::error!("Failed to acquire unprivileged engine bindings write lock: {}", err);
            }
        }
    }

    /// Dispatches a command to the engine. Direct usage is generally not advised unless you know what you are doing.
    /// Instead, create `{Command}Request` instances and call `.send()` directly on them.
    /// This is only made public to support direct usage by CLIs and other features that need direct access.
    pub fn dispatch_command<F>(
        self: &Arc<Self>,
        command: EngineCommand,
        callback: F,
    ) where
        F: FnOnce(EngineResponse) + Send + Sync + 'static,
    {
        match self.engine_bindings.read() {
            Ok(engine_bindings) => {
                if let Err(err) = engine_bindings.dispatch_command(command, Box::new(callback)) {
                    log::error!("Error dispatching engine command: {}", err);
                }
            }
            Err(err) => {
                log::error!("Failed to acquire unprivileged engine bindings read lock for commands: {}", err);
            }
        }
    }

    /// Creates a receiver, allowing the caller to listen to all engine events.
    /// JIRA: Instead of subscribing to every single event, why not have this class serve as a middle man?
    /// We can then allow subscribing to particular events, then dispatching those to where they need go.
    pub fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEvent>, String> {
        match self.engine_bindings.read() {
            Ok(engine_bindings) => engine_bindings.subscribe_to_engine_events(),
            Err(err) => Err(format!("Failed to acquire unprivileged engine bindings read lock for events: {}", err)),
        }
    }
}
