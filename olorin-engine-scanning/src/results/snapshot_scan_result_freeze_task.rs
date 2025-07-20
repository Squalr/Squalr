use crate::scan_settings_config::ScanSettingsConfig;
use olorin_engine_api::registries::freeze_list::freeze_list_registry::FreezeListRegistry;
use olorin_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use olorin_engine_api::structures::tasks::trackable_task::TrackableTask;
use olorin_engine_memory::memory_writer::MemoryWriter;
use olorin_engine_memory::memory_writer::memory_writer_trait::IMemoryWriter;
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
        freeze_list_registry: Arc<RwLock<FreezeListRegistry>>,
    ) -> Arc<TrackableTask> {
        let task = TrackableTask::create(TASK_NAME.to_string(), None);
        let task_clone = task.clone();

        std::thread::spawn(move || {
            loop {
                if task_clone.get_cancellation_token().load(Ordering::Acquire) {
                    break;
                }
                Self::collect_values_task(&process_info, &freeze_list_registry);

                thread::sleep(Duration::from_millis(ScanSettingsConfig::get_results_read_interval()));
            }

            task_clone.complete();
        });

        task
    }

    fn collect_values_task(
        process_info: &Arc<RwLock<Option<OpenedProcessInfo>>>,
        freeze_list_registry: &Arc<RwLock<FreezeListRegistry>>,
    ) {
        let process_info_lock = match process_info.read() {
            Ok(guard) => guard,
            Err(error) => {
                log::error!("Failed to acquire read lock on process info for result freezing: {}", error);

                return;
            }
        };

        let process_info = match process_info_lock.as_ref() {
            Some(process_info) => process_info,
            None => return,
        };

        let freeze_list_registry_guard = match freeze_list_registry.write() {
            Ok(guard) => guard,
            Err(error) => {
                log::error!("Failed to acquire write lock on FreezeListRegistry: {}", error);

                return;
            }
        };

        for address in freeze_list_registry_guard.get_frozen_indicies().keys() {
            if let Some(data_value) = freeze_list_registry_guard.get_address_frozen_data_value(*address) {
                let value_bytes = data_value.get_value_bytes();
                let _success = MemoryWriter::get_instance().write_bytes(process_info, *address, &value_bytes);
            }
        }
    }
}
