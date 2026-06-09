use crate::ui::widgets::controls::data_type_selector::data_type_selection::DataTypeSelection;
use eframe::egui::Pos2;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
use squalr_engine_api::{
    events::debugger::trace_session_updated::debugger_trace_session_updated_event::DebuggerTraceSessionUpdatedEvent,
    structures::debugger::{DebuggerDataBreakpointAccess, DebuggerTraceInstructionRecord, DebuggerTraceSessionDescriptor, DebuggerTraceTargetKind},
};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

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
    pending_trace_start_operation_id: u64,
    active_trace_start_operation_id: Option<u64>,
    pending_trace_start_started_at: Option<Instant>,
    is_starting_pending_trace: bool,
    pending_control_trace_session_ids: HashSet<String>,
    instruction_context_menu_target: Option<DebuggerTraceInstructionContextMenuTarget>,
    // Column splitter positions as fractions of the table width (0 = unset → fall back to defaults). Lets the trace
    // table columns be resized like the scan results / other tables.
    instruction_splitter_ratio: f32,
    address_splitter_ratio: f32,
    value_splitter_ratio: f32,
    // How the accessed memory at each address is interpreted/formatted for the Value column (mirrors the scanner's
    // data-type + display-format selectors). None falls back to defaults (i32 / the type's default format).
    value_data_type_id: Option<String>,
    value_display_format: Option<AnonymousValueStringFormat>,
    // Persistent cache of the last successfully-read value per accessed address. Keeping prior values across frames (and
    // only resetting the snapshot queries when the queried address set / data type changes) avoids the per-frame flicker
    // that occurred when values briefly blanked between virtual-snapshot refreshes. The cache is only cleared when the
    // *interpretation* (data type or display format) changes, so a growing address set never wipes already-read values.
    preview_value_by_address: HashMap<u64, String>,
    preview_query_signature: u64,
    preview_interpretation_signature: u64,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DebuggerTraceInstructionKey {
    trace_session_id: String,
    instruction_address: Option<u64>,
    instruction_bytes: Vec<u8>,
    // Instruction-directed records share one instruction but differ by accessed address, so it is part of the key.
    accessed_address: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct DebuggerTraceInstructionContextMenuTarget {
    instruction_key: DebuggerTraceInstructionKey,
    position: Pos2,
}

#[derive(Clone, Debug)]
pub struct PendingDebuggerTraceStartRequest {
    address: u64,
    size_in_bytes: u8,
    access: DebuggerDataBreakpointAccess,
    label: Option<String>,
    target_kind: DebuggerTraceTargetKind,
}

#[derive(Clone, Debug)]
pub struct PendingDebuggerTraceStartOperation {
    operation_id: u64,
    request: PendingDebuggerTraceStartRequest,
}

#[derive(Clone, Debug, Default)]
pub struct DebuggerTraceSnapshot {
    pub trace_sessions: Vec<DebuggerTraceSessionDescriptor>,
    pub instruction_records: Vec<DebuggerTraceInstructionRecord>,
    pub selected_instruction_keys: Vec<DebuggerTraceInstructionKey>,
    pub pending_trace_start_request: Option<PendingDebuggerTraceStartRequest>,
    pub pending_trace_start_status_message: Option<String>,
    pub is_starting_pending_trace: bool,
    pub pending_control_trace_session_ids: HashSet<String>,
    pub instruction_context_menu_target: Option<DebuggerTraceInstructionContextMenuTarget>,
}

impl DebuggerTraceInstructionKey {
    pub fn from_record(instruction_record: &DebuggerTraceInstructionRecord) -> Self {
        Self {
            trace_session_id: instruction_record.get_trace_session_id().to_string(),
            instruction_address: instruction_record.get_instruction_address(),
            instruction_bytes: if instruction_record.get_instruction_address().is_some() {
                Vec::new()
            } else {
                instruction_record.get_instruction_bytes().to_vec()
            },
            accessed_address: instruction_record.get_accessed_address(),
        }
    }

    pub fn get_trace_session_id(&self) -> &str {
        &self.trace_session_id
    }
}

impl DebuggerTraceInstructionContextMenuTarget {
    pub fn new(
        instruction_key: DebuggerTraceInstructionKey,
        position: Pos2,
    ) -> Self {
        Self { instruction_key, position }
    }

    pub fn get_instruction_key(&self) -> &DebuggerTraceInstructionKey {
        &self.instruction_key
    }

    pub fn get_position(&self) -> Pos2 {
        self.position
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
            target_kind: DebuggerTraceTargetKind::Address,
        }
    }

    /// Instruction-directed trace: execute breakpoint at `instruction_address`, recording what it accesses.
    pub fn new_for_instruction(
        instruction_address: u64,
        access: DebuggerDataBreakpointAccess,
        label: Option<String>,
    ) -> Self {
        Self {
            address: instruction_address,
            size_in_bytes: 0,
            access,
            label,
            target_kind: DebuggerTraceTargetKind::Instruction,
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

    pub fn get_target_kind(&self) -> DebuggerTraceTargetKind {
        self.target_kind
    }
}

impl PendingDebuggerTraceStartOperation {
    pub fn get_operation_id(&self) -> u64 {
        self.operation_id
    }

    pub fn into_request(self) -> PendingDebuggerTraceStartRequest {
        self.request
    }
}

impl DebuggerTraceViewData {
    const DEFAULT_INSTRUCTION_SPLITTER_RATIO: f32 = 0.14;
    const DEFAULT_ADDRESS_SPLITTER_RATIO: f32 = 0.42;
    const DEFAULT_VALUE_SPLITTER_RATIO: f32 = 0.66;
    const DEFAULT_VALUE_DATA_TYPE_ID: &'static str = "i32";

    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the (instruction, address, value) column splitter fractions, falling back to defaults when unset. The Type
    /// and Value controls share a single column whose left edge is the value splitter.
    pub fn get_column_splitter_ratios(&self) -> (f32, f32, f32) {
        self.inner
            .read()
            .ok()
            .map(|state| {
                (
                    Self::effective_ratio(state.instruction_splitter_ratio, Self::DEFAULT_INSTRUCTION_SPLITTER_RATIO),
                    Self::effective_ratio(state.address_splitter_ratio, Self::DEFAULT_ADDRESS_SPLITTER_RATIO),
                    Self::effective_ratio(state.value_splitter_ratio, Self::DEFAULT_VALUE_SPLITTER_RATIO),
                )
            })
            .unwrap_or((
                Self::DEFAULT_INSTRUCTION_SPLITTER_RATIO,
                Self::DEFAULT_ADDRESS_SPLITTER_RATIO,
                Self::DEFAULT_VALUE_SPLITTER_RATIO,
            ))
    }

    pub fn set_column_splitter_ratios(
        &self,
        instruction_splitter_ratio: f32,
        address_splitter_ratio: f32,
        value_splitter_ratio: f32,
    ) {
        if let Ok(mut state) = self.inner.write() {
            // Keep the splitters ordered with a minimum column width so columns cannot collapse or cross over.
            let instruction_splitter_ratio = instruction_splitter_ratio.clamp(0.05, 0.70);
            let address_splitter_ratio = address_splitter_ratio.clamp(instruction_splitter_ratio + 0.05, 0.85);
            let value_splitter_ratio = value_splitter_ratio.clamp(address_splitter_ratio + 0.05, 0.92);

            state.instruction_splitter_ratio = instruction_splitter_ratio;
            state.address_splitter_ratio = address_splitter_ratio;
            state.value_splitter_ratio = value_splitter_ratio;
        }
    }

    fn effective_ratio(
        stored_ratio: f32,
        default_ratio: f32,
    ) -> f32 {
        if stored_ratio > 0.0 && stored_ratio < 1.0 { stored_ratio } else { default_ratio }
    }

    /// The data type id used to interpret the accessed memory for the Value column (defaults to i32).
    pub fn get_value_data_type_id(&self) -> String {
        self.inner
            .read()
            .ok()
            .and_then(|state| state.value_data_type_id.clone())
            .unwrap_or_else(|| Self::DEFAULT_VALUE_DATA_TYPE_ID.to_string())
    }

    /// The display format used to render the Value column, if the user has chosen one.
    pub fn get_value_display_format(&self) -> Option<AnonymousValueStringFormat> {
        self.inner.read().ok().and_then(|state| state.value_display_format)
    }

    /// Runs `render` with mutable access to the value-column data-type selection and display format so the header can host
    /// the data-type + display-format selector widgets, persisting any user changes back into the view state.
    pub fn with_value_format_controls<R>(
        &self,
        default_display_format: AnonymousValueStringFormat,
        render: impl FnOnce(&mut DataTypeSelection, &mut AnonymousValueStringFormat) -> R,
    ) -> Option<R> {
        let mut state = self.inner.write().ok()?;
        let data_type_id = state
            .value_data_type_id
            .clone()
            .unwrap_or_else(|| Self::DEFAULT_VALUE_DATA_TYPE_ID.to_string());
        let mut data_type_selection = DataTypeSelection::new(DataTypeRef::new(&data_type_id));
        let mut display_format = state.value_display_format.unwrap_or(default_display_format);

        let result = render(&mut data_type_selection, &mut display_format);

        state.value_data_type_id = Some(data_type_selection.active_data_type().get_data_type_id().to_string());
        state.value_display_format = Some(display_format);

        Some(result)
    }

    /// Updates the signature describing the queried address set + data type. Returns true when it changed, signalling the
    /// caller to rebuild the virtual snapshot queries. Does NOT clear cached values, so a growing address set keeps the
    /// values already read for existing addresses.
    pub fn update_preview_query_signature(
        &self,
        signature: u64,
    ) -> bool {
        if let Ok(mut state) = self.inner.write() {
            if state.preview_query_signature != signature {
                state.preview_query_signature = signature;
                return true;
            }
        }

        false
    }

    /// Updates the signature describing how values are interpreted (data type + display format). When it changes the
    /// cached values are cleared so they get re-read/re-formatted under the new interpretation.
    pub fn update_preview_interpretation_signature(
        &self,
        signature: u64,
    ) {
        if let Ok(mut state) = self.inner.write() {
            if state.preview_interpretation_signature != signature {
                state.preview_interpretation_signature = signature;
                state.preview_value_by_address.clear();
            }
        }
    }

    /// Merges freshly-read values into the persistent per-address cache (values already present are overwritten with the
    /// newer read; addresses missing from this batch keep their previous value to avoid flicker).
    pub fn merge_preview_values(
        &self,
        fresh_preview_values: HashMap<u64, String>,
    ) {
        if fresh_preview_values.is_empty() {
            return;
        }

        if let Ok(mut state) = self.inner.write() {
            for (accessed_address, preview_value) in fresh_preview_values {
                state.preview_value_by_address.insert(accessed_address, preview_value);
            }
        }
    }

    /// Snapshot of the cached per-address preview values.
    pub fn get_preview_values(&self) -> HashMap<u64, String> {
        self.inner
            .read()
            .ok()
            .map(|state| state.preview_value_by_address.clone())
            .unwrap_or_default()
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
                if !debugger_trace_session_updated_event
                    .trace_session
                    .get_is_active()
                {
                    state.trace_sessions.remove(&trace_session_id);
                    state
                        .instruction_records_by_trace_session_id
                        .remove(&trace_session_id);
                    state
                        .pending_control_trace_session_ids
                        .remove(&trace_session_id);
                    state.retain_valid_selection();

                    return;
                }

                state
                    .trace_sessions
                    .insert(trace_session_id.clone(), debugger_trace_session_updated_event.trace_session.clone());
                state
                    .instruction_records_by_trace_session_id
                    .insert(trace_session_id, debugger_trace_session_updated_event.instruction_records.clone());
                if !debugger_trace_session_updated_event
                    .trace_session
                    .get_is_active()
                {
                    state.pending_control_trace_session_ids.remove(
                        debugger_trace_session_updated_event
                            .trace_session
                            .get_trace_session_id(),
                    );
                }
                state.retain_valid_selection();
            }
            Err(error) => {
                log::error!("Failed to update debugger trace view data: {}", error);
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

    pub fn expire_stale_pending_trace_start(
        &self,
        timeout: Duration,
    ) {
        match self.inner.write() {
            Ok(mut state) => {
                let Some(pending_trace_start_started_at) = state.pending_trace_start_started_at else {
                    return;
                };

                if !state.is_starting_pending_trace || pending_trace_start_started_at.elapsed() < timeout {
                    return;
                }

                state.pending_trace_start_status_message = Some(String::from("Debugger trace start timed out. Try the action again."));
                state.is_starting_pending_trace = false;
                state.active_trace_start_operation_id = None;
                state.pending_trace_start_started_at = None;
            }
            Err(error) => {
                log::error!("Failed to expire stale debugger trace start prompt: {}", error);
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

    pub fn show_instruction_context_menu(
        &self,
        instruction_key: DebuggerTraceInstructionKey,
        position: Pos2,
    ) {
        match self.inner.write() {
            Ok(mut state) => {
                state.selected_instruction_keys.clear();
                state.selected_instruction_keys.push(instruction_key.clone());
                state.instruction_context_menu_target = Some(DebuggerTraceInstructionContextMenuTarget::new(instruction_key, position));
            }
            Err(error) => {
                log::error!("Failed to show debugger trace instruction context menu: {}", error);
            }
        }
    }

    pub fn hide_instruction_context_menu(&self) {
        match self.inner.write() {
            Ok(mut state) => {
                state.instruction_context_menu_target = None;
            }
            Err(error) => {
                log::error!("Failed to hide debugger trace instruction context menu: {}", error);
            }
        }
    }

    pub fn set_pending_trace_start_request(
        &self,
        pending_trace_start_request: PendingDebuggerTraceStartRequest,
    ) {
        match self.inner.write() {
            Ok(mut state) => {
                state.pending_trace_start_operation_id = state.pending_trace_start_operation_id.wrapping_add(1);
                state.pending_trace_start_request = Some(pending_trace_start_request);
                state.pending_trace_start_status_message = None;
                state.active_trace_start_operation_id = None;
                state.pending_trace_start_started_at = None;
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
                state.active_trace_start_operation_id = None;
                state.pending_trace_start_started_at = None;
                state.is_starting_pending_trace = false;
            }
            Err(error) => {
                log::error!("Failed to cancel debugger trace start prompt: {}", error);
            }
        }
    }

    pub fn begin_pending_trace_start(&self) -> Option<PendingDebuggerTraceStartOperation> {
        match self.inner.write() {
            Ok(mut state) => {
                if state.is_starting_pending_trace {
                    return None;
                }

                let pending_trace_start_request = state.pending_trace_start_request.clone()?;
                let operation_id = state.pending_trace_start_operation_id;
                state.pending_trace_start_status_message = None;
                state.active_trace_start_operation_id = Some(operation_id);
                state.pending_trace_start_started_at = Some(Instant::now());
                state.is_starting_pending_trace = true;

                Some(PendingDebuggerTraceStartOperation {
                    operation_id,
                    request: pending_trace_start_request,
                })
            }
            Err(error) => {
                log::error!("Failed to begin debugger trace start prompt: {}", error);
                None
            }
        }
    }

    pub fn complete_pending_trace_start(
        &self,
        operation_id: u64,
    ) {
        match self.inner.write() {
            Ok(mut state) => {
                if state.active_trace_start_operation_id != Some(operation_id) {
                    return;
                }

                state.pending_trace_start_request = None;
                state.pending_trace_start_status_message = None;
                state.active_trace_start_operation_id = None;
                state.pending_trace_start_started_at = None;
                state.is_starting_pending_trace = false;
            }
            Err(error) => {
                log::error!("Failed to complete debugger trace start prompt: {}", error);
            }
        }
    }

    pub fn fail_pending_trace_start(
        &self,
        operation_id: u64,
        status_message: String,
    ) {
        match self.inner.write() {
            Ok(mut state) => {
                if state.active_trace_start_operation_id != Some(operation_id) {
                    return;
                }

                state.pending_trace_start_status_message = Some(status_message);
                state.active_trace_start_operation_id = None;
                state.pending_trace_start_started_at = None;
                state.is_starting_pending_trace = false;
            }
            Err(error) => {
                log::error!("Failed to update debugger trace start prompt: {}", error);
            }
        }
    }

    pub fn begin_trace_control(
        &self,
        trace_session_id: &str,
    ) -> bool {
        match self.inner.write() {
            Ok(mut state) => state
                .pending_control_trace_session_ids
                .insert(trace_session_id.to_string()),
            Err(error) => {
                log::error!("Failed to begin debugger trace control operation: {}", error);
                false
            }
        }
    }

    pub fn complete_trace_control(
        &self,
        trace_session_id: &str,
    ) {
        match self.inner.write() {
            Ok(mut state) => {
                state.pending_control_trace_session_ids.remove(trace_session_id);
            }
            Err(error) => {
                log::error!("Failed to complete debugger trace control operation: {}", error);
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
            pending_control_trace_session_ids: self.pending_control_trace_session_ids.clone(),
            instruction_context_menu_target: self.instruction_context_menu_target.clone(),
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
        if self
            .instruction_context_menu_target
            .as_ref()
            .is_some_and(|context_menu_target| !valid_instruction_keys.contains(context_menu_target.get_instruction_key()))
        {
            self.instruction_context_menu_target = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DebuggerTraceViewData;
    use eframe::egui::pos2;
    use squalr_engine_api::{
        events::debugger::trace_session_updated::debugger_trace_session_updated_event::DebuggerTraceSessionUpdatedEvent,
        structures::debugger::{
            DebuggerBreakpointDescriptor, DebuggerBreakpointKind, DebuggerDataBreakpointAccess, DebuggerRegisterSnapshot, DebuggerTraceEvent,
            DebuggerTraceInstructionRecord, DebuggerTraceSessionDescriptor,
        },
    };

    #[test]
    fn inactive_trace_update_removes_session_and_records_from_snapshot() {
        let debugger_trace_view_data = DebuggerTraceViewData::new();
        let active_trace_session = create_trace_session(true);
        let inactive_trace_session = create_trace_session(false);

        debugger_trace_view_data.apply_trace_session_updated(&DebuggerTraceSessionUpdatedEvent {
            trace_session: active_trace_session,
            instruction_records: Vec::new(),
        });
        assert_eq!(debugger_trace_view_data.get_snapshot().trace_sessions.len(), 1);

        debugger_trace_view_data.apply_trace_session_updated(&DebuggerTraceSessionUpdatedEvent {
            trace_session: inactive_trace_session,
            instruction_records: Vec::new(),
        });
        let snapshot = debugger_trace_view_data.get_snapshot();

        assert!(snapshot.trace_sessions.is_empty());
        assert!(snapshot.instruction_records.is_empty());
    }

    #[test]
    fn instruction_context_menu_target_is_exposed_in_snapshot() {
        let debugger_trace_view_data = DebuggerTraceViewData::new();
        let trace_event = DebuggerTraceEvent::new(
            None,
            DebuggerRegisterSnapshot::default(),
            Some(0x401000),
            vec![0x90],
            Some(String::from("nop")),
            None,
        );
        let instruction_record = DebuggerTraceInstructionRecord::new("trace-1", &trace_event);
        let instruction_key = super::DebuggerTraceInstructionKey::from_record(&instruction_record);

        debugger_trace_view_data.show_instruction_context_menu(instruction_key.clone(), pos2(12.0, 34.0));
        let snapshot = debugger_trace_view_data.get_snapshot();
        let context_menu_target = snapshot
            .instruction_context_menu_target
            .expect("Expected debugger trace instruction context menu target.");

        assert_eq!(context_menu_target.get_instruction_key(), &instruction_key);
        assert_eq!(context_menu_target.get_position(), pos2(12.0, 34.0));
    }

    #[test]
    fn instruction_key_ignores_bytes_when_address_is_known() {
        let first_trace_event = DebuggerTraceEvent::new(
            None,
            DebuggerRegisterSnapshot::default(),
            Some(0x401000),
            vec![0x90],
            Some(String::from("nop")),
            None,
        );
        let second_trace_event = DebuggerTraceEvent::new(
            None,
            DebuggerRegisterSnapshot::default(),
            Some(0x401000),
            vec![0xCC],
            Some(String::from("int3")),
            None,
        );
        let first_instruction_record = DebuggerTraceInstructionRecord::new("trace-1", &first_trace_event);
        let second_instruction_record = DebuggerTraceInstructionRecord::new("trace-1", &second_trace_event);

        assert_eq!(
            super::DebuggerTraceInstructionKey::from_record(&first_instruction_record),
            super::DebuggerTraceInstructionKey::from_record(&second_instruction_record)
        );
    }

    #[test]
    fn instruction_key_uses_bytes_when_address_is_unknown() {
        let first_trace_event = DebuggerTraceEvent::new(None, DebuggerRegisterSnapshot::default(), None, vec![0x90], Some(String::from("nop")), None);
        let second_trace_event = DebuggerTraceEvent::new(None, DebuggerRegisterSnapshot::default(), None, vec![0xCC], Some(String::from("int3")), None);
        let first_instruction_record = DebuggerTraceInstructionRecord::new("trace-1", &first_trace_event);
        let second_instruction_record = DebuggerTraceInstructionRecord::new("trace-1", &second_trace_event);

        assert_ne!(
            super::DebuggerTraceInstructionKey::from_record(&first_instruction_record),
            super::DebuggerTraceInstructionKey::from_record(&second_instruction_record)
        );
    }

    fn create_trace_session(is_active: bool) -> DebuggerTraceSessionDescriptor {
        let breakpoint = DebuggerBreakpointDescriptor::new(
            "bp-1",
            0x5000,
            DebuggerBreakpointKind::hardware_data(DebuggerDataBreakpointAccess::Write, 4),
            is_active,
            Some(String::from("health")),
        );

        DebuggerTraceSessionDescriptor::new(
            "trace-1",
            0x5000,
            4,
            DebuggerDataBreakpointAccess::Write,
            breakpoint,
            Some(String::from("health")),
            is_active,
        )
    }
}
