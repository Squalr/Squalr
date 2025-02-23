use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_response::EngineResponse;
use crate::engine_mode::EngineMode;
use crate::squalr_engine::SqualrEngine;
use interprocess_shell::interprocess_egress::InterprocessEgress;
use interprocess_shell::interprocess_ingress::ExecutableRequest;
use interprocess_shell::interprocess_ingress::InterprocessIngress;
use interprocess_shell::shell::inter_process_privileged_shell::InterProcessPrivilegedShell;
use interprocess_shell::shell::inter_process_unprivileged_host::InterProcessUnprivilegedHost;

/// Orchestrates commands and responses to and from the engine.
pub struct CommandDispatcher {
    host: Option<InterProcessUnprivilegedHost<EngineCommand, EngineResponse>>,
    _shell: Option<InterProcessPrivilegedShell<EngineCommand, EngineResponse>>,
}

impl CommandDispatcher {
    pub fn new(engine_mode: EngineMode) -> Self {
        let mut host = None;
        let mut shell = None;

        if engine_mode == EngineMode::UnprivilegedHost {
            host = Some(InterProcessUnprivilegedHost::new());
        } else if engine_mode == EngineMode::PrivilegedShell {
            shell = Some(InterProcessPrivilegedShell::new());
        }

        let command_dispatcher = CommandDispatcher { host, _shell: shell };

        command_dispatcher
    }

    pub fn dispatch_command<F>(
        &self,
        command: EngineCommand,
        callback: F,
    ) where
        F: FnOnce(EngineResponse) + Send + Sync + 'static,
    {
        let engine_mode = SqualrEngine::get_engine_mode();

        if engine_mode == EngineMode::Standalone {
            // For a standalone engine (the common case), we just immediately execute the command with a callback.
            callback(command.execute());
        } else {
            if let Some(host) = &self.host {
                let _ = host.dispatch_command(
                    InterprocessIngress::EngineCommand(command),
                    move |interprocess_response| match interprocess_response {
                        InterprocessEgress::EngineResponse(engine_response) => {
                            callback(engine_response);
                        }
                    },
                );
            }
        }
    }
}
