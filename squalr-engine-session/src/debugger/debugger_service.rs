use crate::plugins::plugin_registry::PluginRegistry;
use squalr_engine_api::events::debugger::session_state_changed::debugger_session_state_changed_event::DebuggerSessionStateChangedEvent;
use squalr_engine_api::events::debugger::trace_recorded::debugger_trace_recorded_event::DebuggerTraceRecordedEvent;
use squalr_engine_api::events::debugger::trace_session_updated::debugger_trace_session_updated_event::DebuggerTraceSessionUpdatedEvent;
use squalr_engine_api::events::engine_event::{EngineEvent, EngineEventRequest};
use squalr_engine_api::plugins::debugger::{DebuggerSession, DebuggerTraceEventSink};
use squalr_engine_api::structures::debugger::{
    DebuggerBreakpointDescriptor, DebuggerBreakpointKind, DebuggerDataBreakpointAccess, DebuggerRegisterSnapshot, DebuggerSessionState, DebuggerTraceEvent,
    DebuggerTraceInstructionRecord, DebuggerTraceSessionDescriptor, DebuggerTraceTargetKind,
};
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::processes::target_architecture::TargetArchitecture;
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicU64, Ordering},
    },
};

type SharedDebuggerSession = Arc<Mutex<Box<dyn DebuggerSession>>>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DebuggerOperationStatus {
    session_state: DebuggerSessionState,
    active_plugin_id: Option<String>,
}

impl DebuggerOperationStatus {
    fn new(
        session_state: DebuggerSessionState,
        active_plugin_id: Option<String>,
    ) -> Self {
        Self {
            session_state,
            active_plugin_id,
        }
    }

    pub fn get_session_state(&self) -> DebuggerSessionState {
        self.session_state
    }

    pub fn get_active_plugin_id(&self) -> Option<&str> {
        self.active_plugin_id.as_deref()
    }
}

struct CachedDebuggerSession {
    process_id: u32,
    process_handle: u64,
    process_name: String,
    plugin_id: String,
    session: SharedDebuggerSession,
}

impl CachedDebuggerSession {
    fn new(
        process_info: &OpenedProcessInfo,
        plugin_id: String,
        session: SharedDebuggerSession,
    ) -> Self {
        Self {
            process_id: process_info.get_process_id(),
            process_handle: process_info.get_handle(),
            process_name: process_info.get_name().to_string(),
            plugin_id,
            session,
        }
    }

    fn matches(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> bool {
        self.process_id == process_info.get_process_id() && self.process_handle == process_info.get_handle() && self.process_name == process_info.get_name()
    }
}

#[derive(Default)]
struct DebuggerTraceSessionStore {
    sessions: HashMap<String, DebuggerTraceSessionState>,
    breakpoint_to_trace_session_id: HashMap<String, String>,
}

impl DebuggerTraceSessionStore {
    fn insert_session(
        &mut self,
        descriptor: DebuggerTraceSessionDescriptor,
    ) {
        self.breakpoint_to_trace_session_id.insert(
            descriptor.get_breakpoint().get_breakpoint_id().to_string(),
            descriptor.get_trace_session_id().to_string(),
        );
        self.sessions.insert(
            descriptor.get_trace_session_id().to_string(),
            DebuggerTraceSessionState {
                descriptor,
                instruction_records: Vec::new(),
            },
        );
    }

    fn stop_session(
        &mut self,
        trace_session_id: &str,
    ) -> Result<(DebuggerTraceSessionDescriptor, Vec<DebuggerTraceInstructionRecord>), String> {
        let mut trace_session = self
            .sessions
            .remove(trace_session_id)
            .ok_or_else(|| format!("Debugger trace session '{}' does not exist.", trace_session_id))?;

        self.breakpoint_to_trace_session_id
            .remove(trace_session.descriptor.get_breakpoint().get_breakpoint_id());
        let mut breakpoint = trace_session.descriptor.get_breakpoint().clone();
        breakpoint.set_is_enabled(false);
        trace_session.descriptor.set_breakpoint(breakpoint);
        trace_session.descriptor.set_is_active(false);

        Ok((trace_session.descriptor, Vec::new()))
    }

    fn set_collection_enabled(
        &mut self,
        trace_session_id: &str,
        is_enabled: bool,
    ) -> Result<(DebuggerTraceSessionDescriptor, Vec<DebuggerTraceInstructionRecord>), String> {
        let trace_session = self
            .sessions
            .get_mut(trace_session_id)
            .ok_or_else(|| format!("Debugger trace session '{}' does not exist.", trace_session_id))?;

        if !trace_session.descriptor.get_is_active() {
            return Err(format!("Debugger trace session '{}' is stopped.", trace_session_id));
        }

        let mut breakpoint = trace_session.descriptor.get_breakpoint().clone();
        breakpoint.set_is_enabled(is_enabled);
        trace_session.descriptor.set_breakpoint(breakpoint);

        Ok((trace_session.descriptor.clone(), trace_session.instruction_records.clone()))
    }

    fn list_sessions(&self) -> (Vec<DebuggerTraceSessionDescriptor>, Vec<DebuggerTraceInstructionRecord>) {
        let mut trace_sessions = self
            .sessions
            .values()
            .map(|trace_session| trace_session.descriptor.clone())
            .collect::<Vec<_>>();
        let mut instruction_records = self
            .sessions
            .values()
            .flat_map(|trace_session| trace_session.instruction_records.clone())
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

        (trace_sessions, instruction_records)
    }

    fn record_trace_event(
        &mut self,
        trace_event: &DebuggerTraceEvent,
    ) -> Option<(DebuggerTraceSessionDescriptor, Vec<DebuggerTraceInstructionRecord>)> {
        let breakpoint_id = trace_event.get_breakpoint()?.get_breakpoint_id();
        let trace_session_id = self.breakpoint_to_trace_session_id.get(breakpoint_id)?.clone();
        let trace_session = self.sessions.get_mut(&trace_session_id)?;

        if !trace_session.descriptor.get_is_active() || !trace_session.descriptor.get_breakpoint().get_is_enabled() {
            return None;
        }

        // Instruction-directed sessions aggregate by the accessed memory address (what the instruction touched);
        // address-directed sessions aggregate by the accessing instruction. An instruction-directed hit with an
        // unresolved accessed address is dropped (there is nothing meaningful to show or key on).
        let existing_instruction_record = match trace_session.descriptor.get_target_kind() {
            DebuggerTraceTargetKind::Instruction => {
                trace_event.get_accessed_address()?;
                let accessed_address = trace_event.get_accessed_address();

                trace_session
                    .instruction_records
                    .iter_mut()
                    .find(|instruction_record| instruction_record.get_accessed_address() == accessed_address)
            }
            DebuggerTraceTargetKind::Address => {
                let instruction_address = trace_event.get_instruction_address();
                let instruction_bytes = trace_event.get_instruction_bytes();

                trace_session
                    .instruction_records
                    .iter_mut()
                    .find(|instruction_record| {
                        instruction_record.get_instruction_address() == instruction_address && instruction_record.get_instruction_bytes() == instruction_bytes
                    })
            }
        };

        if let Some(instruction_record) = existing_instruction_record {
            instruction_record.record_hit(trace_event);
        } else {
            trace_session
                .instruction_records
                .push(DebuggerTraceInstructionRecord::new(&trace_session_id, trace_event));
        }

        Some((trace_session.descriptor.clone(), trace_session.instruction_records.clone()))
    }

    fn clear(&mut self) {
        self.sessions.clear();
        self.breakpoint_to_trace_session_id.clear();
    }
}

struct DebuggerTraceSessionState {
    descriptor: DebuggerTraceSessionDescriptor,
    instruction_records: Vec<DebuggerTraceInstructionRecord>,
}

pub struct DebuggerService {
    plugin_registry: Arc<PluginRegistry>,
    active_session: RwLock<Option<CachedDebuggerSession>>,
    trace_sessions: Arc<RwLock<DebuggerTraceSessionStore>>,
    next_trace_session_number: AtomicU64,
    event_emitter: Arc<dyn Fn(EngineEvent) + Send + Sync>,
}

impl DebuggerService {
    pub fn new(
        plugin_registry: Arc<PluginRegistry>,
        event_emitter: Arc<dyn Fn(EngineEvent) + Send + Sync>,
    ) -> Self {
        Self {
            plugin_registry,
            active_session: RwLock::new(None),
            trace_sessions: Arc::new(RwLock::new(DebuggerTraceSessionStore::default())),
            next_trace_session_number: AtomicU64::new(1),
            event_emitter,
        }
    }

    pub fn attach(
        &self,
        process_info: &OpenedProcessInfo,
        requested_plugin_id: Option<&str>,
    ) -> Result<DebuggerOperationStatus, String> {
        let debugger_session = self.get_or_create_session(process_info, requested_plugin_id)?;
        let (session_state, active_plugin_id) = self.with_debugger_session(&debugger_session, |debugger_session| {
            let active_plugin_id = debugger_session.plugin_id().to_string();
            let current_session_state = debugger_session.get_state();

            if current_session_state != DebuggerSessionState::Detached {
                return Ok((current_session_state, active_plugin_id));
            }

            debugger_session
                .attach()
                .map(|session_state| (session_state, active_plugin_id))
                .map_err(|error| error.to_string())
        })?;

        self.emit_session_state_changed(session_state, Some(active_plugin_id.clone()));

        Ok(DebuggerOperationStatus::new(session_state, Some(active_plugin_id)))
    }

    pub fn detach(&self) -> Result<DebuggerOperationStatus, String> {
        let cached_session = self.take_cached_session()?;
        let session_state = self.with_debugger_session(&cached_session.session, |debugger_session| {
            debugger_session.detach().map_err(|error| error.to_string())
        })?;

        self.clear_trace_sessions();
        self.emit_session_state_changed(session_state, None);

        Ok(DebuggerOperationStatus::new(session_state, None))
    }

    pub fn pause(&self) -> Result<DebuggerOperationStatus, String> {
        self.with_active_session_state_mutation(|debugger_session| debugger_session.pause().map_err(|error| error.to_string()))
    }

    pub fn resume(&self) -> Result<DebuggerOperationStatus, String> {
        self.with_active_session_state_mutation(|debugger_session| debugger_session.resume().map_err(|error| error.to_string()))
    }

    pub fn set_breakpoint(
        &self,
        address: u64,
        kind: DebuggerBreakpointKind,
        label: Option<String>,
    ) -> Result<DebuggerBreakpointDescriptor, String> {
        let active_session = self.get_cached_session()?;

        self.with_debugger_session(&active_session.session, |debugger_session| {
            debugger_session
                .set_breakpoint(address, kind, label)
                .map_err(|error| error.to_string())
        })
    }

    pub fn remove_breakpoint(
        &self,
        breakpoint_id: &str,
    ) -> Result<(), String> {
        let active_session = self.get_cached_session()?;

        self.with_debugger_session(&active_session.session, |debugger_session| {
            debugger_session
                .remove_breakpoint(breakpoint_id)
                .map_err(|error| error.to_string())
        })
    }

    pub fn set_breakpoint_enabled(
        &self,
        breakpoint_id: &str,
        is_enabled: bool,
    ) -> Result<(), String> {
        let active_session = self.get_cached_session()?;

        self.with_debugger_session(&active_session.session, |debugger_session| {
            debugger_session
                .set_breakpoint_enabled(breakpoint_id, is_enabled)
                .map_err(|error| error.to_string())
        })
    }

    pub fn list_breakpoints(&self) -> Result<Vec<DebuggerBreakpointDescriptor>, String> {
        let active_session = self.get_cached_session()?;

        self.with_debugger_session(&active_session.session, |debugger_session| {
            debugger_session
                .list_breakpoints()
                .map_err(|error| error.to_string())
        })
    }

    pub fn get_active_session_state_for_process(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> Option<DebuggerSessionState> {
        let cached_session = self.active_session.read().ok().and_then(|active_session| {
            active_session.as_ref().and_then(|cached_session| {
                if cached_session.matches(process_info) {
                    Some(CachedDebuggerSessionSnapshot {
                        plugin_id: cached_session.plugin_id.clone(),
                        session: cached_session.session.clone(),
                    })
                } else {
                    None
                }
            })
        })?;

        self.with_debugger_session(&cached_session.session, |debugger_session| Ok(debugger_session.get_state()))
            .ok()
    }

    pub fn read_registers(&self) -> Result<DebuggerRegisterSnapshot, String> {
        let active_session = self.get_cached_session()?;

        self.with_debugger_session(&active_session.session, |debugger_session| {
            debugger_session
                .read_registers()
                .map_err(|error| error.to_string())
        })
    }

    pub fn write_register(
        &self,
        register_name: &str,
        value: u64,
    ) -> Result<DebuggerRegisterSnapshot, String> {
        let active_session = self.get_cached_session()?;

        self.with_debugger_session(&active_session.session, |debugger_session| {
            debugger_session
                .write_register(register_name, value)
                .map_err(|error| error.to_string())
        })
    }

    pub fn start_trace_session(
        &self,
        address: u64,
        size_in_bytes: u8,
        access: DebuggerDataBreakpointAccess,
        label: Option<String>,
    ) -> Result<(DebuggerTraceSessionDescriptor, Vec<DebuggerTraceInstructionRecord>), String> {
        let active_session = self.get_cached_session()?;
        let trace_session_number = self.next_trace_session_number.fetch_add(1, Ordering::Relaxed);
        let trace_session_id = format!("trace-{}", trace_session_number);
        let breakpoint_label = label
            .clone()
            .unwrap_or_else(|| format!("{} data trace at 0x{:X}", access.get_cli_label(), address));
        let breakpoint_result = self.with_debugger_session(&active_session.session, |debugger_session| {
            debugger_session
                .set_breakpoint(address, DebuggerBreakpointKind::hardware_data(access, size_in_bytes), Some(breakpoint_label))
                .map_err(|error| error.to_string())
        });
        let breakpoint = match breakpoint_result {
            Ok(breakpoint) => breakpoint,
            Err(error) => {
                let _ = self.with_debugger_session(&active_session.session, |debugger_session| {
                    debugger_session
                        .resume()
                        .map(|_| ())
                        .map_err(|resume_error| resume_error.to_string())
                });

                return Err(error);
            }
        };
        let trace_session = DebuggerTraceSessionDescriptor::new(trace_session_id, address, size_in_bytes, access, breakpoint, label, true);

        self.trace_sessions
            .write()
            .map_err(|error| format!("Failed to cache debugger trace session: {}", error))?
            .insert_session(trace_session.clone());

        self.emit_trace_session_updated(trace_session.clone(), Vec::new());

        let resume_status = self.with_debugger_session(&active_session.session, |debugger_session| {
            debugger_session.resume().map_err(|error| error.to_string())
        })?;
        self.emit_session_state_changed(resume_status, Some(active_session.plugin_id.clone()));

        Ok((trace_session, Vec::new()))
    }

    /// Instruction-directed trace: sets a hardware execute breakpoint at `instruction_address` and records which memory
    /// addresses the instruction accesses ("find what addresses this instruction accesses").
    pub fn start_instruction_trace_session(
        &self,
        instruction_address: u64,
        access: DebuggerDataBreakpointAccess,
        label: Option<String>,
    ) -> Result<(DebuggerTraceSessionDescriptor, Vec<DebuggerTraceInstructionRecord>), String> {
        let active_session = self.get_cached_session()?;
        let trace_session_number = self.next_trace_session_number.fetch_add(1, Ordering::Relaxed);
        let trace_session_id = format!("trace-{}", trace_session_number);
        let breakpoint_label = label
            .clone()
            .unwrap_or_else(|| format!("{} instruction trace at 0x{:X}", access.get_cli_label(), instruction_address));
        let breakpoint_result = self.with_debugger_session(&active_session.session, |debugger_session| {
            debugger_session
                .set_breakpoint(instruction_address, DebuggerBreakpointKind::hardware_execute(), Some(breakpoint_label))
                .map_err(|error| error.to_string())
        });
        let breakpoint = match breakpoint_result {
            Ok(breakpoint) => breakpoint,
            Err(error) => {
                let _ = self.with_debugger_session(&active_session.session, |debugger_session| {
                    debugger_session
                        .resume()
                        .map(|_| ())
                        .map_err(|resume_error| resume_error.to_string())
                });

                return Err(error);
            }
        };
        let trace_session = DebuggerTraceSessionDescriptor::new_for_instruction(trace_session_id, instruction_address, access, breakpoint, label, true);

        self.trace_sessions
            .write()
            .map_err(|error| format!("Failed to cache debugger trace session: {}", error))?
            .insert_session(trace_session.clone());

        self.emit_trace_session_updated(trace_session.clone(), Vec::new());

        let resume_status = self.with_debugger_session(&active_session.session, |debugger_session| {
            debugger_session.resume().map_err(|error| error.to_string())
        })?;
        self.emit_session_state_changed(resume_status, Some(active_session.plugin_id.clone()));

        Ok((trace_session, Vec::new()))
    }

    pub fn stop_trace_session(
        &self,
        trace_session_id: &str,
    ) -> Result<(DebuggerTraceSessionDescriptor, Vec<DebuggerTraceInstructionRecord>), String> {
        let breakpoint_id = self
            .trace_sessions
            .read()
            .map_err(|error| format!("Failed to read debugger trace sessions: {}", error))?
            .sessions
            .get(trace_session_id)
            .map(|trace_session| {
                trace_session
                    .descriptor
                    .get_breakpoint()
                    .get_breakpoint_id()
                    .to_string()
            })
            .ok_or_else(|| format!("Debugger trace session '{}' does not exist.", trace_session_id))?;

        if let Err(error) = self.remove_breakpoint(&breakpoint_id) {
            log::warn!(
                "Failed to clear backing breakpoint {} while stopping debugger trace session '{}': {}.",
                breakpoint_id,
                trace_session_id,
                error
            );
        }

        let (trace_session, instruction_records) = self
            .trace_sessions
            .write()
            .map_err(|error| format!("Failed to update debugger trace sessions: {}", error))?
            .stop_session(trace_session_id)?;

        self.emit_trace_session_updated(trace_session.clone(), instruction_records.clone());

        Ok((trace_session, instruction_records))
    }

    pub fn pause_trace_session(
        &self,
        trace_session_id: &str,
    ) -> Result<(DebuggerTraceSessionDescriptor, Vec<DebuggerTraceInstructionRecord>), String> {
        self.set_trace_session_collection_enabled(trace_session_id, false)
    }

    pub fn resume_trace_session(
        &self,
        trace_session_id: &str,
    ) -> Result<(DebuggerTraceSessionDescriptor, Vec<DebuggerTraceInstructionRecord>), String> {
        self.set_trace_session_collection_enabled(trace_session_id, true)
    }

    pub fn list_trace_sessions(&self) -> Result<(Vec<DebuggerTraceSessionDescriptor>, Vec<DebuggerTraceInstructionRecord>), String> {
        self.trace_sessions
            .read()
            .map(|trace_sessions| trace_sessions.list_sessions())
            .map_err(|error| format!("Failed to read debugger trace sessions: {}", error))
    }

    pub fn clear_active_session(&self) {
        let cached_session = match self.active_session.write() {
            Ok(mut active_session) => active_session.take(),
            Err(error) => {
                log::error!("Failed to clear active debugger session: {}", error);
                None
            }
        };

        if let Some(cached_session) = cached_session {
            if let Err(error) = self.with_debugger_session(&cached_session.session, |debugger_session| {
                debugger_session.detach().map_err(|error| error.to_string())
            }) {
                log::debug!("Failed to detach debugger session while clearing it: {}", error);
            }

            self.clear_trace_sessions();
            self.emit_session_state_changed(DebuggerSessionState::Detached, None);
        }
    }

    pub fn get_active_plugin_id_for_process(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> Option<String> {
        self.active_session.read().ok().and_then(|active_session| {
            active_session.as_ref().and_then(|cached_session| {
                if cached_session.matches(process_info) {
                    Some(cached_session.plugin_id.clone())
                } else {
                    None
                }
            })
        })
    }

    fn with_active_session_state_mutation<F>(
        &self,
        mutate_session: F,
    ) -> Result<DebuggerOperationStatus, String>
    where
        F: FnOnce(&mut dyn DebuggerSession) -> Result<DebuggerSessionState, String>,
    {
        let active_session = self.get_cached_session()?;
        let (session_state, active_plugin_id) = self.with_debugger_session(&active_session.session, |debugger_session| {
            mutate_session(debugger_session).map(|session_state| (session_state, debugger_session.plugin_id().to_string()))
        })?;

        self.emit_session_state_changed(session_state, Some(active_plugin_id.clone()));

        Ok(DebuggerOperationStatus::new(session_state, Some(active_plugin_id)))
    }

    fn set_trace_session_collection_enabled(
        &self,
        trace_session_id: &str,
        is_enabled: bool,
    ) -> Result<(DebuggerTraceSessionDescriptor, Vec<DebuggerTraceInstructionRecord>), String> {
        let (trace_session, instruction_records) = self
            .trace_sessions
            .write()
            .map_err(|error| format!("Failed to update debugger trace sessions: {}", error))?
            .set_collection_enabled(trace_session_id, is_enabled)?;

        self.emit_trace_session_updated(trace_session.clone(), instruction_records.clone());

        Ok((trace_session, instruction_records))
    }

    fn get_or_create_session(
        &self,
        process_info: &OpenedProcessInfo,
        requested_plugin_id: Option<&str>,
    ) -> Result<SharedDebuggerSession, String> {
        if let Some(active_session) = self.get_matching_cached_session(process_info, requested_plugin_id) {
            return Ok(active_session);
        }

        let plugin_package = self
            .plugin_registry
            .find_debugger_plugin_package(process_info, requested_plugin_id)
            .ok_or_else(|| String::from("No enabled debugger plugin can attach to the opened process."))?;
        let debugger_plugin = plugin_package
            .as_debugger_plugin()
            .ok_or_else(|| format!("Plugin '{}' is not a debugger plugin.", plugin_package.metadata().get_plugin_id()))?;
        let plugin_id = plugin_package.metadata().get_plugin_id().to_string();
        let trace_event_sink = self.create_trace_event_sink(process_info, &plugin_id);
        let debugger_session = debugger_plugin
            .create_session(process_info, trace_event_sink)
            .map_err(|error| error.to_string())?;
        let shared_debugger_session = Arc::new(Mutex::new(debugger_session));

        match self.active_session.write() {
            Ok(mut active_session) => {
                *active_session = Some(CachedDebuggerSession::new(process_info, plugin_id, shared_debugger_session.clone()));
            }
            Err(error) => {
                return Err(format!("Failed to cache debugger session: {}", error));
            }
        }

        Ok(shared_debugger_session)
    }

    fn get_matching_cached_session(
        &self,
        process_info: &OpenedProcessInfo,
        requested_plugin_id: Option<&str>,
    ) -> Option<SharedDebuggerSession> {
        self.active_session.read().ok().and_then(|active_session| {
            active_session.as_ref().and_then(|cached_session| {
                let requested_plugin_matches = requested_plugin_id
                    .map(|requested_plugin_id| requested_plugin_id == cached_session.plugin_id)
                    .unwrap_or(true);

                if cached_session.matches(process_info) && requested_plugin_matches {
                    Some(cached_session.session.clone())
                } else {
                    None
                }
            })
        })
    }

    fn get_cached_session(&self) -> Result<CachedDebuggerSessionSnapshot, String> {
        self.active_session
            .read()
            .map_err(|error| format!("Failed to read active debugger session: {}", error))?
            .as_ref()
            .map(|cached_session| CachedDebuggerSessionSnapshot {
                plugin_id: cached_session.plugin_id.clone(),
                session: cached_session.session.clone(),
            })
            .ok_or_else(|| String::from("No active debugger session."))
    }

    fn take_cached_session(&self) -> Result<CachedDebuggerSessionSnapshot, String> {
        self.active_session
            .write()
            .map_err(|error| format!("Failed to clear active debugger session: {}", error))?
            .take()
            .map(|cached_session| CachedDebuggerSessionSnapshot {
                plugin_id: cached_session.plugin_id,
                session: cached_session.session,
            })
            .ok_or_else(|| String::from("No active debugger session."))
    }

    fn with_debugger_session<T, F>(
        &self,
        debugger_session: &SharedDebuggerSession,
        operation: F,
    ) -> Result<T, String>
    where
        F: FnOnce(&mut dyn DebuggerSession) -> Result<T, String>,
    {
        let mut debugger_session = debugger_session
            .lock()
            .map_err(|error| format!("Failed to lock debugger session: {}", error))?;

        operation(debugger_session.as_mut())
    }

    fn emit_session_state_changed(
        &self,
        session_state: DebuggerSessionState,
        active_plugin_id: Option<String>,
    ) {
        (self.event_emitter)(
            DebuggerSessionStateChangedEvent {
                session_state,
                active_plugin_id,
            }
            .to_engine_event(),
        );
    }

    fn create_trace_event_sink(
        &self,
        process_info: &OpenedProcessInfo,
        plugin_id: &str,
    ) -> DebuggerTraceEventSink {
        let event_emitter = self.event_emitter.clone();
        let plugin_registry = self.plugin_registry.clone();
        let trace_sessions = self.trace_sessions.clone();
        let target_architecture = process_info.get_target_architecture().clone();
        let active_plugin_id = plugin_id.to_string();

        Arc::new(move |trace_event: DebuggerTraceEvent| {
            let trace_event = Self::enrich_trace_event_with_disassembly(&plugin_registry, &target_architecture, trace_event);
            let trace_session_update = trace_sessions
                .write()
                .ok()
                .and_then(|mut trace_sessions| trace_sessions.record_trace_event(&trace_event));
            let session_state = if trace_session_update.is_some()
                || matches!(
                    trace_event
                        .get_breakpoint()
                        .map(DebuggerBreakpointDescriptor::get_kind),
                    Some(DebuggerBreakpointKind::HardwareData { .. })
                ) {
                DebuggerSessionState::Running
            } else {
                DebuggerSessionState::Paused
            };

            event_emitter(
                DebuggerSessionStateChangedEvent {
                    session_state,
                    active_plugin_id: Some(active_plugin_id.clone()),
                }
                .to_engine_event(),
            );
            event_emitter(DebuggerTraceRecordedEvent { trace_event }.to_engine_event());

            if let Some((trace_session, instruction_records)) = trace_session_update {
                event_emitter(
                    DebuggerTraceSessionUpdatedEvent {
                        trace_session,
                        instruction_records,
                    }
                    .to_engine_event(),
                );
            }
        })
    }

    fn enrich_trace_event_with_disassembly(
        plugin_registry: &PluginRegistry,
        target_architecture: &TargetArchitecture,
        trace_event: DebuggerTraceEvent,
    ) -> DebuggerTraceEvent {
        let (instruction_target_architecture, normalized_instruction_address) = trace_event
            .get_instruction_address()
            .map(|instruction_address| target_architecture.normalize_instruction_address(instruction_address))
            .map(|(instruction_target_architecture, normalized_instruction_address)| (instruction_target_architecture, Some(normalized_instruction_address)))
            .unwrap_or_else(|| (target_architecture.clone(), None));
        let trace_event = trace_event.with_instruction_address(normalized_instruction_address);
        let is_instruction_directed = matches!(
            trace_event
                .get_breakpoint()
                .map(DebuggerBreakpointDescriptor::get_kind),
            Some(DebuggerBreakpointKind::HardwareExecute)
        );

        // Already disassembled and not instruction-directed (no accessed address to compute): nothing to do.
        if trace_event.get_instruction_bytes().is_empty() || (trace_event.get_instruction_text().is_some() && !is_instruction_directed) {
            return trace_event.with_target_architecture(instruction_target_architecture);
        }

        let disassembled_instruction = plugin_registry
            .find_instruction_set(instruction_target_architecture.get_instruction_set_id())
            .and_then(|instruction_set| {
                let disassembled_instruction = instruction_set
                    .disassemble_block(trace_event.get_instruction_bytes(), trace_event.get_instruction_address().unwrap_or(0))
                    .ok()
                    .and_then(|instructions| instructions.into_iter().next())?;
                // For instruction-directed traces, resolve which memory address this instruction touched on this hit.
                let accessed_address = if is_instruction_directed {
                    let register_lookup = Self::build_register_lookup(trace_event.get_register_snapshot());

                    instruction_set.resolve_accessed_address(&disassembled_instruction, &register_lookup)
                } else {
                    None
                };

                Some((disassembled_instruction, accessed_address))
            });

        let Some((disassembled_instruction, accessed_address)) = disassembled_instruction else {
            return trace_event.with_target_architecture(instruction_target_architecture);
        };

        let instruction_text = trace_event
            .get_instruction_text()
            .map(String::from)
            .or_else(|| Some(disassembled_instruction.text.clone()))
            .filter(|instruction_text| Self::is_meaningful_instruction_text(instruction_text));

        DebuggerTraceEvent::new(
            trace_event.get_breakpoint().cloned(),
            trace_event.get_register_snapshot().clone(),
            trace_event.get_instruction_address(),
            trace_event.get_instruction_bytes().to_vec(),
            instruction_text,
            trace_event.get_backend_message().map(String::from),
        )
        .with_target_architecture(instruction_target_architecture)
        .with_accessed_address(accessed_address)
    }

    /// Builds a register-value lookup over a snapshot for effective-address computation. Names are matched
    /// case-insensitively; arm64 `wN` falls back to `xN`, and x86 `eN`/`r*d` fall back to their 64-bit form.
    fn build_register_lookup(register_snapshot: &DebuggerRegisterSnapshot) -> impl Fn(&str) -> Option<u64> {
        let register_values = register_snapshot
            .get_registers()
            .iter()
            .map(|register_value| (register_value.get_name().to_ascii_lowercase(), register_value.get_value()))
            .collect::<HashMap<String, u64>>();

        move |register_name: &str| {
            let register_name = register_name.to_ascii_lowercase();

            if let Some(register_value) = register_values.get(&register_name) {
                return Some(*register_value);
            }

            // arm64: wN is the low 32 bits of xN.
            if let Some(register_index) = register_name.strip_prefix('w') {
                if register_index
                    .chars()
                    .all(|character| character.is_ascii_digit())
                    && !register_index.is_empty()
                {
                    return register_values
                        .get(&format!("x{}", register_index))
                        .map(|value| value & 0xFFFF_FFFF);
                }
            }

            // x86: eN is the low 32 bits of rN; r8d/r8w/r8b are sub-registers of r8.
            if let Some(register_suffix) = register_name.strip_prefix('e') {
                if let Some(register_value) = register_values.get(&format!("r{}", register_suffix)) {
                    return Some(register_value & 0xFFFF_FFFF);
                }
            }

            None
        }
    }

    fn is_meaningful_instruction_text(instruction_text: &str) -> bool {
        let normalized_instruction_text = instruction_text
            .trim()
            .trim_start_matches('[')
            .trim_end_matches(']')
            .trim();

        !normalized_instruction_text.is_empty() && normalized_instruction_text != "??"
    }

    fn clear_trace_sessions(&self) {
        match self.trace_sessions.write() {
            Ok(mut trace_sessions) => trace_sessions.clear(),
            Err(error) => {
                log::error!("Failed to clear debugger trace sessions: {}", error);
            }
        }
    }

    fn emit_trace_session_updated(
        &self,
        trace_session: DebuggerTraceSessionDescriptor,
        instruction_records: Vec<DebuggerTraceInstructionRecord>,
    ) {
        (self.event_emitter)(
            DebuggerTraceSessionUpdatedEvent {
                trace_session,
                instruction_records,
            }
            .to_engine_event(),
        );
    }
}

struct CachedDebuggerSessionSnapshot {
    plugin_id: String,
    session: SharedDebuggerSession,
}

#[cfg(test)]
mod tests {
    use super::DebuggerService;
    use crate::plugins::plugin_registry::PluginRegistry;
    use squalr_engine_api::{
        events::{debugger::debugger_event::DebuggerEvent, engine_event::EngineEvent},
        plugins::{
            Plugin, PluginCapability, PluginMetadata, PluginPackage, PluginPermission,
            debugger::{DebuggerPlugin, DebuggerPluginError, DebuggerSession, DebuggerTraceEventSink},
            instruction_set::{DisassembledInstruction, InstructionSet, InstructionSetPlugin},
        },
        structures::{
            debugger::{
                DebuggerBreakpointDescriptor, DebuggerBreakpointKind, DebuggerDataBreakpointAccess, DebuggerRegisterSnapshot, DebuggerRegisterValue,
                DebuggerSessionState, DebuggerTraceEvent,
            },
            memory::bitness::Bitness,
            processes::opened_process_info::OpenedProcessInfo,
            processes::target_architecture::{Endianness, TargetArchitecture},
        },
    };
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct TestDebuggerBackend {
        create_session_count: u32,
        state: DebuggerSessionState,
        breakpoints: Vec<DebuggerBreakpointDescriptor>,
        registers: Vec<DebuggerRegisterValue>,
        trace_event_sink: Option<DebuggerTraceEventSink>,
        resume_count: u32,
        fail_remove_breakpoint: bool,
    }

    struct TestDebuggerPlugin {
        metadata: PluginMetadata,
        process_name: String,
        backend: Arc<Mutex<TestDebuggerBackend>>,
    }

    #[derive(Debug)]
    struct TestInstructionSet {
        instruction_set_id: &'static str,
        display_name: &'static str,
        disassembly_prefix: &'static str,
        include_base_address: bool,
    }

    impl TestInstructionSet {
        fn new(
            instruction_set_id: &'static str,
            display_name: &'static str,
            disassembly_prefix: &'static str,
        ) -> Self {
            Self {
                instruction_set_id,
                display_name,
                disassembly_prefix,
                include_base_address: false,
            }
        }

        fn new_with_base_address(
            instruction_set_id: &'static str,
            display_name: &'static str,
            disassembly_prefix: &'static str,
        ) -> Self {
            Self {
                instruction_set_id,
                display_name,
                disassembly_prefix,
                include_base_address: true,
            }
        }
    }

    impl InstructionSet for TestInstructionSet {
        fn get_instruction_set_id(&self) -> &str {
            self.instruction_set_id
        }

        fn get_display_name(&self) -> &str {
            self.display_name
        }

        fn assemble(
            &self,
            _assembly_source: &str,
        ) -> Result<Vec<u8>, String> {
            Ok(Vec::new())
        }

        fn disassemble(
            &self,
            instruction_bytes: &[u8],
        ) -> Result<String, String> {
            Ok(format!("{}-{}", self.disassembly_prefix, instruction_bytes.len()))
        }

        fn disassemble_block(
            &self,
            instruction_bytes: &[u8],
            base_address: u64,
        ) -> Result<Vec<DisassembledInstruction>, String> {
            let mut instructions = Vec::new();

            if instruction_bytes.first() == Some(&0xCC) {
                instructions.push(DisassembledInstruction {
                    address: base_address,
                    length: 1,
                    bytes: vec![0xCC],
                    text: String::from("??"),
                    branch_target_address: None,
                    is_control_flow: false,
                });
            }

            if !instruction_bytes.is_empty() {
                let instruction_text = if self.include_base_address {
                    format!("{}-block-{}-0x{:X}", self.disassembly_prefix, instruction_bytes.len(), base_address)
                } else {
                    format!("{}-block-{}", self.disassembly_prefix, instruction_bytes.len())
                };

                instructions.push(DisassembledInstruction {
                    address: base_address + instructions.len() as u64,
                    length: instruction_bytes.len(),
                    bytes: instruction_bytes.to_vec(),
                    text: instruction_text,
                    branch_target_address: None,
                    is_control_flow: false,
                });
            }

            Ok(instructions)
        }
    }

    struct TestInstructionSetPlugin {
        metadata: PluginMetadata,
        instruction_sets: Vec<Arc<dyn InstructionSet>>,
    }

    impl TestInstructionSetPlugin {
        fn new() -> Self {
            Self {
                metadata: PluginMetadata::new(
                    "test.instruction-set",
                    "Test Instruction Set",
                    "Test instruction set plugin",
                    vec![PluginCapability::InstructionSet],
                    true,
                    true,
                ),
                instruction_sets: vec![
                    Arc::new(TestInstructionSet::new("x64", "Test x64", "test-disassembly")),
                    Arc::new(TestInstructionSet::new("arm64", "Test ARM64", "arm64-disassembly")),
                    Arc::new(TestInstructionSet::new("thumb", "Test Thumb", "thumb-disassembly")),
                    Arc::new(TestInstructionSet::new_with_base_address("base-test", "Test Base", "base-disassembly")),
                ],
            }
        }
    }

    impl Plugin for TestInstructionSetPlugin {
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }
    }

    impl PluginPackage for TestInstructionSetPlugin {
        fn as_instruction_set_plugin(&self) -> Option<&dyn InstructionSetPlugin> {
            Some(self)
        }
    }

    impl InstructionSetPlugin for TestInstructionSetPlugin {
        fn contributed_instruction_sets(&self) -> &[Arc<dyn InstructionSet>] {
            &self.instruction_sets
        }

        fn contributed_instruction_set_ids(&self) -> &'static [&'static str] {
            &["x64", "arm64"]
        }
    }

    impl TestDebuggerPlugin {
        fn new(
            plugin_id: &str,
            process_name: &str,
            backend: Arc<Mutex<TestDebuggerBackend>>,
        ) -> Self {
            Self {
                metadata: PluginMetadata::new_with_permissions(
                    plugin_id,
                    "Test Debugger",
                    "Test debugger plugin",
                    vec![PluginCapability::Debugger],
                    vec![
                        PluginPermission::AttachDebugger,
                        PluginPermission::ControlDebuggerExecution,
                        PluginPermission::ManageDebuggerBreakpoints,
                        PluginPermission::ReadRegisters,
                        PluginPermission::WriteRegisters,
                    ],
                    true,
                    true,
                ),
                process_name: process_name.to_string(),
                backend,
            }
        }
    }

    impl Plugin for TestDebuggerPlugin {
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }
    }

    impl PluginPackage for TestDebuggerPlugin {
        fn as_debugger_plugin(&self) -> Option<&dyn DebuggerPlugin> {
            Some(self)
        }
    }

    impl DebuggerPlugin for TestDebuggerPlugin {
        fn can_attach(
            &self,
            process_info: &OpenedProcessInfo,
        ) -> bool {
            process_info.get_name() == self.process_name
        }

        fn create_session(
            &self,
            _process_info: &OpenedProcessInfo,
            trace_event_sink: DebuggerTraceEventSink,
        ) -> Result<Box<dyn DebuggerSession>, DebuggerPluginError> {
            self.backend
                .lock()
                .map(|mut backend| {
                    backend.create_session_count += 1;
                    backend.trace_event_sink = Some(trace_event_sink);
                })
                .map_err(|error| DebuggerPluginError::new(self.metadata.get_plugin_id(), error.to_string()))?;

            Ok(Box::new(TestDebuggerSession {
                plugin_id: self.metadata.get_plugin_id().to_string(),
                backend: self.backend.clone(),
            }))
        }
    }

    struct TestDebuggerSession {
        plugin_id: String,
        backend: Arc<Mutex<TestDebuggerBackend>>,
    }

    impl TestDebuggerSession {
        fn mutate_backend<T>(
            &self,
            operation: impl FnOnce(&mut TestDebuggerBackend) -> T,
        ) -> Result<T, DebuggerPluginError> {
            self.backend
                .lock()
                .map(|mut backend| operation(&mut backend))
                .map_err(|error| DebuggerPluginError::new(&self.plugin_id, error.to_string()))
        }

        fn read_backend<T>(
            &self,
            operation: impl FnOnce(&TestDebuggerBackend) -> T,
        ) -> Result<T, DebuggerPluginError> {
            self.backend
                .lock()
                .map(|backend| operation(&backend))
                .map_err(|error| DebuggerPluginError::new(&self.plugin_id, error.to_string()))
        }

        fn create_register_snapshot(registers: &[DebuggerRegisterValue]) -> DebuggerRegisterSnapshot {
            DebuggerRegisterSnapshot::new(Some(0x401000), Some(0x700000), registers.to_vec())
        }

        fn emit_trace_event(&self) -> Result<(), DebuggerPluginError> {
            self.read_backend(|backend| {
                if let Some(trace_event_sink) = backend.trace_event_sink.as_ref() {
                    let breakpoint = backend.breakpoints.first().cloned();
                    let register_snapshot = Self::create_register_snapshot(&backend.registers);

                    trace_event_sink(DebuggerTraceEvent::new(
                        breakpoint,
                        register_snapshot,
                        Some(0x401000),
                        vec![0x90],
                        None,
                        Some(String::from("test trace")),
                    ));
                }
            })
        }
    }

    impl DebuggerSession for TestDebuggerSession {
        fn plugin_id(&self) -> &str {
            &self.plugin_id
        }

        fn get_state(&self) -> DebuggerSessionState {
            self.backend
                .lock()
                .map(|backend| backend.state)
                .unwrap_or(DebuggerSessionState::Detached)
        }

        fn attach(&mut self) -> Result<DebuggerSessionState, DebuggerPluginError> {
            self.mutate_backend(|backend| {
                backend.state = DebuggerSessionState::Attached;
                backend.state
            })
        }

        fn detach(&mut self) -> Result<DebuggerSessionState, DebuggerPluginError> {
            self.mutate_backend(|backend| {
                backend.state = DebuggerSessionState::Detached;
                backend.state
            })
        }

        fn pause(&mut self) -> Result<DebuggerSessionState, DebuggerPluginError> {
            self.mutate_backend(|backend| {
                backend.state = DebuggerSessionState::Paused;
                backend.state
            })
        }

        fn resume(&mut self) -> Result<DebuggerSessionState, DebuggerPluginError> {
            let session_state = self.mutate_backend(|backend| {
                backend.state = DebuggerSessionState::Running;
                backend.resume_count = backend.resume_count.saturating_add(1);
                backend.state
            })?;

            self.emit_trace_event()?;

            Ok(session_state)
        }

        fn set_breakpoint(
            &mut self,
            address: u64,
            kind: DebuggerBreakpointKind,
            label: Option<String>,
        ) -> Result<DebuggerBreakpointDescriptor, DebuggerPluginError> {
            self.mutate_backend(|backend| {
                let breakpoint = DebuggerBreakpointDescriptor::new(format!("bp-{}", backend.breakpoints.len() + 1), address, kind, true, label);

                backend.breakpoints.push(breakpoint.clone());

                breakpoint
            })
        }

        fn remove_breakpoint(
            &mut self,
            breakpoint_id: &str,
        ) -> Result<(), DebuggerPluginError> {
            self.mutate_backend(|backend| {
                if backend.fail_remove_breakpoint {
                    return Err(DebuggerPluginError::new(&self.plugin_id, "test remove failure"));
                }

                backend
                    .breakpoints
                    .retain(|breakpoint| breakpoint.get_breakpoint_id() != breakpoint_id);

                Ok(())
            })
            .and_then(|result| result)
        }

        fn set_breakpoint_enabled(
            &mut self,
            breakpoint_id: &str,
            is_enabled: bool,
        ) -> Result<(), DebuggerPluginError> {
            self.mutate_backend(|backend| {
                if let Some(breakpoint) = backend
                    .breakpoints
                    .iter_mut()
                    .find(|breakpoint| breakpoint.get_breakpoint_id() == breakpoint_id)
                {
                    breakpoint.set_is_enabled(is_enabled);
                }
            })
        }

        fn list_breakpoints(&self) -> Result<Vec<DebuggerBreakpointDescriptor>, DebuggerPluginError> {
            self.read_backend(|backend| backend.breakpoints.clone())
        }

        fn read_registers(&self) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
            self.read_backend(|backend| Self::create_register_snapshot(&backend.registers))
        }

        fn write_register(
            &mut self,
            register_name: &str,
            value: u64,
        ) -> Result<DebuggerRegisterSnapshot, DebuggerPluginError> {
            self.mutate_backend(|backend| {
                if let Some(register) = backend
                    .registers
                    .iter_mut()
                    .find(|register| register.get_name() == register_name)
                {
                    *register = DebuggerRegisterValue::new(register_name, value, register.get_bit_width());
                } else {
                    backend
                        .registers
                        .push(DebuggerRegisterValue::new(register_name, value, 64));
                }

                Self::create_register_snapshot(&backend.registers)
            })
        }
    }

    #[test]
    fn trace_disassembly_rejects_unknown_instruction_text() {
        assert!(DebuggerService::is_meaningful_instruction_text("mov [rsp+18h], rax"));
        assert!(!DebuggerService::is_meaningful_instruction_text("??"));
        assert!(!DebuggerService::is_meaningful_instruction_text("[??]"));
        assert!(!DebuggerService::is_meaningful_instruction_text("   "));
    }

    #[test]
    fn trace_disassembly_uses_target_architecture_instruction_set() {
        let plugin_registry = PluginRegistry::from_plugin_packages(vec![Arc::new(TestInstructionSetPlugin::new())]);
        let trace_event = DebuggerTraceEvent::new(None, DebuggerRegisterSnapshot::default(), Some(0x4000), vec![0x00], None, None);

        let enriched_trace_event = DebuggerService::enrich_trace_event_with_disassembly(&plugin_registry, &TargetArchitecture::arm64(), trace_event);

        assert_eq!(enriched_trace_event.get_instruction_text(), Some("arm64-disassembly-block-1"));
    }

    #[test]
    fn trace_disassembly_selects_thumb_for_arm_interworking_address() {
        let plugin_registry = PluginRegistry::from_plugin_packages(vec![Arc::new(TestInstructionSetPlugin::new())]);
        let trace_event = DebuggerTraceEvent::new(None, DebuggerRegisterSnapshot::default(), Some(0x4001), vec![0x00, 0xBF], None, None);

        let enriched_trace_event = DebuggerService::enrich_trace_event_with_disassembly(&plugin_registry, &TargetArchitecture::arm(), trace_event);

        assert_eq!(enriched_trace_event.get_instruction_text(), Some("thumb-disassembly-block-2"));
        assert_eq!(enriched_trace_event.get_instruction_address(), Some(0x4000));
        assert_eq!(
            enriched_trace_event
                .get_target_architecture()
                .map(TargetArchitecture::get_instruction_set_id),
            Some("thumb")
        );
    }

    #[test]
    fn trace_disassembly_uses_block_decoder_first_instruction() {
        let plugin_registry = PluginRegistry::from_plugin_packages(vec![Arc::new(TestInstructionSetPlugin::new())]);
        let trace_event = DebuggerTraceEvent::new(None, DebuggerRegisterSnapshot::default(), Some(0x4000), vec![0x90, 0x90], None, None);

        let enriched_trace_event = DebuggerService::enrich_trace_event_with_disassembly(&plugin_registry, &TargetArchitecture::x64(), trace_event);

        assert_eq!(enriched_trace_event.get_instruction_text(), Some("test-disassembly-block-2"));
    }

    #[test]
    fn trace_disassembly_passes_instruction_address_to_block_decoder() {
        let plugin_registry = PluginRegistry::from_plugin_packages(vec![Arc::new(TestInstructionSetPlugin::new())]);
        let target_architecture = TargetArchitecture::new("base-test", "i_base_test", Bitness::Bit64, Endianness::Little);
        let trace_event = DebuggerTraceEvent::new(None, DebuggerRegisterSnapshot::default(), Some(0x401234), vec![0x90], None, None);

        let enriched_trace_event = DebuggerService::enrich_trace_event_with_disassembly(&plugin_registry, &target_architecture, trace_event);

        assert_eq!(enriched_trace_event.get_instruction_text(), Some("base-disassembly-block-1-0x401234"));
    }

    #[test]
    fn trace_disassembly_does_not_use_unknown_block_instruction() {
        let plugin_registry = PluginRegistry::from_plugin_packages(vec![Arc::new(TestInstructionSetPlugin::new())]);
        let trace_event = DebuggerTraceEvent::new(None, DebuggerRegisterSnapshot::default(), Some(0x4000), vec![0xCC], None, None);

        let enriched_trace_event = DebuggerService::enrich_trace_event_with_disassembly(&plugin_registry, &TargetArchitecture::x64(), trace_event);

        assert_eq!(enriched_trace_event.get_instruction_text(), None);
    }

    #[test]
    fn service_selects_requested_debugger_and_routes_session_operations() {
        let first_backend = Arc::new(Mutex::new(TestDebuggerBackend::default()));
        let second_backend = Arc::new(Mutex::new(TestDebuggerBackend::default()));
        let plugin_registry = Arc::new(PluginRegistry::from_plugin_packages(vec![
            Arc::new(TestDebuggerPlugin::new("test.debugger.first", "Game.exe", first_backend.clone())),
            Arc::new(TestDebuggerPlugin::new("test.debugger.second", "Game.exe", second_backend.clone())),
            Arc::new(TestInstructionSetPlugin::new()),
        ]));
        let emitted_events = Arc::new(Mutex::new(Vec::new()));
        let event_sink = emitted_events.clone();
        let debugger_service = DebuggerService::new(
            plugin_registry,
            Arc::new(move |engine_event| {
                if let Ok(mut events) = event_sink.lock() {
                    events.push(engine_event);
                }
            }),
        );
        let opened_process_info = OpenedProcessInfo::new(99, String::from("Game.exe"), 1234, Bitness::Bit64, None);

        let attach_status = debugger_service
            .attach(&opened_process_info, Some("test.debugger.second"))
            .expect("Expected requested debugger plugin to attach.");
        assert_eq!(attach_status.get_session_state(), DebuggerSessionState::Attached);
        assert_eq!(attach_status.get_active_plugin_id(), Some("test.debugger.second"));
        assert_eq!(
            debugger_service.get_active_plugin_id_for_process(&opened_process_info),
            Some(String::from("test.debugger.second"))
        );

        let breakpoint = debugger_service
            .set_breakpoint(0x401000, DebuggerBreakpointKind::Software, Some(String::from("entry")))
            .expect("Expected breakpoint creation to route to the active debugger session.");
        assert_eq!(breakpoint.get_breakpoint_id(), "bp-1");
        assert_eq!(
            debugger_service
                .list_breakpoints()
                .expect("Expected breakpoint list to route to the active debugger session.")
                .len(),
            1
        );

        let pause_status = debugger_service
            .pause()
            .expect("Expected pause to route to the active debugger session.");
        assert_eq!(pause_status.get_session_state(), DebuggerSessionState::Paused);
        let resume_status = debugger_service
            .resume()
            .expect("Expected resume to route to the active debugger session.");
        assert_eq!(resume_status.get_session_state(), DebuggerSessionState::Running);

        let register_snapshot = debugger_service
            .write_register("rax", 0xFEED)
            .expect("Expected register writes to route to the active debugger session.");
        assert_eq!(register_snapshot.get_instruction_pointer(), Some(0x401000));
        assert_eq!(
            debugger_service
                .read_registers()
                .expect("Expected register reads to route to the active debugger session.")
                .get_registers()[0]
                .get_value(),
            0xFEED
        );

        let detach_status = debugger_service
            .detach()
            .expect("Expected detach to route to the active debugger session.");
        assert_eq!(detach_status.get_session_state(), DebuggerSessionState::Detached);
        assert_eq!(detach_status.get_active_plugin_id(), None);
        assert_eq!(debugger_service.get_active_plugin_id_for_process(&opened_process_info), None);

        assert_eq!(
            first_backend
                .lock()
                .map(|backend| backend.create_session_count)
                .expect("Expected first backend lock to be available."),
            0
        );
        assert_eq!(
            second_backend
                .lock()
                .map(|backend| backend.create_session_count)
                .expect("Expected second backend lock to be available."),
            1
        );

        let session_events = emitted_events
            .lock()
            .map(|events| {
                events
                    .iter()
                    .filter_map(|engine_event| match engine_event {
                        EngineEvent::Debugger(DebuggerEvent::SessionStateChanged {
                            debugger_session_state_changed_event,
                        }) => Some((
                            debugger_session_state_changed_event.session_state,
                            debugger_session_state_changed_event.active_plugin_id.clone(),
                        )),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
            })
            .expect("Expected emitted event lock to be available.");

        assert_eq!(
            session_events,
            vec![
                (DebuggerSessionState::Attached, Some(String::from("test.debugger.second"))),
                (DebuggerSessionState::Paused, Some(String::from("test.debugger.second"))),
                (DebuggerSessionState::Paused, Some(String::from("test.debugger.second"))),
                (DebuggerSessionState::Running, Some(String::from("test.debugger.second"))),
                (DebuggerSessionState::Detached, None),
            ]
        );

        let trace_event_texts = emitted_events
            .lock()
            .map(|events| {
                events
                    .iter()
                    .filter_map(|engine_event| match engine_event {
                        EngineEvent::Debugger(DebuggerEvent::TraceRecorded { debugger_trace_recorded_event }) => debugger_trace_recorded_event
                            .trace_event
                            .get_instruction_text()
                            .map(String::from),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
            })
            .expect("Expected emitted event lock to be available.");

        assert_eq!(trace_event_texts, vec![String::from("test-disassembly-block-1")]);
    }

    #[test]
    fn service_starts_trace_session_and_aggregates_breakpoint_hits() {
        let debugger_backend = Arc::new(Mutex::new(TestDebuggerBackend::default()));
        let plugin_registry = Arc::new(PluginRegistry::from_plugin_packages(vec![
            Arc::new(TestDebuggerPlugin::new("test.debugger", "Game.exe", debugger_backend.clone())),
            Arc::new(TestInstructionSetPlugin::new()),
        ]));
        let emitted_events = Arc::new(Mutex::new(Vec::new()));
        let event_sink = emitted_events.clone();
        let debugger_service = DebuggerService::new(
            plugin_registry,
            Arc::new(move |engine_event| {
                if let Ok(mut events) = event_sink.lock() {
                    events.push(engine_event);
                }
            }),
        );
        let opened_process_info = OpenedProcessInfo::new(99, String::from("Game.exe"), 1234, Bitness::Bit64, None);

        debugger_service
            .attach(&opened_process_info, Some("test.debugger"))
            .expect("Expected debugger plugin to attach.");
        let (trace_session, initial_records) = debugger_service
            .start_trace_session(0x5000, 4, DebuggerDataBreakpointAccess::Write, Some(String::from("health")))
            .expect("Expected trace session to start.");

        assert_eq!(trace_session.get_trace_session_id(), "trace-1");
        assert_eq!(trace_session.get_address(), 0x5000);
        assert_eq!(trace_session.get_size_in_bytes(), 4);
        assert_eq!(trace_session.get_access(), DebuggerDataBreakpointAccess::Write);
        assert!(trace_session.get_is_active());
        assert!(trace_session.get_breakpoint().get_is_enabled());
        assert!(initial_records.is_empty());
        assert_eq!(
            debugger_backend
                .lock()
                .map(|backend| backend.resume_count)
                .expect("Expected debugger backend lock."),
            1
        );

        let (paused_trace_session, paused_records) = debugger_service
            .pause_trace_session(trace_session.get_trace_session_id())
            .expect("Expected trace collection to pause.");
        assert!(paused_trace_session.get_is_active());
        assert!(!paused_trace_session.get_breakpoint().get_is_enabled());
        assert!(
            debugger_backend
                .lock()
                .map(|backend| backend.breakpoints[0].get_is_enabled())
                .expect("Expected debugger backend lock."),
            "Collection pause should not disable the backend breakpoint."
        );
        assert_eq!(paused_records.len(), 1);
        assert_eq!(paused_records[0].get_hit_count(), 1);

        debugger_service
            .resume()
            .expect("Expected target resume while trace collection is paused.");
        let (_, paused_instruction_records) = debugger_service
            .list_trace_sessions()
            .expect("Expected trace session list after paused target resume.");
        assert_eq!(paused_instruction_records.len(), 1);
        assert_eq!(paused_instruction_records[0].get_hit_count(), 1);

        let (resumed_trace_session, resumed_records) = debugger_service
            .resume_trace_session(trace_session.get_trace_session_id())
            .expect("Expected trace collection to resume.");
        assert!(resumed_trace_session.get_is_active());
        assert!(resumed_trace_session.get_breakpoint().get_is_enabled());
        assert_eq!(resumed_records.len(), 1);
        assert_eq!(resumed_records[0].get_hit_count(), 1);

        debugger_service.resume().expect("Expected first trace hit.");
        debugger_service.resume().expect("Expected second trace hit.");

        let (trace_sessions, instruction_records) = debugger_service
            .list_trace_sessions()
            .expect("Expected trace session list.");
        assert_eq!(trace_sessions.len(), 1);
        assert_eq!(instruction_records.len(), 1);
        assert_eq!(instruction_records[0].get_trace_session_id(), "trace-1");
        assert_eq!(instruction_records[0].get_instruction_address(), Some(0x401000));
        assert_eq!(instruction_records[0].get_instruction_text(), Some("test-disassembly-block-1"));
        assert_eq!(instruction_records[0].get_hit_count(), 3);

        let (stopped_trace_session, stopped_records) = debugger_service
            .stop_trace_session(trace_session.get_trace_session_id())
            .expect("Expected trace session to stop.");
        assert!(!stopped_trace_session.get_is_active());
        assert!(stopped_records.is_empty());
        let (trace_sessions_after_stop, instruction_records_after_stop) = debugger_service
            .list_trace_sessions()
            .expect("Expected trace session list after stop.");
        assert!(trace_sessions_after_stop.is_empty());
        assert!(instruction_records_after_stop.is_empty());
        assert!(
            debugger_backend
                .lock()
                .map(|backend| backend.breakpoints.is_empty())
                .expect("Expected debugger backend lock.")
        );

        let trace_hit_counts = emitted_events
            .lock()
            .map(|events| {
                events
                    .iter()
                    .filter_map(|engine_event| match engine_event {
                        EngineEvent::Debugger(DebuggerEvent::TraceSessionUpdated {
                            debugger_trace_session_updated_event,
                        }) => debugger_trace_session_updated_event
                            .instruction_records
                            .last()
                            .map(|instruction_record| instruction_record.get_hit_count()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
            })
            .expect("Expected emitted event lock to be available.");

        assert_eq!(trace_hit_counts, vec![1, 1, 1, 2, 3]);
    }

    #[test]
    fn service_can_restart_trace_session_after_pause_then_stop() {
        let debugger_backend = Arc::new(Mutex::new(TestDebuggerBackend::default()));
        let plugin_registry = Arc::new(PluginRegistry::from_plugin_packages(vec![
            Arc::new(TestDebuggerPlugin::new("test.debugger", "Game.exe", debugger_backend.clone())),
            Arc::new(TestInstructionSetPlugin::new()),
        ]));
        let debugger_service = DebuggerService::new(plugin_registry, Arc::new(|_engine_event| {}));
        let opened_process_info = OpenedProcessInfo::new(99, String::from("Game.exe"), 1234, Bitness::Bit64, None);

        debugger_service
            .attach(&opened_process_info, Some("test.debugger"))
            .expect("Expected debugger plugin to attach.");
        let (trace_session, _) = debugger_service
            .start_trace_session(0x5000, 4, DebuggerDataBreakpointAccess::Write, Some(String::from("health")))
            .expect("Expected trace session to start.");
        debugger_service.resume().expect("Expected initial trace hit.");
        debugger_service
            .pause_trace_session(trace_session.get_trace_session_id())
            .expect("Expected trace collection to pause.");
        debugger_service
            .stop_trace_session(trace_session.get_trace_session_id())
            .expect("Expected paused trace session to stop.");

        let (trace_sessions_after_stop, instruction_records_after_stop) = debugger_service
            .list_trace_sessions()
            .expect("Expected trace session list after pause-stop.");
        assert!(trace_sessions_after_stop.is_empty());
        assert!(instruction_records_after_stop.is_empty());

        let (restarted_trace_session, restarted_records) = debugger_service
            .start_trace_session(0x5000, 4, DebuggerDataBreakpointAccess::Write, Some(String::from("health restart")))
            .expect("Expected trace session to restart after pause-stop.");
        assert!(restarted_records.is_empty());

        let (restarted_trace_sessions, restarted_instruction_records) = debugger_service
            .list_trace_sessions()
            .expect("Expected restarted trace session list.");
        assert_eq!(restarted_trace_sessions.len(), 1);
        assert_eq!(
            restarted_trace_sessions[0].get_trace_session_id(),
            restarted_trace_session.get_trace_session_id()
        );
        assert_eq!(restarted_instruction_records.len(), 1);
        assert_eq!(restarted_instruction_records[0].get_hit_count(), 1);
    }

    #[test]
    fn service_stops_trace_session_even_when_breakpoint_cleanup_fails() {
        let debugger_backend = Arc::new(Mutex::new(TestDebuggerBackend::default()));
        let plugin_registry = Arc::new(PluginRegistry::from_plugin_packages(vec![
            Arc::new(TestDebuggerPlugin::new("test.debugger", "Game.exe", debugger_backend.clone())),
            Arc::new(TestInstructionSetPlugin::new()),
        ]));
        let debugger_service = DebuggerService::new(plugin_registry, Arc::new(|_engine_event| {}));
        let opened_process_info = OpenedProcessInfo::new(99, String::from("Game.exe"), 1234, Bitness::Bit64, None);

        debugger_service
            .attach(&opened_process_info, Some("test.debugger"))
            .expect("Expected debugger plugin to attach.");
        let (trace_session, _) = debugger_service
            .start_trace_session(0x5000, 4, DebuggerDataBreakpointAccess::Write, Some(String::from("health")))
            .expect("Expected trace session to start.");

        debugger_backend
            .lock()
            .expect("Expected debugger backend lock.")
            .fail_remove_breakpoint = true;

        let (stopped_trace_session, _) = debugger_service
            .stop_trace_session(trace_session.get_trace_session_id())
            .expect("Expected trace session stop to tolerate backend cleanup failure.");

        assert!(!stopped_trace_session.get_is_active());
        assert!(
            debugger_service
                .pause_trace_session(trace_session.get_trace_session_id())
                .is_err()
        );
    }
}
