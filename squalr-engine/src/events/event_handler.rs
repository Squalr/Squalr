use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_response::EngineResponse;
use crate::events::engine_event::EngineEvent;
use crossbeam_channel::{Receiver, Sender, unbounded};
use interprocess_shell::interprocess_egress::InterprocessEgress;
use interprocess_shell::shell::inter_process_privileged_shell::InterProcessPrivilegedShell;
use std::sync::Arc;
use std::thread;

/// Orchestrates commands and responses to and from the engine.
pub struct EngineEventHandler {
    /// An optional interprocess shell-side command handler. This is for Android, where commands go through a privileged shell.
    optional_shell: Option<Arc<InterProcessPrivilegedShell<EngineCommand, EngineResponse, EngineEvent>>>,
    event_sender: Sender<EngineEvent>,
    event_receiver: Receiver<EngineEvent>,
}

impl EngineEventHandler {
    pub fn new(optional_shell: Option<Arc<InterProcessPrivilegedShell<EngineCommand, EngineResponse, EngineEvent>>>) -> Self {
        let (event_sender, event_receiver) = unbounded();

        {
            let event_sender = event_sender.clone();
            let event_receiver = event_receiver.clone();

            thread::spawn(move || {
                let event_sender = event_sender.clone();
                let event_receiver = event_receiver.clone();
                while let Ok(event) = event_receiver.recv() {
                    let _ = event_sender.send(event);
                }
            });
        }

        Self {
            optional_shell,
            event_sender,
            event_receiver,
        }
    }

    pub fn subscribe(&self) -> Receiver<EngineEvent> {
        self.event_receiver.clone()
    }

    pub fn emit_event(
        &self,
        event: EngineEvent,
    ) {
        if let Some(shell) = &self.optional_shell {
            let _ = shell.dispatch_event(InterprocessEgress::EngineEvent(event));
        } else {
            let _ = self.event_sender.send(event);
        }
    }
}
