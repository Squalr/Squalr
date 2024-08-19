use std::sync::{Arc, RwLock, Once};
use squalr_engine_processes::process_info::ProcessInfo;
use squalr_engine_scanning::results::scan_results::ScanResults;

pub struct SessionManager {
    opened_process: Option<ProcessInfo>,
    scan_results: Arc<RwLock<ScanResults>>,
}

impl SessionManager {
    fn new() -> Self {
        SessionManager {
            opened_process: None,
            scan_results: Arc::new(RwLock::new(ScanResults::new())),
        }
    }
    
    pub fn get_instance() -> Arc<RwLock<SessionManager>> {
        static mut INSTANCE: Option<Arc<RwLock<SessionManager>>> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = Arc::new(RwLock::new(SessionManager::new()));
                INSTANCE = Some(instance);
            });

            return INSTANCE.as_ref().unwrap_unchecked().clone();
        }
    }

    pub fn get_scan_results(&self) -> Arc<RwLock<ScanResults>> {
        return self.scan_results.clone();
    }

    pub fn set_opened_process(&mut self, process_info: ProcessInfo) {
        self.opened_process = Some(process_info);
    }

    pub fn clear_opened_process(&mut self) {
        self.opened_process = None;
    }

    pub fn get_opened_process(&self) -> Option<&ProcessInfo> {
        self.opened_process.as_ref()
    }
}
