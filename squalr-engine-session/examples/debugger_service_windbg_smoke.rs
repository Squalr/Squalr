#[cfg(windows)]
use squalr_engine_api::{
    events::{debugger::debugger_event::DebuggerEvent, engine_event::EngineEvent},
    structures::{
        debugger::{DebuggerDataBreakpointAccess, DebuggerTraceEvent},
        memory::bitness::Bitness,
        processes::opened_process_info::OpenedProcessInfo,
    },
};
#[cfg(windows)]
use squalr_engine_session::{debugger::debugger_service::DebuggerService, plugins::plugin_registry::PluginRegistry};
#[cfg(windows)]
use std::{
    error::Error,
    io::{BufRead, BufReader, Write},
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
        mpsc,
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
    let process_info = OpenedProcessInfo::new(
        target_process_id,
        String::from("debugger_service_windbg_smoke_child"),
        u64::from(target_process_id),
        Bitness::Bit64,
        None,
    );
    let plugin_registry = Arc::new(PluginRegistry::new());

    if !plugin_registry.is_plugin_enabled("builtin.debugger.windbg") && !plugin_registry.set_plugin_enabled("builtin.debugger.windbg", true) {
        return Err(String::from("Failed to enable builtin.debugger.windbg for smoke validation.").into());
    }

    let (engine_event_sender, engine_event_receiver) = mpsc::channel::<EngineEvent>();
    let debugger_service = DebuggerService::new(
        plugin_registry,
        Arc::new(move |engine_event| {
            let _ = engine_event_sender.send(engine_event);
        }),
    );

    println!("Attaching session service to child process {target_process_id} at {target_address:#x}.");
    let attach_status = debugger_service.attach(&process_info, Some("builtin.debugger.windbg"))?;
    println!(
        "Attached through {}.",
        attach_status
            .get_active_plugin_id()
            .unwrap_or("<unknown debugger plugin>")
    );

    let (trace_session, _) = debugger_service.start_trace_session(
        target_address,
        8,
        DebuggerDataBreakpointAccess::Write,
        Some(String::from("session-windbg-smoke-write")),
    )?;
    println!(
        "Trace session {} armed at {:#x}.",
        trace_session.get_trace_session_id(),
        trace_session.get_address()
    );

    let listed_breakpoints = debugger_service.list_breakpoints()?;
    println!("Session service reports {} breakpoint(s).", listed_breakpoints.len());

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
        return Err(String::from("Session trace was not enriched with disassembly text.").into());
    }

    let (stopped_trace_session, instruction_records) = debugger_service.stop_trace_session(trace_session.get_trace_session_id())?;
    println!(
        "Stopped trace session {} with {} instruction record(s).",
        stopped_trace_session.get_trace_session_id(),
        instruction_records.len()
    );

    debugger_service.detach()?;
    println!("Detached cleanly through the session service.");

    Ok(())
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
fn wait_for_trace_event(
    engine_event_receiver: &mpsc::Receiver<EngineEvent>,
    timeout: Duration,
) -> Result<DebuggerTraceEvent, Box<dyn Error>> {
    let wait_started_at = Instant::now();

    loop {
        if let Ok(engine_event) = engine_event_receiver.recv_timeout(Duration::from_millis(100)) {
            if let EngineEvent::Debugger(DebuggerEvent::TraceRecorded { debugger_trace_recorded_event }) = engine_event {
                return Ok(debugger_trace_recorded_event.trace_event);
            }
        }

        if wait_started_at.elapsed() >= timeout {
            return Err(String::from("Timed out waiting for a session debugger trace event.").into());
        }
    }
}

#[cfg(windows)]
fn format_optional_address(address: Option<u64>) -> String {
    address
        .map(|address| format!("{address:#x}"))
        .unwrap_or_else(|| String::from("<unknown>"))
}

#[cfg(not(windows))]
fn main() {
    println!("The WinDbg session smoke example is only available on Windows.");
}
