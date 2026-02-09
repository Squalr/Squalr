use crate::os::engine_os_provider::EngineOsProviders;
use squalr_engine_api::registries::freeze_list::freeze_list_registry::FreezeListRegistry;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::tasks::trackable_task::TrackableTask;
use squalr_engine_scanning::scan_settings_config::ScanSettingsConfig;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

const TASK_NAME: &str = "Scan Result Freezer";

pub struct SnapshotScanResultFreezeTask;

/// Implementation of a task that freezes all scan results selected by the user.
impl SnapshotScanResultFreezeTask {
    pub fn start_task(
        process_info: Arc<RwLock<Option<OpenedProcessInfo>>>,
        freeze_list_registry: Arc<RwLock<FreezeListRegistry>>,
        os_providers: EngineOsProviders,
    ) -> Arc<TrackableTask> {
        let task = TrackableTask::create(TASK_NAME.to_string(), None);
        let task_clone = task.clone();

        thread::spawn(move || {
            loop {
                if task_clone.get_cancellation_token().load(Ordering::Acquire) {
                    break;
                }

                Self::freeze_values(&process_info, &freeze_list_registry, &os_providers);
                thread::sleep(Duration::from_millis(ScanSettingsConfig::get_results_read_interval_ms()));
            }

            task_clone.complete();
        });

        task
    }

    fn freeze_values(
        process_info: &Arc<RwLock<Option<OpenedProcessInfo>>>,
        freeze_list_registry: &Arc<RwLock<FreezeListRegistry>>,
        os_providers: &EngineOsProviders,
    ) {
        let process_info_guard = match process_info.read() {
            Ok(process_info_guard) => process_info_guard,
            Err(error) => {
                log::error!("Failed to acquire read lock on process info for result freezing: {}", error);

                return;
            }
        };

        let process_info = match process_info_guard.as_ref() {
            Some(process_info) => process_info,
            None => return,
        };

        let freeze_list_registry_guard = match freeze_list_registry.write() {
            Ok(freeze_list_registry_guard) => freeze_list_registry_guard,
            Err(error) => {
                log::error!("Failed to acquire write lock on FreezeListRegistry: {}", error);

                return;
            }
        };

        let modules = os_providers.memory_query.get_modules(process_info);

        for pointer in freeze_list_registry_guard.get_frozen_pointers().keys() {
            if let Some(value_bytes) = freeze_list_registry_guard.get_address_frozen_bytes(pointer) {
                let module_address = os_providers
                    .memory_query
                    .resolve_module(&modules, pointer.get_module_name());

                let _success = os_providers
                    .memory_write
                    .write_bytes(process_info, module_address.saturating_add(pointer.get_address()), value_bytes);
            }
        }
    }
}
