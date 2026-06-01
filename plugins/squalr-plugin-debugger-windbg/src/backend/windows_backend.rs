use crate::constants::WINDBG_DEBUGGER_PLUGIN_ID;
use squalr_engine_api::structures::debugger::DebuggerRegisterSnapshot;
use squalr_engine_api::{plugins::debugger::DebuggerPluginError, structures::processes::opened_process_info::OpenedProcessInfo};
use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
};
use windows::{
    Win32::System::Diagnostics::Debug::Extensions::{
        DEBUG_ATTACH_DEFAULT, DEBUG_END_ACTIVE_DETACH, DEBUG_INTERRUPT_ACTIVE, DEBUG_STATUS_GO, DebugCreate, IDebugClient, IDebugControl, IDebugRegisters,
    },
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

    pub(crate) fn pause(&self) -> Result<(), DebuggerPluginError> {
        self.worker_handle
            .as_ref()
            .ok_or_else(|| Self::plugin_error("Cannot pause because there is no active DbgEng worker."))?
            .pause()
    }

    pub(crate) fn resume(&self) -> Result<(), DebuggerPluginError> {
        self.worker_handle
            .as_ref()
            .ok_or_else(|| Self::plugin_error("Cannot resume because there is no active DbgEng worker."))?
            .resume()
    }

    pub(crate) fn read_registers(&self) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        self.worker_handle
            .as_ref()
            .ok_or_else(|| Self::plugin_error("Cannot read registers because there is no active DbgEng worker."))?
            .read_registers()
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
    fn pause(&self) -> Result<(), DebuggerPluginError> {
        self.request_worker_result(WindbgWorkerCommandKind::Pause)
    }

    fn resume(&self) -> Result<(), DebuggerPluginError> {
        self.request_worker_result(WindbgWorkerCommandKind::Resume)
    }

    fn read_registers(&self) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        let (result_sender, result_receiver) = mpsc::channel();

        self.command_sender
            .send(WindbgWorkerCommand::ReadRegisters { result_sender })
            .map_err(|error| WindbgBackend::plugin_error(format!("Failed to request DbgEng register snapshot: {}", error)))?;

        result_receiver
            .recv()
            .map_err(|error| WindbgBackend::plugin_error(format!("DbgEng worker exited before register snapshot completed: {}", error)))?
    }

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

    fn request_worker_result(
        &self,
        command_kind: WindbgWorkerCommandKind,
    ) -> Result<(), DebuggerPluginError> {
        let (result_sender, result_receiver) = mpsc::channel();
        let worker_command = match command_kind {
            WindbgWorkerCommandKind::Pause => WindbgWorkerCommand::Pause { result_sender },
            WindbgWorkerCommandKind::Resume => WindbgWorkerCommand::Resume { result_sender },
        };

        self.command_sender
            .send(worker_command)
            .map_err(|error| WindbgBackend::plugin_error(format!("Failed to send DbgEng worker command: {}", error)))?;

        result_receiver
            .recv()
            .map_err(|error| WindbgBackend::plugin_error(format!("DbgEng worker exited before command completed: {}", error)))?
    }
}

enum WindbgWorkerCommand {
    Pause {
        result_sender: Sender<Result<(), DebuggerPluginError>>,
    },
    Resume {
        result_sender: Sender<Result<(), DebuggerPluginError>>,
    },
    ReadRegisters {
        result_sender: Sender<Result<DebuggerRegisterSnapshot, DebuggerPluginError>>,
    },
    Detach {
        result_sender: Sender<Result<(), DebuggerPluginError>>,
    },
}

enum WindbgWorkerCommandKind {
    Pause,
    Resume,
}

struct ActiveWindbgSession {
    client: IDebugClient,
    control: IDebugControl,
    registers: IDebugRegisters,
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
        let registers = client.cast::<IDebugRegisters>().map_err(|error| {
            WindbgBackend::plugin_error(format!(
                "IDebugClient could not be cast to IDebugRegisters while attaching to '{}' ({}): {}",
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

        Ok(Self { client, control, registers })
    }

    fn pause(&self) -> Result<(), DebuggerPluginError> {
        unsafe {
            self.control
                .SetInterrupt(DEBUG_INTERRUPT_ACTIVE)
                .map_err(|error| WindbgBackend::plugin_error(format!("IDebugControl::SetInterrupt(DEBUG_INTERRUPT_ACTIVE) failed: {}", error)))?;
            self.control
                .WaitForEvent(0, INITIAL_ATTACH_WAIT_TIMEOUT_MS)
                .map_err(|error| WindbgBackend::plugin_error(format!("IDebugControl::WaitForEvent failed while pausing: {}", error)))?;
        }

        Ok(())
    }

    fn resume(&self) -> Result<(), DebuggerPluginError> {
        unsafe {
            self.control
                .SetExecutionStatus(DEBUG_STATUS_GO)
                .map_err(|error| WindbgBackend::plugin_error(format!("IDebugControl::SetExecutionStatus(DEBUG_STATUS_GO) failed: {}", error)))?;
        }

        Ok(())
    }

    fn read_registers(&self) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        let instruction_pointer = unsafe { self.registers.GetInstructionOffset() }
            .map_err(|error| WindbgBackend::plugin_error(format!("IDebugRegisters::GetInstructionOffset failed: {}", error)))?;
        let stack_pointer = unsafe { self.registers.GetStackOffset() }
            .map_err(|error| WindbgBackend::plugin_error(format!("IDebugRegisters::GetStackOffset failed: {}", error)))?;

        Ok(DebuggerRegisterSnapshot::new(Some(instruction_pointer), Some(stack_pointer), Vec::new()))
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
    while let Ok(worker_command) = worker_command_receiver.recv() {
        match worker_command {
            WindbgWorkerCommand::Pause { result_sender } => {
                let pause_result = active_session.pause();
                let _ = result_sender.send(pause_result);
            }
            WindbgWorkerCommand::Resume { result_sender } => {
                let resume_result = active_session.resume();
                let _ = result_sender.send(resume_result);
            }
            WindbgWorkerCommand::ReadRegisters { result_sender } => {
                let read_registers_result = active_session.read_registers();
                let _ = result_sender.send(read_registers_result);
            }
            WindbgWorkerCommand::Detach { result_sender } => {
                let detach_result = active_session.detach();
                let _ = result_sender.send(detach_result);
                return;
            }
        }
    }

    let _ = active_session.detach();
}
