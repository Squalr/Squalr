use crate::structures::debugger::{DebuggerRegisterSnapshot, DebuggerTraceEvent};
use crate::structures::processes::target_architecture::TargetArchitecture;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DebuggerTraceInstructionRecord {
    trace_session_id: String,
    instruction_address: Option<u64>,
    instruction_bytes: Vec<u8>,
    instruction_text: Option<String>,
    #[serde(default)]
    target_architecture: Option<TargetArchitecture>,
    hit_count: u64,
    last_register_snapshot: DebuggerRegisterSnapshot,
    last_backend_message: Option<String>,
    /// For instruction-directed traces: the memory address this instruction accessed (records are keyed by this for
    /// instruction sessions). `None` for data-watchpoint records.
    #[serde(default)]
    accessed_address: Option<u64>,
}

impl DebuggerTraceInstructionRecord {
    pub fn new(
        trace_session_id: impl Into<String>,
        trace_event: &DebuggerTraceEvent,
    ) -> Self {
        Self {
            trace_session_id: trace_session_id.into(),
            instruction_address: trace_event.get_instruction_address(),
            instruction_bytes: trace_event.get_instruction_bytes().to_vec(),
            instruction_text: trace_event.get_instruction_text().map(String::from),
            target_architecture: trace_event.get_target_architecture().cloned(),
            hit_count: 1,
            last_register_snapshot: trace_event.get_register_snapshot().clone(),
            last_backend_message: trace_event.get_backend_message().map(String::from),
            accessed_address: trace_event.get_accessed_address(),
        }
    }

    pub fn record_hit(
        &mut self,
        trace_event: &DebuggerTraceEvent,
    ) {
        self.hit_count += 1;
        self.instruction_bytes = trace_event.get_instruction_bytes().to_vec();
        self.instruction_text = trace_event.get_instruction_text().map(String::from);
        self.target_architecture = trace_event.get_target_architecture().cloned();
        self.last_register_snapshot = trace_event.get_register_snapshot().clone();
        self.last_backend_message = trace_event.get_backend_message().map(String::from);
        self.accessed_address = trace_event.get_accessed_address();
    }

    pub fn get_accessed_address(&self) -> Option<u64> {
        self.accessed_address
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

    pub fn get_target_architecture(&self) -> Option<&TargetArchitecture> {
        self.target_architecture.as_ref()
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

#[cfg(test)]
mod tests {
    use super::DebuggerTraceInstructionRecord;
    use crate::structures::debugger::{DebuggerRegisterSnapshot, DebuggerTraceEvent};
    use crate::structures::processes::target_architecture::TargetArchitecture;

    #[test]
    fn trace_instruction_record_preserves_event_target_architecture() {
        let trace_event = DebuggerTraceEvent::new(
            None,
            DebuggerRegisterSnapshot::default(),
            Some(0x1000),
            vec![0x1F, 0x20, 0x03, 0xD5],
            None,
            None,
        )
        .with_target_architecture(TargetArchitecture::arm64());

        let instruction_record = DebuggerTraceInstructionRecord::new("trace-session", &trace_event);

        assert_eq!(
            instruction_record
                .get_target_architecture()
                .expect("Expected target architecture.")
                .get_instruction_set_id(),
            "arm64"
        );
    }
}
