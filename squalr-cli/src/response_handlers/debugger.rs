use squalr_engine_api::{
    commands::debugger::debugger_response::DebuggerResponse,
    structures::debugger::{
        DebuggerBreakpointDescriptor, DebuggerBreakpointKind, DebuggerCommandStatus, DebuggerRegisterSnapshot, DebuggerTraceInstructionRecord,
        DebuggerTraceSessionDescriptor,
    },
};

pub fn handle_debugger_response(response: DebuggerResponse) {
    match response {
        DebuggerResponse::Attach { debugger_attach_response } => {
            print_debugger_status("Debugger attach", &debugger_attach_response.status);
            println!(
                "Session: {:?}{}.",
                debugger_attach_response.session_state,
                debugger_attach_response
                    .active_plugin_id
                    .as_deref()
                    .map(|plugin_id| format!(" via {}", plugin_id))
                    .unwrap_or_default()
            );
        }
        DebuggerResponse::Detach { debugger_detach_response } => {
            print_debugger_status("Debugger detach", &debugger_detach_response.status);
            println!("Session: {:?}.", debugger_detach_response.session_state);
        }
        DebuggerResponse::Pause { debugger_pause_response } => {
            print_debugger_status("Debugger pause", &debugger_pause_response.status);
            println!("Session: {:?}.", debugger_pause_response.session_state);
        }
        DebuggerResponse::Resume { debugger_resume_response } => {
            print_debugger_status("Debugger resume", &debugger_resume_response.status);
            println!("Session: {:?}.", debugger_resume_response.session_state);
        }
        DebuggerResponse::BreakpointSet {
            debugger_breakpoint_set_response,
        } => {
            print_debugger_status("Breakpoint set", &debugger_breakpoint_set_response.status);
            if let Some(breakpoint) = debugger_breakpoint_set_response.breakpoint {
                print_breakpoint(&breakpoint);
            }
        }
        DebuggerResponse::BreakpointRemove {
            debugger_breakpoint_remove_response,
        } => {
            print_debugger_status("Breakpoint remove", &debugger_breakpoint_remove_response.status);
        }
        DebuggerResponse::BreakpointList {
            debugger_breakpoint_list_response,
        } => {
            print_debugger_status("Breakpoint list", &debugger_breakpoint_list_response.status);
            for breakpoint in debugger_breakpoint_list_response.breakpoints {
                print_breakpoint(&breakpoint);
            }
        }
        DebuggerResponse::RegistersRead {
            debugger_registers_read_response,
        } => {
            print_debugger_status("Registers read", &debugger_registers_read_response.status);
            if let Some(register_snapshot) = debugger_registers_read_response.register_snapshot {
                print_register_snapshot(&register_snapshot);
            }
        }
        DebuggerResponse::RegisterWrite {
            debugger_register_write_response,
        } => {
            print_debugger_status("Register write", &debugger_register_write_response.status);
            if let Some(register_snapshot) = debugger_register_write_response.register_snapshot {
                print_register_snapshot(&register_snapshot);
            }
        }
        DebuggerResponse::TraceStart { debugger_trace_start_response } => {
            print_debugger_status("Trace start", &debugger_trace_start_response.status);
            if let Some(trace_session) = debugger_trace_start_response.trace_session {
                print_trace_session(&trace_session);
            }
            print_instruction_records(&debugger_trace_start_response.instruction_records);
        }
        DebuggerResponse::TraceStop { debugger_trace_stop_response } => {
            print_debugger_status("Trace stop", &debugger_trace_stop_response.status);
            if let Some(trace_session) = debugger_trace_stop_response.trace_session {
                print_trace_session(&trace_session);
            }
            print_instruction_records(&debugger_trace_stop_response.instruction_records);
        }
        DebuggerResponse::TracePause { debugger_trace_pause_response } => {
            print_debugger_status("Trace pause", &debugger_trace_pause_response.status);
            if let Some(trace_session) = debugger_trace_pause_response.trace_session {
                print_trace_session(&trace_session);
            }
            print_instruction_records(&debugger_trace_pause_response.instruction_records);
        }
        DebuggerResponse::TraceResume {
            debugger_trace_resume_response,
        } => {
            print_debugger_status("Trace resume", &debugger_trace_resume_response.status);
            if let Some(trace_session) = debugger_trace_resume_response.trace_session {
                print_trace_session(&trace_session);
            }
            print_instruction_records(&debugger_trace_resume_response.instruction_records);
        }
        DebuggerResponse::TraceList { debugger_trace_list_response } => {
            print_debugger_status("Trace list", &debugger_trace_list_response.status);
            for trace_session in debugger_trace_list_response.trace_sessions {
                print_trace_session(&trace_session);
            }
            print_instruction_records(&debugger_trace_list_response.instruction_records);
        }
    }
}

fn print_debugger_status(
    label: &str,
    status: &DebuggerCommandStatus,
) {
    if status.get_success() {
        println!("{} succeeded.", label);
    } else {
        println!("{} failed: {}.", label, status.get_message().unwrap_or("unknown error"));
    }
}

fn print_breakpoint(breakpoint: &DebuggerBreakpointDescriptor) {
    println!(
        "Breakpoint {} at 0x{:X}: kind={}, enabled={}, mechanism={:?}, label={}.",
        breakpoint.get_breakpoint_id(),
        breakpoint.get_address(),
        format_breakpoint_kind(breakpoint.get_kind()),
        breakpoint.get_is_enabled(),
        breakpoint.get_mechanism(),
        breakpoint.get_label().unwrap_or("")
    );
}

fn print_register_snapshot(register_snapshot: &DebuggerRegisterSnapshot) {
    if let Some(instruction_pointer) = register_snapshot.get_instruction_pointer() {
        println!("Instruction pointer: 0x{:X}.", instruction_pointer);
    }
    if let Some(stack_pointer) = register_snapshot.get_stack_pointer() {
        println!("Stack pointer: 0x{:X}.", stack_pointer);
    }

    for register_value in register_snapshot.get_registers() {
        println!(
            "{} = 0x{:X} [{} bits].",
            register_value.get_name(),
            register_value.get_value(),
            register_value.get_bit_width()
        );
    }
}

fn print_trace_session(trace_session: &DebuggerTraceSessionDescriptor) {
    println!(
        "Trace {} at 0x{:X}: access={}, size={} byte(s), active={}, breakpoint={}.",
        trace_session.get_trace_session_id(),
        trace_session.get_address(),
        trace_session.get_access().get_cli_label(),
        trace_session.get_size_in_bytes(),
        trace_session.get_is_active(),
        trace_session.get_breakpoint().get_breakpoint_id()
    );
}

fn print_instruction_records(instruction_records: &[DebuggerTraceInstructionRecord]) {
    for instruction_record in instruction_records {
        let instruction_address = instruction_record
            .get_instruction_address()
            .map(|address| format!("0x{:X}", address))
            .unwrap_or_else(|| String::from("unknown"));
        let instruction_text = instruction_record
            .get_instruction_text()
            .map(str::to_string)
            .unwrap_or_else(|| format_bytes(instruction_record.get_instruction_bytes()));

        println!(
            "Trace record {} at {}: hits={}, instruction={}.",
            instruction_record.get_trace_session_id(),
            instruction_address,
            instruction_record.get_hit_count(),
            instruction_text
        );
    }
}

fn format_breakpoint_kind(kind: &DebuggerBreakpointKind) -> String {
    match kind {
        DebuggerBreakpointKind::Software => String::from("software"),
        DebuggerBreakpointKind::HardwareData { access, size_in_bytes } => {
            format!("hardware-data:{}:{} byte(s)", access.get_cli_label(), size_in_bytes)
        }
    }
}

fn format_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{:02X}", byte))
        .collect::<Vec<_>>()
        .join(" ")
}
