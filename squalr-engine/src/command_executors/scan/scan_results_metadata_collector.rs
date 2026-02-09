use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::structures::scan_results::scan_results_metadata::ScanResultsMetadata;

pub fn collect_scan_results_metadata(engine_privileged_state: &EnginePrivilegedState) -> ScanResultsMetadata {
    match engine_privileged_state.get_snapshot().read() {
        Ok(snapshot) => ScanResultsMetadata {
            result_count: snapshot.get_number_of_results(),
            total_size_in_bytes: snapshot.get_byte_count(),
        },
        Err(error) => {
            log::error!("Failed to acquire snapshot for scan metadata collection: {}", error);

            ScanResultsMetadata::default()
        }
    }
}
