use crate::structures::debugger::{debugger_breakpoint_descriptor::DebuggerBreakpointDescriptor, debugger_register_snapshot::DebuggerRegisterSnapshot};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DebuggerTraceEvent {
    breakpoint: Option<DebuggerBreakpointDescriptor>,
    register_snapshot: DebuggerRegisterSnapshot,
    instruction_bytes: Vec<u8>,
    instruction_text: Option<String>,
    backend_message: Option<String>,
}

impl DebuggerTraceEvent {
    pub fn new(
        breakpoint: Option<DebuggerBreakpointDescriptor>,
        register_snapshot: DebuggerRegisterSnapshot,
        instruction_bytes: Vec<u8>,
        instruction_text: Option<String>,
        backend_message: Option<String>,
    ) -> Self {
        Self {
            breakpoint,
            register_snapshot,
            instruction_bytes,
            instruction_text,
            backend_message,
        }
    }

    pub fn get_breakpoint(&self) -> Option<&DebuggerBreakpointDescriptor> {
        self.breakpoint.as_ref()
    }

    pub fn get_register_snapshot(&self) -> &DebuggerRegisterSnapshot {
        &self.register_snapshot
    }

    pub fn get_instruction_bytes(&self) -> &[u8] {
        &self.instruction_bytes
    }

    pub fn get_instruction_text(&self) -> Option<&str> {
        self.instruction_text.as_deref()
    }

    pub fn get_backend_message(&self) -> Option<&str> {
        self.backend_message.as_deref()
    }
}
