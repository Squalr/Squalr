use crate::commands::command_dispatchers::command_dispatcher::{CommandDispatcher, CommandDispatcherType};
use crate::commands::command_dispatchers::inter_process_command_dispatcher::InterProcessCommandDispatcher;
use crate::commands::command_handlers::command_handler::CommandHandlerType;
use crate::commands::command_handlers::inter_process_command_handler::InterProcessCommandHandler;
use crate::commands::engine_command::EngineCommand;
use squalr_engine_architecture::vectors::vectors;
use squalr_engine_common::logging::{log_level::LogLevel, logger::Logger};
use squalr_engine_processes::{process_info::OpenedProcessInfo, process_query::process_queryer::ProcessQuery};
use squalr_engine_scanning::snapshots::snapshot::Snapshot;
use std::sync::Mutex;
use std::sync::{Arc, Once, RwLock};

static mut INSTANCE: Option<SqualrEngine> = None;
static INIT: Once = Once::new();

/// Defines the mode of operation for Squalr's engine.
pub enum EngineMode {
    /// Standalone mode grants full functionality.
    Standalone,

    /// Client mode defers heavy lifting to the server, and only sends and recieves commands.
    Client,

    /// Server mode waits for commands from the client, does privileged work (scanning, debugging, etc),
    /// and sends responses to client.
    Server,
}

pub struct SqualrEngine {
    /// The process to which Squalr is attached.
    opened_process: RwLock<Option<OpenedProcessInfo>>,

    /// The current snapshot of process memory, which may contain previous and current scan results.
    snapshot: Arc<RwLock<Snapshot>>,

    /// Handles sending commands to the engine.
    command_dispatcher: Arc<Mutex<CommandDispatcherType>>,

    /// Handles receiving commands from the engine.
    _command_handler: Arc<Mutex<CommandHandlerType>>,
}

impl SqualrEngine {
    fn new(engine_mode: EngineMode) -> Self {
        let command_dispatcher = match engine_mode {
            EngineMode::Standalone => CommandDispatcherType::Standard(),
            EngineMode::Client => CommandDispatcherType::InterProcess(InterProcessCommandDispatcher::new()),
            EngineMode::Server => CommandDispatcherType::Standard(),
        };

        let command_handler = match engine_mode {
            EngineMode::Standalone => CommandHandlerType::Standard(),
            EngineMode::Client => CommandHandlerType::Standard(),
            EngineMode::Server => CommandHandlerType::InterProcess(InterProcessCommandHandler::new()),
        };

        SqualrEngine {
            opened_process: RwLock::new(None),
            snapshot: Arc::new(RwLock::new(Snapshot::new(vec![]))),
            command_dispatcher: Arc::new(Mutex::new(command_dispatcher)),
            _command_handler: Arc::new(Mutex::new(command_handler)),
        }
    }

    fn create_instance(engine_mode: EngineMode) {
        unsafe {
            INIT.call_once(|| {
                INSTANCE = Some(SqualrEngine::new(engine_mode));
            });
        }
    }

    fn get_instance() -> &'static SqualrEngine {
        unsafe {
            // If create_instance() has never been called before, default to standalone.
            Self::create_instance(EngineMode::Standalone);

            #[allow(static_mut_refs)]
            INSTANCE.as_ref().unwrap()
        }
    }

    pub fn initialize(engine_mode: EngineMode) {
        Self::create_instance(engine_mode);

        Logger::get_instance().log(LogLevel::Info, "Squalr started", None);
        vectors::log_vector_architecture();

        if let Err(err) = ProcessQuery::start_monitoring() {
            Logger::get_instance().log(LogLevel::Error, &format!("Failed to monitor system processes: {}", err), None);
        }
    }

    pub fn dispatch_command(command: EngineCommand) {
        let mut command = command.clone();
        std::thread::spawn(move || {
            if let Ok(dispatcher) = Self::get_instance().command_dispatcher.lock() {
                dispatcher.dispatch_command(&mut command);
            }
        });
    }

    pub fn set_opened_process(process_info: OpenedProcessInfo) {
        let instance = Self::get_instance();
        if let Ok(mut process) = instance.opened_process.write() {
            Logger::get_instance().log(
                LogLevel::Info,
                &format!("Opened process: {}, pid: {}", process_info.name, process_info.pid),
                None,
            );
            *process = Some(process_info);
        }
    }

    pub fn clear_opened_process() {
        let instance = Self::get_instance();
        if let Ok(mut process) = instance.opened_process.write() {
            *process = None;
            Logger::get_instance().log(LogLevel::Info, "Process closed", None);
        }
    }

    pub fn get_opened_process() -> Option<OpenedProcessInfo> {
        let instance = Self::get_instance();
        instance
            .opened_process
            .read()
            .ok()
            .and_then(|guard| guard.clone())
    }

    pub fn get_snapshot() -> Arc<RwLock<Snapshot>> {
        let instance = Self::get_instance();
        instance.snapshot.clone()
    }
}
