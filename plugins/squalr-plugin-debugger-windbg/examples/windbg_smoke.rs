#[cfg(windows)]
use squalr_engine_api::{
    plugins::debugger::DebuggerPlugin,
    structures::{
        debugger::{DebuggerBreakpointKind, DebuggerDataBreakpointAccess, DebuggerTraceEvent},
        memory::bitness::Bitness,
        processes::opened_process_info::OpenedProcessInfo,
    },
};
#[cfg(windows)]
use squalr_plugin_debugger_windbg::WindbgDebuggerPlugin;
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
        String::from("windbg_smoke_child"),
        u64::from(target_process_id),
        Bitness::Bit64,
        None,
    );

    let (trace_event_sender, trace_event_receiver) = mpsc::channel::<DebuggerTraceEvent>();
    let trace_event_sink = Arc::new(move |trace_event| {
        let _ = trace_event_sender.send(trace_event);
    });

    let plugin = WindbgDebuggerPlugin::new();
    let mut debugger_session = plugin.create_session(&process_info, trace_event_sink)?;

    println!("Attaching to child process {target_process_id} at {target_address:#x}.");
    debugger_session.attach()?;
    let register_snapshot = debugger_session.read_registers()?;
    println!(
        "Attached. IP={}, SP={}, registers={}.",
        format_optional_address(register_snapshot.get_instruction_pointer()),
        format_optional_address(register_snapshot.get_stack_pointer()),
        register_snapshot.get_registers().len()
    );

    let breakpoint_descriptor = debugger_session.set_breakpoint(
        target_address,
        DebuggerBreakpointKind::hardware_data(DebuggerDataBreakpointAccess::Write, 8),
        Some(String::from("windbg-smoke-write")),
    )?;
    println!(
        "Breakpoint {} armed at {:#x}.",
        breakpoint_descriptor.get_breakpoint_id(),
        breakpoint_descriptor.get_address()
    );

    let listed_breakpoints = debugger_session.list_breakpoints()?;
    println!("DbgEng reports {} breakpoint(s).", listed_breakpoints.len());

    debugger_session.resume()?;
    let trace_event = wait_for_trace_event(&trace_event_receiver, Duration::from_secs(10))?;
    let trace_breakpoint_id = trace_event
        .get_breakpoint()
        .map(|breakpoint| breakpoint.get_breakpoint_id())
        .unwrap_or("<none>");
    let trace_register_snapshot = trace_event.get_register_snapshot();
    let trace_instruction_pointer = trace_register_snapshot.get_instruction_pointer();
    println!(
        "Trace received: breakpoint={}, IP={}, instruction_address={}, bytes={}, registers={}, instruction={:?}, backend={:?}.",
        trace_breakpoint_id,
        format_optional_address(trace_instruction_pointer),
        format_optional_address(trace_event.get_instruction_address()),
        trace_event.get_instruction_bytes().len(),
        trace_register_snapshot.get_registers().len(),
        trace_event.get_instruction_text(),
        trace_event.get_backend_message()
    );

    debugger_session.pause()?;
    debugger_session.remove_breakpoint(breakpoint_descriptor.get_breakpoint_id())?;
    debugger_session.detach()?;
    println!("Detached cleanly.");

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
    trace_event_receiver: &mpsc::Receiver<DebuggerTraceEvent>,
    timeout: Duration,
) -> Result<DebuggerTraceEvent, Box<dyn Error>> {
    let wait_started_at = Instant::now();

    loop {
        if let Ok(trace_event) = trace_event_receiver.recv_timeout(Duration::from_millis(100)) {
            return Ok(trace_event);
        }

        if wait_started_at.elapsed() >= timeout {
            return Err(String::from("Timed out waiting for a WinDbg breakpoint trace event.").into());
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
    println!("The WinDbg smoke example is only available on Windows.");
}
