#[cfg(any(windows, target_os = "macos", all(target_os = "linux", target_arch = "x86_64")))]
use squalr_engine_api::{
    plugins::debugger::DebuggerPlugin,
    structures::{
        debugger::{DebuggerBreakpointKind, DebuggerDataBreakpointAccess, DebuggerTraceEvent},
        memory::bitness::Bitness,
        processes::{opened_process_info::OpenedProcessInfo, target_architecture::TargetArchitecture},
    },
};
#[cfg(any(windows, target_os = "macos", all(target_os = "linux", target_arch = "x86_64")))]
use squalr_plugin_debuggers_native::NativeDebuggersPlugin;
#[cfg(any(windows, target_os = "macos", all(target_os = "linux", target_arch = "x86_64")))]
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

#[cfg(target_os = "macos")]
use mach2::{
    kern_return::KERN_SUCCESS,
    mach_port::mach_port_deallocate,
    port::{MACH_PORT_NULL, mach_port_name_t, mach_port_t},
    traps::{mach_task_self, task_for_pid},
};

#[cfg(any(windows, target_os = "macos", all(target_os = "linux", target_arch = "x86_64")))]
static SMOKE_TARGET_VALUE: AtomicU64 = AtomicU64::new(0);

#[cfg(any(windows, target_os = "macos", all(target_os = "linux", target_arch = "x86_64")))]
fn main() -> Result<(), Box<dyn Error>> {
    let is_child_process = std::env::args().any(|argument| argument == "--child");

    if is_child_process {
        run_child_process()?;
    } else {
        run_parent_process()?;
    }

    Ok(())
}

#[cfg(any(windows, target_os = "macos", all(target_os = "linux", target_arch = "x86_64")))]
fn run_child_process() -> Result<(), Box<dyn Error>> {
    let target_address = &SMOKE_TARGET_VALUE as *const AtomicU64 as u64;

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    let _worker_thread_handle = thread::spawn(|| {
        loop {
            SMOKE_TARGET_VALUE.fetch_add(1, Ordering::Relaxed);
            thread::sleep(Duration::from_millis(10));
        }
    });

    println!("{} {target_address:#x}", std::process::id());
    std::io::stdout().flush()?;

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    loop {
        thread::sleep(Duration::from_secs(1));
    }

    #[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
    loop {
        SMOKE_TARGET_VALUE.fetch_add(1, Ordering::Relaxed);
        thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(any(windows, target_os = "macos", all(target_os = "linux", target_arch = "x86_64")))]
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

#[cfg(any(windows, target_os = "macos", all(target_os = "linux", target_arch = "x86_64")))]
fn run_smoke_against_child(child_process: &mut std::process::Child) -> Result<(), Box<dyn Error>> {
    let child_stdout = child_process
        .stdout
        .take()
        .ok_or_else(|| String::from("Smoke child stdout was not available."))?;
    let mut child_stdout_reader = BufReader::new(child_stdout);
    let mut child_ready_line = String::new();
    child_stdout_reader.read_line(&mut child_ready_line)?;

    let (target_process_id, target_address) = parse_child_ready_line(&child_ready_line)?;
    let opened_process = open_smoke_process(target_process_id)?;
    let process_info = OpenedProcessInfo::new(
        target_process_id,
        String::from("native_debugger_smoke_child"),
        opened_process.handle,
        Bitness::Bit64,
        None,
    )
    .with_target_architecture(opened_process.target_architecture.clone());

    let smoke_result = run_debugger_flow(process_info, target_address);
    opened_process.close();

    smoke_result
}

#[cfg(any(windows, target_os = "macos", all(target_os = "linux", target_arch = "x86_64")))]
fn run_debugger_flow(
    process_info: OpenedProcessInfo,
    target_address: u64,
) -> Result<(), Box<dyn Error>> {
    let (trace_event_sender, trace_event_receiver) = mpsc::channel::<DebuggerTraceEvent>();
    let trace_event_sink = Arc::new(move |trace_event| {
        let _ = trace_event_sender.send(trace_event);
    });

    let plugin = NativeDebuggersPlugin::new();
    if !plugin.can_attach(&process_info) {
        return Err(format!(
            "Native debugger plugin refused process architecture '{}'.",
            process_info.get_target_architecture().get_instruction_set_id()
        )
        .into());
    }

    let mut debugger_session = plugin.create_session(&process_info, trace_event_sink)?;

    println!("Attaching to child process {} at {:#x}.", process_info.get_process_id(), target_address);
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
        Some(String::from("native-debugger-smoke-write")),
    )?;
    println!(
        "Breakpoint {} armed at {:#x}.",
        breakpoint_descriptor.get_breakpoint_id(),
        breakpoint_descriptor.get_address()
    );

    let listed_breakpoints = debugger_session.list_breakpoints()?;
    println!("Native debugger reports {} breakpoint(s).", listed_breakpoints.len());

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

#[cfg(any(windows, target_os = "macos", all(target_os = "linux", target_arch = "x86_64")))]
struct SmokeOpenedProcess {
    handle: u64,
    target_architecture: TargetArchitecture,
}

#[cfg(any(windows, target_os = "macos", all(target_os = "linux", target_arch = "x86_64")))]
impl SmokeOpenedProcess {
    fn close(&self) {
        close_smoke_process(self.handle);
    }
}

#[cfg(windows)]
fn open_smoke_process(target_process_id: u32) -> Result<SmokeOpenedProcess, Box<dyn Error>> {
    Ok(SmokeOpenedProcess {
        handle: u64::from(target_process_id),
        target_architecture: TargetArchitecture::x64(),
    })
}

#[cfg(target_os = "macos")]
fn open_smoke_process(target_process_id: u32) -> Result<SmokeOpenedProcess, Box<dyn Error>> {
    let mut task_port: mach_port_t = MACH_PORT_NULL;
    let task_for_pid_status = unsafe { task_for_pid(mach_task_self(), target_process_id as libc::c_int, &mut task_port as *mut mach_port_t) };

    if task_for_pid_status != KERN_SUCCESS || task_port == MACH_PORT_NULL {
        return Err(format!(
            "task_for_pid failed for smoke child {} with status {}. Run with macOS debugging permissions enabled.",
            target_process_id, task_for_pid_status
        )
        .into());
    }

    Ok(SmokeOpenedProcess {
        handle: u64::from(task_port),
        target_architecture: current_macos_target_architecture(),
    })
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn open_smoke_process(target_process_id: u32) -> Result<SmokeOpenedProcess, Box<dyn Error>> {
    Ok(SmokeOpenedProcess {
        handle: u64::from(target_process_id),
        target_architecture: TargetArchitecture::x64(),
    })
}

#[cfg(windows)]
fn close_smoke_process(_handle: u64) {}

#[cfg(target_os = "macos")]
fn close_smoke_process(handle: u64) {
    if handle != 0 {
        let _ = unsafe { mach_port_deallocate(mach_task_self(), handle as mach_port_name_t) };
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn close_smoke_process(_handle: u64) {}

#[cfg(target_os = "macos")]
fn current_macos_target_architecture() -> TargetArchitecture {
    #[cfg(target_arch = "aarch64")]
    {
        TargetArchitecture::arm64()
    }

    #[cfg(not(target_arch = "aarch64"))]
    {
        TargetArchitecture::x64()
    }
}

#[cfg(any(windows, target_os = "macos", all(target_os = "linux", target_arch = "x86_64")))]
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

#[cfg(any(windows, target_os = "macos", all(target_os = "linux", target_arch = "x86_64")))]
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
            return Err(String::from("Timed out waiting for a native debugger breakpoint trace event.").into());
        }
    }
}

#[cfg(any(windows, target_os = "macos", all(target_os = "linux", target_arch = "x86_64")))]
fn format_optional_address(address: Option<u64>) -> String {
    address
        .map(|address| format!("{address:#x}"))
        .unwrap_or_else(|| String::from("<unknown>"))
}

#[cfg(not(any(windows, target_os = "macos", all(target_os = "linux", target_arch = "x86_64"))))]
fn main() {
    println!("The native debugger smoke example is only available on Windows, macOS, and Linux x86_64 right now.");
}
