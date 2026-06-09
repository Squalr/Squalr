use crate::structures::debugger::{debugger_breakpoint_descriptor::DebuggerBreakpointDescriptor, debugger_register_snapshot::DebuggerRegisterSnapshot};
use crate::structures::processes::target_architecture::TargetArchitecture;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DebuggerTraceEvent {
    breakpoint: Option<DebuggerBreakpointDescriptor>,
    register_snapshot: DebuggerRegisterSnapshot,
    instruction_address: Option<u64>,
    instruction_bytes: Vec<u8>,
    instruction_text: Option<String>,
    #[serde(default)]
    target_architecture: Option<TargetArchitecture>,
    backend_message: Option<String>,
    /// For instruction-directed (execute-breakpoint) traces: the memory address this instruction accessed on this hit,
    /// computed from the instruction's memory operand and the register snapshot. `None` for data-watchpoint traces or
    /// when the accessed address could not be resolved.
    #[serde(default)]
    accessed_address: Option<u64>,
}

impl DebuggerTraceEvent {
    pub fn new(
        breakpoint: Option<DebuggerBreakpointDescriptor>,
        register_snapshot: DebuggerRegisterSnapshot,
        instruction_address: Option<u64>,
        instruction_bytes: Vec<u8>,
        instruction_text: Option<String>,
        backend_message: Option<String>,
    ) -> Self {
        Self {
            breakpoint,
            register_snapshot,
            instruction_address,
            instruction_bytes,
            instruction_text,
            target_architecture: None,
            backend_message,
            accessed_address: None,
        }
    }

    pub fn with_accessed_address(
        mut self,
        accessed_address: Option<u64>,
    ) -> Self {
        self.accessed_address = accessed_address;

        self
    }

    pub fn get_accessed_address(&self) -> Option<u64> {
        self.accessed_address
    }

    pub fn with_target_architecture(
        mut self,
        target_architecture: TargetArchitecture,
    ) -> Self {
        self.target_architecture = Some(target_architecture);

        self
    }

    pub fn with_instruction_address(
        mut self,
        instruction_address: Option<u64>,
    ) -> Self {
        self.instruction_address = instruction_address;

        self
    }

    pub fn get_breakpoint(&self) -> Option<&DebuggerBreakpointDescriptor> {
        self.breakpoint.as_ref()
    }

    pub fn get_register_snapshot(&self) -> &DebuggerRegisterSnapshot {
        &self.register_snapshot
    }

    pub fn get_instruction_address(&self) -> Option<u64> {
        self.instruction_address
    }

    pub fn get_instruction_bytes(&self) -> &[u8] {
        &self.instruction_bytes
    }

    pub fn get_instruction_text(&self) -> Option<&str> {
        self.instruction_text.as_deref()
    }

    pub fn get_target_architecture(&self) -> Option<&TargetArchitecture> {
        self.target_architecture.as_ref()
    }

    pub fn get_backend_message(&self) -> Option<&str> {
        self.backend_message.as_deref()
    }
}
