use std::sync::{Arc, RwLock, Once};
use squalr_engine_processes::process_info::ProcessInfo;
use squalr_engine_scanning::snapshots::{snapshot::Snapshot, snapshot_region::SnapshotRegion};
use squalr_engine_memory::memory_queryer::memory_queryer::MemoryQueryer;
use squalr_engine_memory::memory_queryer::memory_queryer::PageRetrievalMode;

pub struct SessionManager {
    opened_process: Option<ProcessInfo>,
    snapshot: Option<Arc<RwLock<Snapshot>>>,
}

impl SessionManager {
    fn new() -> Self {
        SessionManager {
            opened_process: None,
            snapshot: None,
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

    pub fn set_opened_process(
        &mut self,
        process_info: ProcessInfo
    ) {
        self.opened_process = Some(process_info);
    }

    pub fn clear_opened_process(
        &mut self
    ) {
        self.opened_process = None;
    }

    pub fn get_opened_process(
        &self
    ) -> Option<&ProcessInfo> {
        self.opened_process.as_ref()
    }

    pub fn get_or_create_snapshot(
        &mut self,
        process_info: &ProcessInfo
    ) -> Arc<RwLock<Snapshot>> {
        if let Some(snapshot) = &self.snapshot {
            return snapshot.clone();
        }

        let snapshot = Arc::new(RwLock::new(Snapshot::new(self.get_memory_pages_for_scan(process_info))));
        self.snapshot = Some(snapshot.clone());

        return snapshot;
    }

    pub fn get_memory_pages_for_scan(
        &mut self,
        process_info: &ProcessInfo
    ) -> Vec<SnapshotRegion> {
        let memory_pages = MemoryQueryer::get_memory_page_bounds(process_info, PageRetrievalMode::FROM_SETTINGS);
        let snapshot_regions = memory_pages.into_iter().map(|region| SnapshotRegion::new(region)).collect();

        return snapshot_regions;
    }
}
