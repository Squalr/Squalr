use crate::events::engine_event::EngineEvent;
use crate::squalr_engine::SqualrEngine;
use squalr_engine_common::logging::{log_level::LogLevel, logger::Logger};
use squalr_engine_processes::process_info::OpenedProcessInfo;
use squalr_engine_scanning::snapshots::snapshot::Snapshot;
use std::sync::{Arc, Once, RwLock};

pub struct SqualrSession {
    /// The process to which Squalr is attached.
    opened_process: RwLock<Option<OpenedProcessInfo>>,

    /// The current snapshot of process memory, which may contain previous and current scan results.
    snapshot: Arc<RwLock<Snapshot>>,
}

impl SqualrSession {
    pub fn get_instance() -> &'static SqualrSession {
        static mut INSTANCE: Option<SqualrSession> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = SqualrSession::new();
                INSTANCE = Some(instance);
            });

            #[allow(static_mut_refs)]
            INSTANCE.as_ref().unwrap_unchecked()
        }
    }

    fn new() -> Self {
        SqualrSession {
            opened_process: RwLock::new(None),
            snapshot: Arc::new(RwLock::new(Snapshot::new(vec![]))),
        }
    }

    pub fn set_opened_process(process_info: OpenedProcessInfo) {
        if let Ok(mut process) = SqualrSession::get_instance().opened_process.write() {
            Logger::get_instance().log(
                LogLevel::Info,
                &format!("Opened process: {}, pid: {}", process_info.name, process_info.pid),
                None,
            );
            *process = Some(process_info.clone());

            if let Err(err) = SqualrEngine::broadcast_engine_event(EngineEvent::ProcessOpened(process_info)) {
                Logger::get_instance().log(LogLevel::Error, &format!("Error sending opened process event: {}", err), None);
            }
        }
    }

    pub fn clear_opened_process() {
        if let Ok(mut process) = SqualrSession::get_instance().opened_process.write() {
            *process = None;
            Logger::get_instance().log(LogLevel::Info, "Process closed", None);
        }
    }

    pub fn get_opened_process() -> Option<OpenedProcessInfo> {
        SqualrSession::get_instance()
            .opened_process
            .read()
            .ok()
            .and_then(|guard| guard.clone())
    }

    pub fn get_snapshot() -> Arc<RwLock<Snapshot>> {
        SqualrSession::get_instance().snapshot.clone()
    }
}
