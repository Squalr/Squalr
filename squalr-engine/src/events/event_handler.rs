use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_response::EngineResponse;
use crate::engine_execution_context::EngineExecutionContext;
use crate::events::engine_event::EngineEvent;
use crossbeam_channel::{Receiver, Sender};
use interprocess_shell::interprocess_egress::InterprocessEgress;
use interprocess_shell::shell::inter_process_privileged_shell::InterProcessPrivilegedShell;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use std::sync::Arc;
use std::thread;

/// Orchestrates commands and responses to and from the engine.
pub struct EngineEventHandler {
    /// An optional interprocess shell-side command handler. This is for Android, where commands go through a privileged shell.
    optional_shell: Option<Arc<InterProcessPrivilegedShell<EngineCommand, EngineResponse, EngineEvent, EngineExecutionContext>>>,
    event_sender: Sender<EngineEvent>,
    event_receiver: Receiver<EngineEvent>,
}

impl EngineEventHandler {
    pub fn new(optional_shell: Option<Arc<InterProcessPrivilegedShell<EngineCommand, EngineResponse, EngineEvent, EngineExecutionContext>>>) -> Self {
        let (event_sender, event_receiver) = crossbeam_channel::unbounded();

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

    pub fn initialize(
        &self,
        execution_context: &Arc<EngineExecutionContext>,
    ) {
        if let Some(shell) = self.optional_shell.as_ref() {
            if let Err(err) = shell.initialize(execution_context) {
                Logger::log(LogLevel::Error, &format!("Error initializing shell: {}", err), None);
            }
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
