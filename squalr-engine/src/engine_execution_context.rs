use crate::engine_mode::EngineMode;
use crate::engine_privileged_state::EnginePrivilegedState;
use crate::events::engine_event_handler::EngineEventHandler;
use crossbeam_channel::Receiver;
use interprocess_shell::interprocess_egress::InterprocessEgress;
use interprocess_shell::interprocess_ingress::{ExecutableRequest, InterprocessIngress};
use interprocess_shell::shell::inter_process_unprivileged_host::InterProcessUnprivilegedHost;
use squalr_engine_api::commands::engine_response::EngineResponse;
use squalr_engine_api::{commands::engine_command::EngineCommand, events::engine_event::EngineEvent};
use std::sync::Arc;

/// Exposes the ability to send commands to the engine, and handle events from the engine.
pub struct EngineExecutionContext {
    /// The dispatcher that sends commands to the engine.
    ipc_host: Option<Arc<InterProcessUnprivilegedHost<EngineCommand, EngineResponse, EngineEvent, EnginePrivilegedState>>>,

    engine_privileged_state: Option<Arc<EnginePrivilegedState>>,

    /// The event handler for listening to events emitted from the engine.
    event_handler: EngineEventHandler,
}

impl EngineExecutionContext {
    pub fn new(
        engine_mode: EngineMode,
        engine_privileged_state: Option<Arc<EnginePrivilegedState>>,
    ) -> Arc<Self> {
        let mut optional_host = None;

        if engine_mode == EngineMode::UnprivilegedHost {
            optional_host = Some(Arc::new(InterProcessUnprivilegedHost::new()));
        }

        let execution_context = Arc::new(EngineExecutionContext {
            ipc_host: optional_host,
            engine_privileged_state,
            event_handler: EngineEventHandler::new(None),
        });

        execution_context
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
        if let Some(engine_privileged_state) = &self.engine_privileged_state {
            // For a standalone engine (the common case), we just immediately execute the command with a callback.
            callback(command.execute(engine_privileged_state));
        } else if let Some(host) = &self.ipc_host {
            // For an inter-process engine (ie for Android), we dispatch the command to the priviliged root shell.
            let _ = host.dispatch_command(
                InterprocessIngress::EngineCommand(command),
                move |interprocess_response| match interprocess_response {
                    InterprocessEgress::EngineResponse(engine_response) => {
                        callback(engine_response);
                    }
                    _ => {}
                },
            );
        } else {
            log::error!("Unable to dispatch engine command!")
        }
    }

    /// Emits an event from the engine. Direct usage is not advised except by the engine code itself.
    pub fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEvent>, String> {
        self.event_handler.subscribe()
    }
}
