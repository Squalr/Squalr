use crate::structures::debugger::{DebuggerBreakpointDescriptor, DebuggerDataBreakpointAccess};
use serde::{Deserialize, Serialize};

/// Which direction a trace session runs in.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum DebuggerTraceTargetKind {
    /// Address-directed: a hardware data watchpoint on `address`/`size_in_bytes` records which instructions access it.
    #[default]
    Address,
    /// Instruction-directed: a hardware execute breakpoint at `address` (the instruction) records which memory
    /// addresses the instruction accesses.
    Instruction,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DebuggerTraceSessionDescriptor {
    trace_session_id: String,
    address: u64,
    size_in_bytes: u8,
    access: DebuggerDataBreakpointAccess,
    breakpoint: DebuggerBreakpointDescriptor,
    label: Option<String>,
    is_active: bool,
    #[serde(default)]
    target_kind: DebuggerTraceTargetKind,
}

impl DebuggerTraceSessionDescriptor {
    pub fn new(
        trace_session_id: impl Into<String>,
        address: u64,
        size_in_bytes: u8,
        access: DebuggerDataBreakpointAccess,
        breakpoint: DebuggerBreakpointDescriptor,
        label: Option<String>,
        is_active: bool,
    ) -> Self {
        Self {
            trace_session_id: trace_session_id.into(),
            address,
            size_in_bytes,
            access,
            breakpoint,
            label,
            is_active,
            target_kind: DebuggerTraceTargetKind::Address,
        }
    }

    /// Instruction-directed session: `instruction_address` is the execute-breakpoint address. `size_in_bytes` is stored
    /// as the instruction length (informational); `access` is the user-selected filter/label.
    pub fn new_for_instruction(
        trace_session_id: impl Into<String>,
        instruction_address: u64,
        access: DebuggerDataBreakpointAccess,
        breakpoint: DebuggerBreakpointDescriptor,
        label: Option<String>,
        is_active: bool,
    ) -> Self {
        Self {
            trace_session_id: trace_session_id.into(),
            address: instruction_address,
            size_in_bytes: 0,
            access,
            breakpoint,
            label,
            is_active,
            target_kind: DebuggerTraceTargetKind::Instruction,
        }
    }

    pub fn get_target_kind(&self) -> DebuggerTraceTargetKind {
        self.target_kind
    }

    pub fn get_trace_session_id(&self) -> &str {
        &self.trace_session_id
    }

    pub fn get_address(&self) -> u64 {
        self.address
    }

    pub fn get_size_in_bytes(&self) -> u8 {
        self.size_in_bytes
    }

    pub fn get_access(&self) -> DebuggerDataBreakpointAccess {
        self.access
    }

    pub fn get_breakpoint(&self) -> &DebuggerBreakpointDescriptor {
        &self.breakpoint
    }

    pub fn set_breakpoint(
        &mut self,
        breakpoint: DebuggerBreakpointDescriptor,
    ) {
        self.breakpoint = breakpoint;
    }

    pub fn get_label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    pub fn get_is_active(&self) -> bool {
        self.is_active
    }

    pub fn set_is_active(
        &mut self,
        is_active: bool,
    ) {
        self.is_active = is_active;
    }
}
