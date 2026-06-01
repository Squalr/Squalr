use crate::plugins::plugin_registry::PluginRegistry;
use squalr_engine_api::events::debugger::session_state_changed::debugger_session_state_changed_event::DebuggerSessionStateChangedEvent;
use squalr_engine_api::events::engine_event::{EngineEvent, EngineEventRequest};
use squalr_engine_api::plugins::debugger::DebuggerSession;
use squalr_engine_api::structures::debugger::{DebuggerBreakpointDescriptor, DebuggerBreakpointKind, DebuggerRegisterSnapshot, DebuggerSessionState};
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use std::sync::{Arc, Mutex, RwLock};

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

pub struct DebuggerService {
    plugin_registry: Arc<PluginRegistry>,
    active_session: RwLock<Option<CachedDebuggerSession>>,
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
            debugger_session
                .attach()
                .map(|session_state| (session_state, debugger_session.plugin_id().to_string()))
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

    pub fn list_breakpoints(&self) -> Result<Vec<DebuggerBreakpointDescriptor>, String> {
        let active_session = self.get_cached_session()?;

        self.with_debugger_session(&active_session.session, |debugger_session| {
            debugger_session
                .list_breakpoints()
                .map_err(|error| error.to_string())
        })
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
        let debugger_session = debugger_plugin
            .create_session(process_info)
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
}

struct CachedDebuggerSessionSnapshot {
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
            debugger::{DebuggerPlugin, DebuggerPluginError, DebuggerSession},
        },
        structures::{
            debugger::{DebuggerBreakpointDescriptor, DebuggerBreakpointKind, DebuggerRegisterSnapshot, DebuggerRegisterValue, DebuggerSessionState},
            memory::bitness::Bitness,
            processes::opened_process_info::OpenedProcessInfo,
        },
    };
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct TestDebuggerBackend {
        create_session_count: u32,
        state: DebuggerSessionState,
        breakpoints: Vec<DebuggerBreakpointDescriptor>,
        registers: Vec<DebuggerRegisterValue>,
    }

    struct TestDebuggerPlugin {
        metadata: PluginMetadata,
        process_name: String,
        backend: Arc<Mutex<TestDebuggerBackend>>,
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
        ) -> Result<Box<dyn DebuggerSession>, DebuggerPluginError> {
            self.backend
                .lock()
                .map(|mut backend| backend.create_session_count += 1)
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
            self.mutate_backend(|backend| {
                backend.state = DebuggerSessionState::Running;
                backend.state
            })
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
                backend
                    .breakpoints
                    .retain(|breakpoint| breakpoint.get_breakpoint_id() != breakpoint_id);
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
    fn service_selects_requested_debugger_and_routes_session_operations() {
        let first_backend = Arc::new(Mutex::new(TestDebuggerBackend::default()));
        let second_backend = Arc::new(Mutex::new(TestDebuggerBackend::default()));
        let plugin_registry = Arc::new(PluginRegistry::from_plugin_packages(vec![
            Arc::new(TestDebuggerPlugin::new("test.debugger.first", "Game.exe", first_backend.clone())),
            Arc::new(TestDebuggerPlugin::new("test.debugger.second", "Game.exe", second_backend.clone())),
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
                (DebuggerSessionState::Running, Some(String::from("test.debugger.second"))),
                (DebuggerSessionState::Detached, None),
            ]
        );
    }
}
