use crate::constants::WINDBG_DEBUGGER_PLUGIN_ID;
use squalr_engine_api::plugins::debugger::DebuggerTraceEventSink;
use squalr_engine_api::structures::debugger::{
    DebuggerBreakpointDescriptor, DebuggerBreakpointKind, DebuggerDataBreakpointAccess, DebuggerRegisterSnapshot, DebuggerRegisterValue, DebuggerSessionState,
    DebuggerTraceEvent,
};
use squalr_engine_api::{plugins::debugger::DebuggerPluginError, structures::processes::opened_process_info::OpenedProcessInfo};
use std::{
    collections::HashMap,
    ffi::CString,
    mem::size_of,
    sync::mpsc::{self, Receiver, RecvTimeoutError, Sender},
    thread::{self, JoinHandle},
    time::Duration,
};
use windows::{
    Win32::{
        Foundation::{S_FALSE, S_OK},
        System::Diagnostics::Debug::Extensions::{
            DEBUG_ANY_ID, DEBUG_ATTACH_DEFAULT, DEBUG_BREAK_READ, DEBUG_BREAK_WRITE, DEBUG_BREAKPOINT_CODE, DEBUG_BREAKPOINT_DATA, DEBUG_BREAKPOINT_ENABLED,
            DEBUG_BREAKPOINT_PARAMETERS, DEBUG_END_ACTIVE_DETACH, DEBUG_ENGOPT_INITIAL_BREAK, DEBUG_EVENT_BREAKPOINT, DEBUG_EXECUTE_NOT_LOGGED,
            DEBUG_INTERRUPT_ACTIVE, DEBUG_LAST_EVENT_INFO_BREAKPOINT, DEBUG_OUTCTL_IGNORE, DEBUG_REGISTER_DESCRIPTION, DEBUG_STATUS_GO, DEBUG_VALUE,
            DEBUG_VALUE_INT8, DEBUG_VALUE_INT16, DEBUG_VALUE_INT32, DEBUG_VALUE_INT64, DebugCreate, IDebugBreakpoint, IDebugClient, IDebugControl,
            IDebugDataSpaces, IDebugRegisters2, IDebugSystemObjects,
        },
    },
    core::{HSTRING, Interface, PCSTR},
};

const INITIAL_ATTACH_WAIT_TIMEOUT_MS: u32 = 10_000;
const RUNNING_EVENT_WAIT_TIMEOUT_MS: u32 = 50;
const IDLE_COMMAND_WAIT_TIMEOUT_MS: u64 = 50;
const TRACE_INSTRUCTION_BYTE_WINDOW: usize = 16;

pub(crate) struct WindbgBackend {
    process_info: OpenedProcessInfo,
    trace_event_sink: DebuggerTraceEventSink,
    worker_handle: Option<WindbgWorkerHandle>,
}

impl WindbgBackend {
    pub(crate) fn new(
        process_info: OpenedProcessInfo,
        trace_event_sink: DebuggerTraceEventSink,
    ) -> Self {
        Self {
            process_info,
            trace_event_sink,
            worker_handle: None,
        }
    }

    pub(crate) fn attach(&mut self) -> Result<(), DebuggerPluginError> {
        if self.worker_handle.is_some() {
            return Ok(());
        }

        let process_info = self.process_info.clone();
        let trace_event_sink = self.trace_event_sink.clone();
        let (worker_ready_sender, worker_ready_receiver) = mpsc::channel();
        let thread_handle = thread::spawn(move || windbg_worker_main(process_info, trace_event_sink, worker_ready_sender));
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

    pub(crate) fn write_register(
        &self,
        register_name: &str,
        value: u64,
    ) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        self.worker_handle
            .as_ref()
            .ok_or_else(|| Self::plugin_error("Cannot write registers because there is no active DbgEng worker."))?
            .write_register(register_name, value)
    }

    pub(crate) fn set_breakpoint(
        &self,
        address: u64,
        kind: DebuggerBreakpointKind,
        label: Option<String>,
    ) -> Result<DebuggerBreakpointDescriptor, DebuggerPluginError> {
        self.worker_handle
            .as_ref()
            .ok_or_else(|| Self::plugin_error("Cannot set a breakpoint because there is no active DbgEng worker."))?
            .set_breakpoint(address, kind, label)
    }

    pub(crate) fn remove_breakpoint(
        &self,
        breakpoint_id: &str,
    ) -> Result<(), DebuggerPluginError> {
        self.worker_handle
            .as_ref()
            .ok_or_else(|| Self::plugin_error("Cannot remove a breakpoint because there is no active DbgEng worker."))?
            .remove_breakpoint(breakpoint_id)
    }

    pub(crate) fn set_breakpoint_enabled(
        &self,
        breakpoint_id: &str,
        is_enabled: bool,
    ) -> Result<(), DebuggerPluginError> {
        self.worker_handle
            .as_ref()
            .ok_or_else(|| Self::plugin_error("Cannot update a breakpoint because there is no active DbgEng worker."))?
            .set_breakpoint_enabled(breakpoint_id, is_enabled)
    }

    pub(crate) fn list_breakpoints(&self) -> Result<Vec<DebuggerBreakpointDescriptor>, DebuggerPluginError> {
        self.worker_handle
            .as_ref()
            .ok_or_else(|| Self::plugin_error("Cannot list breakpoints because there is no active DbgEng worker."))?
            .list_breakpoints()
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

    fn write_register(
        &self,
        register_name: &str,
        value: u64,
    ) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        let (result_sender, result_receiver) = mpsc::channel();

        self.command_sender
            .send(WindbgWorkerCommand::WriteRegister {
                register_name: register_name.to_string(),
                value,
                result_sender,
            })
            .map_err(|error| WindbgBackend::plugin_error(format!("Failed to request DbgEng register write: {}", error)))?;

        result_receiver
            .recv()
            .map_err(|error| WindbgBackend::plugin_error(format!("DbgEng worker exited before register write completed: {}", error)))?
    }

    fn set_breakpoint(
        &self,
        address: u64,
        kind: DebuggerBreakpointKind,
        label: Option<String>,
    ) -> Result<DebuggerBreakpointDescriptor, DebuggerPluginError> {
        let (result_sender, result_receiver) = mpsc::channel();

        self.command_sender
            .send(WindbgWorkerCommand::SetBreakpoint {
                address,
                kind,
                label,
                result_sender,
            })
            .map_err(|error| WindbgBackend::plugin_error(format!("Failed to request DbgEng breakpoint set: {}", error)))?;

        result_receiver
            .recv()
            .map_err(|error| WindbgBackend::plugin_error(format!("DbgEng worker exited before breakpoint set completed: {}", error)))?
    }

    fn remove_breakpoint(
        &self,
        breakpoint_id: &str,
    ) -> Result<(), DebuggerPluginError> {
        let (result_sender, result_receiver) = mpsc::channel();

        self.command_sender
            .send(WindbgWorkerCommand::RemoveBreakpoint {
                breakpoint_id: breakpoint_id.to_string(),
                result_sender,
            })
            .map_err(|error| WindbgBackend::plugin_error(format!("Failed to request DbgEng breakpoint removal: {}", error)))?;

        result_receiver
            .recv()
            .map_err(|error| WindbgBackend::plugin_error(format!("DbgEng worker exited before breakpoint removal completed: {}", error)))?
    }

    fn set_breakpoint_enabled(
        &self,
        breakpoint_id: &str,
        is_enabled: bool,
    ) -> Result<(), DebuggerPluginError> {
        let (result_sender, result_receiver) = mpsc::channel();

        self.command_sender
            .send(WindbgWorkerCommand::SetBreakpointEnabled {
                breakpoint_id: breakpoint_id.to_string(),
                is_enabled,
                result_sender,
            })
            .map_err(|error| WindbgBackend::plugin_error(format!("Failed to request DbgEng breakpoint state update: {}", error)))?;

        result_receiver
            .recv()
            .map_err(|error| WindbgBackend::plugin_error(format!("DbgEng worker exited before breakpoint state update completed: {}", error)))?
    }

    fn list_breakpoints(&self) -> Result<Vec<DebuggerBreakpointDescriptor>, DebuggerPluginError> {
        let (result_sender, result_receiver) = mpsc::channel();

        self.command_sender
            .send(WindbgWorkerCommand::ListBreakpoints { result_sender })
            .map_err(|error| WindbgBackend::plugin_error(format!("Failed to request DbgEng breakpoint list: {}", error)))?;

        result_receiver
            .recv()
            .map_err(|error| WindbgBackend::plugin_error(format!("DbgEng worker exited before breakpoint list completed: {}", error)))?
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
    WriteRegister {
        register_name: String,
        value: u64,
        result_sender: Sender<Result<DebuggerRegisterSnapshot, DebuggerPluginError>>,
    },
    SetBreakpoint {
        address: u64,
        kind: DebuggerBreakpointKind,
        label: Option<String>,
        result_sender: Sender<Result<DebuggerBreakpointDescriptor, DebuggerPluginError>>,
    },
    RemoveBreakpoint {
        breakpoint_id: String,
        result_sender: Sender<Result<(), DebuggerPluginError>>,
    },
    SetBreakpointEnabled {
        breakpoint_id: String,
        is_enabled: bool,
        result_sender: Sender<Result<(), DebuggerPluginError>>,
    },
    ListBreakpoints {
        result_sender: Sender<Result<Vec<DebuggerBreakpointDescriptor>, DebuggerPluginError>>,
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
    data_spaces: IDebugDataSpaces,
    registers: IDebugRegisters2,
    system_objects: IDebugSystemObjects,
    breakpoint_labels: HashMap<u32, Option<String>>,
    session_state: DebuggerSessionState,
    trace_event_sink: DebuggerTraceEventSink,
}

impl ActiveWindbgSession {
    fn attach(
        process_info: &OpenedProcessInfo,
        trace_event_sink: DebuggerTraceEventSink,
    ) -> Result<Self, DebuggerPluginError> {
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
        let registers = client.cast::<IDebugRegisters2>().map_err(|error| {
            WindbgBackend::plugin_error(format!(
                "IDebugClient could not be cast to IDebugRegisters2 while attaching to '{}' ({}): {}",
                process_info.get_name(),
                process_info.get_process_id(),
                error
            ))
        })?;
        let data_spaces = client.cast::<IDebugDataSpaces>().map_err(|error| {
            WindbgBackend::plugin_error(format!(
                "IDebugClient could not be cast to IDebugDataSpaces while attaching to '{}' ({}): {}",
                process_info.get_name(),
                process_info.get_process_id(),
                error
            ))
        })?;
        let system_objects = client.cast::<IDebugSystemObjects>().map_err(|error| {
            WindbgBackend::plugin_error(format!(
                "IDebugClient could not be cast to IDebugSystemObjects while attaching to '{}' ({}): {}",
                process_info.get_name(),
                process_info.get_process_id(),
                error
            ))
        })?;

        unsafe {
            control
                .AddEngineOptions(DEBUG_ENGOPT_INITIAL_BREAK)
                .map_err(|error| WindbgBackend::plugin_error(format!("IDebugControl::AddEngineOptions(DEBUG_ENGOPT_INITIAL_BREAK) failed: {}", error)))?;
        }

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

        if let Err(error) = wait_for_required_debug_event(&control, INITIAL_ATTACH_WAIT_TIMEOUT_MS, "initial attach") {
            let _ = unsafe { client.DetachProcesses() };
            let _ = unsafe { client.EndSession(DEBUG_END_ACTIVE_DETACH) };

            return Err(error);
        }

        let active_session = Self {
            client,
            control,
            data_spaces,
            registers,
            system_objects,
            breakpoint_labels: HashMap::new(),
            session_state: DebuggerSessionState::Attached,
            trace_event_sink,
        };
        let mut attach_context_message = None;

        active_session.select_last_event_context(&mut attach_context_message)?;
        if let Some(attach_context_message) = attach_context_message {
            log::debug!("{}", attach_context_message);
        }

        Ok(active_session)
    }

    fn pause(&mut self) -> Result<(), DebuggerPluginError> {
        unsafe {
            self.control
                .SetInterrupt(DEBUG_INTERRUPT_ACTIVE)
                .map_err(|error| WindbgBackend::plugin_error(format!("IDebugControl::SetInterrupt(DEBUG_INTERRUPT_ACTIVE) failed: {}", error)))?;
        }
        wait_for_required_debug_event(&self.control, INITIAL_ATTACH_WAIT_TIMEOUT_MS, "pause")?;

        self.session_state = DebuggerSessionState::Paused;

        Ok(())
    }

    fn resume(&mut self) -> Result<(), DebuggerPluginError> {
        unsafe {
            self.control
                .SetExecutionStatus(DEBUG_STATUS_GO)
                .map_err(|error| WindbgBackend::plugin_error(format!("IDebugControl::SetExecutionStatus(DEBUG_STATUS_GO) failed: {}", error)))?;
        }

        self.session_state = DebuggerSessionState::Running;

        Ok(())
    }

    fn read_registers(&self) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        let instruction_pointer = unsafe { self.registers.GetInstructionOffset() }
            .map_err(|error| WindbgBackend::plugin_error(format!("IDebugRegisters::GetInstructionOffset failed: {}", error)))?;
        let stack_pointer = unsafe { self.registers.GetStackOffset() }
            .map_err(|error| WindbgBackend::plugin_error(format!("IDebugRegisters::GetStackOffset failed: {}", error)))?;
        let registers = self.read_integer_registers()?;

        Ok(DebuggerRegisterSnapshot::new(Some(instruction_pointer), Some(stack_pointer), registers))
    }

    fn write_register(
        &self,
        register_name: &str,
        value: u64,
    ) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        let register_ordinal = unsafe {
            self.registers
                .GetIndexByNameWide(&HSTRING::from(register_name))
                .map_err(|error| WindbgBackend::plugin_error(format!("IDebugRegisters::GetIndexByNameWide('{}') failed: {}", register_name, error)))?
        };
        let mut debug_value = DEBUG_VALUE::default();

        unsafe {
            self.registers
                .GetValue(register_ordinal, &mut debug_value)
                .map_err(|error| WindbgBackend::plugin_error(format!("IDebugRegisters::GetValue('{}') failed before write: {}", register_name, error)))?;
        }

        Self::set_debug_value_integer(&mut debug_value, value)?;

        unsafe {
            self.registers
                .SetValue(register_ordinal, &debug_value)
                .map_err(|error| WindbgBackend::plugin_error(format!("IDebugRegisters::SetValue('{}') failed: {}", register_name, error)))?;
        }

        self.read_registers()
    }

    fn set_breakpoint(
        &mut self,
        address: u64,
        kind: DebuggerBreakpointKind,
        label: Option<String>,
    ) -> Result<DebuggerBreakpointDescriptor, DebuggerPluginError> {
        let breakpoint_type = Self::debug_breakpoint_type(&kind);
        let breakpoint = unsafe { self.control.AddBreakpoint(breakpoint_type, DEBUG_ANY_ID) }
            .map_err(|error| WindbgBackend::plugin_error(format!("IDebugControl::AddBreakpoint failed for 0x{:X}: {}", address, error)))?;
        let configure_result = self.configure_breakpoint(&breakpoint, address, &kind);

        if let Err(error) = configure_result {
            let _ = unsafe { self.control.RemoveBreakpoint(&breakpoint) };

            return Err(error);
        }

        let breakpoint_id = unsafe { breakpoint.GetId() }.map_err(|error| {
            WindbgBackend::plugin_error(format!(
                "IDebugBreakpoint::GetId failed after creating breakpoint at 0x{:X}: {}",
                address, error
            ))
        })?;
        self.breakpoint_labels.insert(breakpoint_id, label.clone());

        Ok(DebuggerBreakpointDescriptor::new(breakpoint_id.to_string(), address, kind, true, label))
    }

    fn remove_breakpoint(
        &mut self,
        breakpoint_id: &str,
    ) -> Result<(), DebuggerPluginError> {
        let debug_breakpoint_id = Self::parse_breakpoint_id(breakpoint_id)?;
        let remove_result = self.execute_debugger_command(&format!("bc {}", debug_breakpoint_id), &format!("clear breakpoint {}", breakpoint_id));

        if remove_result.is_ok() {
            self.breakpoint_labels.remove(&debug_breakpoint_id);
        }

        remove_result
    }

    fn set_breakpoint_enabled(
        &mut self,
        breakpoint_id: &str,
        is_enabled: bool,
    ) -> Result<(), DebuggerPluginError> {
        let debug_breakpoint_id = Self::parse_breakpoint_id(breakpoint_id)?;
        let breakpoint = unsafe { self.control.GetBreakpointById(debug_breakpoint_id) }
            .map_err(|error| WindbgBackend::plugin_error(format!("IDebugControl::GetBreakpointById({}) failed: {}", debug_breakpoint_id, error)))?;
        let current_flags = unsafe { breakpoint.GetFlags() }
            .map_err(|error| WindbgBackend::plugin_error(format!("IDebugBreakpoint::GetFlags({}) failed: {}", debug_breakpoint_id, error)))?;
        let updated_flags = if is_enabled {
            current_flags | DEBUG_BREAKPOINT_ENABLED
        } else {
            current_flags & !DEBUG_BREAKPOINT_ENABLED
        };

        unsafe {
            breakpoint.SetFlags(updated_flags).map_err(|error| {
                WindbgBackend::plugin_error(format!(
                    "IDebugBreakpoint::SetFlags({:#X}) failed while trying to {} breakpoint {}: {}",
                    updated_flags,
                    if is_enabled { "enable" } else { "disable" },
                    breakpoint_id,
                    error
                ))
            })
        }
    }

    fn execute_debugger_command(
        &self,
        command: &str,
        context: &str,
    ) -> Result<(), DebuggerPluginError> {
        let command = CString::new(command)
            .map_err(|error| WindbgBackend::plugin_error(format!("DbgEng command for {} contained an interior null byte: {}", context, error)))?;

        unsafe {
            self.control
                .Execute(DEBUG_OUTCTL_IGNORE, PCSTR(command.as_ptr().cast()), DEBUG_EXECUTE_NOT_LOGGED)
                .map_err(|error| WindbgBackend::plugin_error(format!("IDebugControl::Execute failed while trying to {}: {}", context, error)))
        }
    }

    fn list_breakpoints(&self) -> Result<Vec<DebuggerBreakpointDescriptor>, DebuggerPluginError> {
        let breakpoint_count = unsafe { self.control.GetNumberBreakpoints() }
            .map_err(|error| WindbgBackend::plugin_error(format!("IDebugControl::GetNumberBreakpoints failed: {}", error)))?;
        let mut breakpoints = Vec::new();

        for breakpoint_ordinal in 0..breakpoint_count {
            let breakpoint = unsafe { self.control.GetBreakpointByIndex(breakpoint_ordinal) }
                .map_err(|error| WindbgBackend::plugin_error(format!("IDebugControl::GetBreakpointByIndex({}) failed: {}", breakpoint_ordinal, error)))?;

            if let Some(breakpoint_descriptor) = self.describe_breakpoint(&breakpoint)? {
                breakpoints.push(breakpoint_descriptor);
            }
        }

        Ok(breakpoints)
    }

    fn process_pending_debug_event(&mut self) -> Result<(), DebuggerPluginError> {
        if self.session_state != DebuggerSessionState::Running {
            return Ok(());
        }

        if !wait_for_optional_debug_event(&self.control, RUNNING_EVENT_WAIT_TIMEOUT_MS, "running event poll")? {
            return Ok(());
        }

        self.handle_last_debug_event()
    }

    fn handle_last_debug_event(&mut self) -> Result<(), DebuggerPluginError> {
        let mut debug_event_type = 0u32;
        let mut debug_event_process_id = 0u32;
        let mut debug_event_thread_id = 0u32;
        let mut breakpoint_event = DEBUG_LAST_EVENT_INFO_BREAKPOINT::default();
        let mut extra_information_used = 0u32;
        let mut event_description_buffer = [0u8; 512];
        let mut event_description_used = 0u32;

        unsafe {
            self.control
                .GetLastEventInformation(
                    &mut debug_event_type,
                    &mut debug_event_process_id,
                    &mut debug_event_thread_id,
                    Some((&mut breakpoint_event as *mut DEBUG_LAST_EVENT_INFO_BREAKPOINT).cast()),
                    size_of::<DEBUG_LAST_EVENT_INFO_BREAKPOINT>() as u32,
                    Some(&mut extra_information_used),
                    Some(&mut event_description_buffer),
                    Some(&mut event_description_used),
                )
                .map_err(|error| WindbgBackend::plugin_error(format!("IDebugControl::GetLastEventInformation failed: {}", error)))?;
        }

        if debug_event_type != DEBUG_EVENT_BREAKPOINT {
            return Ok(());
        }

        self.session_state = DebuggerSessionState::Paused;
        let breakpoint_descriptor = self.describe_breakpoint_by_id(breakpoint_event.Id)?;
        let mut backend_message = Self::decode_debug_event_description(&event_description_buffer, event_description_used);
        self.set_current_event_context(debug_event_process_id, debug_event_thread_id, &mut backend_message);
        let register_snapshot = match self.read_registers() {
            Ok(register_snapshot) => register_snapshot,
            Err(error) => {
                backend_message = Some(match backend_message {
                    Some(existing_message) => format!("{} Register capture failed: {}", existing_message, error),
                    None => format!("Register capture failed: {}", error),
                });
                DebuggerRegisterSnapshot::default()
            }
        };
        let trace_instruction_pointer = self.resolve_trace_instruction_pointer(
            register_snapshot.get_instruction_pointer(),
            breakpoint_descriptor.as_ref(),
            &mut backend_message,
        );
        let instruction_bytes = self.read_instruction_bytes(trace_instruction_pointer);

        (self.trace_event_sink)(DebuggerTraceEvent::new(
            breakpoint_descriptor.clone(),
            register_snapshot,
            trace_instruction_pointer,
            instruction_bytes,
            None,
            backend_message,
        ));

        if matches!(
            breakpoint_descriptor
                .as_ref()
                .map(DebuggerBreakpointDescriptor::get_kind),
            Some(DebuggerBreakpointKind::HardwareData { .. })
        ) {
            self.resume()?;
        }

        Ok(())
    }

    fn select_last_event_context(
        &self,
        backend_message: &mut Option<String>,
    ) -> Result<(), DebuggerPluginError> {
        let mut debug_event_type = 0u32;
        let mut debug_event_process_id = 0u32;
        let mut debug_event_thread_id = 0u32;

        unsafe {
            self.control
                .GetLastEventInformation(
                    &mut debug_event_type,
                    &mut debug_event_process_id,
                    &mut debug_event_thread_id,
                    None,
                    0,
                    None,
                    None,
                    None,
                )
                .map_err(|error| {
                    WindbgBackend::plugin_error(format!(
                        "IDebugControl::GetLastEventInformation failed while selecting event context: {}",
                        error
                    ))
                })?;
        }

        self.set_current_event_context(debug_event_process_id, debug_event_thread_id, backend_message);

        Ok(())
    }

    fn set_current_event_context(
        &self,
        debug_event_process_id: u32,
        debug_event_thread_id: u32,
        backend_message: &mut Option<String>,
    ) {
        let process_result = unsafe { self.system_objects.SetCurrentProcessId(debug_event_process_id) };
        let thread_result = unsafe { self.system_objects.SetCurrentThreadId(debug_event_thread_id) };

        if process_result.is_ok() && thread_result.is_ok() {
            return;
        }

        let context_message = format!(
            "Failed to select DbgEng event context process={} thread={} before register capture: process={:?}, thread={:?}.",
            debug_event_process_id, debug_event_thread_id, process_result, thread_result
        );

        *backend_message = Some(match backend_message.take() {
            Some(existing_message) => format!("{} {}", existing_message, context_message),
            None => context_message,
        });
    }

    fn configure_breakpoint(
        &self,
        breakpoint: &IDebugBreakpoint,
        address: u64,
        kind: &DebuggerBreakpointKind,
    ) -> Result<(), DebuggerPluginError> {
        unsafe {
            breakpoint
                .SetOffset(address)
                .map_err(|error| WindbgBackend::plugin_error(format!("IDebugBreakpoint::SetOffset(0x{:X}) failed: {}", address, error)))?;
        }

        if let DebuggerBreakpointKind::HardwareData { access, size_in_bytes } = kind {
            let data_size = Self::validate_data_breakpoint_size(*size_in_bytes)?;
            let data_access = Self::debug_data_breakpoint_access(*access);

            unsafe {
                breakpoint
                    .SetDataParameters(data_size, data_access)
                    .map_err(|error| {
                        WindbgBackend::plugin_error(format!(
                            "IDebugBreakpoint::SetDataParameters(size={}, access={}) failed for 0x{:X}: {}",
                            size_in_bytes, data_access, address, error
                        ))
                    })?;
            }
        }

        unsafe {
            breakpoint
                .SetFlags(DEBUG_BREAKPOINT_ENABLED)
                .map_err(|error| WindbgBackend::plugin_error(format!("IDebugBreakpoint::SetFlags(DEBUG_BREAKPOINT_ENABLED) failed: {}", error)))?;
        }

        Ok(())
    }

    fn describe_breakpoint(
        &self,
        breakpoint: &IDebugBreakpoint,
    ) -> Result<Option<DebuggerBreakpointDescriptor>, DebuggerPluginError> {
        let mut parameters = DEBUG_BREAKPOINT_PARAMETERS::default();

        unsafe {
            breakpoint
                .GetParameters(&mut parameters)
                .map_err(|error| WindbgBackend::plugin_error(format!("IDebugBreakpoint::GetParameters failed: {}", error)))?;
        }

        let Some(kind) = Self::breakpoint_kind_from_parameters(&parameters)? else {
            return Ok(None);
        };
        let is_enabled = parameters.Flags & DEBUG_BREAKPOINT_ENABLED != 0;
        let label = self.breakpoint_labels.get(&parameters.Id).cloned().flatten();

        Ok(Some(DebuggerBreakpointDescriptor::new(
            parameters.Id.to_string(),
            parameters.Offset,
            kind,
            is_enabled,
            label,
        )))
    }

    fn describe_breakpoint_by_id(
        &self,
        debug_breakpoint_id: u32,
    ) -> Result<Option<DebuggerBreakpointDescriptor>, DebuggerPluginError> {
        let breakpoint = match unsafe { self.control.GetBreakpointById(debug_breakpoint_id) } {
            Ok(breakpoint) => breakpoint,
            Err(error) => {
                log::debug!(
                    "IDebugControl::GetBreakpointById({}) failed while describing a breakpoint event: {}",
                    debug_breakpoint_id,
                    error
                );

                return Ok(None);
            }
        };

        self.describe_breakpoint(&breakpoint)
    }

    fn read_instruction_bytes(
        &self,
        instruction_pointer: Option<u64>,
    ) -> Vec<u8> {
        let Some(instruction_pointer) = instruction_pointer else {
            return Vec::new();
        };
        let mut instruction_bytes = vec![0u8; TRACE_INSTRUCTION_BYTE_WINDOW];
        let mut bytes_read = 0u32;
        let read_result = unsafe {
            self.data_spaces.ReadVirtual(
                instruction_pointer,
                instruction_bytes.as_mut_ptr().cast(),
                instruction_bytes.len() as u32,
                Some(&mut bytes_read),
            )
        };

        if let Err(error) = read_result {
            log::debug!(
                "IDebugDataSpaces::ReadVirtual failed while reading instruction bytes at 0x{:X}: {}",
                instruction_pointer,
                error
            );

            return Vec::new();
        }

        instruction_bytes.truncate(bytes_read as usize);
        instruction_bytes
    }

    fn resolve_trace_instruction_pointer(
        &self,
        event_instruction_pointer: Option<u64>,
        breakpoint_descriptor: Option<&DebuggerBreakpointDescriptor>,
        backend_message: &mut Option<String>,
    ) -> Option<u64> {
        let Some(event_instruction_pointer) = event_instruction_pointer else {
            return None;
        };
        let is_data_breakpoint = matches!(
            breakpoint_descriptor.map(DebuggerBreakpointDescriptor::get_kind),
            Some(DebuggerBreakpointKind::HardwareData { .. })
        );

        if !is_data_breakpoint {
            return Some(event_instruction_pointer);
        }

        match unsafe { self.control.GetNearInstruction(event_instruction_pointer, -1) } {
            Ok(access_instruction_pointer) => Some(access_instruction_pointer),
            Err(error) => {
                let attribution_message = format!(
                    "Failed to resolve access instruction before post-trap IP 0x{:X}: {}.",
                    event_instruction_pointer, error
                );

                *backend_message = Some(match backend_message.take() {
                    Some(existing_message) => format!("{} {}", existing_message, attribution_message),
                    None => attribution_message,
                });

                Some(event_instruction_pointer)
            }
        }
    }

    fn debug_breakpoint_type(kind: &DebuggerBreakpointKind) -> u32 {
        match kind {
            DebuggerBreakpointKind::Software => DEBUG_BREAKPOINT_CODE,
            DebuggerBreakpointKind::HardwareData { .. } => DEBUG_BREAKPOINT_DATA,
        }
    }

    fn debug_data_breakpoint_access(access: DebuggerDataBreakpointAccess) -> u32 {
        match access {
            DebuggerDataBreakpointAccess::Read => DEBUG_BREAK_READ,
            DebuggerDataBreakpointAccess::Write => DEBUG_BREAK_WRITE,
            DebuggerDataBreakpointAccess::ReadWrite => DEBUG_BREAK_READ | DEBUG_BREAK_WRITE,
        }
    }

    fn breakpoint_kind_from_parameters(parameters: &DEBUG_BREAKPOINT_PARAMETERS) -> Result<Option<DebuggerBreakpointKind>, DebuggerPluginError> {
        match parameters.BreakType {
            DEBUG_BREAKPOINT_CODE => Ok(Some(DebuggerBreakpointKind::Software)),
            DEBUG_BREAKPOINT_DATA => Ok(Some(DebuggerBreakpointKind::hardware_data(
                Self::data_breakpoint_access_from_debug(parameters.DataAccessType)?,
                parameters.DataSize as u8,
            ))),
            _ => Ok(None),
        }
    }

    fn data_breakpoint_access_from_debug(data_access_type: u32) -> Result<DebuggerDataBreakpointAccess, DebuggerPluginError> {
        let has_read = data_access_type & DEBUG_BREAK_READ != 0;
        let has_write = data_access_type & DEBUG_BREAK_WRITE != 0;

        match (has_read, has_write) {
            (true, true) => Ok(DebuggerDataBreakpointAccess::ReadWrite),
            (true, false) => Ok(DebuggerDataBreakpointAccess::Read),
            (false, true) => Ok(DebuggerDataBreakpointAccess::Write),
            (false, false) => Err(WindbgBackend::plugin_error(format!(
                "DbgEng data breakpoint access flags {} do not map to a Squalr access mode.",
                data_access_type
            ))),
        }
    }

    fn validate_data_breakpoint_size(size_in_bytes: u8) -> Result<u32, DebuggerPluginError> {
        match size_in_bytes {
            1 | 2 | 4 | 8 => Ok(size_in_bytes as u32),
            _ => Err(WindbgBackend::plugin_error(format!(
                "Hardware data breakpoint size {} is unsupported. Expected 1, 2, 4, or 8 bytes.",
                size_in_bytes
            ))),
        }
    }

    fn parse_breakpoint_id(breakpoint_id: &str) -> Result<u32, DebuggerPluginError> {
        breakpoint_id
            .parse::<u32>()
            .map_err(|error| WindbgBackend::plugin_error(format!("Breakpoint id '{}' is not a valid DbgEng breakpoint id: {}", breakpoint_id, error)))
    }

    fn decode_debug_event_description(
        event_description_buffer: &[u8],
        event_description_used: u32,
    ) -> Option<String> {
        let candidate_length = if event_description_used > 0 {
            event_description_used as usize
        } else {
            event_description_buffer
                .iter()
                .position(|character| *character == 0)
                .unwrap_or(event_description_buffer.len())
        };
        let bounded_length = candidate_length.min(event_description_buffer.len());
        let trimmed_length = if bounded_length > 0 && event_description_buffer[bounded_length - 1] == 0 {
            bounded_length - 1
        } else {
            bounded_length
        };
        let description = String::from_utf8_lossy(&event_description_buffer[..trimmed_length])
            .trim()
            .to_string();

        if description.is_empty() { None } else { Some(description) }
    }

    fn read_integer_registers(&self) -> Result<Vec<DebuggerRegisterValue>, DebuggerPluginError> {
        let register_count = unsafe { self.registers.GetNumberRegisters() }
            .map_err(|error| WindbgBackend::plugin_error(format!("IDebugRegisters::GetNumberRegisters failed: {}", error)))?;
        let mut register_values = Vec::new();

        for register_ordinal in 0..register_count {
            if let Some(register_value) = self.read_integer_register(register_ordinal)? {
                register_values.push(register_value);
            }
        }

        Ok(register_values)
    }

    fn read_integer_register(
        &self,
        register_ordinal: u32,
    ) -> Result<Option<DebuggerRegisterValue>, DebuggerPluginError> {
        let mut register_name_buffer = [0u16; 128];
        let mut register_name_size = 0u32;
        let mut register_description = DEBUG_REGISTER_DESCRIPTION::default();
        let mut debug_value = DEBUG_VALUE::default();

        unsafe {
            self.registers
                .GetDescriptionWide(
                    register_ordinal,
                    Some(&mut register_name_buffer),
                    Some(&mut register_name_size),
                    Some(&mut register_description),
                )
                .map_err(|error| WindbgBackend::plugin_error(format!("IDebugRegisters::GetDescriptionWide({}) failed: {}", register_ordinal, error)))?;
            if let Err(error) = self.registers.GetValue(register_ordinal, &mut debug_value) {
                log::debug!("IDebugRegisters::GetValue({}) failed while enumerating registers: {}", register_ordinal, error);

                return Ok(None);
            }
        }

        let Some((value, bit_width)) = Self::debug_value_to_u64(&debug_value) else {
            return Ok(None);
        };
        let register_name = Self::decode_register_name(&register_name_buffer, register_name_size);

        if register_name.is_empty() {
            return Ok(None);
        }

        Ok(Some(DebuggerRegisterValue::new(register_name, value, bit_width)))
    }

    fn decode_register_name(
        register_name_buffer: &[u16],
        register_name_size: u32,
    ) -> String {
        let candidate_length = if register_name_size > 0 {
            register_name_size as usize
        } else {
            register_name_buffer
                .iter()
                .position(|character| *character == 0)
                .unwrap_or(register_name_buffer.len())
        };
        let bounded_length = candidate_length.min(register_name_buffer.len());
        let trimmed_length = if bounded_length > 0 && register_name_buffer[bounded_length - 1] == 0 {
            bounded_length - 1
        } else {
            bounded_length
        };

        String::from_utf16_lossy(&register_name_buffer[..trimmed_length])
    }

    fn debug_value_to_u64(debug_value: &DEBUG_VALUE) -> Option<(u64, u16)> {
        match debug_value.Type {
            DEBUG_VALUE_INT8 => Some((unsafe { debug_value.Anonymous.I8 } as u64, 8)),
            DEBUG_VALUE_INT16 => Some((unsafe { debug_value.Anonymous.I16 } as u64, 16)),
            DEBUG_VALUE_INT32 => Some((unsafe { debug_value.Anonymous.I32 } as u64, 32)),
            DEBUG_VALUE_INT64 => Some((unsafe { debug_value.Anonymous.Anonymous.I64 }, 64)),
            _ => None,
        }
    }

    fn set_debug_value_integer(
        debug_value: &mut DEBUG_VALUE,
        value: u64,
    ) -> Result<(), DebuggerPluginError> {
        match debug_value.Type {
            DEBUG_VALUE_INT8 => {
                debug_value.Anonymous.I8 = value as u8;
                Ok(())
            }
            DEBUG_VALUE_INT16 => {
                debug_value.Anonymous.I16 = value as u16;
                Ok(())
            }
            DEBUG_VALUE_INT32 => {
                debug_value.Anonymous.I32 = value as u32;
                Ok(())
            }
            DEBUG_VALUE_INT64 => {
                debug_value.Anonymous.Anonymous.I64 = value;
                Ok(())
            }
            _ => Err(WindbgBackend::plugin_error(format!(
                "Register value type {} is not supported for integer writes.",
                debug_value.Type
            ))),
        }
    }

    fn detach(&self) -> Result<(), DebuggerPluginError> {
        let resume_result = unsafe { self.control.SetExecutionStatus(DEBUG_STATUS_GO) }
            .map_err(|error| WindbgBackend::plugin_error(format!("IDebugControl::SetExecutionStatus(DEBUG_STATUS_GO) before detach failed: {}", error)));
        let detach_processes_result =
            unsafe { self.client.DetachProcesses() }.map_err(|error| WindbgBackend::plugin_error(format!("IDebugClient::DetachProcesses failed: {}", error)));
        let end_session_result = unsafe { self.client.EndSession(DEBUG_END_ACTIVE_DETACH) }
            .map_err(|error| WindbgBackend::plugin_error(format!("IDebugClient::EndSession(DEBUG_END_ACTIVE_DETACH) failed: {}", error)));

        if let Err(error) = resume_result {
            log::debug!("{}", error);
        }

        detach_processes_result?;
        end_session_result
    }
}

fn windbg_worker_main(
    process_info: OpenedProcessInfo,
    trace_event_sink: DebuggerTraceEventSink,
    worker_ready_sender: Sender<Result<Sender<WindbgWorkerCommand>, DebuggerPluginError>>,
) {
    let active_session = match ActiveWindbgSession::attach(&process_info, trace_event_sink) {
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
    mut active_session: ActiveWindbgSession,
    worker_command_receiver: Receiver<WindbgWorkerCommand>,
) {
    loop {
        match worker_command_receiver.recv_timeout(Duration::from_millis(IDLE_COMMAND_WAIT_TIMEOUT_MS)) {
            Ok(worker_command) => {
                if handle_worker_command(&mut active_session, worker_command) {
                    return;
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                if let Err(error) = active_session.process_pending_debug_event() {
                    log::debug!("Failed to process pending DbgEng event: {}", error);
                }
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }

    let _ = active_session.detach();
}

fn handle_worker_command(
    active_session: &mut ActiveWindbgSession,
    worker_command: WindbgWorkerCommand,
) -> bool {
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
        WindbgWorkerCommand::WriteRegister {
            register_name,
            value,
            result_sender,
        } => {
            let write_register_result = active_session.write_register(&register_name, value);
            let _ = result_sender.send(write_register_result);
        }
        WindbgWorkerCommand::SetBreakpoint {
            address,
            kind,
            label,
            result_sender,
        } => {
            let set_breakpoint_result = active_session.set_breakpoint(address, kind, label);
            let _ = result_sender.send(set_breakpoint_result);
        }
        WindbgWorkerCommand::RemoveBreakpoint { breakpoint_id, result_sender } => {
            let remove_breakpoint_result = active_session.remove_breakpoint(&breakpoint_id);
            let _ = result_sender.send(remove_breakpoint_result);
        }
        WindbgWorkerCommand::SetBreakpointEnabled {
            breakpoint_id,
            is_enabled,
            result_sender,
        } => {
            let set_breakpoint_enabled_result = active_session.set_breakpoint_enabled(&breakpoint_id, is_enabled);
            let _ = result_sender.send(set_breakpoint_enabled_result);
        }
        WindbgWorkerCommand::ListBreakpoints { result_sender } => {
            let list_breakpoints_result = active_session.list_breakpoints();
            let _ = result_sender.send(list_breakpoints_result);
        }
        WindbgWorkerCommand::Detach { result_sender } => {
            let detach_result = active_session.detach();
            let _ = result_sender.send(detach_result);

            return true;
        }
    }

    false
}

fn wait_for_required_debug_event(
    control: &IDebugControl,
    timeout_ms: u32,
    context: &str,
) -> Result<(), DebuggerPluginError> {
    if wait_for_optional_debug_event(control, timeout_ms, context)? {
        Ok(())
    } else {
        Err(WindbgBackend::plugin_error(format!(
            "IDebugControl::WaitForEvent timed out during {} after {} ms.",
            context, timeout_ms
        )))
    }
}

fn wait_for_optional_debug_event(
    control: &IDebugControl,
    timeout_ms: u32,
    context: &str,
) -> Result<bool, DebuggerPluginError> {
    let wait_result = unsafe { (Interface::vtable(control).WaitForEvent)(Interface::as_raw(control), 0, timeout_ms) };

    if wait_result == S_OK {
        Ok(true)
    } else if wait_result == S_FALSE {
        Ok(false)
    } else {
        Err(WindbgBackend::plugin_error(format!(
            "IDebugControl::WaitForEvent failed during {} with HRESULT 0x{:08X}.",
            context, wait_result.0 as u32
        )))
    }
}
