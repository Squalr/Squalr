use crate::structures::debugger::{DebuggerRegisterSnapshot, DebuggerTraceEvent};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DebuggerTraceInstructionRecord {
    trace_session_id: String,
    instruction_address: Option<u64>,
    instruction_bytes: Vec<u8>,
    instruction_text: Option<String>,
    hit_count: u64,
    last_register_snapshot: DebuggerRegisterSnapshot,
    last_backend_message: Option<String>,
}

impl DebuggerTraceInstructionRecord {
    pub fn new(
        trace_session_id: impl Into<String>,
        trace_event: &DebuggerTraceEvent,
    ) -> Self {
        Self {
            trace_session_id: trace_session_id.into(),
            instruction_address: trace_event.get_register_snapshot().get_instruction_pointer(),
            instruction_bytes: trace_event.get_instruction_bytes().to_vec(),
            instruction_text: trace_event.get_instruction_text().map(String::from),
            hit_count: 1,
            last_register_snapshot: trace_event.get_register_snapshot().clone(),
            last_backend_message: trace_event.get_backend_message().map(String::from),
        }
    }

    pub fn record_hit(
        &mut self,
        trace_event: &DebuggerTraceEvent,
    ) {
        self.hit_count += 1;
        self.instruction_bytes = trace_event.get_instruction_bytes().to_vec();
        self.instruction_text = trace_event.get_instruction_text().map(String::from);
        self.last_register_snapshot = trace_event.get_register_snapshot().clone();
        self.last_backend_message = trace_event.get_backend_message().map(String::from);
    }

    pub fn get_trace_session_id(&self) -> &str {
        &self.trace_session_id
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

    pub fn get_hit_count(&self) -> u64 {
        self.hit_count
    }

    pub fn get_last_register_snapshot(&self) -> &DebuggerRegisterSnapshot {
        &self.last_register_snapshot
    }

    pub fn get_last_backend_message(&self) -> Option<&str> {
        self.last_backend_message.as_deref()
    }
}
