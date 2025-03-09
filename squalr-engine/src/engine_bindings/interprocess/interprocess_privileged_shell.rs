use crate::engine_bindings::engine_egress::InterprocessEgress;
use crate::engine_bindings::engine_ingress::ExecutableRequest;
use crate::engine_bindings::engine_ingress::InterprocessIngress;
use crate::engine_bindings::engine_priviliged_bindings::EnginePrivilegedBindings;
use crate::engine_bindings::interprocess::pipes::interprocess_pipe_bidirectional::InterProcessPipeBidirectional;
use crate::engine_execution_context::EngineExecutionContext;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::engine_response::EngineResponse;
use squalr_engine_api::events::engine_event::EngineEvent;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use uuid::Uuid;

pub struct InterProcessPrivilegedShell {
    ipc_connection: Arc<RwLock<Option<InterProcessPipeBidirectional>>>,
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
}

impl InterProcessPrivilegedShell {
    pub fn new() -> InterProcessPrivilegedShell {
        let instance = InterProcessPrivilegedShell {
            ipc_connection: Arc::new(RwLock::new(None)),
        };

        instance
    }

    pub fn dispatch_event(
        &self,
        interprocess_event: InterprocessEgress<EngineResponse, EngineEvent>,
    ) -> Result<(), String> {
        Self::dispatch_response(self.ipc_connection.clone(), interprocess_event, Uuid::nil())
    }

    pub fn dispatch_response(
        ipc_connection: Arc<RwLock<Option<InterProcessPipeBidirectional>>>,
        interprocess_response: InterprocessEgress<EngineResponse, EngineEvent>,
        request_id: Uuid,
    ) -> Result<(), String> {
        let ipc_connection = ipc_connection.clone();
        if let Ok(ipc_connection_guard) = ipc_connection.read() {
            if let Some(ipc_connection_pipe) = ipc_connection_guard.as_ref() {
                return ipc_connection_pipe.send(interprocess_response, request_id);
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
                        match ipc_connection_pipe.receive::<InterprocessIngress>() {
                            Ok((interprocess_command, request_id)) => match interprocess_command {
                                InterprocessIngress::EngineCommand(engine_command) => {
                                    let interprocess_response = InterprocessEgress::EngineResponse(engine_command.execute(&engine_privileged_state));
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
