use std::sync::{Arc, RwLock, Once};
use squalr_engine_processes::process_info::ProcessInfo;
use sysinfo::Pid;

pub struct SessionManager {
    opened_process: Option<ProcessInfo>,
}

impl SessionManager {
    fn new() -> Self {
        SessionManager {
            opened_process: None,
        }
    }
    
    pub fn instance() -> Arc<RwLock<SessionManager>> {
        static mut SINGLETON: Option<Arc<RwLock<SessionManager>>> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = Arc::new(RwLock::new(SessionManager::new()));
                SINGLETON = Some(instance);
            });

            SINGLETON.as_ref().unwrap().clone()
        }
    }

    pub fn set_opened_process(&mut self, pid: Pid, handle: u64) {
        self.opened_process = Some(ProcessInfo { pid, handle });
    }

    pub fn clear_opened_process(&mut self) {
        self.opened_process = None;
    }

    pub fn get_opened_process(&self) -> Option<&ProcessInfo> {
        self.opened_process.as_ref()
    }
}
