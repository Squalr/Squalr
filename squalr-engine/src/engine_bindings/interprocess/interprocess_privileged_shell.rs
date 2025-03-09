use crate::engine_bindings::engine_egress::EngineEgress;
use crate::engine_bindings::engine_ingress::EngineIngress;
use crate::engine_bindings::engine_ingress::ExecutableRequest;
use crate::engine_bindings::engine_priviliged_bindings::EnginePrivilegedBindings;
use crate::engine_bindings::interprocess::pipes::interprocess_pipe_bidirectional::InterProcessPipeBidirectional;
use crate::engine_execution_context::EngineExecutionContext;
use crate::engine_privileged_state::EnginePrivilegedState;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use squalr_engine_api::events::engine_event::EngineEvent;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use uuid::Uuid;

pub struct InterProcessPrivilegedShell {
    /// The bidirectional connection to the host process.
    ipc_connection: Arc<RwLock<Option<InterProcessPipeBidirectional>>>,

    /// The list of subscribers to which we send engine events.
    event_senders: Arc<RwLock<Vec<Sender<EngineEvent>>>>,
}

impl EnginePrivilegedBindings for InterProcessPrivilegedShell {
    fn initialize(
        &mut self,
        engine_privileged_state: &Option<Arc<EnginePrivilegedState>>,
        _engine_execution_context: &Option<Arc<EngineExecutionContext>>,
    ) -> Result<(), String> {
        if let Some(engine_privileged_state) = engine_privileged_state {
            if let Ok(mut ipc_connection) = self.ipc_connection.write() {
                match InterProcessPipeBidirectional::create() {
                    Ok(new_connection) => {
                        *ipc_connection = Some(new_connection);
                        self.listen_for_host_requests(&engine_privileged_state);
                        Ok(())
                    }
                    Err(err) => Err(err),
                }
            } else {
                Err("Failed to acquire write lock on bidirectional interprocess connection.".to_string())
            }
        } else {
            Err("No privileged state provided! Engine command dispatching will be non-functional without this.".to_string())
        }
    }

    fn emit_event(
        &self,
        engine_event: EngineEvent,
    ) -> Result<(), String> {
        // First dispatch the invent internally to any listeners.
        if let Ok(senders) = self.event_senders.read() {
            for sender in senders.iter() {
                if let Err(err) = sender.send(engine_event.clone()) {
                    log::error!("Error internally dispatching engine event: {}", err);
                }
            }
        }

        // Next dispatch the event over the interprocess pipe for the unprivileged side to handle.
        Self::dispatch_response(self.ipc_connection.clone(), EngineEgress::EngineEvent(engine_event), Uuid::nil())
    }

    fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEvent>, String> {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let mut sender_lock = self.event_senders.write().map_err(|err| err.to_string())?;
        sender_lock.push(sender);

        Ok(receiver)
    }
}

impl InterProcessPrivilegedShell {
    pub fn new() -> InterProcessPrivilegedShell {
        let instance = InterProcessPrivilegedShell {
            ipc_connection: Arc::new(RwLock::new(None)),
            event_senders: Arc::new(RwLock::new(vec![])),
        };

        instance
    }

    pub fn dispatch_response(
        ipc_connection: Arc<RwLock<Option<InterProcessPipeBidirectional>>>,
        engine_egress: EngineEgress,
        request_id: Uuid,
    ) -> Result<(), String> {
        let ipc_connection = ipc_connection.clone();
        if let Ok(ipc_connection_guard) = ipc_connection.read() {
            if let Some(ipc_connection_pipe) = ipc_connection_guard.as_ref() {
                return ipc_connection_pipe.send(engine_egress, request_id);
            }
        }

        Ok(())
    }

    fn listen_for_host_requests(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) {
        let ipc_connection = self.ipc_connection.clone();
        let engine_privileged_state = engine_privileged_state.clone();

        thread::spawn(move || {
            loop {
                let ipc_connection = ipc_connection.clone();
                let engine_privileged_state = engine_privileged_state.clone();

                if let Ok(ipc_connection_guard) = ipc_connection.read() {
                    if let Some(ipc_connection_pipe) = ipc_connection_guard.as_ref() {
                        match ipc_connection_pipe.receive::<EngineIngress>() {
                            Ok((interprocess_command, request_id)) => match interprocess_command {
                                EngineIngress::EngineCommand(engine_command) => {
                                    let interprocess_response = EngineEgress::EngineResponse(engine_command.execute(&engine_privileged_state));
                                    let _ = Self::dispatch_response(ipc_connection.clone(), interprocess_response, request_id);
                                }
                            },
                            Err(_err) => {
                                std::process::exit(1);
                            }
                        }
                    }
                }

                thread::sleep(Duration::from_millis(1));
            }
        });
    }
}
