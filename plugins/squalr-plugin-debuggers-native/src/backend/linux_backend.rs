use crate::constants::NATIVE_DEBUGGERS_PLUGIN_ID;
use libc::{SIGSTOP, SIGTRAP, WNOHANG, c_void, pid_t};
use squalr_engine_api::{
    plugins::debugger::{DebuggerPluginError, DebuggerTraceEventSink},
    structures::{
        debugger::{
            DebuggerBreakpointDescriptor, DebuggerBreakpointKind, DebuggerDataBreakpointAccess, DebuggerRegisterSnapshot, DebuggerRegisterValue,
            DebuggerSessionState, DebuggerTraceEvent,
        },
        processes::{opened_process_info::OpenedProcessInfo, target_architecture::TargetArchitecture},
    },
};
use std::{
    collections::HashMap,
    mem::zeroed,
    ptr::null_mut,
    sync::mpsc::{self, Receiver, RecvTimeoutError, Sender},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

const ATTACH_WAIT_TIMEOUT: Duration = Duration::from_secs(10);
const IDLE_COMMAND_WAIT_TIMEOUT_MS: u64 = 50;
const RUNNING_EVENT_WAIT_TIMEOUT_MS: u64 = 50;
const TRACE_INSTRUCTION_BYTE_WINDOW: usize = 16;
const WATCHPOINT_SLOT_COUNT: usize = 4;
const X86_64_DEBUG_REGISTER_BASE_OFFSET: usize = 848;
const X86_64_DEBUG_REGISTER_SIZE: usize = 8;
const X86_64_DR6_OFFSET: usize = X86_64_DEBUG_REGISTER_BASE_OFFSET + 6 * X86_64_DEBUG_REGISTER_SIZE;

pub(crate) struct LinuxDebuggerBackend {
    process_info: OpenedProcessInfo,
    trace_event_sink: DebuggerTraceEventSink,
    worker_handle: Option<LinuxWorkerHandle>,
}

impl LinuxDebuggerBackend {
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
        let thread_handle = thread::spawn(move || linux_worker_main(process_info, trace_event_sink, worker_ready_sender));
        let worker_command_sender = match worker_ready_receiver
            .recv()
            .map_err(|error| Self::plugin_error(format!("Linux debugger worker exited before reporting attach status: {}", error)))?
        {
            Ok(worker_command_sender) => worker_command_sender,
            Err(error) => {
                let _ = thread_handle.join();

                return Err(error);
            }
        };

        self.worker_handle = Some(LinuxWorkerHandle {
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
            .ok_or_else(|| Self::plugin_error("Cannot pause because there is no active Linux debugger worker."))?
            .pause()
    }

    pub(crate) fn resume(&self) -> Result<(), DebuggerPluginError> {
        self.worker_handle
            .as_ref()
            .ok_or_else(|| Self::plugin_error("Cannot resume because there is no active Linux debugger worker."))?
            .resume()
    }

    pub(crate) fn read_registers(&self) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        self.worker_handle
            .as_ref()
            .ok_or_else(|| Self::plugin_error("Cannot read registers because there is no active Linux debugger worker."))?
            .read_registers()
    }

    pub(crate) fn write_register(
        &self,
        register_name: &str,
        value: u64,
    ) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        self.worker_handle
            .as_ref()
            .ok_or_else(|| Self::plugin_error("Cannot write registers because there is no active Linux debugger worker."))?
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
            .ok_or_else(|| Self::plugin_error("Cannot set a breakpoint because there is no active Linux debugger worker."))?
            .set_breakpoint(address, kind, label)
    }

    pub(crate) fn remove_breakpoint(
        &self,
        breakpoint_id: &str,
    ) -> Result<(), DebuggerPluginError> {
        self.worker_handle
            .as_ref()
            .ok_or_else(|| Self::plugin_error("Cannot remove a breakpoint because there is no active Linux debugger worker."))?
            .remove_breakpoint(breakpoint_id)
    }

    pub(crate) fn set_breakpoint_enabled(
        &self,
        breakpoint_id: &str,
        is_enabled: bool,
    ) -> Result<(), DebuggerPluginError> {
        self.worker_handle
            .as_ref()
            .ok_or_else(|| Self::plugin_error("Cannot update a breakpoint because there is no active Linux debugger worker."))?
            .set_breakpoint_enabled(breakpoint_id, is_enabled)
    }

    pub(crate) fn list_breakpoints(&self) -> Result<Vec<DebuggerBreakpointDescriptor>, DebuggerPluginError> {
        self.worker_handle
            .as_ref()
            .ok_or_else(|| Self::plugin_error("Cannot list breakpoints because there is no active Linux debugger worker."))?
            .list_breakpoints()
    }

    fn plugin_error(message: impl Into<String>) -> DebuggerPluginError {
        DebuggerPluginError::new(NATIVE_DEBUGGERS_PLUGIN_ID, message)
    }
}

impl Drop for LinuxDebuggerBackend {
    fn drop(&mut self) {
        if let Err(error) = self.detach() {
            log::debug!("Failed to detach Linux debugger backend during drop: {}", error);
        }
    }
}

struct LinuxWorkerHandle {
    command_sender: Sender<LinuxWorkerCommand>,
    thread_handle: Option<JoinHandle<()>>,
}

impl LinuxWorkerHandle {
    fn pause(&self) -> Result<(), DebuggerPluginError> {
        self.request_worker_result(LinuxWorkerCommandKind::Pause)
    }

    fn resume(&self) -> Result<(), DebuggerPluginError> {
        self.request_worker_result(LinuxWorkerCommandKind::Resume)
    }

    fn read_registers(&self) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        let (result_sender, result_receiver) = mpsc::channel();

        self.command_sender
            .send(LinuxWorkerCommand::ReadRegisters { result_sender })
            .map_err(|error| LinuxDebuggerBackend::plugin_error(format!("Failed to request Linux register snapshot: {}", error)))?;

        result_receiver
            .recv()
            .map_err(|error| LinuxDebuggerBackend::plugin_error(format!("Linux debugger worker exited before register snapshot completed: {}", error)))?
    }

    fn write_register(
        &self,
        register_name: &str,
        value: u64,
    ) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        let (result_sender, result_receiver) = mpsc::channel();

        self.command_sender
            .send(LinuxWorkerCommand::WriteRegister {
                register_name: register_name.to_string(),
                value,
                result_sender,
            })
            .map_err(|error| LinuxDebuggerBackend::plugin_error(format!("Failed to request Linux register write: {}", error)))?;

        result_receiver
            .recv()
            .map_err(|error| LinuxDebuggerBackend::plugin_error(format!("Linux debugger worker exited before register write completed: {}", error)))?
    }

    fn set_breakpoint(
        &self,
        address: u64,
        kind: DebuggerBreakpointKind,
        label: Option<String>,
    ) -> Result<DebuggerBreakpointDescriptor, DebuggerPluginError> {
        let (result_sender, result_receiver) = mpsc::channel();

        self.command_sender
            .send(LinuxWorkerCommand::SetBreakpoint {
                address,
                kind,
                label,
                result_sender,
            })
            .map_err(|error| LinuxDebuggerBackend::plugin_error(format!("Failed to request Linux breakpoint set: {}", error)))?;

        result_receiver
            .recv()
            .map_err(|error| LinuxDebuggerBackend::plugin_error(format!("Linux debugger worker exited before breakpoint set completed: {}", error)))?
    }

    fn remove_breakpoint(
        &self,
        breakpoint_id: &str,
    ) -> Result<(), DebuggerPluginError> {
        let (result_sender, result_receiver) = mpsc::channel();

        self.command_sender
            .send(LinuxWorkerCommand::RemoveBreakpoint {
                breakpoint_id: breakpoint_id.to_string(),
                result_sender,
            })
            .map_err(|error| LinuxDebuggerBackend::plugin_error(format!("Failed to request Linux breakpoint removal: {}", error)))?;

        result_receiver
            .recv()
            .map_err(|error| LinuxDebuggerBackend::plugin_error(format!("Linux debugger worker exited before breakpoint removal completed: {}", error)))?
    }

    fn set_breakpoint_enabled(
        &self,
        breakpoint_id: &str,
        is_enabled: bool,
    ) -> Result<(), DebuggerPluginError> {
        let (result_sender, result_receiver) = mpsc::channel();

        self.command_sender
            .send(LinuxWorkerCommand::SetBreakpointEnabled {
                breakpoint_id: breakpoint_id.to_string(),
                is_enabled,
                result_sender,
            })
            .map_err(|error| LinuxDebuggerBackend::plugin_error(format!("Failed to request Linux breakpoint state update: {}", error)))?;

        result_receiver
            .recv()
            .map_err(|error| LinuxDebuggerBackend::plugin_error(format!("Linux debugger worker exited before breakpoint state update completed: {}", error)))?
    }

    fn list_breakpoints(&self) -> Result<Vec<DebuggerBreakpointDescriptor>, DebuggerPluginError> {
        let (result_sender, result_receiver) = mpsc::channel();

        self.command_sender
            .send(LinuxWorkerCommand::ListBreakpoints { result_sender })
            .map_err(|error| LinuxDebuggerBackend::plugin_error(format!("Failed to request Linux breakpoint list: {}", error)))?;

        result_receiver
            .recv()
            .map_err(|error| LinuxDebuggerBackend::plugin_error(format!("Linux debugger worker exited before breakpoint list completed: {}", error)))?
    }

    fn detach(&mut self) -> Result<(), DebuggerPluginError> {
        let (result_sender, result_receiver) = mpsc::channel();

        self.command_sender
            .send(LinuxWorkerCommand::Detach { result_sender })
            .map_err(|error| LinuxDebuggerBackend::plugin_error(format!("Failed to request Linux detach: {}", error)))?;

        let detach_result = result_receiver
            .recv()
            .map_err(|error| LinuxDebuggerBackend::plugin_error(format!("Linux debugger worker exited before detach completed: {}", error)))?;

        if let Some(thread_handle) = self.thread_handle.take() {
            thread_handle
                .join()
                .map_err(|_| LinuxDebuggerBackend::plugin_error("Linux debugger worker thread panicked during detach."))?;
        }

        detach_result
    }

    fn request_worker_result(
        &self,
        command_kind: LinuxWorkerCommandKind,
    ) -> Result<(), DebuggerPluginError> {
        let (result_sender, result_receiver) = mpsc::channel();
        let worker_command = match command_kind {
            LinuxWorkerCommandKind::Pause => LinuxWorkerCommand::Pause { result_sender },
            LinuxWorkerCommandKind::Resume => LinuxWorkerCommand::Resume { result_sender },
        };

        self.command_sender
            .send(worker_command)
            .map_err(|error| LinuxDebuggerBackend::plugin_error(format!("Failed to send Linux debugger worker command: {}", error)))?;

        result_receiver
            .recv()
            .map_err(|error| LinuxDebuggerBackend::plugin_error(format!("Linux debugger worker exited before command completed: {}", error)))?
    }
}

enum LinuxWorkerCommand {
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

enum LinuxWorkerCommandKind {
    Pause,
    Resume,
}

#[derive(Clone)]
struct StoredLinuxBreakpoint {
    descriptor: DebuggerBreakpointDescriptor,
    slot: usize,
}

struct ActiveLinuxSession {
    process_id: pid_t,
    target_architecture: TargetArchitecture,
    breakpoints_by_id: HashMap<String, StoredLinuxBreakpoint>,
    session_state: DebuggerSessionState,
    next_breakpoint_number: u64,
    trace_event_sink: DebuggerTraceEventSink,
}

impl ActiveLinuxSession {
    fn attach(
        process_info: &OpenedProcessInfo,
        trace_event_sink: DebuggerTraceEventSink,
    ) -> Result<Self, DebuggerPluginError> {
        let process_id = process_info.get_process_id() as pid_t;

        if !matches!(process_info.get_target_architecture().get_instruction_set_id(), "x86" | "x64") {
            return Err(LinuxDebuggerBackend::plugin_error(format!(
                "Linux native debugger currently supports x86/x64 targets only; target architecture was '{}'.",
                process_info.get_target_architecture().get_instruction_set_id()
            )));
        }

        ptrace_request(libc::PTRACE_ATTACH, process_id, null_mut(), null_mut(), "PTRACE_ATTACH")?;
        if let Err(error) = wait_for_stop(process_id, ATTACH_WAIT_TIMEOUT, "initial Linux attach") {
            let _ = ptrace_request(libc::PTRACE_DETACH, process_id, null_mut(), null_mut(), "PTRACE_DETACH after failed attach");

            return Err(error);
        }

        Ok(Self {
            process_id,
            target_architecture: process_info.get_target_architecture().clone(),
            breakpoints_by_id: HashMap::new(),
            session_state: DebuggerSessionState::Paused,
            next_breakpoint_number: 1,
            trace_event_sink,
        })
    }

    fn pause(&mut self) -> Result<(), DebuggerPluginError> {
        if self.session_state == DebuggerSessionState::Paused {
            return Ok(());
        }

        let signal_result = unsafe { libc::kill(self.process_id, SIGSTOP) };
        if signal_result != 0 {
            return Err(last_os_error("SIGSTOP Linux debugger pause"));
        }

        wait_for_stop(self.process_id, ATTACH_WAIT_TIMEOUT, "Linux debugger pause")?;
        self.session_state = DebuggerSessionState::Paused;

        Ok(())
    }

    fn resume(&mut self) -> Result<(), DebuggerPluginError> {
        if self.session_state == DebuggerSessionState::Running {
            return Ok(());
        }

        self.apply_watchpoints()?;
        ptrace_request(libc::PTRACE_CONT, self.process_id, null_mut(), null_mut(), "PTRACE_CONT resume")?;
        self.session_state = DebuggerSessionState::Running;

        Ok(())
    }

    fn read_registers(&self) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        self.read_x64_registers()
    }

    fn write_register(
        &mut self,
        register_name: &str,
        value: u64,
    ) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        let should_resume = self.session_state == DebuggerSessionState::Running;

        if should_resume {
            self.pause()?;
        }

        let write_result = self.write_register_while_paused(register_name, value);
        let resume_result = if should_resume { self.resume() } else { Ok(()) };

        match (write_result, resume_result) {
            (Ok(register_snapshot), Ok(())) => Ok(register_snapshot),
            (Err(error), _) => Err(error),
            (Ok(_), Err(error)) => Err(LinuxDebuggerBackend::plugin_error(format!(
                "Linux debugger failed to resume after register write: {}.",
                error
            ))),
        }
    }

    fn write_register_while_paused(
        &mut self,
        register_name: &str,
        value: u64,
    ) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        let mut registers = get_registers(self.process_id)?;
        let normalized_register_name = register_name.trim().to_ascii_lowercase();

        match normalized_register_name.as_str() {
            "rip" | "eip" => registers.rip = value,
            "rsp" | "esp" => registers.rsp = value,
            "rax" | "eax" => registers.rax = value,
            "rbx" | "ebx" => registers.rbx = value,
            "rcx" | "ecx" => registers.rcx = value,
            "rdx" | "edx" => registers.rdx = value,
            "rsi" | "esi" => registers.rsi = value,
            "rdi" | "edi" => registers.rdi = value,
            "rbp" | "ebp" => registers.rbp = value,
            "r8" => registers.r8 = value,
            "r9" => registers.r9 = value,
            "r10" => registers.r10 = value,
            "r11" => registers.r11 = value,
            "r12" => registers.r12 = value,
            "r13" => registers.r13 = value,
            "r14" => registers.r14 = value,
            "r15" => registers.r15 = value,
            unsupported_register_name => {
                return Err(LinuxDebuggerBackend::plugin_error(format!(
                    "Linux x64 register '{}' is not supported for writes.",
                    unsupported_register_name
                )));
            }
        }

        ptrace_request(
            libc::PTRACE_SETREGS,
            self.process_id,
            null_mut(),
            (&mut registers as *mut libc::user_regs_struct).cast::<c_void>(),
            "PTRACE_SETREGS",
        )?;

        self.read_x64_registers()
    }

    fn set_breakpoint(
        &mut self,
        address: u64,
        kind: DebuggerBreakpointKind,
        label: Option<String>,
    ) -> Result<DebuggerBreakpointDescriptor, DebuggerPluginError> {
        self.run_paused_for_breakpoint_mutation("set breakpoint", |active_session| {
            active_session.set_breakpoint_while_paused(address, kind, label)
        })
    }

    fn set_breakpoint_while_paused(
        &mut self,
        address: u64,
        kind: DebuggerBreakpointKind,
        label: Option<String>,
    ) -> Result<DebuggerBreakpointDescriptor, DebuggerPluginError> {
        if !matches!(kind, DebuggerBreakpointKind::HardwareData { .. }) {
            return Err(LinuxDebuggerBackend::plugin_error(
                "The Linux native debugger backend currently supports hardware data breakpoints only.",
            ));
        }

        validate_hardware_breakpoint(address, &kind)?;
        let slot = self.allocate_watchpoint_slot()?;
        let breakpoint_id = format!("linux-{}", self.next_breakpoint_number);
        self.next_breakpoint_number = self.next_breakpoint_number.saturating_add(1);
        let descriptor = DebuggerBreakpointDescriptor::new(breakpoint_id.clone(), address, kind, true, label);

        self.breakpoints_by_id.insert(
            breakpoint_id.clone(),
            StoredLinuxBreakpoint {
                descriptor: descriptor.clone(),
                slot,
            },
        );

        if let Err(error) = self.apply_watchpoints() {
            self.breakpoints_by_id.remove(&breakpoint_id);
            let _ = self.apply_watchpoints();

            return Err(error);
        }

        Ok(descriptor)
    }

    fn remove_breakpoint(
        &mut self,
        breakpoint_id: &str,
    ) -> Result<(), DebuggerPluginError> {
        self.run_paused_for_breakpoint_mutation("remove breakpoint", |active_session| {
            if active_session.breakpoints_by_id.remove(breakpoint_id).is_none() {
                return Err(LinuxDebuggerBackend::plugin_error(format!(
                    "Linux breakpoint '{}' does not exist.",
                    breakpoint_id
                )));
            }

            active_session.apply_watchpoints()
        })
    }

    fn set_breakpoint_enabled(
        &mut self,
        breakpoint_id: &str,
        is_enabled: bool,
    ) -> Result<(), DebuggerPluginError> {
        self.run_paused_for_breakpoint_mutation("set breakpoint enabled", |active_session| {
            let stored_breakpoint = active_session
                .breakpoints_by_id
                .get_mut(breakpoint_id)
                .ok_or_else(|| LinuxDebuggerBackend::plugin_error(format!("Linux breakpoint '{}' does not exist.", breakpoint_id)))?;

            stored_breakpoint.descriptor.set_is_enabled(is_enabled);
            active_session.apply_watchpoints()
        })
    }

    fn list_breakpoints(&self) -> Result<Vec<DebuggerBreakpointDescriptor>, DebuggerPluginError> {
        let mut breakpoints = self
            .breakpoints_by_id
            .values()
            .map(|stored_breakpoint| stored_breakpoint.descriptor.clone())
            .collect::<Vec<_>>();

        breakpoints.sort_by(|left, right| {
            left.get_address()
                .cmp(&right.get_address())
                .then_with(|| left.get_breakpoint_id().cmp(right.get_breakpoint_id()))
        });

        Ok(breakpoints)
    }

    fn detach(&mut self) -> Result<(), DebuggerPluginError> {
        if self.session_state == DebuggerSessionState::Running {
            self.pause()?;
        }

        self.breakpoints_by_id.clear();
        let clear_result = self.apply_watchpoints();
        let detach_result = ptrace_request(libc::PTRACE_DETACH, self.process_id, null_mut(), null_mut(), "PTRACE_DETACH");

        clear_result?;
        detach_result?;
        self.session_state = DebuggerSessionState::Detached;

        Ok(())
    }

    fn process_pending_debug_event(&mut self) -> Result<(), DebuggerPluginError> {
        if self.session_state != DebuggerSessionState::Running {
            return Ok(());
        }

        let mut wait_status = 0;
        let wait_result = unsafe { libc::waitpid(self.process_id, &mut wait_status, WNOHANG) };

        if wait_result == 0 {
            thread::sleep(Duration::from_millis(RUNNING_EVENT_WAIT_TIMEOUT_MS));
            return Ok(());
        }

        if wait_result < 0 {
            return Err(last_os_error("waitpid while polling Linux debug events"));
        }

        if libc::WIFEXITED(wait_status) || libc::WIFSIGNALED(wait_status) {
            self.session_state = DebuggerSessionState::Detached;
            return Ok(());
        }

        if libc::WIFSTOPPED(wait_status) {
            let stop_signal = libc::WSTOPSIG(wait_status);
            self.session_state = DebuggerSessionState::Paused;

            if stop_signal == SIGTRAP {
                self.handle_breakpoint_trap()?;
            }

            if self.session_state != DebuggerSessionState::Detached {
                self.apply_watchpoints()?;
                ptrace_request(libc::PTRACE_CONT, self.process_id, null_mut(), null_mut(), "PTRACE_CONT after debug event")?;
                self.session_state = DebuggerSessionState::Running;
            }
        }

        Ok(())
    }

    fn handle_breakpoint_trap(&mut self) -> Result<(), DebuggerPluginError> {
        let dr6 = read_debug_register(self.process_id, 6)?;
        let breakpoint_descriptor = self.describe_hit_breakpoint(dr6);
        let register_snapshot = self.read_registers()?;
        let instruction_address = register_snapshot.get_instruction_pointer();
        let instruction_bytes = self.read_instruction_bytes(instruction_address);
        let backend_message = Some(String::from(
            "Linux x64 hardware data breakpoint hit; instruction pointer is the post-trap RIP and may point after the accessing instruction.",
        ));

        clear_debug_status(self.process_id)?;

        if breakpoint_descriptor.is_some() {
            let trace_event = DebuggerTraceEvent::new(
                breakpoint_descriptor,
                register_snapshot,
                instruction_address,
                instruction_bytes,
                None,
                backend_message,
            )
            .with_target_architecture(self.target_architecture.clone());

            (self.trace_event_sink)(trace_event);
        }

        Ok(())
    }

    fn describe_hit_breakpoint(
        &self,
        dr6: u64,
    ) -> Option<DebuggerBreakpointDescriptor> {
        self.breakpoints_by_id
            .values()
            .find(|stored_breakpoint| dr6 & (1u64 << stored_breakpoint.slot) != 0)
            .map(|stored_breakpoint| stored_breakpoint.descriptor.clone())
    }

    fn read_instruction_bytes(
        &self,
        instruction_pointer: Option<u64>,
    ) -> Vec<u8> {
        let Some(instruction_pointer) = instruction_pointer else {
            return Vec::new();
        };
        let mut instruction_bytes = Vec::with_capacity(TRACE_INSTRUCTION_BYTE_WINDOW);

        while instruction_bytes.len() < TRACE_INSTRUCTION_BYTE_WINDOW {
            let read_address = instruction_pointer.saturating_add(instruction_bytes.len() as u64);
            let word = match ptrace_peek_data(self.process_id, read_address) {
                Ok(word) => word,
                Err(error) => {
                    log::debug!("Failed to read Linux instruction bytes at 0x{:X}: {}", read_address, error);
                    break;
                }
            };
            let word_bytes = word.to_ne_bytes();
            let remaining_byte_count = TRACE_INSTRUCTION_BYTE_WINDOW - instruction_bytes.len();
            let copied_byte_count = remaining_byte_count.min(word_bytes.len());

            instruction_bytes.extend_from_slice(&word_bytes[..copied_byte_count]);
        }

        instruction_bytes
    }

    fn apply_watchpoints(&self) -> Result<(), DebuggerPluginError> {
        let mut dr7 = 0u64;

        clear_debug_status(self.process_id)?;
        for slot in 0..WATCHPOINT_SLOT_COUNT {
            write_debug_register(self.process_id, slot, 0)?;
        }

        for stored_breakpoint in self.breakpoints_by_id.values() {
            if !stored_breakpoint.descriptor.get_is_enabled() {
                continue;
            }

            let DebuggerBreakpointKind::HardwareData { access, size_in_bytes } = *stored_breakpoint.descriptor.get_kind() else {
                continue;
            };
            let slot = stored_breakpoint.slot;
            let length_bits = x64_watchpoint_length_bits(size_in_bytes)?;
            let access_bits = x64_watchpoint_access_bits(access);

            write_debug_register(self.process_id, slot, stored_breakpoint.descriptor.get_address())?;
            dr7 |= 1u64 << (slot * 2);
            dr7 |= access_bits << (16 + slot * 4);
            dr7 |= length_bits << (18 + slot * 4);
        }

        write_debug_register(self.process_id, 7, dr7)
    }

    fn allocate_watchpoint_slot(&self) -> Result<usize, DebuggerPluginError> {
        for slot in 0..WATCHPOINT_SLOT_COUNT {
            let is_slot_available = self
                .breakpoints_by_id
                .values()
                .all(|stored_breakpoint| stored_breakpoint.slot != slot);

            if is_slot_available {
                return Ok(slot);
            }
        }

        Err(LinuxDebuggerBackend::plugin_error(format!(
            "No Linux x64 hardware watchpoint slots are available. This backend currently uses {} slots.",
            WATCHPOINT_SLOT_COUNT
        )))
    }

    fn run_paused_for_breakpoint_mutation<T, F>(
        &mut self,
        operation_name: &str,
        mutation: F,
    ) -> Result<T, DebuggerPluginError>
    where
        F: FnOnce(&mut Self) -> Result<T, DebuggerPluginError>,
    {
        let should_resume = self.session_state == DebuggerSessionState::Running;

        if should_resume {
            self.pause()?;
        }

        let mutation_result = mutation(self);
        let resume_result = if should_resume { self.resume() } else { Ok(()) };

        match (mutation_result, resume_result) {
            (Ok(value), Ok(())) => Ok(value),
            (Err(error), _) => Err(error),
            (Ok(_), Err(error)) => Err(LinuxDebuggerBackend::plugin_error(format!(
                "Linux debugger failed to resume after {}: {}.",
                operation_name, error
            ))),
        }
    }

    fn read_x64_registers(&self) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        let registers = get_registers(self.process_id)?;
        let register_values = vec![
            DebuggerRegisterValue::new("rax", registers.rax, 64),
            DebuggerRegisterValue::new("rbx", registers.rbx, 64),
            DebuggerRegisterValue::new("rcx", registers.rcx, 64),
            DebuggerRegisterValue::new("rdx", registers.rdx, 64),
            DebuggerRegisterValue::new("rsi", registers.rsi, 64),
            DebuggerRegisterValue::new("rdi", registers.rdi, 64),
            DebuggerRegisterValue::new("rbp", registers.rbp, 64),
            DebuggerRegisterValue::new("rsp", registers.rsp, 64),
            DebuggerRegisterValue::new("rip", registers.rip, 64),
            DebuggerRegisterValue::new("r8", registers.r8, 64),
            DebuggerRegisterValue::new("r9", registers.r9, 64),
            DebuggerRegisterValue::new("r10", registers.r10, 64),
            DebuggerRegisterValue::new("r11", registers.r11, 64),
            DebuggerRegisterValue::new("r12", registers.r12, 64),
            DebuggerRegisterValue::new("r13", registers.r13, 64),
            DebuggerRegisterValue::new("r14", registers.r14, 64),
            DebuggerRegisterValue::new("r15", registers.r15, 64),
            DebuggerRegisterValue::new("eflags", registers.eflags, 64),
        ];

        Ok(DebuggerRegisterSnapshot::new(Some(registers.rip), Some(registers.rsp), register_values))
    }
}

fn linux_worker_main(
    process_info: OpenedProcessInfo,
    trace_event_sink: DebuggerTraceEventSink,
    worker_ready_sender: Sender<Result<Sender<LinuxWorkerCommand>, DebuggerPluginError>>,
) {
    let active_session = match ActiveLinuxSession::attach(&process_info, trace_event_sink) {
        Ok(active_session) => active_session,
        Err(error) => {
            let _ = worker_ready_sender.send(Err(error));
            return;
        }
    };
    let (worker_command_sender, worker_command_receiver) = mpsc::channel();

    if worker_ready_sender.send(Ok(worker_command_sender)).is_err() {
        let mut active_session = active_session;
        let _ = active_session.detach();
        return;
    }

    wait_for_worker_commands(active_session, worker_command_receiver);
}

fn wait_for_worker_commands(
    mut active_session: ActiveLinuxSession,
    worker_command_receiver: Receiver<LinuxWorkerCommand>,
) {
    loop {
        if let Err(error) = active_session.process_pending_debug_event() {
            log::debug!("Failed to process pending Linux debugger event before worker command poll: {}", error);
        }

        match worker_command_receiver.recv_timeout(Duration::from_millis(IDLE_COMMAND_WAIT_TIMEOUT_MS)) {
            Ok(worker_command) => {
                if handle_worker_command(&mut active_session, worker_command) {
                    return;
                }
            }
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }

    let _ = active_session.detach();
}

fn handle_worker_command(
    active_session: &mut ActiveLinuxSession,
    worker_command: LinuxWorkerCommand,
) -> bool {
    match worker_command {
        LinuxWorkerCommand::Pause { result_sender } => {
            let _ = result_sender.send(active_session.pause());
        }
        LinuxWorkerCommand::Resume { result_sender } => {
            let _ = result_sender.send(active_session.resume());
        }
        LinuxWorkerCommand::ReadRegisters { result_sender } => {
            let _ = result_sender.send(active_session.read_registers());
        }
        LinuxWorkerCommand::WriteRegister {
            register_name,
            value,
            result_sender,
        } => {
            let _ = result_sender.send(active_session.write_register(&register_name, value));
        }
        LinuxWorkerCommand::SetBreakpoint {
            address,
            kind,
            label,
            result_sender,
        } => {
            let _ = result_sender.send(active_session.set_breakpoint(address, kind, label));
        }
        LinuxWorkerCommand::RemoveBreakpoint { breakpoint_id, result_sender } => {
            let _ = result_sender.send(active_session.remove_breakpoint(&breakpoint_id));
        }
        LinuxWorkerCommand::SetBreakpointEnabled {
            breakpoint_id,
            is_enabled,
            result_sender,
        } => {
            let _ = result_sender.send(active_session.set_breakpoint_enabled(&breakpoint_id, is_enabled));
        }
        LinuxWorkerCommand::ListBreakpoints { result_sender } => {
            let _ = result_sender.send(active_session.list_breakpoints());
        }
        LinuxWorkerCommand::Detach { result_sender } => {
            let _ = result_sender.send(active_session.detach());
            return true;
        }
    }

    false
}

fn get_registers(process_id: pid_t) -> Result<libc::user_regs_struct, DebuggerPluginError> {
    let mut registers = unsafe { zeroed::<libc::user_regs_struct>() };

    ptrace_request(
        libc::PTRACE_GETREGS,
        process_id,
        null_mut(),
        (&mut registers as *mut libc::user_regs_struct).cast::<c_void>(),
        "PTRACE_GETREGS",
    )?;

    Ok(registers)
}

fn validate_hardware_breakpoint(
    address: u64,
    kind: &DebuggerBreakpointKind,
) -> Result<(), DebuggerPluginError> {
    let DebuggerBreakpointKind::HardwareData { size_in_bytes, .. } = *kind else {
        return Ok(());
    };

    x64_watchpoint_length_bits(size_in_bytes)?;

    if address % u64::from(size_in_bytes) != 0 {
        return Err(LinuxDebuggerBackend::plugin_error(format!(
            "Linux x64 hardware data breakpoint at 0x{:X} must be aligned to its {} byte size.",
            address, size_in_bytes
        )));
    }

    Ok(())
}

fn clear_debug_status(process_id: pid_t) -> Result<(), DebuggerPluginError> {
    write_debug_user_offset(process_id, X86_64_DR6_OFFSET, 0)
}

fn read_debug_register(
    process_id: pid_t,
    debug_register_index: usize,
) -> Result<u64, DebuggerPluginError> {
    let offset = X86_64_DEBUG_REGISTER_BASE_OFFSET + debug_register_index * X86_64_DEBUG_REGISTER_SIZE;
    read_debug_user_offset(process_id, offset)
}

fn write_debug_register(
    process_id: pid_t,
    debug_register_index: usize,
    value: u64,
) -> Result<(), DebuggerPluginError> {
    let offset = X86_64_DEBUG_REGISTER_BASE_OFFSET + debug_register_index * X86_64_DEBUG_REGISTER_SIZE;
    write_debug_user_offset(process_id, offset, value)
}

fn read_debug_user_offset(
    process_id: pid_t,
    offset: usize,
) -> Result<u64, DebuggerPluginError> {
    let ptrace_result = ptrace_peek_user(process_id, offset)?;

    Ok(ptrace_result as u64)
}

fn write_debug_user_offset(
    process_id: pid_t,
    offset: usize,
    value: u64,
) -> Result<(), DebuggerPluginError> {
    ptrace_request(
        libc::PTRACE_POKEUSER,
        process_id,
        offset as *mut c_void,
        value as usize as *mut c_void,
        "PTRACE_POKEUSER debug register",
    )
}

fn ptrace_peek_user(
    process_id: pid_t,
    offset: usize,
) -> Result<libc::c_long, DebuggerPluginError> {
    clear_errno();
    let ptrace_result = unsafe { libc::ptrace(libc::PTRACE_PEEKUSER, process_id, offset as *mut c_void, null_mut::<c_void>()) };
    let errno = current_errno();

    if ptrace_result == -1 && errno != 0 {
        Err(last_os_error("PTRACE_PEEKUSER"))
    } else {
        Ok(ptrace_result)
    }
}

fn ptrace_peek_data(
    process_id: pid_t,
    address: u64,
) -> Result<usize, DebuggerPluginError> {
    clear_errno();
    let ptrace_result = unsafe { libc::ptrace(libc::PTRACE_PEEKDATA, process_id, address as *mut c_void, null_mut::<c_void>()) };
    let errno = current_errno();

    if ptrace_result == -1 && errno != 0 {
        Err(last_os_error("PTRACE_PEEKDATA"))
    } else {
        Ok(ptrace_result as usize)
    }
}

fn ptrace_request(
    request: libc::c_uint,
    process_id: pid_t,
    address: *mut c_void,
    data: *mut c_void,
    operation_name: &str,
) -> Result<(), DebuggerPluginError> {
    let ptrace_result = unsafe { libc::ptrace(request, process_id, address, data) };

    if ptrace_result == 0 { Ok(()) } else { Err(last_os_error(operation_name)) }
}

fn wait_for_stop(
    process_id: pid_t,
    timeout: Duration,
    operation_name: &str,
) -> Result<(), DebuggerPluginError> {
    let wait_started_at = Instant::now();

    loop {
        let mut wait_status = 0;
        let wait_result = unsafe { libc::waitpid(process_id, &mut wait_status, WNOHANG) };

        if wait_result < 0 {
            return Err(last_os_error(operation_name));
        }

        if wait_result > 0 && libc::WIFSTOPPED(wait_status) {
            return Ok(());
        }

        if wait_started_at.elapsed() >= timeout {
            return Err(LinuxDebuggerBackend::plugin_error(format!(
                "Timed out waiting for Linux debug stop during {}.",
                operation_name
            )));
        }

        thread::sleep(Duration::from_millis(25));
    }
}

fn last_os_error(operation_name: &str) -> DebuggerPluginError {
    LinuxDebuggerBackend::plugin_error(format!("{} failed: {}.", operation_name, std::io::Error::last_os_error()))
}

fn clear_errno() {
    unsafe {
        *libc::__errno_location() = 0;
    }
}

fn current_errno() -> i32 {
    unsafe { *libc::__errno_location() }
}

fn x64_watchpoint_access_bits(access: DebuggerDataBreakpointAccess) -> u64 {
    match access {
        DebuggerDataBreakpointAccess::Write => 0b01,
        DebuggerDataBreakpointAccess::Read | DebuggerDataBreakpointAccess::ReadWrite => 0b11,
    }
}

fn x64_watchpoint_length_bits(size_in_bytes: u8) -> Result<u64, DebuggerPluginError> {
    match size_in_bytes {
        1 => Ok(0b00),
        2 => Ok(0b01),
        4 => Ok(0b11),
        8 => Ok(0b10),
        _ => Err(LinuxDebuggerBackend::plugin_error(format!(
            "Linux x64 hardware data breakpoint size {} is unsupported. Expected 1, 2, 4, or 8 bytes.",
            size_in_bytes
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::{x64_watchpoint_access_bits, x64_watchpoint_length_bits};
    use squalr_engine_api::structures::debugger::DebuggerDataBreakpointAccess;

    #[test]
    fn x64_watchpoint_lengths_match_debug_register_encoding() {
        assert_eq!(x64_watchpoint_length_bits(1).unwrap_or(u64::MAX), 0b00);
        assert_eq!(x64_watchpoint_length_bits(2).unwrap_or(u64::MAX), 0b01);
        assert_eq!(x64_watchpoint_length_bits(4).unwrap_or(u64::MAX), 0b11);
        assert_eq!(x64_watchpoint_length_bits(8).unwrap_or(u64::MAX), 0b10);
        assert!(x64_watchpoint_length_bits(3).is_err());
    }

    #[test]
    fn x64_watchpoint_accesses_match_debug_register_encoding() {
        assert_eq!(x64_watchpoint_access_bits(DebuggerDataBreakpointAccess::Write), 0b01);
        assert_eq!(x64_watchpoint_access_bits(DebuggerDataBreakpointAccess::Read), 0b11);
        assert_eq!(x64_watchpoint_access_bits(DebuggerDataBreakpointAccess::ReadWrite), 0b11);
    }
}
