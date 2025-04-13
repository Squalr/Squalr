use crate::results::snapshot_scan_result_freeze_list::SnapshotScanResultFreezeList;
use crate::scan_settings_config::ScanSettingsConfig;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::tasks::trackable_task::TrackableTask;
use squalr_engine_memory::memory_writer::MemoryWriter;
use squalr_engine_memory::memory_writer::memory_writer_trait::IMemoryWriter;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

const TASK_NAME: &'static str = "Scan Result Freezer";

pub struct SnapshotScanResultFreezeTask;

/// Implementation of a task that freezes all scan results selected by the user.
impl SnapshotScanResultFreezeTask {
    pub fn start_task(
        process_info: Arc<RwLock<Option<OpenedProcessInfo>>>,
        snapshot_scan_result_freeze_list: Arc<RwLock<SnapshotScanResultFreezeList>>,
    ) -> Arc<TrackableTask> {
        let task = TrackableTask::create(TASK_NAME.to_string(), None);
        let task_clone = task.clone();

        std::thread::spawn(move || {
            loop {
                if task_clone.get_cancellation_token().load(Ordering::Acquire) {
                    break;
                }
                Self::collect_values_task(&process_info, &snapshot_scan_result_freeze_list);

                thread::sleep(Duration::from_millis(ScanSettingsConfig::get_results_read_interval()));
            }

            task_clone.complete();
        });

        task
    }

    fn collect_values_task(
        process_info: &Arc<RwLock<Option<OpenedProcessInfo>>>,
        snapshot_scan_result_freeze_list: &Arc<RwLock<SnapshotScanResultFreezeList>>,
    ) {
        let process_info_lock = match process_info.read() {
            Ok(guard) => guard,
            Err(err) => {
                log::error!("Failed to acquire read lock on process info for result freezing: {}", err);

                return;
            }
        };

        let process_info = match process_info_lock.as_ref() {
            Some(process_info) => process_info,
            None => return,
        };

        let snapshot_scan_result_freeze_list = match snapshot_scan_result_freeze_list.read() {
            Ok(guard) => guard,
            Err(err) => {
                log::error!("Failed to acquire write lock on snapshot for result freezing: {}", err);

                return;
            }
        };

        if let Ok(freeze_entries) = snapshot_scan_result_freeze_list.get_frozen_indicies().read() {
            for address in freeze_entries.keys() {
                if let Some(data_value) = snapshot_scan_result_freeze_list.get_address_frozen_data_value(*address) {
                    let value_bytes = data_value.get_value_bytes();
                    let _success = MemoryWriter::get_instance().write_bytes(process_info.get_handle(), *address, &value_bytes);
                }
            }
        }
    }
}
