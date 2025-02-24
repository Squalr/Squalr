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

pub struct InterProcessPrivilegedShell<
    RequestType: ExecutableRequest<ResponseType, ExecutionContextType> + DeserializeOwned + Serialize,
    ResponseType: DeserializeOwned + Serialize,
    EventType: DeserializeOwned + Serialize,
    ExecutionContextType: Send + Sync + 'static,
> {
    ipc_connection: Arc<RwLock<Option<InterProcessPipeBidirectional>>>,
    _phantom_data: PhantomData<(RequestType, ResponseType, EventType, ExecutionContextType)>,
}

impl<
        RequestType: ExecutableRequest<ResponseType, ExecutionContextType> + DeserializeOwned + Serialize,
        ResponseType: DeserializeOwned + Serialize,
        EventType: DeserializeOwned + Serialize,
        ExecutionContextType: Send + Sync,
    > InterProcessPrivilegedShell<RequestType, ResponseType, EventType, ExecutionContextType>
{
    pub fn new() -> InterProcessPrivilegedShell<RequestType, ResponseType, EventType, ExecutionContextType> {
        let instance = InterProcessPrivilegedShell {
            ipc_connection: Arc::new(RwLock::new(None)),
            _phantom_data: PhantomData,
        };

        instance
    }

    pub fn initialize(
        &self,
        execution_context: &Arc<ExecutionContextType>,
    ) -> io::Result<()> {
        if let Ok(mut ipc_connection) = self.ipc_connection.write() {
            match InterProcessPipeBidirectional::create() {
                Ok(new_connection) => {
                    *ipc_connection = Some(new_connection);
                    self.listen_for_host_requests(&execution_context);
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

    pub fn dispatch_event(
        &self,
        interprocess_event: InterprocessEgress<ResponseType, EventType>,
    ) -> io::Result<()> {
        Self::dispatch_response(self.ipc_connection.clone(), interprocess_event, Uuid::nil())
    }

    pub fn dispatch_response(
        ipc_connection: Arc<RwLock<Option<InterProcessPipeBidirectional>>>,
        interprocess_response: InterprocessEgress<ResponseType, EventType>,
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

    fn listen_for_host_requests(
        &self,
        execution_context: &Arc<ExecutionContextType>,
    ) {
        let ipc_connection = self.ipc_connection.clone();
        let execution_context = execution_context.clone();

        thread::spawn(move || loop {
            let ipc_connection = ipc_connection.clone();
            let execution_context = execution_context.clone();

            if let Ok(ipc_connection_guard) = ipc_connection.read() {
                if let Some(ipc_connection_pipe) = ipc_connection_guard.as_ref() {
                    match ipc_connection_pipe.receive::<InterprocessIngress<RequestType, ResponseType, ExecutionContextType>>() {
                        Ok((interprocess_command, request_id)) => match interprocess_command {
                            InterprocessIngress::EngineCommand(engine_command) => {
                                let interprocess_response = InterprocessEgress::EngineResponse(engine_command.execute(&execution_context));
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
