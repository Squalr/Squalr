#[cfg(windows)]
use crossbeam_channel::{Receiver, Sender, unbounded};
#[cfg(windows)]
use squalr_engine::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
#[cfg(windows)]
use squalr_engine::engine_privileged_state::EnginePrivilegedState;
#[cfg(windows)]
use squalr_engine_api::{
    commands::{
        debugger::{
            attach::debugger_attach_request::DebuggerAttachRequest, detach::debugger_detach_request::DebuggerDetachRequest,
            registers_read::debugger_registers_read_request::DebuggerRegistersReadRequest, trace_list::debugger_trace_list_request::DebuggerTraceListRequest,
            trace_pause::debugger_trace_pause_request::DebuggerTracePauseRequest, trace_resume::debugger_trace_resume_request::DebuggerTraceResumeRequest,
            trace_start::debugger_trace_start_request::DebuggerTraceStartRequest, trace_stop::debugger_trace_stop_request::DebuggerTraceStopRequest,
        },
        privileged_command::PrivilegedCommand,
        privileged_command_response::PrivilegedCommandResponse,
    },
    engine::{
        engine_api_priviliged_bindings::EngineApiPrivilegedBindings, engine_binding_error::EngineBindingError, engine_event_envelope::EngineEventEnvelope,
    },
    events::{debugger::debugger_event::DebuggerEvent, engine_event::EngineEvent},
    structures::{
        debugger::{DebuggerCommandStatus, DebuggerDataBreakpointAccess, DebuggerTraceEvent},
        memory::bitness::Bitness,
        processes::{opened_process_info::OpenedProcessInfo, process_info::ProcessInfo},
    },
};
#[cfg(windows)]
use squalr_engine_session::os::engine_os_provider::{EngineOsProviders, ProcessQueryError, ProcessQueryOptions, ProcessQueryProvider};
#[cfg(windows)]
use std::{
    error::Error,
    io::{BufRead, BufReader, Write},
    process::{Command, Stdio},
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

#[cfg(windows)]
static SMOKE_TARGET_VALUE: AtomicU64 = AtomicU64::new(0);

#[cfg(windows)]
fn main() -> Result<(), Box<dyn Error>> {
    let is_child_process = std::env::args().any(|argument| argument == "--child");

    if is_child_process {
        run_child_process()?;
    } else {
        run_parent_process()?;
    }

    Ok(())
}

#[cfg(windows)]
fn run_child_process() -> Result<(), Box<dyn Error>> {
    let target_address = &SMOKE_TARGET_VALUE as *const AtomicU64 as u64;

    println!("{} {target_address:#x}", std::process::id());
    std::io::stdout().flush()?;

    loop {
        SMOKE_TARGET_VALUE.fetch_add(1, Ordering::Relaxed);
        thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(windows)]
fn run_parent_process() -> Result<(), Box<dyn Error>> {
    let current_executable_path = std::env::current_exe()?;
    let mut child_process = Command::new(current_executable_path)
        .arg("--child")
        .stdout(Stdio::piped())
        .spawn()?;

    let smoke_result = run_smoke_against_child(&mut child_process);
    let _ = child_process.kill();
    let _ = child_process.wait();

    smoke_result
}

#[cfg(windows)]
fn run_smoke_against_child(child_process: &mut std::process::Child) -> Result<(), Box<dyn Error>> {
    let child_stdout = child_process
        .stdout
        .take()
        .ok_or_else(|| String::from("Smoke child stdout was not available."))?;
    let mut child_stdout_reader = BufReader::new(child_stdout);
    let mut child_ready_line = String::new();
    child_stdout_reader.read_line(&mut child_ready_line)?;

    let (target_process_id, target_address) = parse_child_ready_line(&child_ready_line)?;
    let opened_process_info = OpenedProcessInfo::new(
        target_process_id,
        String::from("debugger_command_dbgeng_smoke_child"),
        u64::from(target_process_id),
        Bitness::Bit64,
        None,
    );
    let engine_privileged_state = create_smoke_engine_privileged_state()?;
    let engine_event_receiver = engine_privileged_state.subscribe_to_engine_events()?;

    let plugin_registry = engine_privileged_state.get_plugin_registry();

    if !plugin_registry.is_plugin_enabled("builtin.debugger.dbgeng") && !plugin_registry.set_plugin_enabled("builtin.debugger.dbgeng", true) {
        return Err(String::from("Failed to enable builtin.debugger.dbgeng for command smoke validation.").into());
    }

    engine_privileged_state
        .get_process_manager()
        .set_opened_process(opened_process_info);

    println!("Executing debugger commands for child process {target_process_id} at {target_address:#x}.");
    let attach_response = DebuggerAttachRequest {
        plugin_id: Some(String::from("builtin.debugger.dbgeng")),
    }
    .execute(&engine_privileged_state);
    require_status(&attach_response.status, "debugger attach")?;
    println!(
        "Attached through {}.",
        attach_response
            .active_plugin_id
            .as_deref()
            .unwrap_or("<unknown debugger plugin>")
    );

    let register_response = DebuggerRegistersReadRequest {}.execute(&engine_privileged_state);
    require_status(&register_response.status, "debugger register read after attach")?;
    let attach_register_snapshot = register_response
        .register_snapshot
        .ok_or_else(|| String::from("Register read response did not include a snapshot."))?;
    println!(
        "Attach registers: IP={}, SP={}, registers={}.",
        format_optional_address(attach_register_snapshot.get_instruction_pointer()),
        format_optional_address(attach_register_snapshot.get_stack_pointer()),
        attach_register_snapshot.get_registers().len()
    );

    let trace_start_response = DebuggerTraceStartRequest {
        address: target_address,
        size_in_bytes: 8,
        access: DebuggerDataBreakpointAccess::Write,
        label: Some(String::from("command-dbgeng-smoke-write")),
    }
    .execute(&engine_privileged_state);
    require_status(&trace_start_response.status, "debugger trace start")?;
    let trace_session = trace_start_response
        .trace_session
        .ok_or_else(|| String::from("Trace start response did not include a trace session descriptor."))?;
    println!(
        "Trace session {} armed at {:#x}.",
        trace_session.get_trace_session_id(),
        trace_session.get_address()
    );

    let trace_event = wait_for_trace_event(&engine_event_receiver, Duration::from_secs(10))?;
    let trace_register_snapshot = trace_event.get_register_snapshot();
    println!(
        "Trace event: IP={}, instruction_address={}, bytes={}, registers={}, instruction={:?}, backend={:?}.",
        format_optional_address(trace_register_snapshot.get_instruction_pointer()),
        format_optional_address(trace_event.get_instruction_address()),
        trace_event.get_instruction_bytes().len(),
        trace_register_snapshot.get_registers().len(),
        trace_event.get_instruction_text(),
        trace_event.get_backend_message()
    );

    if trace_event.get_instruction_text().is_none() {
        return Err(String::from("Command trace was not enriched with disassembly text.").into());
    }

    let trace_pause_response = DebuggerTracePauseRequest {
        trace_session_id: trace_session.get_trace_session_id().to_string(),
    }
    .execute(&engine_privileged_state);
    require_status(&trace_pause_response.status, "debugger trace collection pause")?;
    println!(
        "Paused trace collection for {} at {} instruction record(s).",
        trace_session.get_trace_session_id(),
        trace_pause_response.instruction_records.len()
    );
    let paused_hit_count = trace_pause_response
        .instruction_records
        .first()
        .map(|instruction_record| instruction_record.get_hit_count())
        .unwrap_or(0);
    thread::sleep(Duration::from_millis(250));
    let hit_count_while_paused = list_maximum_trace_hit_count(&engine_privileged_state, trace_session.get_trace_session_id())?;

    if hit_count_while_paused != paused_hit_count {
        return Err(format!(
            "Trace collection recorded hits while paused: before={}, after={}.",
            paused_hit_count, hit_count_while_paused
        )
        .into());
    }

    let trace_resume_response = DebuggerTraceResumeRequest {
        trace_session_id: trace_session.get_trace_session_id().to_string(),
    }
    .execute(&engine_privileged_state);
    require_status(&trace_resume_response.status, "debugger trace collection resume")?;
    println!(
        "Resumed trace collection for {} at {} instruction record(s).",
        trace_session.get_trace_session_id(),
        trace_resume_response.instruction_records.len()
    );
    wait_for_trace_session_hit_count(
        &engine_privileged_state,
        trace_session.get_trace_session_id(),
        paused_hit_count.saturating_add(1),
        Duration::from_secs(10),
    )?;
    println!("Resume captured a new trace hit.");

    let trace_stop_response = DebuggerTraceStopRequest {
        trace_session_id: trace_session.get_trace_session_id().to_string(),
    }
    .execute(&engine_privileged_state);
    require_status(&trace_stop_response.status, "debugger trace stop")?;
    println!(
        "Stopped trace session {} with {} instruction record(s).",
        trace_session.get_trace_session_id(),
        trace_stop_response.instruction_records.len()
    );

    let restarted_trace_start_response = DebuggerTraceStartRequest {
        address: target_address,
        size_in_bytes: 8,
        access: DebuggerDataBreakpointAccess::Write,
        label: Some(String::from("command-dbgeng-smoke-write-restart")),
    }
    .execute(&engine_privileged_state);
    require_status(&restarted_trace_start_response.status, "debugger restarted trace start")?;
    let restarted_trace_session = restarted_trace_start_response
        .trace_session
        .ok_or_else(|| String::from("Restarted trace start response did not include a trace session descriptor."))?;
    wait_for_trace_session_hit_count(
        &engine_privileged_state,
        restarted_trace_session.get_trace_session_id(),
        1,
        Duration::from_secs(10),
    )?;
    println!("Restarted trace session {} captured a hit.", restarted_trace_session.get_trace_session_id());

    let restarted_trace_stop_response = DebuggerTraceStopRequest {
        trace_session_id: restarted_trace_session.get_trace_session_id().to_string(),
    }
    .execute(&engine_privileged_state);
    require_status(&restarted_trace_stop_response.status, "debugger restarted trace stop")?;

    let detach_response = DebuggerDetachRequest {}.execute(&engine_privileged_state);
    require_status(&detach_response.status, "debugger detach")?;
    println!("Detached cleanly through debugger command executors.");

    Ok(())
}

#[cfg(windows)]
fn create_smoke_engine_privileged_state() -> Result<Arc<EnginePrivilegedState>, Box<dyn Error>> {
    let engine_bindings = Arc::new(RwLock::new(CapturingEngineBindings::new()));
    let mut engine_os_providers = EngineOsProviders::default();
    engine_os_providers.process_query = Arc::new(NoOpProcessQueryProvider);

    Ok(EnginePrivilegedState::new(engine_bindings, engine_os_providers)?)
}

#[cfg(windows)]
fn list_maximum_trace_hit_count(
    engine_privileged_state: &Arc<EnginePrivilegedState>,
    trace_session_id: &str,
) -> Result<u64, Box<dyn Error>> {
    let trace_list_response = DebuggerTraceListRequest {}.execute(engine_privileged_state);
    require_status(&trace_list_response.status, "debugger trace list")?;

    Ok(trace_list_response
        .instruction_records
        .iter()
        .filter(|instruction_record| instruction_record.get_trace_session_id() == trace_session_id)
        .map(|instruction_record| instruction_record.get_hit_count())
        .max()
        .unwrap_or(0))
}

#[cfg(windows)]
fn wait_for_trace_session_hit_count(
    engine_privileged_state: &Arc<EnginePrivilegedState>,
    trace_session_id: &str,
    minimum_hit_count: u64,
    timeout: Duration,
) -> Result<(), Box<dyn Error>> {
    let wait_started_at = Instant::now();

    loop {
        let maximum_hit_count = list_maximum_trace_hit_count(engine_privileged_state, trace_session_id)?;

        if maximum_hit_count >= minimum_hit_count {
            return Ok(());
        }

        if wait_started_at.elapsed() >= timeout {
            return Err(format!(
                "Timed out waiting for trace session {} to reach hit count {}.",
                trace_session_id, minimum_hit_count
            )
            .into());
        }

        thread::sleep(Duration::from_millis(50));
    }
}

#[cfg(windows)]
fn parse_child_ready_line(child_ready_line: &str) -> Result<(u32, u64), Box<dyn Error>> {
    let mut child_ready_parts = child_ready_line.split_whitespace();
    let target_process_id = child_ready_parts
        .next()
        .ok_or_else(|| String::from("Smoke child did not print a process id."))?
        .parse::<u32>()?;
    let target_address_text = child_ready_parts
        .next()
        .ok_or_else(|| String::from("Smoke child did not print a target address."))?;
    let target_address = u64::from_str_radix(target_address_text.trim_start_matches("0x"), 16)?;

    Ok((target_process_id, target_address))
}

#[cfg(windows)]
fn require_status(
    status: &DebuggerCommandStatus,
    operation_name: &str,
) -> Result<(), Box<dyn Error>> {
    if status.get_success() {
        Ok(())
    } else {
        Err(format!("{} failed: {}", operation_name, status.get_message().unwrap_or("no diagnostic message")).into())
    }
}

#[cfg(windows)]
fn wait_for_trace_event(
    engine_event_receiver: &Receiver<EngineEventEnvelope>,
    timeout: Duration,
) -> Result<DebuggerTraceEvent, Box<dyn Error>> {
    let wait_started_at = Instant::now();

    loop {
        if let Ok(engine_event_envelope) = engine_event_receiver.recv_timeout(Duration::from_millis(100)) {
            if let EngineEvent::Debugger(DebuggerEvent::TraceRecorded { debugger_trace_recorded_event }) = engine_event_envelope.into_engine_event() {
                return Ok(debugger_trace_recorded_event.trace_event);
            }
        }

        if wait_started_at.elapsed() >= timeout {
            return Err(String::from("Timed out waiting for a command debugger trace event.").into());
        }
    }
}

#[cfg(windows)]
fn format_optional_address(address: Option<u64>) -> String {
    address
        .map(|address| format!("{address:#x}"))
        .unwrap_or_else(|| String::from("<unknown>"))
}

#[cfg(windows)]
struct CapturingEngineBindings {
    subscribers: Mutex<Vec<Sender<EngineEventEnvelope>>>,
}

#[cfg(windows)]
impl CapturingEngineBindings {
    fn new() -> Self {
        Self {
            subscribers: Mutex::new(Vec::new()),
        }
    }
}

#[cfg(windows)]
impl EngineApiPrivilegedBindings for CapturingEngineBindings {
    fn emit_event(
        &self,
        event: EngineEvent,
    ) -> Result<(), EngineBindingError> {
        let event_envelope = EngineEventEnvelope::new(0, event);

        self.subscribers
            .lock()
            .map_err(|error| EngineBindingError::lock_failure("publishing smoke engine event", error.to_string()))?
            .retain(|subscriber| subscriber.send(event_envelope.clone()).is_ok());

        Ok(())
    }

    fn dispatch_internal_command(
        &self,
        _engine_command: PrivilegedCommand,
        _callback: Box<dyn FnOnce(PrivilegedCommandResponse) + Send + Sync + 'static>,
    ) -> Result<(), EngineBindingError> {
        Err(EngineBindingError::unavailable(
            "dispatching internal commands in the DbgEng command smoke example",
        ))
    }

    fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEventEnvelope>, EngineBindingError> {
        let (event_sender, event_receiver) = unbounded();

        self.subscribers
            .lock()
            .map_err(|error| EngineBindingError::lock_failure("subscribing to smoke engine events", error.to_string()))?
            .push(event_sender);

        Ok(event_receiver)
    }
}

#[cfg(windows)]
struct NoOpProcessQueryProvider;

#[cfg(windows)]
impl ProcessQueryProvider for NoOpProcessQueryProvider {
    fn start_monitoring(&self) -> Result<(), ProcessQueryError> {
        Ok(())
    }

    fn get_processes(
        &self,
        _process_query_options: ProcessQueryOptions,
    ) -> Vec<ProcessInfo> {
        Vec::new()
    }

    fn open_process(
        &self,
        _process_info: &ProcessInfo,
    ) -> Result<OpenedProcessInfo, ProcessQueryError> {
        Err(ProcessQueryError::internal("open_process", "not used by the DbgEng command smoke example"))
    }

    fn close_process(
        &self,
        _handle: u64,
    ) -> Result<(), ProcessQueryError> {
        Ok(())
    }
}

#[cfg(not(windows))]
fn main() {
    println!("The DbgEng command smoke example is only available on Windows.");
}
