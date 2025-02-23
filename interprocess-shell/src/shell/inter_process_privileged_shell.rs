use crate::interprocess_egress::InterprocessEgress;
use crate::interprocess_ingress::ExecutableRequest;
use crate::interprocess_ingress::InterprocessIngress;
use crate::pipes::inter_process_pipe_bidirectional::InterProcessPipeBidirectional;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use uuid::Uuid;

pub struct InterProcessPrivilegedShell<RequestType: ExecutableRequest<ResponseType> + DeserializeOwned + Serialize, ResponseType: DeserializeOwned + Serialize>
{
    ipc_connection: Arc<RwLock<Option<InterProcessPipeBidirectional>>>,
    _phantom: PhantomData<RequestType>,
    _phantom2: PhantomData<ResponseType>,
}

impl<RequestType: ExecutableRequest<ResponseType> + DeserializeOwned + Serialize, ResponseType: DeserializeOwned + Serialize>
    InterProcessPrivilegedShell<RequestType, ResponseType>
{
    pub fn new() -> InterProcessPrivilegedShell<RequestType, ResponseType> {
        let instance = InterProcessPrivilegedShell {
            ipc_connection: Arc::new(RwLock::new(None)),
            _phantom: PhantomData,
            _phantom2: PhantomData,
        };

        instance
    }

    pub fn initialize(&self) -> io::Result<()> {
        if let Ok(mut ipc_connection) = self.ipc_connection.write() {
            match InterProcessPipeBidirectional::create() {
                Ok(new_connection) => {
                    *ipc_connection = Some(new_connection);
                    self.listen_for_host_requests();
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

    pub fn dispatch_response(
        ipc_connection: Arc<RwLock<Option<InterProcessPipeBidirectional>>>,
        interprocess_response: InterprocessEgress<ResponseType>,
        request_id: Uuid,
    ) -> io::Result<()> {
        let ipc_connection = ipc_connection.clone();
        if let Ok(ipc_connection_guard) = ipc_connection.read() {
            if let Some(ipc_connection_pipe) = ipc_connection_guard.as_ref() {
                return ipc_connection_pipe.send(interprocess_response, request_id);
            }
        }

        Ok(())
    }

    fn listen_for_host_requests(&self) {
        let ipc_connection = self.ipc_connection.clone();

        thread::spawn(move || loop {
            let ipc_connection = ipc_connection.clone();

            if let Ok(ipc_connection_guard) = ipc_connection.read() {
                if let Some(ipc_connection_pipe) = ipc_connection_guard.as_ref() {
                    match ipc_connection_pipe.receive::<InterprocessIngress<RequestType, ResponseType>>() {
                        Ok((interprocess_command, request_id)) => match interprocess_command {
                            InterprocessIngress::EngineCommand(engine_command) => {
                                let interprocess_response = InterprocessEgress::EngineResponse(engine_command.execute());
                                let _ = Self::dispatch_response(ipc_connection.clone(), interprocess_response, request_id);
                            }
                            InterprocessIngress::_Phantom(_) => {}
                        },
                        Err(_err) => {
                            std::process::exit(1);
                        }
                    }
                }
            }

            thread::sleep(Duration::from_millis(1));
        });
    }
}
