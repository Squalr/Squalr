use crate::process_query::{process_query_options::ProcessQueryOptions, process_queryer::ProcessQuery};
use squalr_engine_api::{
    events::{
        engine_event::{EngineEvent, EngineEventRequest},
        process::changed::process_changed_event::ProcessChangedEvent,
    },
    structures::processes::opened_process_info::OpenedProcessInfo,
};
use std::{
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};

pub struct ProcessManager {
    opened_process: Arc<RwLock<Option<OpenedProcessInfo>>>,
    event_emitter: Arc<dyn Fn(EngineEvent) + Send + Sync>,
}

impl ProcessManager {
    pub fn new(event_emitter: Arc<dyn Fn(EngineEvent) + Send + Sync>) -> Self {
        let instance = Self {
            opened_process: Arc::new(RwLock::new(None)),
            event_emitter: event_emitter.clone(),
        };

        Self::listen_for_open_process_death(event_emitter, instance.opened_process.clone());

        instance
    }

    /// Sets the process to which we are currently attached.
    pub fn set_opened_process(
        &self,
        process_info: OpenedProcessInfo,
    ) {
        if let Ok(mut process) = self.opened_process.write() {
            log::info!("Opened process: {}, pid: {}", process_info.get_name(), process_info.get_process_id());
            *process = Some(process_info.clone());

            (self.event_emitter)(
                ProcessChangedEvent {
                    process_info: Some(process_info),
                }
                .to_engine_event(),
            );
        }
    }

    /// Clears the process to which we are currently attached.
    pub fn clear_opened_process(&self) {
        if let Ok(mut process) = self.opened_process.write() {
            *process = None;

            log::info!("Process closed.");

            (self.event_emitter)(ProcessChangedEvent { process_info: None }.to_engine_event());
        }
    }

    /// Gets the process to which we are currently attached, if any.
    pub fn get_opened_process(&self) -> Option<OpenedProcessInfo> {
        match self.opened_process.read() {
            Ok(opened_process) => opened_process.clone(),
            Err(error) => {
                log::error!("Failed to access opened process: {}", error);
                None
            }
        }
    }

    /// Gets a reference to the shared lock containing the currently opened process.
    pub fn get_opened_process_ref(&self) -> Arc<RwLock<Option<OpenedProcessInfo>>> {
        self.opened_process.clone()
    }

    /// Listens for the death of the currently opened process by polling for it repeatedly.
    fn listen_for_open_process_death(
        event_emitter: Arc<dyn Fn(EngineEvent) + Send + Sync>,
        opened_process: Arc<RwLock<Option<OpenedProcessInfo>>>,
    ) {
        std::thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(100));

                let opened_process_id = {
                    let read_result = opened_process.read();
                    if let Ok(guard) = read_result {
                        if let Some(opened_process_info) = guard.as_ref() {
                            opened_process_info.get_process_id()
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                };

                let process_query_options = ProcessQueryOptions {
                    required_process_id: Some(opened_process_id),
                    search_name: None,
                    require_windowed: false,
                    match_case: false,
                    fetch_icons: false,
                    limit: Some(1),
                };

                let processes = ProcessQuery::get_processes(process_query_options);

                if processes.len() <= 0 {
                    if let Ok(mut opened_process) = opened_process.write() {
                        *opened_process = None;
                        log::info!("Process no longer open, detaching.");
                        (event_emitter)(ProcessChangedEvent { process_info: None }.to_engine_event());
                    }
                }
            }
        });
    }
}
