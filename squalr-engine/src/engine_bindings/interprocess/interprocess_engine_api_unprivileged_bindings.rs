use crate::engine_bindings::engine_egress::EngineEgress;
use crate::engine_bindings::executable_command_unprivileged::ExecutableCommandUnprivleged;
use crate::engine_bindings::interprocess::pipes::interprocess_pipe_bidirectional::InterprocessPipeBidirectional;
use crate::engine_initialization_error::EngineInitializationError;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use squalr_engine_api::commands::privileged_command::PrivilegedCommand;
use squalr_engine_api::commands::privileged_command_response::PrivilegedCommandResponse;
use squalr_engine_api::commands::unprivileged_command::UnprivilegedCommand;
use squalr_engine_api::commands::unprivileged_command_response::UnprivilegedCommandResponse;
use squalr_engine_api::engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings;
use squalr_engine_api::engine::engine_binding_error::EngineBindingError;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
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

pub struct InterprocessEngineApiUnprivilegedBindings {
    /// The spawned shell process with system privileges.
    privileged_shell_process: Arc<RwLock<Option<Child>>>,

    /// The bidirectional connection to the shell process.
    ipc_connection: Arc<RwLock<Option<InterprocessPipeBidirectional>>>,

    /// A map of outgoing requests that are awaiting an engine response.
    request_handles: Arc<Mutex<HashMap<Uuid, Box<dyn FnOnce(PrivilegedCommandResponse) + Send + Sync>>>>,

    /// The list of subscribers to which we send engine events, after having received them from the engine.
    event_senders: Arc<RwLock<Vec<Sender<EngineEvent>>>>,
}

impl EngineApiUnprivilegedBindings for InterprocessEngineApiUnprivilegedBindings {
    /// Dispatches an unprivileged command to be immediately handled on the client side.
    fn dispatch_privileged_command(
        &self,
        privileged_command: PrivilegedCommand,
        callback: Box<dyn FnOnce(PrivilegedCommandResponse) + Send + Sync + 'static>,
    ) -> Result<(), EngineBindingError> {
        let request_id = Uuid::new_v4();

        if let Ok(mut request_handles) = self.request_handles.lock() {
            request_handles.insert(request_id, Box::new(callback));
        }

        let ipc_connection_guard = self
            .ipc_connection
            .read()
            .map_err(|error| EngineBindingError::lock_failure("dispatching privileged command to IPC", error.to_string()))?;

        if let Some(ipc_connection) = ipc_connection_guard.as_ref() {
            ipc_connection
                .send(privileged_command, request_id)
                .map_err(|error| EngineBindingError::operation_failed("sending privileged command over IPC", error))?;

            return Ok(());
        }

        Err(EngineBindingError::unavailable("dispatching privileged command over IPC"))
    }

    /// Dispatches an unprivileged command to be immediately handled on the client side.
    fn dispatch_unprivileged_command(
        &self,
        unprivileged_command: UnprivilegedCommand,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
        callback: Box<dyn FnOnce(UnprivilegedCommandResponse) + Send + Sync + 'static>,
    ) -> Result<(), EngineBindingError> {
        let response = unprivileged_command.execute(engine_unprivileged_state);

        callback(response);

        Ok(())
    }

    /// Requests to listen to all engine events.
    fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEvent>, EngineBindingError> {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let mut sender_lock = self
            .event_senders
            .write()
            .map_err(|error| EngineBindingError::lock_failure("subscribing to IPC engine events", error.to_string()))?;
        sender_lock.push(sender);

        Ok(receiver)
    }
}

impl InterprocessEngineApiUnprivilegedBindings {
    pub fn new() -> Result<InterprocessEngineApiUnprivilegedBindings, EngineInitializationError> {
        let instance = InterprocessEngineApiUnprivilegedBindings {
            privileged_shell_process: Arc::new(RwLock::new(None)),
            ipc_connection: Arc::new(RwLock::new(None)),
            request_handles: Arc::new(Mutex::new(HashMap::new())),
            event_senders: Arc::new(RwLock::new(vec![])),
        };

        instance.initialize()?;

        Ok(instance)
    }

    fn initialize(&self) -> Result<(), EngineInitializationError> {
        self.initialize_with_hooks(Self::spawn_privileged_cli, Self::bind_to_interprocess_pipe)
    }

    fn initialize_with_hooks(
        &self,
        spawn_privileged_cli: fn(Arc<RwLock<Option<Child>>>) -> io::Result<()>,
        bind_to_interprocess_pipe: fn(Arc<RwLock<Option<InterprocessPipeBidirectional>>>) -> Result<(), EngineBindingError>,
    ) -> Result<(), EngineInitializationError> {
        let privileged_shell_process = self.privileged_shell_process.clone();
        let ipc_connection = self.ipc_connection.clone();
        let request_handles = self.request_handles.clone();
        let event_senders = self.event_senders.clone();

        spawn_privileged_cli(privileged_shell_process).map_err(EngineInitializationError::spawn_privileged_cli_failed)?;
        bind_to_interprocess_pipe(ipc_connection.clone()).map_err(EngineInitializationError::bind_unprivileged_ipc_failed)?;

        Self::listen_for_shell_responses(request_handles, event_senders, ipc_connection);

        Ok(())
    }

    fn handle_engine_response(
        request_handles: &Arc<Mutex<HashMap<Uuid, Box<dyn FnOnce(PrivilegedCommandResponse) + Send + Sync>>>>,
        engine_response: PrivilegedCommandResponse,
        request_id: Uuid,
    ) {
        if let Ok(mut request_handles) = request_handles.lock() {
            if let Some(callback) = request_handles.remove(&request_id) {
                callback(engine_response);
            }
        }
    }

    fn handle_engine_event(
        event_senders: &Arc<RwLock<Vec<Sender<EngineEvent>>>>,
        engine_event: EngineEvent,
    ) {
        if let Ok(senders) = event_senders.read() {
            for sender in senders.iter() {
                if let Err(error) = sender.send(engine_event.clone()) {
                    log::error!("Error broadcasting received engine event: {}", error);
                }
            }
        }
    }

    fn listen_for_shell_responses(
        request_handles: Arc<Mutex<HashMap<Uuid, Box<dyn FnOnce(PrivilegedCommandResponse) + Send + Sync>>>>,
        event_senders: Arc<RwLock<Vec<Sender<EngineEvent>>>>,
        ipc_connection: Arc<RwLock<Option<InterprocessPipeBidirectional>>>,
    ) {
        let request_handles = request_handles.clone();
        let event_senders = event_senders.clone();

        thread::spawn(move || {
            loop {
                if let Ok(ipc_connection) = ipc_connection.read() {
                    if let Some(ipc_connection) = ipc_connection.as_ref() {
                        match ipc_connection.receive::<EngineEgress>() {
                            Ok((interprocess_egress, request_id)) => match interprocess_egress {
                                EngineEgress::PrivilegedCommandResponse(engine_response) => {
                                    Self::handle_engine_response(&request_handles, engine_response, request_id)
                                }
                                EngineEgress::EngineEvent(engine_event) => Self::handle_engine_event(&event_senders, engine_event),
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

    fn spawn_privileged_cli(privileged_shell_process: Arc<RwLock<Option<Child>>>) -> io::Result<()> {
        match Self::spawn_squalr_cli_as_root() {
            Ok(child) => {
                // Update the server handle
                if let Ok(mut server) = privileged_shell_process.write() {
                    *server = Some(child);
                }

                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    fn bind_to_interprocess_pipe(ipc_connection: Arc<RwLock<Option<InterprocessPipeBidirectional>>>) -> Result<(), EngineBindingError> {
        let mut ipc_connection_guard = ipc_connection
            .write()
            .map_err(|error| EngineBindingError::lock_failure("binding unprivileged IPC connection", error.to_string()))?;
        let bound_connection =
            InterprocessPipeBidirectional::bind().map_err(|error| EngineBindingError::operation_failed("binding bidirectional IPC connection", error))?;
        *ipc_connection_guard = Some(bound_connection);

        Ok(())
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

#[cfg(test)]
mod tests {
    use super::InterprocessEngineApiUnprivilegedBindings;
    use crate::engine_bindings::interprocess::pipes::interprocess_pipe_bidirectional::InterprocessPipeBidirectional;
    use squalr_engine_api::engine::engine_binding_error::EngineBindingError;
    use std::io;
    use std::sync::{Arc, RwLock};

    #[test]
    fn initialize_fails_fast_when_privileged_cli_spawn_fails() {
        fn failing_spawn(_privileged_shell_process: Arc<RwLock<Option<std::process::Child>>>) -> io::Result<()> {
            Err(io::Error::other("spawn failed"))
        }

        fn successful_bind(_ipc_connection: Arc<RwLock<Option<InterprocessPipeBidirectional>>>) -> Result<(), EngineBindingError> {
            Ok(())
        }

        let interprocess_bindings = InterprocessEngineApiUnprivilegedBindings {
            privileged_shell_process: Arc::new(RwLock::new(None)),
            ipc_connection: Arc::new(RwLock::new(None)),
            request_handles: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            event_senders: Arc::new(RwLock::new(vec![])),
        };

        let initialize_result = interprocess_bindings.initialize_with_hooks(failing_spawn, successful_bind);

        assert!(initialize_result.is_err());

        if let Err(initialization_error) = initialize_result {
            assert!(
                initialization_error
                    .to_string()
                    .contains("Failed to spawn privileged CLI process for unprivileged host startup")
            );
        }
    }

    #[test]
    fn initialize_fails_fast_when_ipc_bind_fails() {
        fn successful_spawn(_privileged_shell_process: Arc<RwLock<Option<std::process::Child>>>) -> io::Result<()> {
            Ok(())
        }

        fn failing_bind(_ipc_connection: Arc<RwLock<Option<InterprocessPipeBidirectional>>>) -> Result<(), EngineBindingError> {
            Err(EngineBindingError::unavailable("binding bidirectional IPC connection"))
        }

        let interprocess_bindings = InterprocessEngineApiUnprivilegedBindings {
            privileged_shell_process: Arc::new(RwLock::new(None)),
            ipc_connection: Arc::new(RwLock::new(None)),
            request_handles: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            event_senders: Arc::new(RwLock::new(vec![])),
        };

        let initialize_result = interprocess_bindings.initialize_with_hooks(successful_spawn, failing_bind);

        assert!(initialize_result.is_err());

        if let Err(initialization_error) = initialize_result {
            assert!(
                initialization_error
                    .to_string()
                    .contains("Failed to bind unprivileged host IPC channel during startup")
            );
        }
    }
}
