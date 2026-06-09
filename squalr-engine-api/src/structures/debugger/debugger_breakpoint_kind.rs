use crate::structures::debugger::debugger_data_breakpoint_access::DebuggerDataBreakpointAccess;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum DebuggerBreakpointKind {
    Software,
    HardwareData {
        access: DebuggerDataBreakpointAccess,
        size_in_bytes: u8,
    },
    /// A hardware instruction (execute) breakpoint: the target stops when the instruction at the breakpoint address is
    /// executed. Used by instruction-directed traces ("find what addresses this instruction accesses").
    HardwareExecute,
}

impl DebuggerBreakpointKind {
    pub fn hardware_data(
        access: DebuggerDataBreakpointAccess,
        size_in_bytes: u8,
    ) -> Self {
        Self::HardwareData { access, size_in_bytes }
    }

    pub fn hardware_execute() -> Self {
        Self::HardwareExecute
    }
}
