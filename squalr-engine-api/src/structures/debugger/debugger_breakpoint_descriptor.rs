use crate::structures::debugger::debugger_breakpoint_kind::DebuggerBreakpointKind;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum DebuggerBreakpointMechanism {
    #[default]
    Debugger,
    MemoryPatch,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DebuggerBreakpointDescriptor {
    breakpoint_id: String,
    address: u64,
    kind: DebuggerBreakpointKind,
    is_enabled: bool,
    label: Option<String>,
    #[serde(default)]
    mechanism: DebuggerBreakpointMechanism,
}

impl DebuggerBreakpointDescriptor {
    pub fn new(
        breakpoint_id: impl Into<String>,
        address: u64,
        kind: DebuggerBreakpointKind,
        is_enabled: bool,
        label: Option<String>,
    ) -> Self {
        Self {
            breakpoint_id: breakpoint_id.into(),
            address,
            kind,
            is_enabled,
            label,
            mechanism: DebuggerBreakpointMechanism::Debugger,
        }
    }

    pub fn new_memory_patch(
        breakpoint_id: impl Into<String>,
        address: u64,
        kind: DebuggerBreakpointKind,
        is_enabled: bool,
        label: Option<String>,
    ) -> Self {
        Self {
            breakpoint_id: breakpoint_id.into(),
            address,
            kind,
            is_enabled,
            label,
            mechanism: DebuggerBreakpointMechanism::MemoryPatch,
        }
    }

    pub fn get_breakpoint_id(&self) -> &str {
        &self.breakpoint_id
    }

    pub fn get_address(&self) -> u64 {
        self.address
    }

    pub fn get_kind(&self) -> &DebuggerBreakpointKind {
        &self.kind
    }

    pub fn get_mechanism(&self) -> DebuggerBreakpointMechanism {
        self.mechanism
    }

    pub fn get_is_enabled(&self) -> bool {
        self.is_enabled
    }

    pub fn set_is_enabled(
        &mut self,
        is_enabled: bool,
    ) {
        self.is_enabled = is_enabled;
    }

    pub fn get_label(&self) -> Option<&str> {
        self.label.as_deref()
    }
}
