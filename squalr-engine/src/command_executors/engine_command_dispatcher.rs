use crate::engine_execution_context::EngineExecutionContext;
use interprocess_shell::interprocess_egress::InterprocessEgress;
use interprocess_shell::interprocess_ingress::ExecutableRequest;
use interprocess_shell::interprocess_ingress::InterprocessIngress;
use interprocess_shell::shell::inter_process_unprivileged_host::InterProcessUnprivilegedHost;
use squalr_engine_api::commands::engine_command::EngineCommand;
use squalr_engine_api::commands::engine_response::EngineResponse;
use squalr_engine_api::events::engine_event::EngineEvent;
use std::sync::Arc;

/// Orchestrates commands and responses to and from the engine.
pub struct EngineCommandDispatcher {
    /// An optional interprocess command dispatcher to a privileged shell. This is for Android where the GUI is unprivileged.
    optional_host: Option<Arc<InterProcessUnprivilegedHost<EngineCommand, EngineResponse, EngineEvent, EngineExecutionContext>>>,
}

impl EngineCommandDispatcher {
    pub fn new(optional_host: Option<Arc<InterProcessUnprivilegedHost<EngineCommand, EngineResponse, EngineEvent, EngineExecutionContext>>>) -> Self {
        Self { optional_host }
    }

    /// Sends a command to the engine, with the raw response returned in a callback.
    /// The contract of the engine is that it ALWAYS returns a response.
    pub fn dispatch_command<F>(
        &self,
        command: EngineCommand,
        execution_context: &Arc<EngineExecutionContext>,
        callback: F,
    ) where
        F: FnOnce(EngineResponse) + Send + Sync + 'static,
    {
        // For an inter-process engine (ie for Android), we dispatch the command to the priviliged root shell.
        if let Some(host) = &self.optional_host {
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
            // For a standalone engine (the common case), we just immediately execute the command with a callback.
            callback(command.execute(execution_context));
        }
    }
}
