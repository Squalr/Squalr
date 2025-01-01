use squalr_engine_processes::process_info::ProcessInfo;
use squalr_engine_scanning::snapshots::snapshot::Snapshot;
use std::sync::{Arc, Once, RwLock};

pub struct SessionManager {
    opened_process: Option<ProcessInfo>,
    snapshot: Arc<RwLock<Snapshot>>,
}

impl SessionManager {
    fn new() -> Self {
        SessionManager {
            opened_process: None,
            snapshot: Arc::new(RwLock::new(Snapshot::new(vec![]))),
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

            #[allow(static_mut_refs)]
            return INSTANCE.as_ref().unwrap_unchecked().clone();
        }
    }

    pub fn set_opened_process(
        &mut self,
        process_info: ProcessInfo,
    ) {
        self.opened_process = Some(process_info);
    }

    pub fn clear_opened_process(&mut self) {
        self.opened_process = None;
    }

    pub fn get_opened_process(&self) -> Option<&ProcessInfo> {
        self.opened_process.as_ref()
    }

    pub fn get_snapshot(&self) -> Arc<RwLock<Snapshot>> {
        return self.snapshot.clone();
    }
}
