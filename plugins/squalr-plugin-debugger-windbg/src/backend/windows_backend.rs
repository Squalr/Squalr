use crate::constants::WINDBG_DEBUGGER_PLUGIN_ID;
use squalr_engine_api::{plugins::debugger::DebuggerPluginError, structures::processes::opened_process_info::OpenedProcessInfo};
use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
};
use windows::{
    Win32::System::Diagnostics::Debug::Extensions::{DEBUG_ATTACH_DEFAULT, DEBUG_END_ACTIVE_DETACH, DebugCreate, IDebugClient, IDebugControl},
    core::Interface,
};

const INITIAL_ATTACH_WAIT_TIMEOUT_MS: u32 = 10_000;

pub(crate) struct WindbgBackend {
    process_info: OpenedProcessInfo,
    worker_handle: Option<WindbgWorkerHandle>,
}

impl WindbgBackend {
    pub(crate) fn new(process_info: OpenedProcessInfo) -> Self {
        Self {
            process_info,
            worker_handle: None,
        }
    }

    pub(crate) fn attach(&mut self) -> Result<(), DebuggerPluginError> {
        if self.worker_handle.is_some() {
            return Ok(());
        }

        let process_info = self.process_info.clone();
        let (worker_ready_sender, worker_ready_receiver) = mpsc::channel();
        let thread_handle = thread::spawn(move || windbg_worker_main(process_info, worker_ready_sender));
        let worker_command_sender = match worker_ready_receiver
            .recv()
            .map_err(|error| Self::plugin_error(format!("DbgEng worker exited before reporting attach status: {}", error)))?
        {
            Ok(worker_command_sender) => worker_command_sender,
            Err(error) => {
                let _ = thread_handle.join();

                return Err(error);
            }
        };

        self.worker_handle = Some(WindbgWorkerHandle {
            command_sender: worker_command_sender,
            thread_handle: Some(thread_handle),
        });

        Ok(())
    }

    pub(crate) fn detach(&mut self) -> Result<(), DebuggerPluginError> {
        if let Some(mut worker_handle) = self.worker_handle.take() {
            worker_handle.detach()
        } else {
            Ok(())
        }
    }

    pub(crate) fn unavailable_error(&self) -> DebuggerPluginError {
        DebuggerPluginError::new(WINDBG_DEBUGGER_PLUGIN_ID, "DbgEng debugger backend currently supports attach/detach only.")
    }

    fn plugin_error(message: impl Into<String>) -> DebuggerPluginError {
        DebuggerPluginError::new(WINDBG_DEBUGGER_PLUGIN_ID, message)
    }
}

impl Drop for WindbgBackend {
    fn drop(&mut self) {
        if let Err(error) = self.detach() {
            log::debug!("Failed to detach WinDbg backend during drop: {}", error);
        }
    }
}

struct WindbgWorkerHandle {
    command_sender: Sender<WindbgWorkerCommand>,
    thread_handle: Option<JoinHandle<()>>,
}

impl WindbgWorkerHandle {
    fn detach(&mut self) -> Result<(), DebuggerPluginError> {
        let (detach_result_sender, detach_result_receiver) = mpsc::channel();

        self.command_sender
            .send(WindbgWorkerCommand::Detach {
                result_sender: detach_result_sender,
            })
            .map_err(|error| WindbgBackend::plugin_error(format!("Failed to request DbgEng detach: {}", error)))?;

        let detach_result = detach_result_receiver
            .recv()
            .map_err(|error| WindbgBackend::plugin_error(format!("DbgEng worker exited before detach completed: {}", error)))?;

        if let Some(thread_handle) = self.thread_handle.take() {
            thread_handle
                .join()
                .map_err(|_| WindbgBackend::plugin_error("DbgEng worker thread panicked during detach."))?;
        }

        detach_result
    }
}

enum WindbgWorkerCommand {
    Detach { result_sender: Sender<Result<(), DebuggerPluginError>> },
}

struct ActiveWindbgSession {
    client: IDebugClient,
}

impl ActiveWindbgSession {
    fn attach(process_info: &OpenedProcessInfo) -> Result<Self, DebuggerPluginError> {
        let client = unsafe { DebugCreate::<IDebugClient>() }.map_err(|error| {
            WindbgBackend::plugin_error(format!(
                "DebugCreate<IDebugClient> failed while attaching to '{}' ({}): {}",
                process_info.get_name(),
                process_info.get_process_id(),
                error
            ))
        })?;
        let control = client.cast::<IDebugControl>().map_err(|error| {
            WindbgBackend::plugin_error(format!(
                "IDebugClient could not be cast to IDebugControl while attaching to '{}' ({}): {}",
                process_info.get_name(),
                process_info.get_process_id(),
                error
            ))
        })?;

        unsafe {
            client
                .AttachProcess(0, process_info.get_process_id(), DEBUG_ATTACH_DEFAULT)
                .map_err(|error| {
                    WindbgBackend::plugin_error(format!(
                        "IDebugClient::AttachProcess failed for '{}' ({}): {}",
                        process_info.get_name(),
                        process_info.get_process_id(),
                        error
                    ))
                })?;
        }

        if let Err(error) = unsafe { control.WaitForEvent(0, INITIAL_ATTACH_WAIT_TIMEOUT_MS) } {
            let _ = unsafe { client.DetachProcesses() };
            let _ = unsafe { client.EndSession(DEBUG_END_ACTIVE_DETACH) };

            return Err(WindbgBackend::plugin_error(format!(
                "IDebugControl::WaitForEvent timed out or failed after attaching to '{}' ({}): {}",
                process_info.get_name(),
                process_info.get_process_id(),
                error
            )));
        }

        Ok(Self { client })
    }

    fn detach(&self) -> Result<(), DebuggerPluginError> {
        let detach_processes_result =
            unsafe { self.client.DetachProcesses() }.map_err(|error| WindbgBackend::plugin_error(format!("IDebugClient::DetachProcesses failed: {}", error)));
        let end_session_result = unsafe { self.client.EndSession(DEBUG_END_ACTIVE_DETACH) }
            .map_err(|error| WindbgBackend::plugin_error(format!("IDebugClient::EndSession(DEBUG_END_ACTIVE_DETACH) failed: {}", error)));

        detach_processes_result?;
        end_session_result
    }
}

fn windbg_worker_main(
    process_info: OpenedProcessInfo,
    worker_ready_sender: Sender<Result<Sender<WindbgWorkerCommand>, DebuggerPluginError>>,
) {
    let active_session = match ActiveWindbgSession::attach(&process_info) {
        Ok(active_session) => active_session,
        Err(error) => {
            let _ = worker_ready_sender.send(Err(error));

            return;
        }
    };
    let (worker_command_sender, worker_command_receiver) = mpsc::channel();

    if worker_ready_sender.send(Ok(worker_command_sender)).is_err() {
        let _ = active_session.detach();

        return;
    }

    wait_for_worker_commands(active_session, worker_command_receiver);
}

fn wait_for_worker_commands(
    active_session: ActiveWindbgSession,
    worker_command_receiver: Receiver<WindbgWorkerCommand>,
) {
    if let Ok(worker_command) = worker_command_receiver.recv() {
        match worker_command {
            WindbgWorkerCommand::Detach { result_sender } => {
                let detach_result = active_session.detach();
                let _ = result_sender.send(detach_result);
            }
        }
    } else {
        let _ = active_session.detach();
    }
}
