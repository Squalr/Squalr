use std::sync::{Arc, Mutex};
use sysinfo::Pid;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref SESSION_MANAGER: Arc<Mutex<SessionManager>> = Arc::new(Mutex::new(SessionManager::new()));
}

pub struct SessionManager {
    opened_process: Option<Pid>,
}

impl SessionManager {
    pub fn new() -> Self {
        SessionManager {
            opened_process: None,
        }
    }

    pub fn set_opened_process(&mut self, pid: Option<Pid>) {
        self.opened_process = pid;
    }

    pub fn get_opened_process(&self) -> Option<Pid> {
        self.opened_process
    }
}
