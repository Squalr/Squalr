use crate::constants::NATIVE_DEBUGGERS_PLUGIN_ID;
use libc::{SIGSTOP, WNOHANG, c_char, pid_t};
use mach2::{
    kern_return::KERN_SUCCESS,
    mach_port::mach_port_deallocate,
    mach_types::{thread_act_array_t, thread_act_t},
    message::mach_msg_type_number_t,
    port::{MACH_PORT_NULL, mach_port_t},
    task::{task_resume, task_threads},
    thread_act::{thread_get_state, thread_set_state},
    thread_status::thread_state_t,
    traps::mach_task_self,
    vm::mach_vm_deallocate,
    vm::mach_vm_read_overwrite,
    vm_types::{mach_vm_address_t, mach_vm_size_t},
};
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
    mem::{size_of, zeroed},
    ptr::null_mut,
    sync::mpsc::{self, Receiver, RecvTimeoutError, Sender},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

const PT_CONTINUE: libc::c_int = 7;
const PT_DETACH: libc::c_int = 11;
const PT_ATTACHEXC: libc::c_int = 14;

const X86_THREAD_STATE64: libc::c_int = 4;
const X86_DEBUG_STATE64: libc::c_int = 11;
const ARM_THREAD_STATE64: libc::c_int = 6;
const ARM_DEBUG_STATE64: libc::c_int = 15;

const TRACE_INSTRUCTION_BYTE_WINDOW: usize = 16;
const RUNNING_EVENT_WAIT_TIMEOUT_MS: u64 = 50;
const ATTACH_WAIT_TIMEOUT: Duration = Duration::from_secs(10);
const WATCHPOINT_SLOT_COUNT: usize = 4;
const ARM64_WATCH_GRANULE_SIZE: u64 = 8;

pub(crate) struct MacOsDebuggerBackend {
    process_info: OpenedProcessInfo,
    trace_event_sink: DebuggerTraceEventSink,
    worker_handle: Option<MacOsWorkerHandle>,
}

impl MacOsDebuggerBackend {
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
        let thread_handle = thread::spawn(move || macos_worker_main(process_info, trace_event_sink, worker_ready_sender));
        let worker_command_sender = match worker_ready_receiver
            .recv()
            .map_err(|error| Self::plugin_error(format!("macOS debugger worker exited before reporting attach status: {}", error)))?
        {
            Ok(worker_command_sender) => worker_command_sender,
            Err(error) => {
                let _ = thread_handle.join();

                return Err(error);
            }
        };

        self.worker_handle = Some(MacOsWorkerHandle {
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
            .ok_or_else(|| Self::plugin_error("Cannot pause because there is no active macOS debugger worker."))?
            .pause()
    }

    pub(crate) fn resume(&self) -> Result<(), DebuggerPluginError> {
        self.worker_handle
            .as_ref()
            .ok_or_else(|| Self::plugin_error("Cannot resume because there is no active macOS debugger worker."))?
            .resume()
    }

    pub(crate) fn read_registers(&self) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        self.worker_handle
            .as_ref()
            .ok_or_else(|| Self::plugin_error("Cannot read registers because there is no active macOS debugger worker."))?
            .read_registers()
    }

    pub(crate) fn write_register(
        &self,
        register_name: &str,
        value: u64,
    ) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        self.worker_handle
            .as_ref()
            .ok_or_else(|| Self::plugin_error("Cannot write registers because there is no active macOS debugger worker."))?
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
            .ok_or_else(|| Self::plugin_error("Cannot set a breakpoint because there is no active macOS debugger worker."))?
            .set_breakpoint(address, kind, label)
    }

    pub(crate) fn remove_breakpoint(
        &self,
        breakpoint_id: &str,
    ) -> Result<(), DebuggerPluginError> {
        self.worker_handle
            .as_ref()
            .ok_or_else(|| Self::plugin_error("Cannot remove a breakpoint because there is no active macOS debugger worker."))?
            .remove_breakpoint(breakpoint_id)
    }

    pub(crate) fn set_breakpoint_enabled(
        &self,
        breakpoint_id: &str,
        is_enabled: bool,
    ) -> Result<(), DebuggerPluginError> {
        self.worker_handle
            .as_ref()
            .ok_or_else(|| Self::plugin_error("Cannot update a breakpoint because there is no active macOS debugger worker."))?
            .set_breakpoint_enabled(breakpoint_id, is_enabled)
    }

    pub(crate) fn list_breakpoints(&self) -> Result<Vec<DebuggerBreakpointDescriptor>, DebuggerPluginError> {
        self.worker_handle
            .as_ref()
            .ok_or_else(|| Self::plugin_error("Cannot list breakpoints because there is no active macOS debugger worker."))?
            .list_breakpoints()
    }

    fn plugin_error(message: impl Into<String>) -> DebuggerPluginError {
        DebuggerPluginError::new(NATIVE_DEBUGGERS_PLUGIN_ID, message)
    }
}

impl Drop for MacOsDebuggerBackend {
    fn drop(&mut self) {
        if let Err(error) = self.detach() {
            log::debug!("Failed to detach macOS debugger backend during drop: {}", error);
        }
    }
}

struct MacOsWorkerHandle {
    command_sender: Sender<MacOsWorkerCommand>,
    thread_handle: Option<JoinHandle<()>>,
}

impl MacOsWorkerHandle {
    fn pause(&self) -> Result<(), DebuggerPluginError> {
        let (result_sender, result_receiver) = mpsc::channel();

        self.command_sender
            .send(MacOsWorkerCommand::Pause { result_sender })
            .map_err(|error| MacOsDebuggerBackend::plugin_error(format!("Failed to request macOS debugger pause: {}", error)))?;

        result_receiver
            .recv()
            .map_err(|error| MacOsDebuggerBackend::plugin_error(format!("macOS debugger worker exited before pause completed: {}", error)))?
    }

    fn resume(&self) -> Result<(), DebuggerPluginError> {
        let (result_sender, result_receiver) = mpsc::channel();

        self.command_sender
            .send(MacOsWorkerCommand::Resume { result_sender })
            .map_err(|error| MacOsDebuggerBackend::plugin_error(format!("Failed to request macOS debugger resume: {}", error)))?;

        result_receiver
            .recv()
            .map_err(|error| MacOsDebuggerBackend::plugin_error(format!("macOS debugger worker exited before resume completed: {}", error)))?
    }

    fn read_registers(&self) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        let (result_sender, result_receiver) = mpsc::channel();

        self.command_sender
            .send(MacOsWorkerCommand::ReadRegisters { result_sender })
            .map_err(|error| MacOsDebuggerBackend::plugin_error(format!("Failed to request macOS register snapshot: {}", error)))?;

        result_receiver
            .recv()
            .map_err(|error| MacOsDebuggerBackend::plugin_error(format!("macOS debugger worker exited before register snapshot completed: {}", error)))?
    }

    fn write_register(
        &self,
        register_name: &str,
        value: u64,
    ) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        let (result_sender, result_receiver) = mpsc::channel();

        self.command_sender
            .send(MacOsWorkerCommand::WriteRegister {
                register_name: register_name.to_string(),
                value,
                result_sender,
            })
            .map_err(|error| MacOsDebuggerBackend::plugin_error(format!("Failed to request macOS register write: {}", error)))?;

        result_receiver
            .recv()
            .map_err(|error| MacOsDebuggerBackend::plugin_error(format!("macOS debugger worker exited before register write completed: {}", error)))?
    }

    fn set_breakpoint(
        &self,
        address: u64,
        kind: DebuggerBreakpointKind,
        label: Option<String>,
    ) -> Result<DebuggerBreakpointDescriptor, DebuggerPluginError> {
        let (result_sender, result_receiver) = mpsc::channel();

        self.command_sender
            .send(MacOsWorkerCommand::SetBreakpoint {
                address,
                kind,
                label,
                result_sender,
            })
            .map_err(|error| MacOsDebuggerBackend::plugin_error(format!("Failed to request macOS breakpoint set: {}", error)))?;

        result_receiver
            .recv()
            .map_err(|error| MacOsDebuggerBackend::plugin_error(format!("macOS debugger worker exited before breakpoint set completed: {}", error)))?
    }

    fn remove_breakpoint(
        &self,
        breakpoint_id: &str,
    ) -> Result<(), DebuggerPluginError> {
        let (result_sender, result_receiver) = mpsc::channel();

        self.command_sender
            .send(MacOsWorkerCommand::RemoveBreakpoint {
                breakpoint_id: breakpoint_id.to_string(),
                result_sender,
            })
            .map_err(|error| MacOsDebuggerBackend::plugin_error(format!("Failed to request macOS breakpoint removal: {}", error)))?;

        result_receiver
            .recv()
            .map_err(|error| MacOsDebuggerBackend::plugin_error(format!("macOS debugger worker exited before breakpoint removal completed: {}", error)))?
    }

    fn set_breakpoint_enabled(
        &self,
        breakpoint_id: &str,
        is_enabled: bool,
    ) -> Result<(), DebuggerPluginError> {
        let (result_sender, result_receiver) = mpsc::channel();

        self.command_sender
            .send(MacOsWorkerCommand::SetBreakpointEnabled {
                breakpoint_id: breakpoint_id.to_string(),
                is_enabled,
                result_sender,
            })
            .map_err(|error| MacOsDebuggerBackend::plugin_error(format!("Failed to request macOS breakpoint state update: {}", error)))?;

        result_receiver
            .recv()
            .map_err(|error| MacOsDebuggerBackend::plugin_error(format!("macOS debugger worker exited before breakpoint state update completed: {}", error)))?
    }

    fn list_breakpoints(&self) -> Result<Vec<DebuggerBreakpointDescriptor>, DebuggerPluginError> {
        let (result_sender, result_receiver) = mpsc::channel();

        self.command_sender
            .send(MacOsWorkerCommand::ListBreakpoints { result_sender })
            .map_err(|error| MacOsDebuggerBackend::plugin_error(format!("Failed to request macOS breakpoint list: {}", error)))?;

        result_receiver
            .recv()
            .map_err(|error| MacOsDebuggerBackend::plugin_error(format!("macOS debugger worker exited before breakpoint list completed: {}", error)))?
    }

    fn detach(&mut self) -> Result<(), DebuggerPluginError> {
        let (result_sender, result_receiver) = mpsc::channel();

        self.command_sender
            .send(MacOsWorkerCommand::Detach { result_sender })
            .map_err(|error| MacOsDebuggerBackend::plugin_error(format!("Failed to request macOS debugger detach: {}", error)))?;

        let detach_result = result_receiver
            .recv()
            .map_err(|error| MacOsDebuggerBackend::plugin_error(format!("macOS debugger worker exited before detach completed: {}", error)))?;

        if let Some(thread_handle) = self.thread_handle.take() {
            thread_handle
                .join()
                .map_err(|_| MacOsDebuggerBackend::plugin_error("macOS debugger worker thread panicked during detach."))?;
        }

        detach_result
    }
}

enum MacOsWorkerCommand {
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

struct ActiveMacOsSession {
    process_id: pid_t,
    task_port: mach_port_t,
    target_architecture: TargetArchitecture,
    breakpoints_by_id: HashMap<String, StoredMacOsBreakpoint>,
    session_state: DebuggerSessionState,
    next_breakpoint_number: u64,
    trace_event_sink: DebuggerTraceEventSink,
}

#[derive(Clone)]
struct StoredMacOsBreakpoint {
    descriptor: DebuggerBreakpointDescriptor,
    slot: usize,
}

impl ActiveMacOsSession {
    fn attach(
        process_info: &OpenedProcessInfo,
        trace_event_sink: DebuggerTraceEventSink,
    ) -> Result<Self, DebuggerPluginError> {
        let process_id = process_info.get_process_id() as pid_t;
        ptrace_request(PT_ATTACHEXC, process_id, null_mut(), 0, "PT_ATTACHEXC attach")?;
        wait_for_stop(process_id, ATTACH_WAIT_TIMEOUT, "initial attach")?;

        Ok(Self {
            process_id,
            task_port: process_info.get_handle() as mach_port_t,
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
            return Err(last_os_error("SIGSTOP debugger pause"));
        }

        wait_for_stop(self.process_id, ATTACH_WAIT_TIMEOUT, "debugger pause")?;

        self.session_state = DebuggerSessionState::Paused;

        Ok(())
    }

    fn resume(&mut self) -> Result<(), DebuggerPluginError> {
        if self.session_state == DebuggerSessionState::Running {
            return Ok(());
        }

        self.refresh_threads_and_apply_watchpoints()?;

        let resume_status = unsafe { task_resume(self.task_port) };
        if resume_status != KERN_SUCCESS {
            log::debug!("task_resume returned status {} before ptrace continue.", resume_status);
        }

        ptrace_request(PT_CONTINUE, self.process_id, 1usize as *mut c_char, 0, "PT_CONTINUE resume")?;
        self.session_state = DebuggerSessionState::Running;

        Ok(())
    }

    fn read_registers(&self) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        let thread_list = ThreadList::for_task(self.task_port)?;
        let event_thread = thread_list
            .threads()
            .first()
            .copied()
            .ok_or_else(|| MacOsDebuggerBackend::plugin_error("Cannot read registers because the target has no threads."))?;

        self.read_registers_for_thread(event_thread)
    }

    fn write_register(
        &mut self,
        register_name: &str,
        value: u64,
    ) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        let thread_list = ThreadList::for_task(self.task_port)?;
        let event_thread = thread_list
            .threads()
            .first()
            .copied()
            .ok_or_else(|| MacOsDebuggerBackend::plugin_error("Cannot write registers because the target has no threads."))?;

        match self.target_architecture.get_instruction_set_id() {
            "x64" => self.write_x64_register(event_thread, register_name, value)?,
            "arm64" => self.write_arm64_register(event_thread, register_name, value)?,
            instruction_set_id => {
                return Err(MacOsDebuggerBackend::plugin_error(format!(
                    "macOS debugger register writes are unsupported for architecture '{}'.",
                    instruction_set_id
                )));
            }
        }

        self.read_registers_for_thread(event_thread)
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
            return Err(MacOsDebuggerBackend::plugin_error(
                "The macOS native debugger backend currently supports hardware data breakpoints only.",
            ));
        }

        self.validate_hardware_breakpoint(address, &kind)?;

        let slot = self.allocate_watchpoint_slot()?;
        let breakpoint_id = format!("mach-{}", self.next_breakpoint_number);
        self.next_breakpoint_number = self.next_breakpoint_number.saturating_add(1);
        let descriptor = DebuggerBreakpointDescriptor::new(breakpoint_id.clone(), address, kind, true, label);

        self.breakpoints_by_id.insert(
            breakpoint_id.clone(),
            StoredMacOsBreakpoint {
                descriptor: descriptor.clone(),
                slot,
            },
        );

        if let Err(error) = self.refresh_threads_and_apply_watchpoints() {
            self.breakpoints_by_id.remove(&breakpoint_id);
            let _ = self.refresh_threads_and_apply_watchpoints();

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
                return Err(MacOsDebuggerBackend::plugin_error(format!(
                    "macOS breakpoint '{}' does not exist.",
                    breakpoint_id
                )));
            }

            active_session.refresh_threads_and_apply_watchpoints()
        })
    }

    fn set_breakpoint_enabled(
        &mut self,
        breakpoint_id: &str,
        is_enabled: bool,
    ) -> Result<(), DebuggerPluginError> {
        self.run_paused_for_breakpoint_mutation("update breakpoint", |active_session| {
            let stored_breakpoint = active_session
                .breakpoints_by_id
                .get_mut(breakpoint_id)
                .ok_or_else(|| MacOsDebuggerBackend::plugin_error(format!("macOS breakpoint '{}' does not exist.", breakpoint_id)))?;

            stored_breakpoint.descriptor.set_is_enabled(is_enabled);
            active_session.refresh_threads_and_apply_watchpoints()
        })
    }

    fn list_breakpoints(&self) -> Vec<DebuggerBreakpointDescriptor> {
        let mut breakpoints = self
            .breakpoints_by_id
            .values()
            .map(|stored_breakpoint| stored_breakpoint.descriptor.clone())
            .collect::<Vec<_>>();

        breakpoints.sort_by(|left, right| left.get_breakpoint_id().cmp(right.get_breakpoint_id()));
        breakpoints
    }

    fn detach(&mut self) -> Result<(), DebuggerPluginError> {
        self.breakpoints_by_id.clear();
        let _ = self.refresh_threads_and_apply_watchpoints();
        let _ = unsafe { task_resume(self.task_port) };
        let detach_result = ptrace_request(PT_DETACH, self.process_id, null_mut(), 0, "PT_DETACH detach");
        self.session_state = DebuggerSessionState::Detached;

        detach_result
    }

    fn run_paused_for_breakpoint_mutation<T>(
        &mut self,
        operation_name: &str,
        mutation: impl FnOnce(&mut Self) -> Result<T, DebuggerPluginError>,
    ) -> Result<T, DebuggerPluginError> {
        let should_resume_after_mutation = self.session_state == DebuggerSessionState::Running;

        if should_resume_after_mutation {
            self.pause()
                .map_err(|error| MacOsDebuggerBackend::plugin_error(format!("Failed to pause before macOS {}: {}", operation_name, error)))?;
        }

        let mutation_result = mutation(self);
        let resume_result = if should_resume_after_mutation {
            self.resume()
                .map_err(|error| MacOsDebuggerBackend::plugin_error(format!("Failed to resume after macOS {}: {}", operation_name, error)))
        } else {
            Ok(())
        };

        match (mutation_result, resume_result) {
            (Ok(value), Ok(())) => Ok(value),
            (Err(error), _) => Err(error),
            (Ok(_), Err(error)) => Err(error),
        }
    }

    fn process_pending_debug_event(&mut self) -> Result<(), DebuggerPluginError> {
        if self.session_state != DebuggerSessionState::Running {
            return Ok(());
        }

        let mut wait_status = 0;
        let wait_result = unsafe { libc::waitpid(self.process_id, &mut wait_status, WNOHANG) };

        if wait_result == 0 {
            return Ok(());
        }

        if wait_result < 0 {
            return Err(last_os_error("waitpid while polling macOS debug events"));
        }

        if !libc::WIFSTOPPED(wait_status) {
            return Ok(());
        }

        self.session_state = DebuggerSessionState::Paused;
        self.handle_breakpoint_stop()?;

        if self
            .breakpoints_by_id
            .values()
            .any(|stored_breakpoint| stored_breakpoint.descriptor.get_is_enabled())
        {
            self.resume()?;
        }

        Ok(())
    }

    fn handle_breakpoint_stop(&mut self) -> Result<(), DebuggerPluginError> {
        let thread_list = ThreadList::for_task(self.task_port)?;
        let event_thread = self.select_event_thread(thread_list.threads())?;
        let register_snapshot = self.read_registers_for_thread(event_thread).unwrap_or_default();
        let (breakpoint_descriptor, backend_message) = self.describe_hit_breakpoint(event_thread);
        let instruction_pointer = register_snapshot.get_instruction_pointer();
        let instruction_address = self.resolve_instruction_address(instruction_pointer);
        let instruction_bytes = self.read_instruction_bytes(instruction_address);

        (self.trace_event_sink)(DebuggerTraceEvent::new(
            breakpoint_descriptor,
            register_snapshot,
            instruction_address,
            instruction_bytes,
            None,
            backend_message,
        ));

        Ok(())
    }

    fn describe_hit_breakpoint(
        &self,
        event_thread: thread_act_t,
    ) -> (Option<DebuggerBreakpointDescriptor>, Option<String>) {
        if self.target_architecture.get_instruction_set_id() == "x64" {
            if let Ok(debug_state) = self.get_x64_debug_state(event_thread) {
                for stored_breakpoint in self.breakpoints_by_id.values() {
                    if stored_breakpoint.descriptor.get_is_enabled() && debug_state.dr6 & (1u64 << stored_breakpoint.slot) != 0 {
                        return (
                            Some(stored_breakpoint.descriptor.clone()),
                            Some(String::from(
                                "macOS hardware data breakpoint hit. x86_64 reports the instruction pointer after the trapped access; attribution needs human verification.",
                            )),
                        );
                    }
                }
            }
        }

        let fallback_breakpoint = self
            .breakpoints_by_id
            .values()
            .find(|stored_breakpoint| stored_breakpoint.descriptor.get_is_enabled())
            .map(|stored_breakpoint| stored_breakpoint.descriptor.clone());

        (
            fallback_breakpoint,
            Some(String::from(
                "macOS hardware data breakpoint hit. Event thread attribution uses the first stopped thread in this MVP backend.",
            )),
        )
    }

    fn select_event_thread(
        &self,
        threads: &[thread_act_t],
    ) -> Result<thread_act_t, DebuggerPluginError> {
        if self.target_architecture.get_instruction_set_id() == "x64" {
            for thread in threads {
                if let Ok(debug_state) = self.get_x64_debug_state(*thread) {
                    let has_hit_slot = self
                        .breakpoints_by_id
                        .values()
                        .any(|stored_breakpoint| stored_breakpoint.descriptor.get_is_enabled() && debug_state.dr6 & (1u64 << stored_breakpoint.slot) != 0);

                    if has_hit_slot {
                        return Ok(*thread);
                    }
                }
            }
        }

        threads
            .first()
            .copied()
            .ok_or_else(|| MacOsDebuggerBackend::plugin_error("Cannot select a macOS event thread because the target has no threads."))
    }

    fn read_registers_for_thread(
        &self,
        thread: thread_act_t,
    ) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        match self.target_architecture.get_instruction_set_id() {
            "x64" => self.read_x64_registers(thread),
            "arm64" => self.read_arm64_registers(thread),
            instruction_set_id => Err(MacOsDebuggerBackend::plugin_error(format!(
                "macOS debugger register snapshots are unsupported for architecture '{}'.",
                instruction_set_id
            ))),
        }
    }

    fn refresh_threads_and_apply_watchpoints(&self) -> Result<(), DebuggerPluginError> {
        let thread_list = ThreadList::for_task(self.task_port)?;

        for thread in thread_list.threads() {
            match self.target_architecture.get_instruction_set_id() {
                "x64" => self.apply_x64_watchpoints(*thread)?,
                "arm64" => self.apply_arm64_watchpoints(*thread)?,
                instruction_set_id => {
                    return Err(MacOsDebuggerBackend::plugin_error(format!(
                        "macOS debugger watchpoints are unsupported for architecture '{}'.",
                        instruction_set_id
                    )));
                }
            }
        }

        Ok(())
    }

    fn apply_x64_watchpoints(
        &self,
        thread: thread_act_t,
    ) -> Result<(), DebuggerPluginError> {
        let mut debug_state = X64DebugState::default();

        debug_state.dr6 = 0;
        debug_state.dr7 = 0;

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

            debug_state.set_slot_address(slot, stored_breakpoint.descriptor.get_address())?;
            debug_state.dr7 |= 1u64 << (slot * 2);
            debug_state.dr7 |= access_bits << (16 + slot * 4);
            debug_state.dr7 |= length_bits << (18 + slot * 4);
        }

        self.set_x64_debug_state(thread, debug_state)
    }

    fn apply_arm64_watchpoints(
        &self,
        thread: thread_act_t,
    ) -> Result<(), DebuggerPluginError> {
        let mut debug_state = Arm64DebugState::default();

        for stored_breakpoint in self.breakpoints_by_id.values() {
            if !stored_breakpoint.descriptor.get_is_enabled() {
                continue;
            }

            let DebuggerBreakpointKind::HardwareData { access, size_in_bytes } = *stored_breakpoint.descriptor.get_kind() else {
                continue;
            };
            let address = stored_breakpoint.descriptor.get_address();
            let granule_base = address & !(ARM64_WATCH_GRANULE_SIZE - 1);
            let byte_offset = address - granule_base;
            let byte_count = u64::from(size_in_bytes);
            let byte_mask = ((1u64 << byte_count) - 1) << byte_offset;
            let slot = stored_breakpoint.slot;

            debug_state.wvr[slot] = granule_base;
            debug_state.wcr[slot] = arm64_watch_control(access, byte_mask);
        }

        self.set_arm64_debug_state(thread, debug_state)
    }

    fn validate_hardware_breakpoint(
        &self,
        address: u64,
        kind: &DebuggerBreakpointKind,
    ) -> Result<(), DebuggerPluginError> {
        let DebuggerBreakpointKind::HardwareData { size_in_bytes, .. } = kind else {
            return Ok(());
        };

        match size_in_bytes {
            1 | 2 | 4 | 8 => {}
            _ => {
                return Err(MacOsDebuggerBackend::plugin_error(format!(
                    "Hardware data breakpoint size {} is unsupported. Expected 1, 2, 4, or 8 bytes.",
                    size_in_bytes
                )));
            }
        }

        if self.target_architecture.get_instruction_set_id() == "arm64" {
            let byte_offset = address & (ARM64_WATCH_GRANULE_SIZE - 1);
            if byte_offset + u64::from(*size_in_bytes) > ARM64_WATCH_GRANULE_SIZE {
                return Err(MacOsDebuggerBackend::plugin_error(format!(
                    "ARM64 watchpoint at 0x{:X} with size {} crosses an 8-byte watchpoint granule.",
                    address, size_in_bytes
                )));
            }
        }

        Ok(())
    }

    fn allocate_watchpoint_slot(&self) -> Result<usize, DebuggerPluginError> {
        for slot in 0..WATCHPOINT_SLOT_COUNT {
            if !self
                .breakpoints_by_id
                .values()
                .any(|stored_breakpoint| stored_breakpoint.slot == slot)
            {
                return Ok(slot);
            }
        }

        Err(MacOsDebuggerBackend::plugin_error(format!(
            "No macOS hardware watchpoint slots are available. This backend currently uses {} slots.",
            WATCHPOINT_SLOT_COUNT
        )))
    }

    fn resolve_instruction_address(
        &self,
        instruction_pointer: Option<u64>,
    ) -> Option<u64> {
        let instruction_pointer = instruction_pointer?;

        match self.target_architecture.get_instruction_set_id() {
            "x64" => Some(instruction_pointer),
            "arm64" => Some(instruction_pointer),
            _ => Some(instruction_pointer),
        }
    }

    fn read_instruction_bytes(
        &self,
        instruction_address: Option<u64>,
    ) -> Vec<u8> {
        let Some(instruction_address) = instruction_address else {
            return Vec::new();
        };
        let mut instruction_bytes = vec![0u8; TRACE_INSTRUCTION_BYTE_WINDOW];
        let mut copied_bytes: mach_vm_size_t = 0;
        let read_status = unsafe {
            mach_vm_read_overwrite(
                self.task_port,
                instruction_address as mach_vm_address_t,
                instruction_bytes.len() as mach_vm_size_t,
                instruction_bytes.as_mut_ptr() as mach_vm_address_t,
                &mut copied_bytes as *mut mach_vm_size_t,
            )
        };

        if read_status != KERN_SUCCESS {
            log::debug!(
                "mach_vm_read_overwrite failed while reading instruction bytes at 0x{:X}: {}.",
                instruction_address,
                read_status
            );

            return Vec::new();
        }

        instruction_bytes.truncate(copied_bytes as usize);
        instruction_bytes
    }

    fn get_x64_thread_state(
        &self,
        thread: thread_act_t,
    ) -> Result<X64ThreadState, DebuggerPluginError> {
        let mut thread_state = X64ThreadState::default();
        let mut state_count = state_count::<X64ThreadState>();
        let state_status = unsafe { thread_get_state(thread, X86_THREAD_STATE64, (&mut thread_state as *mut X64ThreadState).cast(), &mut state_count) };

        if state_status == KERN_SUCCESS {
            Ok(thread_state)
        } else {
            Err(MacOsDebuggerBackend::plugin_error(format!(
                "thread_get_state(x86_THREAD_STATE64) failed with status {}.",
                state_status
            )))
        }
    }

    fn set_x64_thread_state(
        &self,
        thread: thread_act_t,
        thread_state: X64ThreadState,
    ) -> Result<(), DebuggerPluginError> {
        let state_status = unsafe {
            thread_set_state(
                thread,
                X86_THREAD_STATE64,
                (&thread_state as *const X64ThreadState).cast::<u32>() as thread_state_t,
                state_count::<X64ThreadState>(),
            )
        };

        if state_status == KERN_SUCCESS {
            Ok(())
        } else {
            Err(MacOsDebuggerBackend::plugin_error(format!(
                "thread_set_state(x86_THREAD_STATE64) failed with status {}.",
                state_status
            )))
        }
    }

    fn get_x64_debug_state(
        &self,
        thread: thread_act_t,
    ) -> Result<X64DebugState, DebuggerPluginError> {
        let mut debug_state = X64DebugState::default();
        let mut state_count = state_count::<X64DebugState>();
        let state_status = unsafe { thread_get_state(thread, X86_DEBUG_STATE64, (&mut debug_state as *mut X64DebugState).cast(), &mut state_count) };

        if state_status == KERN_SUCCESS {
            Ok(debug_state)
        } else {
            Err(MacOsDebuggerBackend::plugin_error(format!(
                "thread_get_state(x86_DEBUG_STATE64) failed with status {}.",
                state_status
            )))
        }
    }

    fn set_x64_debug_state(
        &self,
        thread: thread_act_t,
        debug_state: X64DebugState,
    ) -> Result<(), DebuggerPluginError> {
        let state_status = unsafe {
            thread_set_state(
                thread,
                X86_DEBUG_STATE64,
                (&debug_state as *const X64DebugState).cast::<u32>() as thread_state_t,
                state_count::<X64DebugState>(),
            )
        };

        if state_status == KERN_SUCCESS {
            Ok(())
        } else {
            Err(MacOsDebuggerBackend::plugin_error(format!(
                "thread_set_state(x86_DEBUG_STATE64) failed with status {}.",
                state_status
            )))
        }
    }

    fn get_arm64_thread_state(
        &self,
        thread: thread_act_t,
    ) -> Result<Arm64ThreadState, DebuggerPluginError> {
        let mut thread_state = Arm64ThreadState::default();
        let mut state_count = state_count::<Arm64ThreadState>();
        let state_status = unsafe {
            thread_get_state(
                thread,
                ARM_THREAD_STATE64,
                (&mut thread_state as *mut Arm64ThreadState).cast(),
                &mut state_count,
            )
        };

        if state_status == KERN_SUCCESS {
            Ok(thread_state)
        } else {
            Err(MacOsDebuggerBackend::plugin_error(format!(
                "thread_get_state(ARM_THREAD_STATE64) failed with status {}.",
                state_status
            )))
        }
    }

    fn set_arm64_thread_state(
        &self,
        thread: thread_act_t,
        thread_state: Arm64ThreadState,
    ) -> Result<(), DebuggerPluginError> {
        let state_status = unsafe {
            thread_set_state(
                thread,
                ARM_THREAD_STATE64,
                (&thread_state as *const Arm64ThreadState).cast::<u32>() as thread_state_t,
                state_count::<Arm64ThreadState>(),
            )
        };

        if state_status == KERN_SUCCESS {
            Ok(())
        } else {
            Err(MacOsDebuggerBackend::plugin_error(format!(
                "thread_set_state(ARM_THREAD_STATE64) failed with status {}.",
                state_status
            )))
        }
    }

    fn set_arm64_debug_state(
        &self,
        thread: thread_act_t,
        debug_state: Arm64DebugState,
    ) -> Result<(), DebuggerPluginError> {
        let state_status = unsafe {
            thread_set_state(
                thread,
                ARM_DEBUG_STATE64,
                (&debug_state as *const Arm64DebugState).cast::<u32>() as thread_state_t,
                state_count::<Arm64DebugState>(),
            )
        };

        if state_status == KERN_SUCCESS {
            Ok(())
        } else {
            Err(MacOsDebuggerBackend::plugin_error(format!(
                "thread_set_state(ARM_DEBUG_STATE64) failed with status {}.",
                state_status
            )))
        }
    }

    fn read_x64_registers(
        &self,
        thread: thread_act_t,
    ) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        let thread_state = self.get_x64_thread_state(thread)?;
        let registers = vec![
            DebuggerRegisterValue::new("rax", thread_state.rax, 64),
            DebuggerRegisterValue::new("rbx", thread_state.rbx, 64),
            DebuggerRegisterValue::new("rcx", thread_state.rcx, 64),
            DebuggerRegisterValue::new("rdx", thread_state.rdx, 64),
            DebuggerRegisterValue::new("rdi", thread_state.rdi, 64),
            DebuggerRegisterValue::new("rsi", thread_state.rsi, 64),
            DebuggerRegisterValue::new("rbp", thread_state.rbp, 64),
            DebuggerRegisterValue::new("rsp", thread_state.rsp, 64),
            DebuggerRegisterValue::new("r8", thread_state.r8, 64),
            DebuggerRegisterValue::new("r9", thread_state.r9, 64),
            DebuggerRegisterValue::new("r10", thread_state.r10, 64),
            DebuggerRegisterValue::new("r11", thread_state.r11, 64),
            DebuggerRegisterValue::new("r12", thread_state.r12, 64),
            DebuggerRegisterValue::new("r13", thread_state.r13, 64),
            DebuggerRegisterValue::new("r14", thread_state.r14, 64),
            DebuggerRegisterValue::new("r15", thread_state.r15, 64),
            DebuggerRegisterValue::new("rip", thread_state.rip, 64),
            DebuggerRegisterValue::new("rflags", thread_state.rflags, 64),
        ];

        Ok(DebuggerRegisterSnapshot::new(Some(thread_state.rip), Some(thread_state.rsp), registers))
    }

    fn read_arm64_registers(
        &self,
        thread: thread_act_t,
    ) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
        let thread_state = self.get_arm64_thread_state(thread)?;
        let mut registers = Vec::with_capacity(34);

        for register_number in 0..thread_state.x.len() {
            registers.push(DebuggerRegisterValue::new(format!("x{}", register_number), thread_state.x[register_number], 64));
        }

        registers.push(DebuggerRegisterValue::new("fp", thread_state.fp, 64));
        registers.push(DebuggerRegisterValue::new("lr", thread_state.lr, 64));
        registers.push(DebuggerRegisterValue::new("sp", thread_state.sp, 64));
        registers.push(DebuggerRegisterValue::new("pc", thread_state.pc, 64));
        registers.push(DebuggerRegisterValue::new("cpsr", u64::from(thread_state.cpsr), 32));

        Ok(DebuggerRegisterSnapshot::new(Some(thread_state.pc), Some(thread_state.sp), registers))
    }

    fn write_x64_register(
        &self,
        thread: thread_act_t,
        register_name: &str,
        value: u64,
    ) -> Result<(), DebuggerPluginError> {
        let mut thread_state = self.get_x64_thread_state(thread)?;

        match register_name.to_ascii_lowercase().as_str() {
            "rax" => thread_state.rax = value,
            "rbx" => thread_state.rbx = value,
            "rcx" => thread_state.rcx = value,
            "rdx" => thread_state.rdx = value,
            "rdi" => thread_state.rdi = value,
            "rsi" => thread_state.rsi = value,
            "rbp" => thread_state.rbp = value,
            "rsp" => thread_state.rsp = value,
            "r8" => thread_state.r8 = value,
            "r9" => thread_state.r9 = value,
            "r10" => thread_state.r10 = value,
            "r11" => thread_state.r11 = value,
            "r12" => thread_state.r12 = value,
            "r13" => thread_state.r13 = value,
            "r14" => thread_state.r14 = value,
            "r15" => thread_state.r15 = value,
            "rip" => thread_state.rip = value,
            "rflags" => thread_state.rflags = value,
            _ => {
                return Err(MacOsDebuggerBackend::plugin_error(format!(
                    "Register '{}' is not a supported x64 integer register.",
                    register_name
                )));
            }
        }

        self.set_x64_thread_state(thread, thread_state)
    }

    fn write_arm64_register(
        &self,
        thread: thread_act_t,
        register_name: &str,
        value: u64,
    ) -> Result<(), DebuggerPluginError> {
        let mut thread_state = self.get_arm64_thread_state(thread)?;
        let normalized_register_name = register_name.to_ascii_lowercase();

        if let Some(register_number_text) = normalized_register_name.strip_prefix('x') {
            let register_number = register_number_text.parse::<usize>().map_err(|error| {
                MacOsDebuggerBackend::plugin_error(format!("Register '{}' is not a valid ARM64 x-register name: {}.", register_name, error))
            })?;

            if register_number < thread_state.x.len() {
                thread_state.x[register_number] = value;
                return self.set_arm64_thread_state(thread, thread_state);
            }
        }

        match normalized_register_name.as_str() {
            "fp" | "x29" => thread_state.fp = value,
            "lr" | "x30" => thread_state.lr = value,
            "sp" => thread_state.sp = value,
            "pc" => thread_state.pc = value,
            "cpsr" => {
                thread_state.cpsr = u32::try_from(value)
                    .map_err(|error| MacOsDebuggerBackend::plugin_error(format!("ARM64 cpsr value 0x{:X} does not fit in 32 bits: {}.", value, error)))?
            }
            _ => {
                return Err(MacOsDebuggerBackend::plugin_error(format!(
                    "Register '{}' is not a supported ARM64 integer register.",
                    register_name
                )));
            }
        }

        self.set_arm64_thread_state(thread, thread_state)
    }
}

impl Drop for ActiveMacOsSession {
    fn drop(&mut self) {
        if self.session_state != DebuggerSessionState::Detached {
            if let Err(error) = self.detach() {
                log::debug!("Failed to detach active macOS debugger session during drop: {}", error);
            }
        }
    }
}

fn macos_worker_main(
    process_info: OpenedProcessInfo,
    trace_event_sink: DebuggerTraceEventSink,
    worker_ready_sender: Sender<Result<Sender<MacOsWorkerCommand>, DebuggerPluginError>>,
) {
    let (command_sender, command_receiver) = mpsc::channel();
    let mut active_session = match ActiveMacOsSession::attach(&process_info, trace_event_sink) {
        Ok(active_session) => active_session,
        Err(error) => {
            let _ = worker_ready_sender.send(Err(error));

            return;
        }
    };

    if worker_ready_sender.send(Ok(command_sender)).is_err() {
        return;
    }

    run_macos_worker_loop(&command_receiver, &mut active_session);
}

fn run_macos_worker_loop(
    command_receiver: &Receiver<MacOsWorkerCommand>,
    active_session: &mut ActiveMacOsSession,
) {
    loop {
        match command_receiver.recv_timeout(Duration::from_millis(RUNNING_EVENT_WAIT_TIMEOUT_MS)) {
            Ok(MacOsWorkerCommand::Pause { result_sender }) => {
                let _ = result_sender.send(active_session.pause());
            }
            Ok(MacOsWorkerCommand::Resume { result_sender }) => {
                let _ = result_sender.send(active_session.resume());
            }
            Ok(MacOsWorkerCommand::ReadRegisters { result_sender }) => {
                let _ = result_sender.send(active_session.read_registers());
            }
            Ok(MacOsWorkerCommand::WriteRegister {
                register_name,
                value,
                result_sender,
            }) => {
                let _ = result_sender.send(active_session.write_register(&register_name, value));
            }
            Ok(MacOsWorkerCommand::SetBreakpoint {
                address,
                kind,
                label,
                result_sender,
            }) => {
                let _ = result_sender.send(active_session.set_breakpoint(address, kind, label));
            }
            Ok(MacOsWorkerCommand::RemoveBreakpoint { breakpoint_id, result_sender }) => {
                let _ = result_sender.send(active_session.remove_breakpoint(&breakpoint_id));
            }
            Ok(MacOsWorkerCommand::SetBreakpointEnabled {
                breakpoint_id,
                is_enabled,
                result_sender,
            }) => {
                let _ = result_sender.send(active_session.set_breakpoint_enabled(&breakpoint_id, is_enabled));
            }
            Ok(MacOsWorkerCommand::ListBreakpoints { result_sender }) => {
                let _ = result_sender.send(Ok(active_session.list_breakpoints()));
            }
            Ok(MacOsWorkerCommand::Detach { result_sender }) => {
                let detach_result = active_session.detach();
                let _ = result_sender.send(detach_result);

                break;
            }
            Err(RecvTimeoutError::Timeout) => {
                if let Err(error) = active_session.process_pending_debug_event() {
                    log::debug!("macOS debugger event processing failed: {}", error);
                }
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
}

struct ThreadList {
    thread_list: thread_act_array_t,
    thread_count: mach_msg_type_number_t,
}

impl ThreadList {
    fn for_task(task_port: mach_port_t) -> Result<Self, DebuggerPluginError> {
        let mut thread_list: thread_act_array_t = null_mut();
        let mut thread_count = 0;
        let thread_status = unsafe { task_threads(task_port, &mut thread_list, &mut thread_count) };

        if thread_status != KERN_SUCCESS {
            return Err(MacOsDebuggerBackend::plugin_error(format!(
                "task_threads failed with status {}.",
                thread_status
            )));
        }

        Ok(Self { thread_list, thread_count })
    }

    fn threads(&self) -> &[thread_act_t] {
        if self.thread_list.is_null() || self.thread_count == 0 {
            return &[];
        }

        unsafe { std::slice::from_raw_parts(self.thread_list, self.thread_count as usize) }
    }
}

impl Drop for ThreadList {
    fn drop(&mut self) {
        for thread in self.threads() {
            if *thread != MACH_PORT_NULL {
                let _ = unsafe { mach_port_deallocate(mach_task_self(), *thread) };
            }
        }

        if !self.thread_list.is_null() && self.thread_count > 0 {
            let byte_size = u64::from(self.thread_count).saturating_mul(size_of::<thread_act_t>() as u64);
            let _ = unsafe { mach_vm_deallocate(mach_task_self(), self.thread_list as mach_vm_address_t, byte_size) };
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct X64ThreadState {
    rax: u64,
    rbx: u64,
    rcx: u64,
    rdx: u64,
    rdi: u64,
    rsi: u64,
    rbp: u64,
    rsp: u64,
    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
    rip: u64,
    rflags: u64,
    cs: u64,
    fs: u64,
    gs: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct X64DebugState {
    dr0: u64,
    dr1: u64,
    dr2: u64,
    dr3: u64,
    dr4: u64,
    dr5: u64,
    dr6: u64,
    dr7: u64,
}

impl X64DebugState {
    fn set_slot_address(
        &mut self,
        slot: usize,
        address: u64,
    ) -> Result<(), DebuggerPluginError> {
        match slot {
            0 => self.dr0 = address,
            1 => self.dr1 = address,
            2 => self.dr2 = address,
            3 => self.dr3 = address,
            _ => {
                return Err(MacOsDebuggerBackend::plugin_error(format!("Invalid x64 watchpoint slot {}.", slot)));
            }
        }

        Ok(())
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Arm64ThreadState {
    x: [u64; 29],
    fp: u64,
    lr: u64,
    sp: u64,
    pc: u64,
    cpsr: u32,
    pad: u32,
}

impl Default for Arm64ThreadState {
    fn default() -> Self {
        unsafe { zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Arm64DebugState {
    bvr: [u64; 16],
    bcr: [u64; 16],
    wvr: [u64; 16],
    wcr: [u64; 16],
    mdscr_el1: u64,
}

impl Default for Arm64DebugState {
    fn default() -> Self {
        unsafe { zeroed() }
    }
}

fn ptrace_request(
    request: libc::c_int,
    process_id: pid_t,
    address: *mut c_char,
    data: libc::c_int,
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
            return Err(MacOsDebuggerBackend::plugin_error(format!(
                "Timed out waiting for macOS debug stop during {}.",
                operation_name
            )));
        }

        thread::sleep(Duration::from_millis(25));
    }
}

fn last_os_error(operation_name: &str) -> DebuggerPluginError {
    MacOsDebuggerBackend::plugin_error(format!("{} failed: {}.", operation_name, std::io::Error::last_os_error()))
}

fn state_count<T>() -> mach_msg_type_number_t {
    (size_of::<T>() / size_of::<u32>()) as mach_msg_type_number_t
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
        _ => Err(MacOsDebuggerBackend::plugin_error(format!(
            "x64 hardware data breakpoint size {} is unsupported. Expected 1, 2, 4, or 8 bytes.",
            size_in_bytes
        ))),
    }
}

fn arm64_watch_control(
    access: DebuggerDataBreakpointAccess,
    byte_mask: u64,
) -> u64 {
    const ENABLE: u64 = 1 << 0;
    const USER_ACCESS_ONLY: u64 = 0b10 << 1;
    const LOAD_STORE_CONTROL_SHIFT: u64 = 3;
    const BYTE_ADDRESS_SELECT_SHIFT: u64 = 5;
    let load_store_control = match access {
        DebuggerDataBreakpointAccess::Read => 0b01,
        DebuggerDataBreakpointAccess::Write => 0b10,
        DebuggerDataBreakpointAccess::ReadWrite => 0b11,
    };

    ENABLE | USER_ACCESS_ONLY | (load_store_control << LOAD_STORE_CONTROL_SHIFT) | (byte_mask << BYTE_ADDRESS_SELECT_SHIFT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn x64_watchpoint_lengths_match_debug_register_encoding() {
        assert_eq!(x64_watchpoint_length_bits(1).unwrap_or(u64::MAX), 0b00);
        assert_eq!(x64_watchpoint_length_bits(2).unwrap_or(u64::MAX), 0b01);
        assert_eq!(x64_watchpoint_length_bits(4).unwrap_or(u64::MAX), 0b11);
        assert_eq!(x64_watchpoint_length_bits(8).unwrap_or(u64::MAX), 0b10);
        assert!(x64_watchpoint_length_bits(3).is_err());
    }

    #[test]
    fn arm64_watch_control_sets_access_and_byte_mask() {
        let control = arm64_watch_control(DebuggerDataBreakpointAccess::Write, 0b1111);

        assert_eq!(control & 1, 1);
        assert_eq!((control >> 3) & 0b11, 0b10);
        assert_eq!((control >> 5) & 0b1111, 0b1111);
    }
}
