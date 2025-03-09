use crate::engine_bindings::engine_egress::InterprocessEgress;
use crate::engine_bindings::engine_unprivileged_bindings::EngineUnprivilegedBindings;
use crate::engine_bindings::interprocess::pipes::interprocess_pipe_bidirectional::InterProcessPipeBidirectional;
use crate::engine_execution_context::EngineExecutionContext;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::engine_command::EngineCommand;
use squalr_engine_api::commands::engine_response::EngineResponse;
use squalr_engine_api::events::engine_event::EngineEvent;
use std::collections::HashMap;
use std::io;
use std::process::Child;
use std::process::Command;
use std::sync::Mutex;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use uuid::Uuid;

pub struct InterProcessUnprivilegedHost {
    /// The spawned shell process with system privileges.
    privileged_shell_process: Arc<RwLock<Option<Child>>>,

    /// The bidirectional connection to the shell process.
    ipc_connection: Arc<RwLock<Option<InterProcessPipeBidirectional>>>,

    /// A map of outgoing requests that are awaiting an engine response.
    request_handles: Arc<Mutex<HashMap<Uuid, Box<dyn FnOnce(EngineResponse) + Send + Sync>>>>,

    // The callback function to handle all interprocess events.
    event_handler: Arc<dyn Fn(EngineEvent) + Send + Sync>,
}

impl EngineUnprivilegedBindings for InterProcessUnprivilegedHost {
    fn initialize(
        &mut self,
        _engine_privileged_state: &Option<Arc<EnginePrivilegedState>>,
        _engine_execution_context: &Option<Arc<EngineExecutionContext>>,
    ) -> Result<(), String> {
        Ok(())
    }

    fn dispatch_command(
        &self,
        command: EngineCommand,
        callback: Box<dyn FnOnce(EngineResponse) + Send + Sync + 'static>,
    ) -> Result<(), String> {
        let request_id = Uuid::new_v4();

        if let Ok(mut request_handles) = self.request_handles.lock() {
            request_handles.insert(request_id, Box::new(callback));
        }

        if let Ok(ipc_connection) = self.ipc_connection.read() {
            if let Some(ipc_connection) = ipc_connection.as_ref() {
                if let Err(err) = ipc_connection.send(command, request_id) {
                    return Err(err.to_string());
                } else {
                    return Ok(());
                }
            }
        }

        Err("Failed to dispatch command.".to_string())
    }
}

impl InterProcessUnprivilegedHost {
    pub fn new<F>(callback: F) -> InterProcessUnprivilegedHost
    where
        F: Fn(EngineEvent) + Send + Sync + 'static,
    {
        let instance = InterProcessUnprivilegedHost {
            privileged_shell_process: Arc::new(RwLock::new(None)),
            ipc_connection: Arc::new(RwLock::new(None)),
            request_handles: Arc::new(Mutex::new(HashMap::new())),
            event_handler: Arc::new(callback),
        };

        instance.initialize();

        instance
    }

    fn initialize(&self) {
        let privileged_shell_process = self.privileged_shell_process.clone();
        let ipc_connection = self.ipc_connection.clone();
        let request_handles = self.request_handles.clone();
        let event_handler = self.event_handler.clone();

        thread::spawn(move || {
            if let Err(err) = Self::spawn_privileged_cli(privileged_shell_process) {
                log::error!("Failed to spawn privileged cli: {}", err);
            }

            if let Err(err) = Self::bind_to_interprocess_pipe(ipc_connection.clone()) {
                log::error!("Failed to bind to inter process pipe: {}", err);
            }

            Self::listen_for_shell_responses(event_handler, request_handles, ipc_connection);
        });
    }

    fn handle_engine_response(
        request_handles: &Arc<Mutex<HashMap<Uuid, Box<dyn FnOnce(EngineResponse) + Send + Sync>>>>,
        engine_response: EngineResponse,
        request_id: Uuid,
    ) {
        if let Ok(mut request_handles) = request_handles.lock() {
            if let Some(callback) = request_handles.remove(&request_id) {
                callback(engine_response);
            }
        }
    }

    fn handle_engine_event(
        event_handler: &Arc<dyn Fn(EngineEvent) + Send + Sync>,
        engine_response: EngineEvent,
    ) {
        event_handler(engine_response);
    }

    fn listen_for_shell_responses(
        event_handler: Arc<dyn Fn(EngineEvent) + Send + Sync>,
        request_handles: Arc<Mutex<HashMap<Uuid, Box<dyn FnOnce(EngineResponse) + Send + Sync>>>>,
        ipc_connection: Arc<RwLock<Option<InterProcessPipeBidirectional>>>,
    ) {
        let request_handles = request_handles.clone();
        let event_handler = event_handler.clone();

        thread::spawn(move || {
            loop {
                if let Ok(ipc_connection) = ipc_connection.read() {
                    if let Some(ipc_connection) = ipc_connection.as_ref() {
                        match ipc_connection.receive::<InterprocessEgress<EngineResponse, EngineEvent>>() {
                            Ok((interprocess_egress, request_id)) => match interprocess_egress {
                                InterprocessEgress::EngineResponse(engine_response) => {
                                    Self::handle_engine_response(&request_handles, engine_response, request_id)
                                }
                                InterprocessEgress::EngineEvent(engine_event) => Self::handle_engine_event(&event_handler, engine_event),
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

    fn spawn_privileged_cli(privileged_shell_process: Arc<RwLock<Option<Child>>>) -> io::Result<()> {
        match Self::spawn_squalr_cli_as_root() {
            Ok(child) => {
                // Update the server handle
                if let Ok(mut server) = privileged_shell_process.write() {
                    *server = Some(child);
                }

                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    fn bind_to_interprocess_pipe(ipc_connection: Arc<RwLock<Option<InterProcessPipeBidirectional>>>) -> Result<(), String> {
        if let Ok(mut ipc_connection) = ipc_connection.write() {
            match InterProcessPipeBidirectional::bind() {
                Ok(bound_connection) => {
                    *ipc_connection = Some(bound_connection);
                    Ok(())
                }
                Err(err) => Err(err),
            }
        } else {
            Err("Failed to acquire write lock on bidirectional interprocess connection.".to_string())
        }
    }

    #[cfg(any(target_os = "android"))]
    fn spawn_squalr_cli_as_root() -> std::io::Result<std::process::Child> {
        Logger::log(LogLevel::Info, "Spawning privileged worker...", None);

        let child = Command::new("su")
            .arg("-c")
            .arg("/data/data/rust.squalr_android/files/squalr-cli")
            .arg("--ipc-mode")
            .spawn()?;

        Ok(child)
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    fn spawn_squalr_cli_as_root() -> io::Result<Child> {
        Command::new("sudo").arg("squalr-cli").arg("--ipc-mode").spawn()
    }

    #[cfg(windows)]
    fn spawn_squalr_cli_as_root() -> io::Result<Child> {
        // No actual privilege escallation for windows -- this feature is not supposed to be used on windows at all.
        // So, just spawn it normally for the rare occasion that we are testing this feature on windows.
        Command::new("squalr-cli").arg("--ipc-mode").spawn()
    }
}
