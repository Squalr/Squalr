use squalr_engine_api::structures::snapshots::snapshot::Snapshot;
use std::sync::{Arc, RwLock};

/// Clears any manually deleted scan result indices after a command produces a fresh result set.
pub fn clear_deleted_scan_result_indices(snapshot: &Arc<RwLock<Snapshot>>) {
    match snapshot.write() {
        Ok(mut snapshot_guard) => {
            snapshot_guard.clear_deleted_scan_result_indices();
        }
        Err(error) => {
            log::error!("Failed to acquire write lock on snapshot to clear deleted scan result indices: {}", error);
        }
    }
}
