use crate::snapshots::snapshot::Snapshot;
use crate::snapshots::snapshot_queryer::{SnapshotQueryer, SnapshotRetrievalMode};

use std::sync::{Arc, RwLock, Mutex, Once};
use std::collections::VecDeque;
use sysinfo::Pid;

const SIZE_LIMIT: u64 = 1 << 28; // 256MB

pub struct SnapshotManager {
    current_snapshot: Option<Arc<RwLock<Snapshot>>>,
    historical_snapshots: Mutex<VecDeque<Arc<RwLock<Snapshot>>>>,
    on_snapshots_updated: Option<Box<dyn Fn(&SnapshotManager) + Send + Sync>>,
    on_new_snapshot: Option<Box<dyn Fn(&SnapshotManager) + Send + Sync>>,
}

impl SnapshotManager {
    fn new() -> Self {
        Self {
            current_snapshot: None,
            historical_snapshots: Mutex::new(VecDeque::new()),
            on_snapshots_updated: None,
            on_new_snapshot: None,
        }
    }

    pub fn instance() -> Arc<RwLock<SnapshotManager>> {
        static mut SINGLETON: Option<Arc<RwLock<SnapshotManager>>> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = Arc::new(RwLock::new(SnapshotManager::new()));
                SINGLETON = Some(instance);
            });

            return SINGLETON.as_ref().unwrap().clone();
        }
    }

    pub fn get_active_snapshot(&self) -> Option<Arc<RwLock<Snapshot>>> {
        return self.current_snapshot.clone();
    }

    pub fn get_active_snapshot_create_if_none(&mut self, process_id: &Pid) -> Arc<RwLock<Snapshot>> {
        if let Some(snapshot) = &self.current_snapshot {
            return snapshot.clone();
        }

        let snapshot = Arc::new(RwLock::new(SnapshotQueryer::get_snapshot(process_id, SnapshotRetrievalMode::FROM_SETTINGS)));
        self.current_snapshot = Some(snapshot.clone());

        if let Some(callback) = &self.on_new_snapshot {
            callback(self);
        }

        snapshot
    }

    pub fn save_snapshot(&mut self, snapshot: Snapshot) {
        let new_snapshot = Arc::new(RwLock::new(snapshot));
        if let Some(current) = self.current_snapshot.take() {
            let mut historical_snapshots = self.historical_snapshots.lock().unwrap();
            if current.read().unwrap().get_byte_count() > SIZE_LIMIT {
                historical_snapshots.pop_front();
            }
            historical_snapshots.push_front(current);
        }
        self.current_snapshot = Some(new_snapshot.clone());

        if let Some(callback) = &self.on_snapshots_updated {
            callback(self);
        }

        if let Some(callback) = &self.on_new_snapshot {
            callback(self);
        }
    }

    pub fn redo_snapshot(&mut self) {
        let mut historical_snapshots = self.historical_snapshots.lock().unwrap();
        if let Some(snapshot) = historical_snapshots.pop_back() {
            self.current_snapshot = Some(snapshot);

            if let Some(callback) = &self.on_snapshots_updated {
                callback(self);
            }
        }
    }

    pub fn undo_snapshot(&mut self) {
        if let Some(current) = self.current_snapshot.take() {
            let mut historical_snapshots = self.historical_snapshots.lock().unwrap();
            historical_snapshots.push_front(current);

            if let Some(snapshot) = historical_snapshots.pop_front() {
                self.current_snapshot = Some(snapshot);
            }

            if let Some(callback) = &self.on_snapshots_updated {
                callback(self);
            }
        }
    }

    pub fn clear_snapshots(&mut self) {
        let mut historical_snapshots = self.historical_snapshots.lock().unwrap();
        historical_snapshots.clear();
        self.current_snapshot = None;

        if let Some(callback) = &self.on_snapshots_updated {
            callback(self);
        }
    }

    pub fn set_on_snapshots_updated<F>(&mut self, callback: F) where F: 'static + Fn(&SnapshotManager) + Send + Sync,
    {
        self.on_snapshots_updated = Some(Box::new(callback));
    }

    pub fn set_on_new_snapshot<F>(&mut self, callback: F) where F: 'static + Fn(&SnapshotManager) + Send + Sync,
    {
        self.on_new_snapshot = Some(Box::new(callback));
    }
}
