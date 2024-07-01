use sysinfo::{Pid, System};
use squalr_engine_common::logging::logger::Logger;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ProcessSession {
    opened_process: Arc<Mutex<Option<Pid>>>,
    system: Arc<Mutex<System>>,
}

impl ProcessSession {
    pub fn new(pid: Option<Pid>, system: Arc<Mutex<System>>) -> Self {
        if let Some(pid) = pid {
            let system_guard = system.lock().unwrap();
            if let Some(process) = system_guard.process(pid) {
                Logger::instance().log(
                    squalr_engine_common::logging::log_level::LogLevel::Info,
                    &format!("Attached to process: {} ({})", process.name(), process.pid().as_u32()),
                    None,
                );
            }
        }

        let session = ProcessSession {
            opened_process: Arc::new(Mutex::new(pid)),
            system,
        };

        session.listen_for_process_death();
        session
    }

    pub fn get_opened_process(&self) -> Option<Pid> {
        *self.opened_process.lock().unwrap()
    }

    pub fn set_opened_process(&self, process_id: Option<Pid>) {
        let mut opened_process = self.opened_process.lock().unwrap();
        *opened_process = process_id;
    }

    fn listen_for_process_death(&self) {
        let opened_process = self.opened_process.clone();
        let system = self.system.clone();
        thread::spawn(move || loop {
            {
                let mut process_guard = opened_process.lock().unwrap();
                let system_guard = system.lock().unwrap();
                if let Some(process_id) = *process_guard {
                    if system_guard.process(process_id).is_none() {
                        *process_guard = None;
                    }
                }
            }
            thread::sleep(Duration::from_millis(50));
        });
    }
}
