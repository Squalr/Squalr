use crate::os::{ProcessManager, engine_os_provider::EngineOsProviders};
use crate::settings::scan_settings_store::ScanSettingsStore;
use squalr_engine_api::registries::freeze_list::freeze_list_registry::FreezeListRegistry;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::utils::periodic_worker::PeriodicWorker;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;

pub struct SnapshotScanResultFreezeTask;

/// Implementation of a task that freezes all scan results selected by the user.
impl SnapshotScanResultFreezeTask {
    pub fn start_task(
        process_manager: Arc<ProcessManager>,
        freeze_list_registry: Arc<RwLock<FreezeListRegistry>>,
        os_providers: EngineOsProviders,
    ) -> PeriodicWorker {
        PeriodicWorker::start(move || {
            process_manager.with_opened_process(|target| Self::freeze_values(target, &freeze_list_registry, &os_providers));
            Duration::from_millis(ScanSettingsStore::get_results_read_interval_ms())
        })
    }

    fn freeze_values(
        process_info: &OpenedProcessInfo,
        freeze_list_registry: &Arc<RwLock<FreezeListRegistry>>,
        os_providers: &EngineOsProviders,
    ) {
        let freeze_list_registry_guard = match freeze_list_registry.read() {
            Ok(freeze_list_registry_guard) => freeze_list_registry_guard,
            Err(error) => {
                log::error!("Failed to acquire read lock on FreezeListRegistry: {}", error);

                return;
            }
        };

        if freeze_list_registry_guard.get_frozen_pointers().is_empty() {
            return;
        }

        let modules = os_providers.memory_query.get_modules(process_info);

        for pointer in freeze_list_registry_guard.get_frozen_pointers().keys() {
            if let Some(value_bytes) = freeze_list_registry_guard.get_address_frozen_bytes(pointer) {
                let resolved_address = pointer.resolve_final_address(
                    |module_name, module_offset| {
                        os_providers
                            .memory_query
                            .resolve_module_address(&modules, module_name, module_offset)
                    },
                    |address, pointer_size| {
                        let mut pointer_bytes = vec![0_u8; pointer_size.get_size_in_bytes() as usize];

                        if !os_providers
                            .memory_read
                            .read_bytes(process_info, address, &mut pointer_bytes)
                        {
                            return None;
                        }

                        pointer_size.read_address_value(&squalr_engine_api::structures::data_values::data_value::DataValue::new(
                            pointer_size.to_data_type_ref(),
                            pointer_bytes,
                        ))
                    },
                );

                if let Some(resolved_address) = resolved_address {
                    let _success = os_providers
                        .memory_write
                        .write_bytes(process_info, resolved_address, value_bytes);
                }
            }
        }
    }
}
