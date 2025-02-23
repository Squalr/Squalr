use crate::interprocess_egress::InterprocessEgress;
use crate::interprocess_ingress::ExecutableRequest;
use crate::interprocess_ingress::InterprocessIngress;
use crate::pipes::inter_process_pipe_bidirectional::InterProcessPipeBidirectional;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::io;
use std::marker::PhantomData;
use std::process::Child;
use std::process::Command;
use std::sync::Mutex;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use uuid::Uuid;

pub struct InterProcessUnprivilegedHost<
    RequestType: ExecutableRequest<ResponseType> + DeserializeOwned + Serialize,
    ResponseType: DeserializeOwned + Serialize + 'static,
    EventType: DeserializeOwned + Serialize + 'static,
> {
    privileged_shell_process: Arc<RwLock<Option<Child>>>,
    ipc_connection: Arc<RwLock<Option<InterProcessPipeBidirectional>>>,
    /// A map of outgoing requests that are awaiting an engine response.
    request_handles: Arc<Mutex<HashMap<Uuid, Box<dyn FnOnce(InterprocessEgress<ResponseType, EventType>) + Send + Sync>>>>,
    _phantom: PhantomData<RequestType>,
}

impl<
        RequestType: ExecutableRequest<ResponseType> + DeserializeOwned + Serialize,
        ResponseType: DeserializeOwned + Serialize,
        EventType: DeserializeOwned + Serialize,
    > InterProcessUnprivilegedHost<RequestType, ResponseType, EventType>
{
    pub fn new() -> InterProcessUnprivilegedHost<RequestType, ResponseType, EventType> {
        let instance = InterProcessUnprivilegedHost {
            privileged_shell_process: Arc::new(RwLock::new(None)),
            ipc_connection: Arc::new(RwLock::new(None)),
            request_handles: Arc::new(Mutex::new(HashMap::new())),
            _phantom: PhantomData,
        };

        instance.initialize();

        instance
    }

    fn initialize(&self) {
        let privileged_shell_process = self.privileged_shell_process.clone();
        let ipc_connection = self.ipc_connection.clone();
        let request_handles = self.request_handles.clone();

        thread::spawn(move || {
            let _ = Self::spawn_privileged_cli(privileged_shell_process);
            let _ = Self::bind_to_inter_process_pipe(ipc_connection.clone());
            Self::listen_for_shell_responses(request_handles, ipc_connection);
        });
    }

    pub fn dispatch_command<F>(
        &self,
        command: InterprocessIngress<RequestType, ResponseType>,
        callback: F,
    ) -> io::Result<()>
    where
        F: FnOnce(InterprocessEgress<ResponseType, EventType>) + Send + Sync + 'static,
    {
        let request_id = Uuid::new_v4();

        if let Ok(mut request_handles) = self.request_handles.lock() {
            request_handles.insert(request_id, Box::new(callback));
        }

        if let Ok(ipc_connection) = self.ipc_connection.read() {
            if let Some(ipc_connection) = ipc_connection.as_ref() {
                if let Err(err) = ipc_connection.send(command, request_id) {
                    return Err(err);
                } else {
                    return Ok(());
                }
            }
        }

        Err(io::Error::new(io::ErrorKind::Other, "Failed to dispatch command."))
    }

    fn handle_command_response(
        request_handles: &Arc<Mutex<HashMap<Uuid, Box<dyn FnOnce(InterprocessEgress<ResponseType, EventType>) + Send + Sync>>>>,
        engine_response: InterprocessEgress<ResponseType, EventType>,
        request_id: Uuid,
    ) {
        if let Ok(mut request_handles) = request_handles.lock() {
            if let Some(callback) = request_handles.remove(&request_id) {
                callback(engine_response);
            }
        }
    }

    fn listen_for_shell_responses(
        request_handles: Arc<Mutex<HashMap<Uuid, Box<dyn FnOnce(InterprocessEgress<ResponseType, EventType>) + Send + Sync>>>>,
        ipc_connection: Arc<RwLock<Option<InterProcessPipeBidirectional>>>,
    ) {
        let request_handles = request_handles.clone();

        thread::spawn(move || loop {
            if let Ok(ipc_connection) = ipc_connection.read() {
                if let Some(ipc_connection) = ipc_connection.as_ref() {
                    match ipc_connection.receive::<InterprocessEgress<ResponseType, EventType>>() {
                        Ok((engine_response, request_id)) => {
                            Self::handle_command_response(&request_handles, engine_response, request_id);
                        }
                        Err(_err) => {
                            std::process::exit(1);
                        }
                    }
                }
            }

            thread::sleep(Duration::from_millis(1));
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

    fn bind_to_inter_process_pipe(ipc_connection: Arc<RwLock<Option<InterProcessPipeBidirectional>>>) -> io::Result<()> {
        if let Ok(mut ipc_connection) = ipc_connection.write() {
            match InterProcessPipeBidirectional::bind() {
                Ok(bound_connection) => {
                    *ipc_connection = Some(bound_connection);
                    Ok(())
                }
                Err(err) => Err(err),
            }
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to acquire write lock on bidirectional interprocess connection.",
            ))
        }
    }

    #[cfg(any(target_os = "android"))]
    fn spawn_squalr_cli_as_root() -> std::io::Result<std::process::Child> {
        Logger::get_instance().log(LogLevel::Info, "Spawning privileged worker...", None);

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
