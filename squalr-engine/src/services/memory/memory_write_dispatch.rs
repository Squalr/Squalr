use squalr_engine_api::commands::memory::write::memory_write_request::MemoryWriteRequest;
use squalr_engine_api::commands::memory::write::memory_write_response::MemoryWriteResponse;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use std::sync::{Arc, mpsc};
use std::time::Duration;

pub fn dispatch_memory_write_request(
    engine_execution_context: &Arc<dyn EngineExecutionContext>,
    memory_write_request: MemoryWriteRequest,
    timeout: Duration,
) -> Result<MemoryWriteResponse, String> {
    let memory_write_command = memory_write_request.to_engine_command();
    let (memory_write_response_sender, memory_write_response_receiver) = mpsc::channel();

    let dispatch_result = match engine_execution_context.get_bindings().read() {
        Ok(engine_bindings) => engine_bindings.dispatch_privileged_command(
            memory_write_command,
            Box::new(move |engine_response| {
                let conversion_result = MemoryWriteResponse::from_engine_response(engine_response)
                    .map_err(|unexpected_response| format!("Unexpected response variant for memory write request: {:?}.", unexpected_response));
                let _ = memory_write_response_sender.send(conversion_result);
            }),
        ),
        Err(error) => {
            return Err(format!("Failed to acquire engine bindings lock for memory write request: {}.", error));
        }
    };

    if let Err(error) = dispatch_result {
        return Err(format!("Failed to dispatch memory write request: {}.", error));
    }

    match memory_write_response_receiver.recv_timeout(timeout) {
        Ok(Ok(memory_write_response)) => Ok(memory_write_response),
        Ok(Err(error)) => Err(format!("Failed to convert memory write response: {}.", error)),
        Err(error) => Err(format!("Timed out waiting for memory write response: {}.", error)),
    }
}
