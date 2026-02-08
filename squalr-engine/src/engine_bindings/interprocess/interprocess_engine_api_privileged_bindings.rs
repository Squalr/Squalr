use crate::engine_bindings::engine_egress::EngineEgress;
use crate::engine_bindings::engine_ingress::EngineIngress;
use crate::engine_bindings::executable_command_privileged::ExecutableCommandPrivileged;
use crate::engine_bindings::interprocess::pipes::interprocess_pipe_bidirectional::InterprocessPipeBidirectional;
use crate::engine_privileged_state::EnginePrivilegedState;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use squalr_engine_api::commands::privileged_command::PrivilegedCommand;
use squalr_engine_api::commands::privileged_command_response::PrivilegedCommandResponse;
use squalr_engine_api::engine::engine_api_priviliged_bindings::EngineApiPrivilegedBindings;
use squalr_engine_api::engine::engine_binding_error::EngineBindingError;
use squalr_engine_api::events::engine_event::EngineEvent;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use uuid::Uuid;

pub struct InterprocessEngineApiPrivilegedBindings {
    engine_privileged_state: Option<Arc<EnginePrivilegedState>>,

    /// The bidirectional connection to the host process.
    ipc_connection: Arc<RwLock<Option<InterprocessPipeBidirectional>>>,

    /// A map of outgoing requests that are awaiting an engine response.
    request_handles: Arc<Mutex<HashMap<Uuid, Box<dyn FnOnce(PrivilegedCommandResponse) + Send + Sync>>>>,

    /// The list of subscribers to which we send engine events.
    event_senders: Arc<RwLock<Vec<Sender<EngineEvent>>>>,
}

impl EngineApiPrivilegedBindings for InterprocessEngineApiPrivilegedBindings {
    fn emit_event(
        &self,
        engine_event: EngineEvent,
    ) -> Result<(), EngineBindingError> {
        // First dispatch the invent internally to any listeners.
        if let Ok(senders) = self.event_senders.read() {
            for sender in senders.iter() {
                if let Err(error) = sender.send(engine_event.clone()) {
                    log::error!("Error internally dispatching engine event: {}", error);
                }
            }
        }

        // Next dispatch the event over the interprocess pipe for the unprivileged side to handle.
        Self::dispatch_response(self.ipc_connection.clone(), EngineEgress::EngineEvent(engine_event), Uuid::nil())
    }

    fn dispatch_internal_command(
        &self,
        engine_command: PrivilegedCommand,
        callback: Box<dyn FnOnce(PrivilegedCommandResponse) + Send + Sync + 'static>,
    ) -> Result<(), EngineBindingError> {
        let request_id = Uuid::new_v4();

        if let Ok(mut request_handles) = self.request_handles.lock() {
            request_handles.insert(request_id, Box::new(callback));
        }

        if let Some(engine_privileged_state) = &self.engine_privileged_state {
            let interprocess_response = EngineEgress::PrivilegedCommandResponse(engine_command.execute(&engine_privileged_state));

            Self::dispatch_response(self.ipc_connection.clone(), interprocess_response, request_id)
        } else {
            Err(EngineBindingError::unavailable("dispatching privileged command in IPC mode"))
        }
    }

    fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEvent>, EngineBindingError> {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let mut sender_lock = self
            .event_senders
            .write()
            .map_err(|error| EngineBindingError::lock_failure("subscribing to privileged IPC engine events", error.to_string()))?;
        sender_lock.push(sender);

        Ok(receiver)
    }
}

impl InterprocessEngineApiPrivilegedBindings {
    pub fn new() -> InterprocessEngineApiPrivilegedBindings {
        let instance = InterprocessEngineApiPrivilegedBindings {
            engine_privileged_state: None,
            ipc_connection: Arc::new(RwLock::new(None)),
            request_handles: Arc::new(Mutex::new(HashMap::new())),
            event_senders: Arc::new(RwLock::new(vec![])),
        };

        instance
    }

    pub fn initialize(
        &mut self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> Result<(), EngineBindingError> {
        let mut ipc_connection_guard = self
            .ipc_connection
            .write()
            .map_err(|error| EngineBindingError::lock_failure("initializing privileged IPC connection", error.to_string()))?;
        let new_connection =
            InterprocessPipeBidirectional::create().map_err(|error| EngineBindingError::operation_failed("creating bidirectional IPC connection", error))?;
        *ipc_connection_guard = Some(new_connection);
        self.listen_for_host_requests(&engine_privileged_state);

        Ok(())
    }

    pub fn dispatch_response(
        ipc_connection: Arc<RwLock<Option<InterprocessPipeBidirectional>>>,
        engine_egress: EngineEgress,
        request_id: Uuid,
    ) -> Result<(), EngineBindingError> {
        let ipc_connection = ipc_connection.clone();
        let ipc_connection_guard = ipc_connection
            .read()
            .map_err(|error| EngineBindingError::lock_failure("dispatching IPC response", error.to_string()))?;

        if let Some(ipc_connection_pipe) = ipc_connection_guard.as_ref() {
            ipc_connection_pipe
                .send(engine_egress, request_id)
                .map_err(|error| EngineBindingError::operation_failed("sending IPC response", error))?;
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
                                EngineIngress::PrivilegedCommand(engine_command) => {
                                    let interprocess_response = EngineEgress::PrivilegedCommandResponse(engine_command.execute(&engine_privileged_state));
                                    let _ = Self::dispatch_response(ipc_connection.clone(), interprocess_response, request_id);
                                }
                            },
                            Err(_error) => {
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
