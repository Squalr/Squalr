use crate::commands::command_handlers::memory;
use crate::commands::command_handlers::process;
use crate::commands::command_handlers::project;
use crate::commands::command_handlers::results;
use crate::commands::command_handlers::scan;
use crate::commands::command_handlers::settings;
use crate::commands::engine_command::EngineCommand;
use squalr_engine_architecture::vectors::vectors;
use squalr_engine_common::logging::{log_level::LogLevel, logger::Logger};
use squalr_engine_processes::{process_info::OpenedProcessInfo, process_query::process_queryer::ProcessQuery};
use squalr_engine_scanning::snapshots::snapshot::Snapshot;
use std::io;
use std::process::Child;
use std::process::Command;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Once, RwLock};
use std::thread;

pub struct SqualrEngine {
    /// The process to which Squalr is attached.
    opened_process: RwLock<Option<OpenedProcessInfo>>,

    /// The current snapshot of process memory, which may contain previous and current scan results.
    snapshot: Arc<RwLock<Snapshot>>,

    /// Whether Squalr is in inter-process communication mode. This is necessary on platforms like Android,
    /// where the GUI app is unprivileged, and needs to spawn a root access client to do the heavy lifting.
    ipc_mode: AtomicBool,

    /// The (optional) spawned child process with elevated privileges.
    ipc_server: Arc<RwLock<Option<Child>>>,
}

impl SqualrEngine {
    fn new() -> Self {
        SqualrEngine {
            opened_process: RwLock::new(None),
            snapshot: Arc::new(RwLock::new(Snapshot::new(vec![]))),
            ipc_mode: AtomicBool::new(false),
            ipc_server: Arc::new(RwLock::new(None)),
        }
    }

    fn get_instance() -> &'static SqualrEngine {
        static mut INSTANCE: Option<SqualrEngine> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                INSTANCE = Some(SqualrEngine::new());
            });

            #[allow(static_mut_refs)]
            INSTANCE.as_ref().unwrap_unchecked()
        }
    }

    pub fn initialize(ipc_mode: bool) {
        Self::get_instance()
            .ipc_mode
            .store(ipc_mode, std::sync::atomic::Ordering::Relaxed);

        Logger::get_instance().log(LogLevel::Info, "Squalr started", None);
        vectors::log_vector_architecture();

        if let Err(err) = ProcessQuery::start_monitoring() {
            Logger::get_instance().log(LogLevel::Error, &format!("Failed to monitor system processes: {}", err), None);
        }

        if ipc_mode {
            Logger::get_instance().log(LogLevel::Info, &"Spawning squalr-cli privileged shell...", None);

            thread::spawn(|| match Self::spawn_squalr_cli_as_root() {
                Ok(child) => {
                    Logger::get_instance().log(LogLevel::Info, &"Spawned squalr-cli as root.", None);
                    if let Ok(mut ipc_server) = SqualrEngine::get_instance().ipc_server.write() {
                        *ipc_server = Some(child);
                    }
                }
                Err(err) => {
                    Logger::get_instance().log(LogLevel::Info, &format!("Failed to spawn squalr-cli as root: {}", err), None);
                }
            });
        }
    }

    pub fn dispatch_command(command: &mut EngineCommand) {
        match command {
            EngineCommand::Memory(cmd) => memory::handle_memory_command(cmd),
            EngineCommand::Process(cmd) => process::handle_process_command(cmd),
            EngineCommand::Project(cmd) => project::handle_project_command(cmd),
            EngineCommand::Results(cmd) => results::handle_results_command(cmd),
            EngineCommand::Scan(cmd) => scan::handle_scan_command(cmd),
            EngineCommand::Settings(cmd) => settings::handle_settings_command(cmd),
        }
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

    #[cfg(target_os = "android")]
    fn spawn_squalr_cli_as_root() -> io::Result<Child> {
        Command::new("su")
            .arg("-c")
            .arg("squalr-cli --ipc-mode")
            .spawn()
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    fn spawn_squalr_cli_as_root() -> io::Result<Child> {
        Command::new("sudo").arg("squalr-cli").arg("--ipc-mode").spawn()
    }

    #[cfg(windows)]
    fn spawn_squalr_cli_as_root() -> io::Result<Child> {
        // No actual privilege escallation for windows -- this feature is not supposed to be used on windows at all.
        // So, just spawn it normally for the rare occasion that we are testing this feature on windows.
        Command::new("squalr-cli").arg("--ipc-mode").spawn()
    }
}
