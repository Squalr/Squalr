use crate::snapshots::snapshot::Snapshot;
use crate::snapshots::snapshot_queryer::{SnapshotQueryer, SnapshotRetrievalMode};
use sysinfo::Pid;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

const SIZE_LIMIT: u64 = 1 << 28; // 256MB

pub struct SnapshotManager {
    snapshots: Arc<Mutex<VecDeque<Snapshot>>>,
    deleted_snapshots: Arc<Mutex<Vec<Snapshot>>>,
    on_snapshots_updated: Option<Box<dyn Fn(&SnapshotManager)>>,
    on_new_snapshot: Option<Box<dyn Fn(&SnapshotManager)>>,
}

impl SnapshotManager {
    pub fn new() -> Self {
        Self {
            snapshots: Arc::new(Mutex::new(VecDeque::new())),
            deleted_snapshots: Arc::new(Mutex::new(Vec::new())),
            on_snapshots_updated: None,
            on_new_snapshot: None,
        }
    }

    pub fn get_active_snapshot_create_if_none(
        &self,
        process_id: &Pid,
    ) -> Option<Snapshot> {
        let mut snapshots = self.snapshots.lock().unwrap();

        if snapshots.is_empty() || snapshots.front().unwrap().element_count == 0 {
            let snapshot = SnapshotQueryer::get_snapshot(process_id, SnapshotRetrievalMode::FROM_SETTINGS);
            snapshots.push_front(snapshot.clone());
            return Some(snapshot);
        }

        Some(snapshots.front().unwrap().clone())
    }

    pub fn get_active_snapshot(&self) -> Option<Snapshot> {
        let snapshots = self.snapshots.lock().unwrap();

        if snapshots.is_empty() || snapshots.front().unwrap().element_count == 0 {
            return None;
        }

        Some(snapshots.front().unwrap().clone())
    }

    pub fn redo_snapshot(&self) {
        let mut snapshots = self.snapshots.lock().unwrap();
        let mut deleted_snapshots = self.deleted_snapshots.lock().unwrap();

        if let Some(snapshot) = deleted_snapshots.pop() {
            snapshots.push_front(snapshot);
            if let Some(callback) = &self.on_snapshots_updated {
                callback(self);
            }
        }
    }

    pub fn undo_snapshot(&self) {
        let mut snapshots = self.snapshots.lock().unwrap();
        let mut deleted_snapshots = self.deleted_snapshots.lock().unwrap();

        if let Some(snapshot) = snapshots.pop_front() {
            deleted_snapshots.push(snapshot);
            if let Some(callback) = &self.on_snapshots_updated {
                callback(self);
            }
        }
    }

    pub fn clear_snapshots(&self) {
        let mut snapshots = self.snapshots.lock().unwrap();
        let mut deleted_snapshots = self.deleted_snapshots.lock().unwrap();

        snapshots.clear();
        deleted_snapshots.clear();

        if let Some(callback) = &self.on_snapshots_updated {
            callback(self);
        }

        drop(snapshots);
        drop(deleted_snapshots);
    }

    pub fn save_snapshot(&self, snapshot: Snapshot) {
        let mut snapshots = self.snapshots.lock().unwrap();
        let mut deleted_snapshots = self.deleted_snapshots.lock().unwrap();

        if snapshots.front().map_or(false, |s| s.byte_count > SIZE_LIMIT) {
            snapshots.pop_front();
        }

        snapshots.push_front(snapshot);

        deleted_snapshots.clear();

        if let Some(callback) = &self.on_snapshots_updated {
            callback(self);
        }

        if let Some(callback) = &self.on_new_snapshot {
            callback(self);
        }
    }

    pub fn set_on_snapshots_updated<F>(&mut self, callback: F)
    where
        F: 'static + Fn(&SnapshotManager),
    {
        self.on_snapshots_updated = Some(Box::new(callback));
    }

    pub fn set_on_new_snapshot<F>(&mut self, callback: F)
    where
        F: 'static + Fn(&SnapshotManager),
    {
        self.on_new_snapshot = Some(Box::new(callback));
    }
}
