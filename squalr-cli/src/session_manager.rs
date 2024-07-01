use std::sync::{Arc, RwLock, Once};
use sysinfo::Pid;

pub struct SessionManager {
    opened_process: Option<Pid>,
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

    pub fn set_opened_process(&mut self, pid: Option<Pid>) {
        self.opened_process = pid;
    }

    pub fn get_opened_process(&self) -> Option<Pid> {
        self.opened_process
    }
}
