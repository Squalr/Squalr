use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_response::EngineResponse;
use crate::engine_execution_context::EngineExecutionContext;
use crate::events::engine_event::EngineEvent;
use crossbeam_channel::{Receiver, Sender};
use interprocess_shell::interprocess_egress::InterprocessEgress;
use interprocess_shell::shell::inter_process_privileged_shell::InterProcessPrivilegedShell;
use std::sync::{Arc, RwLock};

/// Orchestrates commands and responses to and from the engine.
pub struct EngineEventHandler {
    /// An optional interprocess shell-side command handler. This is for Android, where commands go through a privileged shell.
    optional_shell: Option<Arc<InterProcessPrivilegedShell<EngineCommand, EngineResponse, EngineEvent, EngineExecutionContext>>>,
    event_senders: Arc<RwLock<Vec<Sender<EngineEvent>>>>,
}

impl EngineEventHandler {
    pub fn new(optional_shell: Option<Arc<InterProcessPrivilegedShell<EngineCommand, EngineResponse, EngineEvent, EngineExecutionContext>>>) -> Self {
        Self {
            optional_shell,
            event_senders: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn initialize(
        &self,
        execution_context: &Arc<EngineExecutionContext>,
    ) {
        if let Some(shell) = self.optional_shell.as_ref() {
            if let Err(err) = shell.initialize(execution_context) {
                log::error!("Error initializing shell: {}", err);
            }
        }
    }

    pub fn subscribe(&self) -> Result<Receiver<EngineEvent>, String> {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let mut sender_lock = self.event_senders.write().map_err(|err| err.to_string())?;
        sender_lock.push(sender);
        Ok(receiver)
    }

    pub fn emit_event(
        &self,
        event: EngineEvent,
    ) {
        if let Some(shell) = &self.optional_shell {
            let _ = shell.dispatch_event(InterprocessEgress::EngineEvent(event.clone()));
        } else {
            if let Ok(senders) = self.event_senders.read() {
                for sender in senders.iter() {
                    let _ = sender.send(event.clone());
                }
            }
        }
    }
}
