use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::{
    events::debugger::{
        session_state_changed::debugger_session_state_changed_event::DebuggerSessionStateChangedEvent,
        trace_session_updated::debugger_trace_session_updated_event::DebuggerTraceSessionUpdatedEvent,
    },
    structures::debugger::{DebuggerDataBreakpointAccess, DebuggerSessionState, DebuggerTraceInstructionRecord, DebuggerTraceSessionDescriptor},
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug, Default)]
pub struct DebuggerTraceViewData {
    inner: Arc<RwLock<DebuggerTraceViewState>>,
}

#[derive(Clone, Debug, Default)]
struct DebuggerTraceViewState {
    trace_sessions: HashMap<String, DebuggerTraceSessionDescriptor>,
    instruction_records_by_trace_session_id: HashMap<String, Vec<DebuggerTraceInstructionRecord>>,
    selected_instruction_keys: Vec<DebuggerTraceInstructionKey>,
    pending_trace_start_request: Option<PendingDebuggerTraceStartRequest>,
    pending_trace_start_status_message: Option<String>,
    is_starting_pending_trace: bool,
    debugger_session_state: DebuggerSessionState,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DebuggerTraceInstructionKey {
    trace_session_id: String,
    instruction_address: Option<u64>,
    instruction_bytes: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct PendingDebuggerTraceStartRequest {
    address: u64,
    size_in_bytes: u8,
    access: DebuggerDataBreakpointAccess,
    label: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct DebuggerTraceSnapshot {
    pub trace_sessions: Vec<DebuggerTraceSessionDescriptor>,
    pub instruction_records: Vec<DebuggerTraceInstructionRecord>,
    pub selected_instruction_keys: Vec<DebuggerTraceInstructionKey>,
    pub pending_trace_start_request: Option<PendingDebuggerTraceStartRequest>,
    pub pending_trace_start_status_message: Option<String>,
    pub is_starting_pending_trace: bool,
    pub debugger_session_state: DebuggerSessionState,
}

impl DebuggerTraceInstructionKey {
    pub fn from_record(instruction_record: &DebuggerTraceInstructionRecord) -> Self {
        Self {
            trace_session_id: instruction_record.get_trace_session_id().to_string(),
            instruction_address: instruction_record.get_instruction_address(),
            instruction_bytes: instruction_record.get_instruction_bytes().to_vec(),
        }
    }

    pub fn get_trace_session_id(&self) -> &str {
        &self.trace_session_id
    }
}

impl PendingDebuggerTraceStartRequest {
    pub fn new(
        address: u64,
        size_in_bytes: u8,
        access: DebuggerDataBreakpointAccess,
        label: Option<String>,
    ) -> Self {
        Self {
            address,
            size_in_bytes,
            access,
            label,
        }
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

    pub fn get_label(&self) -> Option<&str> {
        self.label.as_deref()
    }
}

impl DebuggerTraceViewData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn request_trace_start(
        debugger_trace_view_data: Dependency<Self>,
        pending_trace_start_request: PendingDebuggerTraceStartRequest,
    ) {
        let Some(debugger_trace_view_data) = debugger_trace_view_data.read("Debugger trace request trace start") else {
            return;
        };

        debugger_trace_view_data.set_pending_trace_start_request(pending_trace_start_request);
    }

    pub fn apply_trace_session_updated(
        &self,
        debugger_trace_session_updated_event: &DebuggerTraceSessionUpdatedEvent,
    ) {
        let trace_session_id = debugger_trace_session_updated_event
            .trace_session
            .get_trace_session_id()
            .to_string();

        match self.inner.write() {
            Ok(mut state) => {
                state
                    .trace_sessions
                    .insert(trace_session_id.clone(), debugger_trace_session_updated_event.trace_session.clone());
                state
                    .instruction_records_by_trace_session_id
                    .insert(trace_session_id, debugger_trace_session_updated_event.instruction_records.clone());
                state.retain_valid_selection();
            }
            Err(error) => {
                log::error!("Failed to update debugger trace view data: {}", error);
            }
        }
    }

    pub fn apply_debugger_session_state_changed(
        &self,
        debugger_session_state_changed_event: &DebuggerSessionStateChangedEvent,
    ) {
        self.set_debugger_session_state(debugger_session_state_changed_event.session_state);
    }

    pub fn set_debugger_session_state(
        &self,
        debugger_session_state: DebuggerSessionState,
    ) {
        match self.inner.write() {
            Ok(mut state) => {
                state.debugger_session_state = debugger_session_state;
            }
            Err(error) => {
                log::error!("Failed to update debugger session state in trace view data: {}", error);
            }
        }
    }

    pub fn get_snapshot(&self) -> DebuggerTraceSnapshot {
        match self.inner.read() {
            Ok(state) => state.snapshot(),
            Err(error) => {
                log::error!("Failed to read debugger trace view data: {}", error);
                DebuggerTraceSnapshot::default()
            }
        }
    }

    pub fn set_single_instruction_selection(
        &self,
        instruction_key: DebuggerTraceInstructionKey,
    ) {
        match self.inner.write() {
            Ok(mut state) => {
                state.selected_instruction_keys.clear();
                state.selected_instruction_keys.push(instruction_key);
            }
            Err(error) => {
                log::error!("Failed to set debugger trace instruction selection: {}", error);
            }
        }
    }

    pub fn set_pending_trace_start_request(
        &self,
        pending_trace_start_request: PendingDebuggerTraceStartRequest,
    ) {
        match self.inner.write() {
            Ok(mut state) => {
                state.pending_trace_start_request = Some(pending_trace_start_request);
                state.pending_trace_start_status_message = None;
                state.is_starting_pending_trace = false;
            }
            Err(error) => {
                log::error!("Failed to queue debugger trace start prompt: {}", error);
            }
        }
    }

    pub fn cancel_pending_trace_start(&self) {
        match self.inner.write() {
            Ok(mut state) => {
                state.pending_trace_start_request = None;
                state.pending_trace_start_status_message = None;
                state.is_starting_pending_trace = false;
            }
            Err(error) => {
                log::error!("Failed to cancel debugger trace start prompt: {}", error);
            }
        }
    }

    pub fn begin_pending_trace_start(&self) -> Option<PendingDebuggerTraceStartRequest> {
        match self.inner.write() {
            Ok(mut state) => {
                if state.is_starting_pending_trace {
                    return None;
                }

                let pending_trace_start_request = state.pending_trace_start_request.clone()?;
                state.pending_trace_start_status_message = None;
                state.is_starting_pending_trace = true;

                Some(pending_trace_start_request)
            }
            Err(error) => {
                log::error!("Failed to begin debugger trace start prompt: {}", error);
                None
            }
        }
    }

    pub fn complete_pending_trace_start(&self) {
        self.cancel_pending_trace_start();
    }

    pub fn fail_pending_trace_start(
        &self,
        status_message: String,
    ) {
        match self.inner.write() {
            Ok(mut state) => {
                state.pending_trace_start_status_message = Some(status_message);
                state.is_starting_pending_trace = false;
            }
            Err(error) => {
                log::error!("Failed to update debugger trace start prompt: {}", error);
            }
        }
    }
}

impl DebuggerTraceViewState {
    fn snapshot(&self) -> DebuggerTraceSnapshot {
        let mut trace_sessions = self.trace_sessions.values().cloned().collect::<Vec<_>>();
        let mut instruction_records = self
            .instruction_records_by_trace_session_id
            .values()
            .flat_map(|instruction_records| instruction_records.iter().cloned())
            .collect::<Vec<_>>();

        trace_sessions.sort_by(|left, right| left.get_trace_session_id().cmp(right.get_trace_session_id()));
        instruction_records.sort_by(|left, right| {
            left.get_trace_session_id()
                .cmp(right.get_trace_session_id())
                .then(
                    left.get_instruction_address()
                        .cmp(&right.get_instruction_address()),
                )
        });

        DebuggerTraceSnapshot {
            trace_sessions,
            instruction_records,
            selected_instruction_keys: self.selected_instruction_keys.clone(),
            pending_trace_start_request: self.pending_trace_start_request.clone(),
            pending_trace_start_status_message: self.pending_trace_start_status_message.clone(),
            is_starting_pending_trace: self.is_starting_pending_trace,
            debugger_session_state: self.debugger_session_state,
        }
    }

    fn retain_valid_selection(&mut self) {
        let valid_instruction_keys = self
            .instruction_records_by_trace_session_id
            .values()
            .flat_map(|instruction_records| {
                instruction_records
                    .iter()
                    .map(DebuggerTraceInstructionKey::from_record)
            })
            .collect::<Vec<_>>();

        self.selected_instruction_keys
            .retain(|selected_instruction_key| valid_instruction_keys.contains(selected_instruction_key));
    }
}
