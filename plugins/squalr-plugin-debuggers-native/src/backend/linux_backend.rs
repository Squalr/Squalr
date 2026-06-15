use crate::constants::NATIVE_DEBUGGERS_PLUGIN_ID;
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
use iced_x86::{Decoder, DecoderOptions};
use libc::{SIGTRAP, WNOHANG, c_void, pid_t};
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
    fs,
    mem::zeroed,
    ptr::null_mut,
    sync::mpsc::{self, Receiver, RecvTimeoutError, Sender},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

const ATTACH_WAIT_TIMEOUT: Duration = Duration::from_secs(10);
const IDLE_COMMAND_WAIT_TIMEOUT_MS: u64 = 50;
const WATCHPOINT_REARM_POLL_TIMEOUT_MS: u64 = 2;
// How long a hit thread runs with its watchpoint disabled before we interrupt it to re-arm. Long enough for the
// accessing instruction (including an LDXR/STXR atomic sequence) to retire so re-arming does not livelock the atomic.
const WATCHPOINT_REARM_DELAY_MS: u64 = 30;
const RESUME_STOP_DRAIN_TIMEOUT_MS: u64 = 250;
const MAX_DEBUG_EVENTS_PER_DRAIN: usize = 4096;
#[cfg(all(target_os = "android", target_arch = "aarch64"))]
const ARM64_INSTRUCTION_BYTE_LENGTH: usize = 4;
#[cfg(any(test, all(target_os = "android", target_arch = "aarch64")))]
const ARM64_WATCH_GRANULE_SIZE: u64 = 8;
#[cfg(all(target_os = "android", target_arch = "aarch64"))]
const NT_ARM_HW_WATCH: usize = 0x403;
#[cfg(all(target_os = "android", target_arch = "aarch64"))]
const NT_ARM_HW_BREAK: usize = 0x402;
#[cfg(all(target_os = "android", target_arch = "aarch64"))]
const NT_PRSTATUS: usize = 1;
const TRACE_INSTRUCTION_BYTE_WINDOW: usize = 16;
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
const X64_MAX_INSTRUCTION_LENGTH: usize = 15;
const WATCHPOINT_SLOT_COUNT: usize = 4;
const WAIT_ALL_TRACED_THREADS: libc::c_int = WNOHANG | 0x40000000;
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
const X86_64_DEBUG_REGISTER_BASE_OFFSET: usize = 848;
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
const X86_64_DEBUG_REGISTER_SIZE: usize = 8;
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
const X86_64_DR6_OFFSET: usize = X86_64_DEBUG_REGISTER_BASE_OFFSET + 6 * X86_64_DEBUG_REGISTER_SIZE;

#[cfg(all(target_os = "android", target_arch = "aarch64"))]
type PtraceRequest = libc::c_int;
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
type PtraceRequest = libc::c_uint;

// PTRACE_SEIZE attaches a tracee without injecting a SIGSTOP. Unlike PTRACE_ATTACH, it does not group-stop the whole
// thread group, so resuming a heavily-multithreaded target (hundreds of threads) does not strand threads in ptrace_stop.
// These are not exposed by the libc crate for all targets, so define them from the kernel ABI (uapi/linux/ptrace.h).
const PTRACE_SEIZE_REQUEST: PtraceRequest = 0x4206;
const PTRACE_INTERRUPT_REQUEST: PtraceRequest = 0x4207;
// PTRACE_O_TRACECLONE: auto-attach threads cloned by a SEIZEd tracee, so new threads are watched without re-attaching
// (which would re-deliver SIGSTOP and re-trigger the group-stop deadlock).
const PTRACE_CLONE_TRACE_OPTION: libc::c_long = 0x0000_0008;

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

#[derive(Clone, Copy)]
struct TracedLinuxThread {
    thread_id: pid_t,
    is_stopped: bool,
}

struct ActiveLinuxSession {
    process_id: pid_t,
    target_architecture: TargetArchitecture,
    traced_threads_by_id: HashMap<pid_t, TracedLinuxThread>,
    breakpoints_by_id: HashMap<String, StoredLinuxBreakpoint>,
    session_state: DebuggerSessionState,
    next_breakpoint_number: u64,
    trace_event_sink: DebuggerTraceEventSink,
    // Threads whose watchpoint we disabled right after a hit (so they run past the accessing instruction without
    // immediately re-trapping), mapped to when they should be re-armed. We re-arm by interrupting them only after a
    // delay long enough for the access to retire: on arm64 the watched location is often accessed via an LDXR/STXR
    // exclusive pair, and stopping the thread between LDXR and STXR clears the monitor and livelocks the atomic. The
    // delay lets the atomic complete first, then a proactive interrupt re-arms reliably (unlike waiting for an
    // incidental stop, which left hot watchpoints silent after the first hit).
    rearm_at_by_thread: HashMap<pid_t, Instant>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AttachThreadResult {
    Attached,
    SkippedStaleThread,
}

impl ActiveLinuxSession {
    fn attach(
        process_info: &OpenedProcessInfo,
        trace_event_sink: DebuggerTraceEventSink,
    ) -> Result<Self, DebuggerPluginError> {
        let process_id = process_info.get_process_id() as pid_t;

        if !Self::is_supported_target_architecture(process_info.get_target_architecture()) {
            return Err(LinuxDebuggerBackend::plugin_error(format!(
                "Native ptrace debugger does not support target architecture '{}'.",
                process_info.get_target_architecture().get_instruction_set_id()
            )));
        }

        let mut active_session = Self {
            process_id,
            target_architecture: process_info.get_target_architecture().clone(),
            traced_threads_by_id: HashMap::new(),
            breakpoints_by_id: HashMap::new(),
            session_state: DebuggerSessionState::Paused,
            next_breakpoint_number: 1,
            trace_event_sink,
            rearm_at_by_thread: HashMap::new(),
        };

        // Android (and recent Linux) freeze backgrounded apps with the cgroup v2 freezer. A frozen task never processes
        // the SIGSTOP that PTRACE_ATTACH delivers, so the attach wait times out. Thaw the target before attaching.
        thaw_process_cgroup_freezer(process_id);

        if let Err(error) = active_session.attach_existing_threads() {
            let _ = active_session.detach();

            return Err(error);
        }

        if active_session.traced_threads_by_id.is_empty() {
            return Err(LinuxDebuggerBackend::plugin_error(format!(
                "Native ptrace debugger found no traceable threads for process {}.",
                process_id
            )));
        }

        Ok(active_session)
    }

    fn is_supported_target_architecture(target_architecture: &TargetArchitecture) -> bool {
        let instruction_set_id = target_architecture.get_instruction_set_id();

        (cfg!(all(target_os = "linux", target_arch = "x86_64")) && matches!(instruction_set_id, "x86" | "x64"))
            || (cfg!(all(target_os = "android", target_arch = "aarch64")) && instruction_set_id == "arm64")
    }

    fn pause(&mut self) -> Result<(), DebuggerPluginError> {
        if self.session_state == DebuggerSessionState::Paused {
            return Ok(());
        }

        self.sync_thread_list()?;
        let running_thread_ids = self
            .traced_threads_by_id
            .values()
            .filter(|traced_thread| !traced_thread.is_stopped)
            .map(|traced_thread| traced_thread.thread_id)
            .collect::<Vec<_>>();

        for thread_id in &running_thread_ids {
            ptrace_interrupt(*thread_id, "PTRACE_INTERRUPT Linux debugger pause")?;
        }

        for thread_id in running_thread_ids {
            wait_for_stop(thread_id, ATTACH_WAIT_TIMEOUT, "Linux debugger pause")?;
            if let Some(traced_thread) = self.traced_threads_by_id.get_mut(&thread_id) {
                traced_thread.is_stopped = true;
            }
        }

        self.session_state = DebuggerSessionState::Paused;

        Ok(())
    }

    fn resume(&mut self) -> Result<(), DebuggerPluginError> {
        let has_stopped_threads = self
            .traced_threads_by_id
            .values()
            .any(|traced_thread| traced_thread.is_stopped);

        if self.session_state == DebuggerSessionState::Running && !has_stopped_threads {
            return Ok(());
        }

        self.sync_thread_list()?;
        self.apply_watchpoints()?;
        self.resume_stopped_threads("PTRACE_CONT resume")?;
        self.session_state = DebuggerSessionState::Running;
        self.drain_pending_stops_after_resume()?;

        Ok(())
    }

    fn read_registers(&self) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        self.read_registers_for_thread(self.select_register_thread_id()?)
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
        let register_thread_id = self.select_register_thread_id()?;

        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        if self.target_architecture.get_instruction_set_id() == "x64" {
            return self.write_x64_register(register_thread_id, register_name, value);
        }

        #[cfg(all(target_os = "android", target_arch = "aarch64"))]
        if self.target_architecture.get_instruction_set_id() == "arm64" {
            return self.write_arm64_register(register_thread_id, register_name, value);
        }

        Err(LinuxDebuggerBackend::plugin_error(format!(
            "Native ptrace debugger register writes are unsupported for architecture '{}'.",
            self.target_architecture.get_instruction_set_id()
        )))
    }

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    fn write_x64_register(
        &self,
        register_thread_id: pid_t,
        register_name: &str,
        value: u64,
    ) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        let mut registers = get_registers(register_thread_id)?;
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
            register_thread_id,
            null_mut(),
            (&mut registers as *mut libc::user_regs_struct).cast::<c_void>(),
            "PTRACE_SETREGS",
        )?;

        self.read_x64_registers(register_thread_id)
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
        if !matches!(kind, DebuggerBreakpointKind::HardwareData { .. } | DebuggerBreakpointKind::HardwareExecute) {
            return Err(LinuxDebuggerBackend::plugin_error(
                "The native ptrace debugger backend currently supports hardware data and hardware execute breakpoints only.",
            ));
        }

        self.validate_hardware_breakpoint(address, &kind)?;
        let slot = self.allocate_watchpoint_slot()?;
        let breakpoint_id = format!("ptrace-{}", self.next_breakpoint_number);
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
        let thread_ids = self.traced_threads_by_id.keys().copied().collect::<Vec<_>>();
        let mut detach_result = Ok(());

        for thread_id in thread_ids {
            if let Err(error) = ptrace_request(libc::PTRACE_DETACH, thread_id, null_mut(), null_mut(), "PTRACE_DETACH") {
                if detach_result.is_ok() {
                    detach_result = Err(error);
                }
            }
        }

        clear_result?;
        detach_result?;
        self.traced_threads_by_id.clear();
        self.session_state = DebuggerSessionState::Detached;

        Ok(())
    }

    fn process_pending_debug_event(&mut self) -> Result<(), DebuggerPluginError> {
        if self.session_state != DebuggerSessionState::Running {
            return Ok(());
        }

        // Keep the target thawed: a backgrounded app can be re-frozen mid-session by the cgroup v2 freezer, which would
        // wedge ptrace stops and strand stopped threads holding runtime locks.
        thaw_process_cgroup_freezer(self.process_id);
        self.sync_thread_list()?;

        // Drain every stop that is already pending this tick. Handling one event per idle cycle let watchpoint-stopped
        // threads pile up faster than they were continued; while they sit in tracing-stop they hold runtime locks and
        // deadlock the whole target. The per-tick cap keeps the worker responsive to pause/stop/detach commands.
        let mut drained_event_count = 0;

        while self.session_state == DebuggerSessionState::Running && self.try_process_one_debug_event()? {
            drained_event_count += 1;
            if drained_event_count >= MAX_DEBUG_EVENTS_PER_DRAIN {
                break;
            }
        }

        // Re-arm threads whose post-hit disable window has elapsed. They have now run past the access (and finished any
        // LDXR/STXR atomic), so interrupting them to re-arm is safe and reliable.
        self.interrupt_due_rearm_threads();

        Ok(())
    }

    /// Returns whether any thread is awaiting re-arm after a hit, so the worker loop can poll quickly (keeping the
    /// disabled window close to the configured delay) instead of idling for the full command timeout.
    fn has_pending_rearms(&self) -> bool {
        !self.rearm_at_by_thread.is_empty()
    }

    fn interrupt_due_rearm_threads(&mut self) {
        if self.rearm_at_by_thread.is_empty() {
            return;
        }

        let now = Instant::now();

        for (thread_id, rearm_at) in self
            .rearm_at_by_thread
            .iter()
            .map(|(thread_id, rearm_at)| (*thread_id, *rearm_at))
            .collect::<Vec<_>>()
        {
            if !self.traced_threads_by_id.contains_key(&thread_id) {
                self.rearm_at_by_thread.remove(&thread_id);
                continue;
            }

            if rearm_at > now {
                continue;
            }

            // Interrupt the (now safely-past-the-access) thread so its stop is reaped by the event loop, which re-arms
            // its watchpoint. Interrupt errors (e.g. it is already stopping) are non-fatal; the stop handler re-arms it.
            let _ = ptrace_interrupt(thread_id, "PTRACE_INTERRUPT to re-arm watchpoint");
        }
    }

    fn try_process_one_debug_event(&mut self) -> Result<bool, DebuggerPluginError> {
        let mut wait_status = 0;
        let wait_result = unsafe { libc::waitpid(-1, &mut wait_status, WAIT_ALL_TRACED_THREADS) };

        if wait_result == 0 {
            return Ok(false);
        }

        if wait_result < 0 {
            return Err(last_os_error("waitpid while polling Linux debug events"));
        }

        self.handle_wait_status(wait_result, wait_status)?;

        Ok(true)
    }

    fn handle_wait_status(
        &mut self,
        event_thread_id: pid_t,
        wait_status: libc::c_int,
    ) -> Result<(), DebuggerPluginError> {
        if libc::WIFEXITED(wait_status) || libc::WIFSIGNALED(wait_status) {
            self.traced_threads_by_id.remove(&event_thread_id);
            self.rearm_at_by_thread.remove(&event_thread_id);
            if self.traced_threads_by_id.is_empty() {
                self.session_state = DebuggerSessionState::Detached;
            }

            return Ok(());
        }

        if !libc::WIFSTOPPED(wait_status) {
            return Ok(());
        }

        let stop_signal = libc::WSTOPSIG(wait_status);
        let ptrace_event_code = (wait_status >> 16) & 0xff;
        let is_known_thread = self.traced_threads_by_id.contains_key(&event_thread_id);

        if is_known_thread {
            if let Some(traced_thread) = self.traced_threads_by_id.get_mut(&event_thread_id) {
                traced_thread.is_stopped = true;
            }
        } else {
            // A thread cloned by a SEIZEd tracee is auto-attached and reported here the first time it stops. Register it,
            // mirror clone tracing onto it, and let handle_stopped_thread_event arm its watchpoints and continue it.
            let _ = ptrace_set_clone_tracing(event_thread_id);
            self.traced_threads_by_id.insert(
                event_thread_id,
                TracedLinuxThread {
                    thread_id: event_thread_id,
                    is_stopped: true,
                },
            );
        }

        // Decide which signal to re-inject when continuing the thread. A tracee that stopped because a signal was about
        // to be delivered (a signal-delivery-stop) must be restarted WITH that signal, otherwise the signal is silently
        // discarded. This matters enormously on Android: the ART runtime relies on SIGSEGV (implicit null checks, GC
        // read barriers) and SIGABRT for normal operation. Swallowing them makes the faulting instruction re-fault
        // forever, which storms the tracer and deadlocks the whole app. We only swallow our own debug traps:
        //   - ptrace event/interrupt/group stops (event_code != 0): not real pending signals; restart with 0.
        //   - a watchpoint SIGTRAP (event_code == 0): our hardware breakpoint; swallow it.
        //   - anything else (SIGSEGV, SIGBUS, SIGABRT, ...): a real signal the tracee must still receive; re-inject it.
        let signal_to_inject = if ptrace_event_code != 0 || stop_signal == SIGTRAP { 0 } else { stop_signal };

        // This thread's watchpoint was disabled after a hit so it could run past the access. This stop (our re-arm
        // interrupt, or a real signal that arrived first) is where we re-arm and continue, forwarding any real signal.
        // Re-arming early on a signal is harmless: the access already retired while the thread ran free.
        if self.rearm_at_by_thread.remove(&event_thread_id).is_some() {
            let rearm_result = self.apply_watchpoints_to_thread(event_thread_id);
            let continue_result = self.continue_stopped_thread_with_signal(event_thread_id, signal_to_inject, "PTRACE_CONT after watchpoint re-arm");

            return self.finalize_thread_event(event_thread_id, [rearm_result, continue_result]);
        }

        // Only a pure signal-delivery SIGTRAP (no ptrace event code) can be a hardware watchpoint hit. Clone events,
        // interrupt/group stops (PTRACE_EVENT_STOP), and a brand-new thread's first stop must not be mis-read as hits.
        let is_watchpoint_candidate = is_known_thread && stop_signal == SIGTRAP && ptrace_event_code == 0;

        self.handle_stopped_thread_event(event_thread_id, is_watchpoint_candidate, signal_to_inject)
    }

    fn drain_pending_stops_after_resume(&mut self) -> Result<(), DebuggerPluginError> {
        let drain_started_at = Instant::now();

        while drain_started_at.elapsed() < Duration::from_millis(RESUME_STOP_DRAIN_TIMEOUT_MS) {
            let mut wait_status = 0;
            let wait_result = unsafe { libc::waitpid(-1, &mut wait_status, WAIT_ALL_TRACED_THREADS) };

            if wait_result == 0 {
                return Ok(());
            }

            if wait_result < 0 {
                return Err(last_os_error("waitpid while draining Linux resume stops"));
            }

            self.handle_wait_status(wait_result, wait_status)?;
        }

        Ok(())
    }

    fn handle_stopped_thread_event(
        &mut self,
        event_thread_id: pid_t,
        is_watchpoint_candidate: bool,
        signal_to_inject: i32,
    ) -> Result<(), DebuggerPluginError> {
        if self.session_state == DebuggerSessionState::Detached {
            return Ok(());
        }

        if is_watchpoint_candidate {
            match self.handle_breakpoint_trap(event_thread_id) {
                // A real watchpoint hit was recorded. On arm64 the hit is reported before the accessing instruction
                // retires, so continuing with the watchpoint armed would immediately re-trap forever. Disable the
                // watchpoint on this thread and let it run; interrupt_due_rearm_threads() re-arms it after a short delay
                // (once the access — possibly an LDXR/STXR atomic — has retired), so subsequent accesses keep counting
                // without a re-trap livelock.
                Ok(true) => {
                    let disarm_result = self.clear_watchpoints_for_thread(event_thread_id);
                    if disarm_result.is_ok() {
                        self.rearm_at_by_thread
                            .insert(event_thread_id, Instant::now() + Duration::from_millis(WATCHPOINT_REARM_DELAY_MS));
                    }
                    // signal_to_inject is 0 here (a watchpoint hit is reported as SIGTRAP, which we swallow).
                    let continue_result = self.continue_stopped_thread_with_signal(event_thread_id, signal_to_inject, "PTRACE_CONT after watchpoint hit");

                    return self.finalize_thread_event(event_thread_id, [disarm_result, continue_result]);
                }
                // A SIGTRAP that is not one of our watchpoints: re-arm and continue normally (handled below).
                Ok(false) => {}
                Err(record_error) => {
                    // Recording failed; clear watchpoints and continue so the thread is never stranded stopped.
                    let clear_result = self.clear_watchpoints_for_thread(event_thread_id);
                    let continue_result = self.continue_stopped_thread_with_signal(event_thread_id, signal_to_inject, "PTRACE_CONT after failed trace record");

                    return self.finalize_thread_event(event_thread_id, [Err(record_error), clear_result, continue_result]);
                }
            }
        }

        let watchpoint_result = self.apply_watchpoints_to_thread(event_thread_id);
        let continue_result = self.continue_stopped_thread_with_signal(event_thread_id, signal_to_inject, "PTRACE_CONT after debug event");

        self.finalize_thread_event(event_thread_id, [watchpoint_result, continue_result])
    }

    fn finalize_thread_event<const RESULT_COUNT: usize>(
        &mut self,
        event_thread_id: pid_t,
        results: [Result<(), DebuggerPluginError>; RESULT_COUNT],
    ) -> Result<(), DebuggerPluginError> {
        let error_messages = results
            .into_iter()
            .filter_map(|result| result.err().map(|error| error.to_string()))
            .collect::<Vec<_>>();

        if error_messages.is_empty() {
            self.session_state = DebuggerSessionState::Running;

            return Ok(());
        }

        log::warn!(
            "Linux debugger recovered stopped thread {} after debug event error(s): {}.",
            event_thread_id,
            error_messages.join("; ")
        );

        Err(LinuxDebuggerBackend::plugin_error(format!(
            "Linux debugger recovered stopped thread {} after debug event error(s): {}.",
            event_thread_id,
            error_messages.join("; ")
        )))
    }

    /// Handles a SIGTRAP that may be a hardware watchpoint hit. Returns `true` when the trap was attributed to one of
    /// our watchpoints and a trace event was emitted; the caller then single-steps the thread past the accessing
    /// instruction before re-arming, so every subsequent access is caught (and counted) instead of re-trapping forever.
    fn handle_breakpoint_trap(
        &mut self,
        event_thread_id: pid_t,
    ) -> Result<bool, DebuggerPluginError> {
        let breakpoint_descriptor = self.describe_hit_breakpoint(event_thread_id)?;
        let is_execute_breakpoint = matches!(
            breakpoint_descriptor
                .as_ref()
                .map(DebuggerBreakpointDescriptor::get_kind),
            Some(DebuggerBreakpointKind::HardwareExecute)
        );
        let register_snapshot = self.read_registers_for_thread(event_thread_id)?;
        let (instruction_address, instruction_bytes, backend_message) =
            self.resolve_trace_instruction(event_thread_id, register_snapshot.get_instruction_pointer(), is_execute_breakpoint);

        self.clear_debug_status(event_thread_id)?;

        let Some(breakpoint_descriptor) = breakpoint_descriptor else {
            return Ok(false);
        };

        let trace_event = DebuggerTraceEvent::new(
            Some(breakpoint_descriptor),
            register_snapshot,
            instruction_address,
            instruction_bytes,
            None,
            backend_message,
        )
        .with_target_architecture(self.target_architecture.clone());

        (self.trace_event_sink)(trace_event);

        Ok(true)
    }

    fn resolve_trace_instruction(
        &self,
        thread_id: pid_t,
        post_trap_instruction_pointer: Option<u64>,
        is_execute_breakpoint: bool,
    ) -> (Option<u64>, Vec<u8>, Option<String>) {
        let Some(post_trap_instruction_pointer) = post_trap_instruction_pointer else {
            return (None, Vec::new(), None);
        };

        // For an execute breakpoint the trap is taken before the instruction runs, so the instruction pointer already
        // points at the instruction itself (no post-trap recovery needed). The bytes there are exactly what executed.
        if is_execute_breakpoint {
            let execute_message = Some(String::from("Hardware execute breakpoint hit; instruction pointer is the instruction that trapped."));

            // arm64's block disassembler joins a whole byte window into one line, so read exactly one instruction to
            // keep the trace's instruction text (and accessed-address resolution) to the single trapped instruction.
            #[cfg(all(target_os = "android", target_arch = "aarch64"))]
            if self.target_architecture.get_instruction_set_id() == "arm64" {
                return (
                    Some(post_trap_instruction_pointer),
                    self.read_memory_bytes(thread_id, post_trap_instruction_pointer, ARM64_INSTRUCTION_BYTE_LENGTH)
                        .unwrap_or_default(),
                    execute_message,
                );
            }

            return (
                Some(post_trap_instruction_pointer),
                self.read_instruction_bytes(thread_id, Some(post_trap_instruction_pointer)),
                execute_message,
            );
        }

        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        if self.target_architecture.get_instruction_set_id() == "x64" {
            if let Some((instruction_address, instruction_bytes)) = self.resolve_x64_post_trap_instruction(thread_id, post_trap_instruction_pointer) {
                return (
                    Some(instruction_address),
                    instruction_bytes,
                    Some(String::from(
                        "Linux x64 hardware data breakpoint hit; trace instruction was recovered from the post-trap RIP.",
                    )),
                );
            }

            return (
                Some(post_trap_instruction_pointer),
                self.read_instruction_bytes(thread_id, Some(post_trap_instruction_pointer)),
                Some(String::from(
                    "Linux x64 hardware data breakpoint hit; instruction pointer is the post-trap RIP and may point after the accessing instruction.",
                )),
            );
        }

        #[cfg(all(target_os = "android", target_arch = "aarch64"))]
        if self.target_architecture.get_instruction_set_id() == "arm64" {
            return (
                Some(post_trap_instruction_pointer),
                self.read_memory_bytes(thread_id, post_trap_instruction_pointer, ARM64_INSTRUCTION_BYTE_LENGTH)
                    .unwrap_or_default(),
                Some(String::from(
                    "Android arm64 hardware data breakpoint hit; instruction pointer was reported by ptrace and needs human verification.",
                )),
            );
        }

        (
            Some(post_trap_instruction_pointer),
            self.read_instruction_bytes(thread_id, Some(post_trap_instruction_pointer)),
            Some(String::from(
                "Native ptrace hardware data breakpoint hit; instruction attribution is unsupported for this architecture.",
            )),
        )
    }

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    fn resolve_x64_post_trap_instruction(
        &self,
        thread_id: pid_t,
        post_trap_instruction_pointer: u64,
    ) -> Option<(u64, Vec<u8>)> {
        let window_start_address = post_trap_instruction_pointer.saturating_sub(X64_MAX_INSTRUCTION_LENGTH as u64);
        let window_byte_count = post_trap_instruction_pointer.saturating_sub(window_start_address) as usize;
        let window_bytes = self.read_memory_bytes(thread_id, window_start_address, window_byte_count)?;

        resolve_x64_post_trap_instruction_from_window(post_trap_instruction_pointer, window_start_address, &window_bytes)
    }

    fn describe_hit_breakpoint(
        &self,
        thread_id: pid_t,
    ) -> Result<Option<DebuggerBreakpointDescriptor>, DebuggerPluginError> {
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        if self.target_architecture.get_instruction_set_id() == "x64" {
            let dr6 = read_debug_register(thread_id, 6)?;
            return Ok(self.describe_x64_hit_breakpoint(dr6));
        }

        #[cfg(all(target_os = "android", target_arch = "aarch64"))]
        if self.target_architecture.get_instruction_set_id() == "arm64" {
            return self.describe_arm64_hit_breakpoint(thread_id);
        }

        Ok(None)
    }

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    fn describe_x64_hit_breakpoint(
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
        thread_id: pid_t,
        instruction_pointer: Option<u64>,
    ) -> Vec<u8> {
        let Some(instruction_pointer) = instruction_pointer else {
            return Vec::new();
        };
        let mut instruction_bytes = Vec::with_capacity(TRACE_INSTRUCTION_BYTE_WINDOW);

        while instruction_bytes.len() < TRACE_INSTRUCTION_BYTE_WINDOW {
            let read_address = instruction_pointer.saturating_add(instruction_bytes.len() as u64);
            let word = match ptrace_peek_data(thread_id, read_address) {
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

    fn read_memory_bytes(
        &self,
        thread_id: pid_t,
        base_address: u64,
        byte_count: usize,
    ) -> Option<Vec<u8>> {
        let mut memory_bytes = Vec::with_capacity(byte_count);

        while memory_bytes.len() < byte_count {
            let read_address = base_address.saturating_add(memory_bytes.len() as u64);
            let word = match ptrace_peek_data(thread_id, read_address) {
                Ok(word) => word,
                Err(error) => {
                    log::debug!("Failed to read Linux memory bytes at 0x{:X}: {}", read_address, error);
                    return None;
                }
            };
            let word_bytes = word.to_ne_bytes();
            let remaining_byte_count = byte_count - memory_bytes.len();
            let copied_byte_count = remaining_byte_count.min(word_bytes.len());

            memory_bytes.extend_from_slice(&word_bytes[..copied_byte_count]);
        }

        Some(memory_bytes)
    }

    fn apply_watchpoints(&self) -> Result<(), DebuggerPluginError> {
        for traced_thread in self.traced_threads_by_id.values() {
            if traced_thread.is_stopped {
                self.apply_watchpoints_to_thread(traced_thread.thread_id)?;
            }
        }

        Ok(())
    }

    fn apply_watchpoints_to_thread(
        &self,
        thread_id: pid_t,
    ) -> Result<(), DebuggerPluginError> {
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        if self.target_architecture.get_instruction_set_id() == "x64" {
            return self.apply_x64_watchpoints_to_thread(thread_id);
        }

        #[cfg(all(target_os = "android", target_arch = "aarch64"))]
        if self.target_architecture.get_instruction_set_id() == "arm64" {
            return self.apply_arm64_watchpoints_to_thread(thread_id);
        }

        Err(LinuxDebuggerBackend::plugin_error(format!(
            "Native ptrace debugger watchpoints are unsupported for architecture '{}'.",
            self.target_architecture.get_instruction_set_id()
        )))
    }

    fn clear_watchpoints_for_thread(
        &self,
        thread_id: pid_t,
    ) -> Result<(), DebuggerPluginError> {
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        if self.target_architecture.get_instruction_set_id() == "x64" {
            return self.clear_x64_watchpoints_for_thread(thread_id);
        }

        #[cfg(all(target_os = "android", target_arch = "aarch64"))]
        if self.target_architecture.get_instruction_set_id() == "arm64" {
            return self.clear_arm64_watchpoints_for_thread(thread_id);
        }

        Err(LinuxDebuggerBackend::plugin_error(format!(
            "Native ptrace debugger watchpoints are unsupported for architecture '{}'.",
            self.target_architecture.get_instruction_set_id()
        )))
    }

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    fn apply_x64_watchpoints_to_thread(
        &self,
        thread_id: pid_t,
    ) -> Result<(), DebuggerPluginError> {
        let mut dr7 = 0u64;

        clear_debug_status(thread_id)?;
        for slot in 0..WATCHPOINT_SLOT_COUNT {
            write_debug_register(thread_id, slot, 0)?;
        }

        for stored_breakpoint in self.breakpoints_by_id.values() {
            if !Self::is_watchpoint_armed(stored_breakpoint) {
                continue;
            }

            let slot = stored_breakpoint.slot;

            match stored_breakpoint.descriptor.get_kind() {
                DebuggerBreakpointKind::HardwareData { access, size_in_bytes } => {
                    let length_bits = x64_watchpoint_length_bits(*size_in_bytes)?;
                    let access_bits = x64_watchpoint_access_bits(*access);

                    write_debug_register(thread_id, slot, stored_breakpoint.descriptor.get_address())?;
                    dr7 |= 1u64 << (slot * 2);
                    dr7 |= access_bits << (16 + slot * 4);
                    dr7 |= length_bits << (18 + slot * 4);
                }
                DebuggerBreakpointKind::HardwareExecute => {
                    // Execute breakpoint: rw = 00 and len = 00 (those bit fields stay zero); just enable the slot.
                    write_debug_register(thread_id, slot, stored_breakpoint.descriptor.get_address())?;
                    dr7 |= 1u64 << (slot * 2);
                }
                DebuggerBreakpointKind::Software => {}
            }
        }

        write_debug_register(thread_id, 7, dr7)
    }

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    fn clear_x64_watchpoints_for_thread(
        &self,
        thread_id: pid_t,
    ) -> Result<(), DebuggerPluginError> {
        clear_debug_status(thread_id)?;
        for slot in 0..WATCHPOINT_SLOT_COUNT {
            write_debug_register(thread_id, slot, 0)?;
        }

        write_debug_register(thread_id, 7, 0)
    }

    fn is_watchpoint_armed(stored_breakpoint: &StoredLinuxBreakpoint) -> bool {
        stored_breakpoint.descriptor.get_is_enabled()
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

    fn validate_hardware_breakpoint(
        &self,
        address: u64,
        kind: &DebuggerBreakpointKind,
    ) -> Result<(), DebuggerPluginError> {
        // Instruction (execute) breakpoints: arm64 requires the instruction be 4-byte aligned; x86 has no alignment rule.
        if matches!(kind, DebuggerBreakpointKind::HardwareExecute) {
            #[cfg(all(target_os = "android", target_arch = "aarch64"))]
            if self.target_architecture.get_instruction_set_id() == "arm64" && address % 4 != 0 {
                return Err(LinuxDebuggerBackend::plugin_error(format!(
                    "Android arm64 hardware execute breakpoint at 0x{:X} must be 4-byte aligned.",
                    address
                )));
            }

            return Ok(());
        }

        let DebuggerBreakpointKind::HardwareData { size_in_bytes, .. } = *kind else {
            return Ok(());
        };

        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        if self.target_architecture.get_instruction_set_id() == "x64" {
            x64_watchpoint_length_bits(size_in_bytes)?;

            if address % u64::from(size_in_bytes) != 0 {
                return Err(LinuxDebuggerBackend::plugin_error(format!(
                    "Linux x64 hardware data breakpoint at 0x{:X} must be aligned to its {} byte size.",
                    address, size_in_bytes
                )));
            }

            return Ok(());
        }

        #[cfg(all(target_os = "android", target_arch = "aarch64"))]
        if self.target_architecture.get_instruction_set_id() == "arm64" {
            validate_arm64_watchpoint(address, size_in_bytes)?;

            return Ok(());
        }

        Err(LinuxDebuggerBackend::plugin_error(format!(
            "Native ptrace hardware data breakpoints are unsupported for architecture '{}'.",
            self.target_architecture.get_instruction_set_id()
        )))
    }

    fn read_registers_for_thread(
        &self,
        thread_id: pid_t,
    ) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        if self.target_architecture.get_instruction_set_id() == "x64" {
            return self.read_x64_registers(thread_id);
        }

        #[cfg(all(target_os = "android", target_arch = "aarch64"))]
        if self.target_architecture.get_instruction_set_id() == "arm64" {
            return self.read_arm64_registers(thread_id);
        }

        Err(LinuxDebuggerBackend::plugin_error(format!(
            "Native ptrace debugger register reads are unsupported for architecture '{}'.",
            self.target_architecture.get_instruction_set_id()
        )))
    }

    fn clear_debug_status(
        &self,
        thread_id: pid_t,
    ) -> Result<(), DebuggerPluginError> {
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        if self.target_architecture.get_instruction_set_id() == "x64" {
            return clear_debug_status(thread_id);
        }

        let _ = thread_id;
        Ok(())
    }

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    fn read_x64_registers(
        &self,
        thread_id: pid_t,
    ) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        let registers = get_registers(thread_id)?;
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

    #[cfg(all(target_os = "android", target_arch = "aarch64"))]
    fn read_arm64_registers(
        &self,
        thread_id: pid_t,
    ) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        let registers = read_arm64_raw_registers(thread_id)?;
        let mut register_values = Vec::with_capacity(34);

        for register_number in 0..31 {
            register_values.push(DebuggerRegisterValue::new(format!("x{}", register_number), registers.regs[register_number], 64));
        }
        register_values.push(DebuggerRegisterValue::new("sp", registers.sp, 64));
        register_values.push(DebuggerRegisterValue::new("pc", registers.pc, 64));
        register_values.push(DebuggerRegisterValue::new("pstate", registers.pstate, 64));

        Ok(DebuggerRegisterSnapshot::new(Some(registers.pc), Some(registers.sp), register_values))
    }

    #[cfg(all(target_os = "android", target_arch = "aarch64"))]
    fn write_arm64_register(
        &self,
        register_thread_id: pid_t,
        register_name: &str,
        value: u64,
    ) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        let mut registers = read_arm64_raw_registers(register_thread_id)?;
        let normalized_register_name = register_name.trim().to_ascii_lowercase();

        match normalized_register_name.as_str() {
            "sp" => registers.sp = value,
            "pc" => registers.pc = value,
            "pstate" => registers.pstate = value,
            "fp" => registers.regs[29] = value,
            "lr" => registers.regs[30] = value,
            register_name if register_name.starts_with('x') => {
                let register_number = register_name
                    .trim_start_matches('x')
                    .parse::<usize>()
                    .map_err(|_| LinuxDebuggerBackend::plugin_error(format!("Android arm64 register '{}' is not supported for writes.", register_name)))?;
                if register_number >= registers.regs.len() {
                    return Err(LinuxDebuggerBackend::plugin_error(format!(
                        "Android arm64 register '{}' is not supported for writes.",
                        register_name
                    )));
                }

                registers.regs[register_number] = value;
            }
            unsupported_register_name => {
                return Err(LinuxDebuggerBackend::plugin_error(format!(
                    "Android arm64 register '{}' is not supported for writes.",
                    unsupported_register_name
                )));
            }
        }

        write_arm64_raw_registers(register_thread_id, &mut registers)?;
        self.read_arm64_registers(register_thread_id)
    }

    #[cfg(all(target_os = "android", target_arch = "aarch64"))]
    fn apply_arm64_watchpoints_to_thread(
        &self,
        thread_id: pid_t,
    ) -> Result<(), DebuggerPluginError> {
        let mut debug_state = read_arm64_hardware_watchpoint_state(thread_id)?;
        let supported_slot_count = (debug_state.dbg_info & 0xff) as usize;

        for debug_register in &mut debug_state.dbg_regs {
            *debug_register = Arm64HardwareDebugRegister::default();
        }

        for stored_breakpoint in self.breakpoints_by_id.values() {
            if !Self::is_watchpoint_armed(stored_breakpoint) {
                continue;
            }

            let DebuggerBreakpointKind::HardwareData { access, size_in_bytes } = *stored_breakpoint.descriptor.get_kind() else {
                continue;
            };
            let slot = stored_breakpoint.slot;
            if slot >= supported_slot_count {
                return Err(LinuxDebuggerBackend::plugin_error(format!(
                    "Android arm64 hardware watchpoint slot {} is unavailable; this device reported {} slot(s).",
                    slot, supported_slot_count
                )));
            }

            let address = stored_breakpoint.descriptor.get_address();
            let granule_base = address & !(ARM64_WATCH_GRANULE_SIZE - 1);
            let byte_offset = address - granule_base;
            let byte_count = u64::from(size_in_bytes);
            let byte_mask = ((1u64 << byte_count) - 1) << byte_offset;

            debug_state.dbg_regs[slot].address = granule_base;
            debug_state.dbg_regs[slot].control = arm64_watch_control(access, byte_mask);
        }

        write_arm64_hardware_watchpoint_state(thread_id, &mut debug_state)?;
        self.apply_arm64_breakpoints_to_thread(thread_id)
    }

    /// Programs the hardware instruction-breakpoint bank (NT_ARM_HW_BREAK) for any armed HardwareExecute breakpoints.
    #[cfg(all(target_os = "android", target_arch = "aarch64"))]
    fn apply_arm64_breakpoints_to_thread(
        &self,
        thread_id: pid_t,
    ) -> Result<(), DebuggerPluginError> {
        let mut debug_state = read_arm64_hardware_breakpoint_state(thread_id)?;
        let supported_slot_count = (debug_state.dbg_info & 0xff) as usize;

        for debug_register in &mut debug_state.dbg_regs {
            *debug_register = Arm64HardwareDebugRegister::default();
        }

        for stored_breakpoint in self.breakpoints_by_id.values() {
            if !Self::is_watchpoint_armed(stored_breakpoint) || !matches!(stored_breakpoint.descriptor.get_kind(), DebuggerBreakpointKind::HardwareExecute) {
                continue;
            }

            let slot = stored_breakpoint.slot;
            if slot >= supported_slot_count {
                return Err(LinuxDebuggerBackend::plugin_error(format!(
                    "Android arm64 hardware breakpoint slot {} is unavailable; this device reported {} slot(s).",
                    slot, supported_slot_count
                )));
            }

            debug_state.dbg_regs[slot].address = stored_breakpoint.descriptor.get_address();
            debug_state.dbg_regs[slot].control = arm64_breakpoint_control();
        }

        write_arm64_hardware_breakpoint_state(thread_id, &mut debug_state)
    }

    #[cfg(all(target_os = "android", target_arch = "aarch64"))]
    fn clear_arm64_watchpoints_for_thread(
        &self,
        thread_id: pid_t,
    ) -> Result<(), DebuggerPluginError> {
        let mut debug_state = read_arm64_hardware_watchpoint_state(thread_id)?;

        for debug_register in &mut debug_state.dbg_regs {
            *debug_register = Arm64HardwareDebugRegister::default();
        }

        write_arm64_hardware_watchpoint_state(thread_id, &mut debug_state)?;

        let mut breakpoint_state = read_arm64_hardware_breakpoint_state(thread_id)?;

        for debug_register in &mut breakpoint_state.dbg_regs {
            *debug_register = Arm64HardwareDebugRegister::default();
        }

        write_arm64_hardware_breakpoint_state(thread_id, &mut breakpoint_state)
    }

    #[cfg(all(target_os = "android", target_arch = "aarch64"))]
    fn describe_arm64_hit_breakpoint(
        &self,
        thread_id: pid_t,
    ) -> Result<Option<DebuggerBreakpointDescriptor>, DebuggerPluginError> {
        let fault_address = read_signal_fault_address(thread_id)?;
        // Execute breakpoints are identified by the program counter (the instruction that trapped), data watchpoints by
        // the faulting access address reported in the signal info.
        let program_counter = read_arm64_raw_registers(thread_id)
            .ok()
            .map(|registers| registers.pc);

        Ok(self
            .breakpoints_by_id
            .values()
            .find(|stored_breakpoint| {
                if !stored_breakpoint.descriptor.get_is_enabled() {
                    return false;
                }

                match *stored_breakpoint.descriptor.get_kind() {
                    DebuggerBreakpointKind::HardwareData { size_in_bytes, .. } => {
                        let Some(fault_address) = fault_address else {
                            return false;
                        };

                        arm64_fault_matches_watchpoint(fault_address, stored_breakpoint.descriptor.get_address(), size_in_bytes)
                    }
                    DebuggerBreakpointKind::HardwareExecute => program_counter == Some(stored_breakpoint.descriptor.get_address()),
                    DebuggerBreakpointKind::Software => false,
                }
            })
            .map(|stored_breakpoint| stored_breakpoint.descriptor.clone()))
    }

    fn attach_existing_threads(&mut self) -> Result<(), DebuggerPluginError> {
        let mut thread_ids = enumerate_linux_thread_ids(self.process_id)?;
        thread_ids.sort_by_key(|thread_id| if *thread_id == self.process_id { 0 } else { 1 });

        for thread_id in thread_ids {
            if !self.traced_threads_by_id.contains_key(&thread_id) {
                let attach_result = self.attach_thread(thread_id)?;

                if attach_result == AttachThreadResult::SkippedStaleThread {
                    continue;
                }
            }
        }

        Ok(())
    }

    fn sync_thread_list(&mut self) -> Result<(), DebuggerPluginError> {
        let known_thread_ids = self.traced_threads_by_id.keys().copied().collect::<Vec<_>>();

        for thread_id in known_thread_ids {
            if !linux_task_exists(self.process_id, thread_id) {
                self.traced_threads_by_id.remove(&thread_id);
            }
        }

        // NOTE: We deliberately do NOT re-attach newly-created threads here. PTRACE_ATTACH delivers a SIGSTOP, which
        // is a process-wide job-control stop: every attach group-stops all ~300 threads of the target. PTRACE_CONT
        // does not lift a group-stop (only SIGCONT does), and the kernel will not re-report an already-reported
        // group-stop, so repeatedly attaching strands threads in ptrace_stop and deadlocks the app. New threads are
        // therefore not watched in this build; the watchpoint is mirrored across the threads present at attach time.
        Ok(())
    }

    fn attach_thread(
        &mut self,
        thread_id: pid_t,
    ) -> Result<AttachThreadResult, DebuggerPluginError> {
        let attach_operation_name = format!("PTRACE_SEIZE thread {} in process {}", thread_id, self.process_id);

        // SEIZE attaches without delivering a SIGSTOP (no group-stop). PTRACE_O_TRACECLONE makes threads cloned later
        // auto-attach, so we never have to re-deliver SIGSTOP to pick up new threads.
        if let Err(error) = ptrace_seize(thread_id, &attach_operation_name) {
            if is_linux_task_stale_attach_error(&error, self.process_id, thread_id) {
                log::debug!(
                    "Skipping stale Linux thread {} during debugger attach for process {}.",
                    thread_id,
                    self.process_id
                );

                return Ok(AttachThreadResult::SkippedStaleThread);
            }

            return Err(error);
        }

        // SEIZE leaves the thread running; interrupt it so its debug (watchpoint) registers can be programmed.
        if let Err(error) = ptrace_interrupt(thread_id, "PTRACE_INTERRUPT initial Linux thread attach")
            .and_then(|_| wait_for_stop(thread_id, ATTACH_WAIT_TIMEOUT, "initial Linux thread attach"))
        {
            let _ = ptrace_request(
                libc::PTRACE_DETACH,
                thread_id,
                null_mut(),
                null_mut(),
                "PTRACE_DETACH after failed thread attach",
            );

            if is_linux_task_stale_attach_error(&error, self.process_id, thread_id) {
                log::debug!(
                    "Skipping stale Linux thread {} after attach wait failed for process {}.",
                    thread_id,
                    self.process_id
                );

                return Ok(AttachThreadResult::SkippedStaleThread);
            }

            return Err(error);
        }

        self.traced_threads_by_id
            .insert(thread_id, TracedLinuxThread { thread_id, is_stopped: true });
        self.apply_watchpoints_to_thread(thread_id)?;

        if self.session_state == DebuggerSessionState::Running {
            let resume_operation_name = format!("PTRACE_CONT newly attached Linux thread {}", thread_id);
            ptrace_request(libc::PTRACE_CONT, thread_id, null_mut(), null_mut(), &resume_operation_name)?;
            if let Some(traced_thread) = self.traced_threads_by_id.get_mut(&thread_id) {
                traced_thread.is_stopped = false;
            }
        }

        Ok(AttachThreadResult::Attached)
    }

    fn resume_stopped_threads(
        &mut self,
        operation_name: &str,
    ) -> Result<(), DebuggerPluginError> {
        let stopped_thread_ids = self
            .traced_threads_by_id
            .values()
            .filter(|traced_thread| traced_thread.is_stopped)
            .map(|traced_thread| traced_thread.thread_id)
            .collect::<Vec<_>>();

        for thread_id in stopped_thread_ids {
            self.continue_stopped_thread(thread_id, operation_name)?;
        }

        Ok(())
    }

    fn continue_stopped_thread(
        &mut self,
        thread_id: pid_t,
        operation_name: &str,
    ) -> Result<(), DebuggerPluginError> {
        self.continue_stopped_thread_with_signal(thread_id, 0, operation_name)
    }

    fn continue_stopped_thread_with_signal(
        &mut self,
        thread_id: pid_t,
        signal_to_inject: i32,
        operation_name: &str,
    ) -> Result<(), DebuggerPluginError> {
        // The signal number is passed in ptrace's `data` argument; 0 means "deliver no signal" (swallow it).
        ptrace_request(
            libc::PTRACE_CONT,
            thread_id,
            null_mut(),
            signal_to_inject as usize as *mut c_void,
            operation_name,
        )?;
        if let Some(traced_thread) = self.traced_threads_by_id.get_mut(&thread_id) {
            traced_thread.is_stopped = false;
        }

        Ok(())
    }

    fn select_register_thread_id(&self) -> Result<pid_t, DebuggerPluginError> {
        if self
            .traced_threads_by_id
            .get(&self.process_id)
            .map(|traced_thread| traced_thread.is_stopped)
            .unwrap_or(false)
        {
            return Ok(self.process_id);
        }

        self.traced_threads_by_id
            .values()
            .find(|traced_thread| traced_thread.is_stopped)
            .map(|traced_thread| traced_thread.thread_id)
            .ok_or_else(|| LinuxDebuggerBackend::plugin_error("Cannot access Linux registers because no traced thread is currently stopped."))
    }
}

fn enumerate_linux_thread_ids(process_id: pid_t) -> Result<Vec<pid_t>, DebuggerPluginError> {
    let task_directory_path = format!("/proc/{}/task", process_id);
    let task_directory_entries = fs::read_dir(&task_directory_path)
        .map_err(|error| LinuxDebuggerBackend::plugin_error(format!("Failed to enumerate Linux process threads from '{}': {}.", task_directory_path, error)))?;
    let mut thread_ids = Vec::new();

    for task_directory_entry_result in task_directory_entries {
        let task_directory_entry = match task_directory_entry_result {
            Ok(task_directory_entry) => task_directory_entry,
            Err(error) => {
                log::debug!("Failed to read Linux task directory entry: {}", error);
                continue;
            }
        };
        let thread_id_text = task_directory_entry.file_name().to_string_lossy().to_string();
        let Ok(thread_id) = thread_id_text.parse::<pid_t>() else {
            continue;
        };

        thread_ids.push(thread_id);
    }

    thread_ids.sort_unstable();
    Ok(thread_ids)
}

fn linux_task_exists(
    process_id: pid_t,
    thread_id: pid_t,
) -> bool {
    fs::metadata(format!("/proc/{}/task/{}", process_id, thread_id)).is_ok()
}

fn is_linux_task_stale_attach_error(
    error: &DebuggerPluginError,
    process_id: pid_t,
    thread_id: pid_t,
) -> bool {
    if thread_id == process_id {
        return false;
    }

    let error_message = error.get_message();
    let is_no_such_process = error_message.contains("No such process") || error_message.contains("os error 3");

    is_no_such_process && !linux_task_exists(process_id, thread_id)
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

        // While watchpoints are awaiting re-arm after a hit, poll quickly so the disabled window stays close to the
        // configured delay (tighter hit counts) instead of idling for the full command timeout. Otherwise idle until a
        // command or the next tick.
        let command_wait_timeout = if active_session.has_pending_rearms() {
            Duration::from_millis(WATCHPOINT_REARM_POLL_TIMEOUT_MS)
        } else {
            Duration::from_millis(IDLE_COMMAND_WAIT_TIMEOUT_MS)
        };

        match worker_command_receiver.recv_timeout(command_wait_timeout) {
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

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
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

#[cfg(all(target_os = "android", target_arch = "aarch64"))]
#[repr(C)]
#[derive(Clone, Copy)]
struct Arm64HardwareDebugRegister {
    address: u64,
    control: u32,
    padding: u32,
}

#[cfg(all(target_os = "android", target_arch = "aarch64"))]
impl Default for Arm64HardwareDebugRegister {
    fn default() -> Self {
        unsafe { zeroed() }
    }
}

#[cfg(all(target_os = "android", target_arch = "aarch64"))]
#[repr(C)]
#[derive(Clone, Copy)]
struct Arm64HardwareDebugState {
    dbg_info: u32,
    pad: u32,
    dbg_regs: [Arm64HardwareDebugRegister; 16],
}

#[cfg(all(target_os = "android", target_arch = "aarch64"))]
impl Default for Arm64HardwareDebugState {
    fn default() -> Self {
        unsafe { zeroed() }
    }
}

#[cfg(all(target_os = "android", target_arch = "aarch64"))]
fn read_arm64_raw_registers(thread_id: pid_t) -> Result<libc::user_regs_struct, DebuggerPluginError> {
    let mut registers = unsafe { zeroed::<libc::user_regs_struct>() };

    ptrace_get_regset(thread_id, NT_PRSTATUS, &mut registers, "PTRACE_GETREGSET NT_PRSTATUS")?;

    Ok(registers)
}

#[cfg(all(target_os = "android", target_arch = "aarch64"))]
fn write_arm64_raw_registers(
    thread_id: pid_t,
    registers: &mut libc::user_regs_struct,
) -> Result<(), DebuggerPluginError> {
    ptrace_set_regset(thread_id, NT_PRSTATUS, registers, "PTRACE_SETREGSET NT_PRSTATUS")
}

#[cfg(all(target_os = "android", target_arch = "aarch64"))]
fn read_arm64_hardware_watchpoint_state(thread_id: pid_t) -> Result<Arm64HardwareDebugState, DebuggerPluginError> {
    let mut debug_state = Arm64HardwareDebugState::default();

    ptrace_get_regset(thread_id, NT_ARM_HW_WATCH, &mut debug_state, "PTRACE_GETREGSET NT_ARM_HW_WATCH")?;

    Ok(debug_state)
}

#[cfg(all(target_os = "android", target_arch = "aarch64"))]
fn write_arm64_hardware_watchpoint_state(
    thread_id: pid_t,
    debug_state: &mut Arm64HardwareDebugState,
) -> Result<(), DebuggerPluginError> {
    let reported_slot_count = (debug_state.dbg_info & 0xff) as usize;
    let written_slot_count = reported_slot_count.min(debug_state.dbg_regs.len());
    let header_byte_count = std::mem::offset_of!(Arm64HardwareDebugState, dbg_regs);
    let register_byte_count = written_slot_count.saturating_mul(std::mem::size_of::<Arm64HardwareDebugRegister>());
    let mut io_vector = libc::iovec {
        iov_base: (debug_state as *mut Arm64HardwareDebugState).cast::<c_void>(),
        iov_len: header_byte_count.saturating_add(register_byte_count),
    };

    ptrace_request(
        libc::PTRACE_SETREGSET,
        thread_id,
        NT_ARM_HW_WATCH as *mut c_void,
        (&mut io_vector as *mut libc::iovec).cast::<c_void>(),
        "PTRACE_SETREGSET NT_ARM_HW_WATCH",
    )
}

#[cfg(all(target_os = "android", target_arch = "aarch64"))]
fn read_arm64_hardware_breakpoint_state(thread_id: pid_t) -> Result<Arm64HardwareDebugState, DebuggerPluginError> {
    let mut debug_state = Arm64HardwareDebugState::default();

    ptrace_get_regset(thread_id, NT_ARM_HW_BREAK, &mut debug_state, "PTRACE_GETREGSET NT_ARM_HW_BREAK")?;

    Ok(debug_state)
}

#[cfg(all(target_os = "android", target_arch = "aarch64"))]
fn write_arm64_hardware_breakpoint_state(
    thread_id: pid_t,
    debug_state: &mut Arm64HardwareDebugState,
) -> Result<(), DebuggerPluginError> {
    let reported_slot_count = (debug_state.dbg_info & 0xff) as usize;
    let written_slot_count = reported_slot_count.min(debug_state.dbg_regs.len());
    let header_byte_count = std::mem::offset_of!(Arm64HardwareDebugState, dbg_regs);
    let register_byte_count = written_slot_count.saturating_mul(std::mem::size_of::<Arm64HardwareDebugRegister>());
    let mut io_vector = libc::iovec {
        iov_base: (debug_state as *mut Arm64HardwareDebugState).cast::<c_void>(),
        iov_len: header_byte_count.saturating_add(register_byte_count),
    };

    ptrace_request(
        libc::PTRACE_SETREGSET,
        thread_id,
        NT_ARM_HW_BREAK as *mut c_void,
        (&mut io_vector as *mut libc::iovec).cast::<c_void>(),
        "PTRACE_SETREGSET NT_ARM_HW_BREAK",
    )
}

#[cfg(all(target_os = "android", target_arch = "aarch64"))]
fn ptrace_get_regset<T>(
    thread_id: pid_t,
    register_set_id: usize,
    register_set: &mut T,
    operation_name: &str,
) -> Result<(), DebuggerPluginError> {
    let mut io_vector = libc::iovec {
        iov_base: (register_set as *mut T).cast::<c_void>(),
        iov_len: std::mem::size_of::<T>(),
    };

    ptrace_request(
        libc::PTRACE_GETREGSET,
        thread_id,
        register_set_id as *mut c_void,
        (&mut io_vector as *mut libc::iovec).cast::<c_void>(),
        operation_name,
    )
}

#[cfg(all(target_os = "android", target_arch = "aarch64"))]
fn ptrace_set_regset<T>(
    thread_id: pid_t,
    register_set_id: usize,
    register_set: &mut T,
    operation_name: &str,
) -> Result<(), DebuggerPluginError> {
    let mut io_vector = libc::iovec {
        iov_base: (register_set as *mut T).cast::<c_void>(),
        iov_len: std::mem::size_of::<T>(),
    };

    ptrace_request(
        libc::PTRACE_SETREGSET,
        thread_id,
        register_set_id as *mut c_void,
        (&mut io_vector as *mut libc::iovec).cast::<c_void>(),
        operation_name,
    )
}

#[cfg(all(target_os = "android", target_arch = "aarch64"))]
fn read_signal_fault_address(thread_id: pid_t) -> Result<Option<u64>, DebuggerPluginError> {
    let mut signal_info = unsafe { zeroed::<libc::siginfo_t>() };

    ptrace_request(
        libc::PTRACE_GETSIGINFO,
        thread_id,
        null_mut(),
        (&mut signal_info as *mut libc::siginfo_t).cast::<c_void>(),
        "PTRACE_GETSIGINFO",
    )?;

    let fault_address = unsafe { signal_info.si_addr() };
    if fault_address.is_null() { Ok(None) } else { Ok(Some(fault_address as u64)) }
}

#[cfg(all(target_os = "android", target_arch = "aarch64"))]
fn validate_arm64_watchpoint(
    address: u64,
    size_in_bytes: u8,
) -> Result<(), DebuggerPluginError> {
    match size_in_bytes {
        1 | 2 | 4 | 8 => {}
        _ => {
            return Err(LinuxDebuggerBackend::plugin_error(format!(
                "Android arm64 hardware data breakpoint size {} is unsupported. Expected 1, 2, 4, or 8 bytes.",
                size_in_bytes
            )));
        }
    }

    let byte_offset = address & (ARM64_WATCH_GRANULE_SIZE - 1);
    if byte_offset + u64::from(size_in_bytes) > ARM64_WATCH_GRANULE_SIZE {
        return Err(LinuxDebuggerBackend::plugin_error(format!(
            "Android arm64 watchpoint at 0x{:X} with size {} crosses an 8-byte watchpoint granule.",
            address, size_in_bytes
        )));
    }

    Ok(())
}

#[cfg(any(test, all(target_os = "android", target_arch = "aarch64")))]
fn arm64_fault_matches_watchpoint(
    fault_address: u64,
    watch_start: u64,
    size_in_bytes: u8,
) -> bool {
    let watch_end = watch_start.saturating_add(u64::from(size_in_bytes));
    let granule_start = watch_start & !(ARM64_WATCH_GRANULE_SIZE - 1);
    let granule_end = granule_start.saturating_add(ARM64_WATCH_GRANULE_SIZE);

    (fault_address >= watch_start && fault_address < watch_end) || (fault_address >= granule_start && fault_address < granule_end)
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn clear_debug_status(process_id: pid_t) -> Result<(), DebuggerPluginError> {
    write_debug_user_offset(process_id, X86_64_DR6_OFFSET, 0)
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn read_debug_register(
    process_id: pid_t,
    debug_register_index: usize,
) -> Result<u64, DebuggerPluginError> {
    let offset = X86_64_DEBUG_REGISTER_BASE_OFFSET + debug_register_index * X86_64_DEBUG_REGISTER_SIZE;
    read_debug_user_offset(process_id, offset)
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn write_debug_register(
    process_id: pid_t,
    debug_register_index: usize,
    value: u64,
) -> Result<(), DebuggerPluginError> {
    let offset = X86_64_DEBUG_REGISTER_BASE_OFFSET + debug_register_index * X86_64_DEBUG_REGISTER_SIZE;
    write_debug_user_offset(process_id, offset, value)
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn read_debug_user_offset(
    process_id: pid_t,
    offset: usize,
) -> Result<u64, DebuggerPluginError> {
    let ptrace_result = ptrace_peek_user(process_id, offset)?;

    Ok(ptrace_result as u64)
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
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

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
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

fn ptrace_seize(
    thread_id: pid_t,
    operation_name: &str,
) -> Result<(), DebuggerPluginError> {
    ptrace_request(
        PTRACE_SEIZE_REQUEST,
        thread_id,
        null_mut(),
        PTRACE_CLONE_TRACE_OPTION as *mut c_void,
        operation_name,
    )
}

fn ptrace_interrupt(
    thread_id: pid_t,
    operation_name: &str,
) -> Result<(), DebuggerPluginError> {
    ptrace_request(PTRACE_INTERRUPT_REQUEST, thread_id, null_mut(), null_mut(), operation_name)
}

fn ptrace_set_clone_tracing(thread_id: pid_t) -> Result<(), DebuggerPluginError> {
    ptrace_request(
        libc::PTRACE_SETOPTIONS as PtraceRequest,
        thread_id,
        null_mut(),
        PTRACE_CLONE_TRACE_OPTION as *mut c_void,
        "PTRACE_SETOPTIONS clone tracing",
    )
}

/// Best-effort thaw of the target's cgroup v2 freezer. Android (`CachedAppOptimizer`) and recent Linux kernels freeze
/// backgrounded tasks; a frozen task never processes the SIGSTOP that `PTRACE_ATTACH` and pausing rely on, so attach
/// hangs and stopped threads cannot make progress. This only ever thaws (writes `0`); it never freezes the target.
fn thaw_process_cgroup_freezer(process_id: pid_t) {
    let Some(freeze_control_path) = resolve_cgroup_v2_freeze_path(process_id) else {
        return;
    };

    // Avoid a needless write (and noisy audit) when the target is already thawed.
    match fs::read_to_string(&freeze_control_path) {
        Ok(current_state) if current_state.trim() == "0" => return,
        Ok(_) => {}
        Err(_) => return,
    }

    if let Err(error) = fs::write(&freeze_control_path, b"0") {
        log::debug!("Failed to thaw cgroup freezer at '{}': {}", freeze_control_path, error);
    }
}

fn resolve_cgroup_v2_freeze_path(process_id: pid_t) -> Option<String> {
    let cgroup_contents = fs::read_to_string(format!("/proc/{}/cgroup", process_id)).ok()?;

    // The unified (v2) hierarchy is the entry with an empty controller list, formatted as "0::<path>".
    for cgroup_line in cgroup_contents.lines() {
        let mut cgroup_fields = cgroup_line.splitn(3, ':');
        let hierarchy_id = cgroup_fields.next()?;
        let controllers = cgroup_fields.next()?;
        let cgroup_path = cgroup_fields.next()?;

        if hierarchy_id != "0" || !controllers.is_empty() {
            continue;
        }

        let normalized_path = cgroup_path.trim_end_matches('/');

        // The root cgroup has no freeze control; only descend into a real app cgroup path.
        if normalized_path.is_empty() {
            return None;
        }

        return Some(format!("/sys/fs/cgroup{}/cgroup.freeze", normalized_path));
    }

    None
}

fn ptrace_request(
    request: PtraceRequest,
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
    // An interrupt/attach stop usually lands within microseconds, but enumerating hundreds of threads with a flat
    // coarse poll added seconds of attach latency. Back off from a tight poll so the common fast case returns almost
    // immediately while a genuinely slow/stuck thread still falls back to a cheap wait.
    let mut poll_interval = Duration::from_micros(100);
    let max_poll_interval = Duration::from_millis(5);

    loop {
        let mut wait_status = 0;
        let wait_result = unsafe { libc::waitpid(process_id, &mut wait_status, WAIT_ALL_TRACED_THREADS) };

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

        thread::sleep(poll_interval);
        poll_interval = (poll_interval * 2).min(max_poll_interval);
    }
}

fn last_os_error(operation_name: &str) -> DebuggerPluginError {
    LinuxDebuggerBackend::plugin_error(format!("{} failed: {}.", operation_name, std::io::Error::last_os_error()))
}

fn clear_errno() {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    unsafe {
        *libc::__errno_location() = 0;
    }

    #[cfg(all(target_os = "android", target_arch = "aarch64"))]
    unsafe {
        *libc::__errno() = 0;
    }
}

fn current_errno() -> i32 {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    unsafe {
        *libc::__errno_location()
    }

    #[cfg(all(target_os = "android", target_arch = "aarch64"))]
    unsafe {
        *libc::__errno()
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn x64_watchpoint_access_bits(access: DebuggerDataBreakpointAccess) -> u64 {
    match access {
        DebuggerDataBreakpointAccess::Write => 0b01,
        DebuggerDataBreakpointAccess::Read | DebuggerDataBreakpointAccess::ReadWrite => 0b11,
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
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

#[cfg(any(test, all(target_os = "android", target_arch = "aarch64")))]
fn arm64_watch_control(
    access: DebuggerDataBreakpointAccess,
    byte_mask: u64,
) -> u32 {
    const BYTE_ADDRESS_SELECT_SHIFT: u64 = 5;
    const ENABLE: u64 = 1 << 0;
    const LOAD_STORE_CONTROL_SHIFT: u64 = 3;
    const USER_ACCESS_ONLY: u64 = 0b10 << 1;
    let load_store_control = match access {
        DebuggerDataBreakpointAccess::Read => 0b01,
        DebuggerDataBreakpointAccess::Write => 0b10,
        DebuggerDataBreakpointAccess::ReadWrite => 0b11,
    };

    (ENABLE | USER_ACCESS_ONLY | (load_store_control << LOAD_STORE_CONTROL_SHIFT) | (byte_mask << BYTE_ADDRESS_SELECT_SHIFT)) as u32
}

/// Control word (DBGBCR) for an EL0 instruction (execute) breakpoint matching a whole 4-byte instruction:
/// enable, user-mode (EL0) privilege, byte-address-select = 0xF, breakpoint type = unlinked instruction address match.
#[cfg(any(test, all(target_os = "android", target_arch = "aarch64")))]
fn arm64_breakpoint_control() -> u32 {
    const ENABLE: u64 = 1 << 0;
    const USER_ACCESS_ONLY: u64 = 0b10 << 1;
    const BYTE_ADDRESS_SELECT_SHIFT: u64 = 5;
    const BYTE_ADDRESS_SELECT_ALL: u64 = 0xF;

    (ENABLE | USER_ACCESS_ONLY | (BYTE_ADDRESS_SELECT_ALL << BYTE_ADDRESS_SELECT_SHIFT)) as u32
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn resolve_x64_post_trap_instruction_from_window(
    post_trap_instruction_pointer: u64,
    window_start_address: u64,
    window_bytes: &[u8],
) -> Option<(u64, Vec<u8>)> {
    let mut resolved_instruction = None;

    for candidate_offset in 0..window_bytes.len() {
        let candidate_address = window_start_address.saturating_add(candidate_offset as u64);
        let candidate_bytes = &window_bytes[candidate_offset..];
        let mut decoder = Decoder::with_ip(64, candidate_bytes, candidate_address, DecoderOptions::NONE);
        let instruction = decoder.decode();
        let instruction_length = instruction.len();

        if instruction.is_invalid() || instruction_length == 0 {
            continue;
        }

        if candidate_address.saturating_add(instruction_length as u64) == post_trap_instruction_pointer {
            resolved_instruction = Some((candidate_address, candidate_bytes[..instruction_length].to_vec()));
            break;
        }
    }

    resolved_instruction
}

#[cfg(test)]
mod tests {
    use super::{arm64_fault_matches_watchpoint, arm64_watch_control, is_linux_task_stale_attach_error};
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    use super::{resolve_x64_post_trap_instruction_from_window, x64_watchpoint_access_bits, x64_watchpoint_length_bits};
    use squalr_engine_api::plugins::debugger::DebuggerPluginError;
    use squalr_engine_api::structures::debugger::DebuggerDataBreakpointAccess;

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    #[test]
    fn x64_watchpoint_lengths_match_debug_register_encoding() {
        assert_eq!(x64_watchpoint_length_bits(1).unwrap_or(u64::MAX), 0b00);
        assert_eq!(x64_watchpoint_length_bits(2).unwrap_or(u64::MAX), 0b01);
        assert_eq!(x64_watchpoint_length_bits(4).unwrap_or(u64::MAX), 0b11);
        assert_eq!(x64_watchpoint_length_bits(8).unwrap_or(u64::MAX), 0b10);
        assert!(x64_watchpoint_length_bits(3).is_err());
    }

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    #[test]
    fn x64_watchpoint_accesses_match_debug_register_encoding() {
        assert_eq!(x64_watchpoint_access_bits(DebuggerDataBreakpointAccess::Write), 0b01);
        assert_eq!(x64_watchpoint_access_bits(DebuggerDataBreakpointAccess::Read), 0b11);
        assert_eq!(x64_watchpoint_access_bits(DebuggerDataBreakpointAccess::ReadWrite), 0b11);
    }

    #[test]
    fn arm64_watch_control_sets_access_and_byte_mask() {
        let control = arm64_watch_control(DebuggerDataBreakpointAccess::Write, 0b1111);

        assert_eq!(control & 1, 1);
        assert_eq!((control >> 1) & 0b11, 0b10);
        assert_eq!((control >> 3) & 0b11, 0b10);
        assert_eq!((control >> 5) & 0xff, 0b1111);
    }

    #[test]
    fn arm64_fault_matching_accepts_reported_granule_base_for_offset_watchpoint() {
        let watch_start = 0x1004;

        assert!(arm64_fault_matches_watchpoint(0x1004, watch_start, 4));
        assert!(arm64_fault_matches_watchpoint(0x1000, watch_start, 4));
        assert!(!arm64_fault_matches_watchpoint(0x1008, watch_start, 4));
    }

    #[test]
    fn stale_attach_filter_skips_only_missing_non_leader_threads() {
        let missing_thread_error = DebuggerPluginError::new("test.debugger", "PTRACE_ATTACH thread 12 in process 10 failed: No such process (os error 3).");

        assert!(is_linux_task_stale_attach_error(&missing_thread_error, 10, 12));
        assert!(!is_linux_task_stale_attach_error(&missing_thread_error, 10, 10));
    }

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    #[test]
    fn resolves_x64_post_trap_memory_instruction_from_prior_bytes() {
        let window_start_address = 0x1000;
        let post_trap_instruction_pointer = 0x1004;
        let window_bytes = [0x90, 0x48, 0xFF, 0x00];

        let resolved_instruction = resolve_x64_post_trap_instruction_from_window(post_trap_instruction_pointer, window_start_address, &window_bytes);

        assert_eq!(resolved_instruction, Some((0x1001, vec![0x48, 0xFF, 0x00])));
    }
}
