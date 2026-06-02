use crate::structures::debugger::{DebuggerBreakpointDescriptor, DebuggerDataBreakpointAccess};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DebuggerTraceSessionDescriptor {
    trace_session_id: String,
    address: u64,
    size_in_bytes: u8,
    access: DebuggerDataBreakpointAccess,
    breakpoint: DebuggerBreakpointDescriptor,
    label: Option<String>,
    is_active: bool,
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
        }
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
