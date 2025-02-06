use crate::command_handlers::command_handler::CommandHandlerType;
use crate::events::engine_event::EngineEvent;
use crate::requests::command_dispatcher::CommandDispatcher;
use crate::requests::command_dispatcher::CommandDispatcherType;
use crate::requests::engine_command::EngineCommand;
use squalr_engine_architecture::vectors::vectors;
use squalr_engine_common::logging::{log_level::LogLevel, logger::Logger};
use squalr_engine_processes::process_query::process_queryer::ProcessQuery;
use std::sync::mpsc::SendError;
use std::sync::{Arc, Once};
use std::sync::{Mutex, mpmc};

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

/// Orchestrates commands and responses to and from the engine. This is leveraged by the GUI, CLI, etc.
pub struct SqualrEngine {
    /// Handles sending commands to the engine.
    command_dispatcher: Arc<Mutex<CommandDispatcherType>>,

    /// Handles receiving commands from the engine.
    _command_handler: Arc<Mutex<CommandHandlerType>>,

    /// Handles broadcasting events from the engine.
    event_sender: mpmc::Sender<EngineEvent>,

    /// Clonable receiver for receiving events from the engine.
    event_receiver: mpmc::Receiver<EngineEvent>,
}

impl SqualrEngine {
    fn new(engine_mode: EngineMode) -> Self {
        let command_dispatcher = match engine_mode {
            EngineMode::Standalone => CommandDispatcherType::Standalone(),
            EngineMode::Client => CommandDispatcherType::InterProcess(),
            EngineMode::Server => CommandDispatcherType::Standalone(),
        };

        let command_handler = match engine_mode {
            EngineMode::Standalone => CommandHandlerType::Standalone(),
            EngineMode::Client => CommandHandlerType::Standalone(),
            EngineMode::Server => CommandHandlerType::InterProcess(),
        };

        let (event_sender, event_receiver) = mpmc::channel();

        SqualrEngine {
            command_dispatcher: Arc::new(Mutex::new(command_dispatcher)),
            _command_handler: Arc::new(Mutex::new(command_handler)),
            event_sender: event_sender,
            event_receiver: event_receiver,
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
        Logger::get_instance().log(LogLevel::Info, "Squalr started", None);
        vectors::log_vector_architecture();

        Self::create_instance(engine_mode);

        if let Err(err) = ProcessQuery::start_monitoring() {
            Logger::get_instance().log(LogLevel::Error, &format!("Failed to monitor system processes: {}", err), None);
        }
    }

    pub fn dispatch_command(command: EngineCommand) {
        let mut command = command.clone();
        if let Ok(dispatcher) = Self::get_instance().command_dispatcher.lock() {
            dispatcher.dispatch_command(&mut command);
        }
    }

    pub fn dispatch_command_async(command: EngineCommand) {
        let mut command = command.clone();
        std::thread::spawn(move || {
            if let Ok(dispatcher) = Self::get_instance().command_dispatcher.lock() {
                dispatcher.dispatch_command(&mut command);
            }
        });
    }

    pub fn get_engine_event_receiver() -> mpmc::Receiver<EngineEvent> {
        SqualrEngine::get_instance().event_receiver.clone()
    }

    pub fn broadcast_engine_event(event: EngineEvent) -> Result<(), SendError<EngineEvent>> {
        SqualrEngine::get_instance().event_sender.send(event)
    }
}
