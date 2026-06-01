use crate::structures::debugger::debugger_data_breakpoint_access::DebuggerDataBreakpointAccess;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum DebuggerBreakpointKind {
    Software,
    HardwareData { access: DebuggerDataBreakpointAccess, size_in_bytes: u8 },
}

impl DebuggerBreakpointKind {
    pub fn hardware_data(
        access: DebuggerDataBreakpointAccess,
        size_in_bytes: u8,
    ) -> Self {
        Self::HardwareData { access, size_in_bytes }
    }
}
