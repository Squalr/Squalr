use crate::command_executors::snapshot_region_builder::merge_memory_regions_into_snapshot_regions;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::registries::symbols::symbol_registry::SymbolRegistry;
use squalr_engine_api::structures::{
    data_types::data_type_ref::DataTypeRef, memory::memory_alignment::MemoryAlignment, processes::opened_process_info::OpenedProcessInfo,
    snapshots::snapshot::Snapshot,
};
use squalr_engine_scanning::scan_settings_config::ScanSettingsConfig;
use squalr_engine_session::os::PageRetrievalMode;

pub fn ensure_snapshot_regions_for_scan(
    engine_privileged_state: &EnginePrivilegedState,
    opened_process_info: &OpenedProcessInfo,
    snapshot: &mut Snapshot,
) {
    if snapshot.get_region_count() > 0 {
        return;
    }

    let memory_pages = engine_privileged_state
        .get_os_providers()
        .memory_query
        .get_memory_page_bounds(opened_process_info, effective_page_retrieval_mode());
    let merged_snapshot_regions = merge_memory_regions_into_snapshot_regions(memory_pages);

    if !merged_snapshot_regions.is_empty() {
        snapshot.set_snapshot_regions(merged_snapshot_regions);
    }
}

fn effective_page_retrieval_mode() -> PageRetrievalMode {
    match ScanSettingsConfig::get_page_retrieval_mode() {
        PageRetrievalMode::FromSettings => PageRetrievalMode::FromSettings,
        PageRetrievalMode::FromUserMode => PageRetrievalMode::FromUserMode,
        PageRetrievalMode::FromNonModules => PageRetrievalMode::FromNonModules,
        PageRetrievalMode::FromModules => PageRetrievalMode::FromModules,
        PageRetrievalMode::FromVirtualModules => PageRetrievalMode::FromVirtualModules,
    }
}

pub fn initialize_snapshot_scan_results_if_empty(
    snapshot: &mut Snapshot,
    symbol_registry: &SymbolRegistry,
    data_type_refs: &[DataTypeRef],
    memory_alignment: MemoryAlignment,
) -> bool {
    if data_type_refs.is_empty() {
        return false;
    }

    let mut did_initialize_scan_results = false;

    for snapshot_region in snapshot.get_snapshot_regions_mut() {
        let had_scan_results = !snapshot_region
            .get_scan_results()
            .get_filter_collections()
            .is_empty();
        snapshot_region.initialize_scan_results(symbol_registry, data_type_refs.iter(), memory_alignment);
        let has_scan_results = !snapshot_region
            .get_scan_results()
            .get_filter_collections()
            .is_empty();

        did_initialize_scan_results |= !had_scan_results && has_scan_results;
    }

    if did_initialize_scan_results {
        snapshot.clear_deleted_scan_result_indices();
    }

    did_initialize_scan_results
}

#[cfg(test)]
mod tests {
    use super::initialize_snapshot_scan_results_if_empty;
    use squalr_engine_api::structures::{
        data_types::data_type_ref::DataTypeRef,
        memory::{memory_alignment::MemoryAlignment, normalized_region::NormalizedRegion},
        results::snapshot_region_scan_results::SnapshotRegionScanResults,
        scanning::filters::{snapshot_region_filter::SnapshotRegionFilter, snapshot_region_filter_collection::SnapshotRegionFilterCollection},
        snapshots::{snapshot::Snapshot, snapshot_region::SnapshotRegion},
    };

    fn create_snapshot_with_region(region_size: u64) -> Snapshot {
        let mut snapshot = Snapshot::new();
        let snapshot_region = SnapshotRegion::new(NormalizedRegion::new(0x1000, region_size), Vec::new());

        snapshot.set_snapshot_regions(vec![snapshot_region]);

        snapshot
    }

    #[test]
    fn initialize_snapshot_scan_results_if_empty_creates_full_region_filters() {
        let mut snapshot = create_snapshot_with_region(0x10);
        let symbol_registry = squalr_engine_api::registries::symbols::symbol_registry::SymbolRegistry::new();
        let did_initialize_scan_results =
            initialize_snapshot_scan_results_if_empty(&mut snapshot, &symbol_registry, &[DataTypeRef::new("u32")], MemoryAlignment::Alignment4);

        assert!(did_initialize_scan_results);
        assert_eq!(snapshot.get_number_of_results(), 4);
    }

    #[test]
    fn initialize_snapshot_scan_results_if_empty_is_noop_without_data_types() {
        let mut snapshot = create_snapshot_with_region(0x10);
        let symbol_registry = squalr_engine_api::registries::symbols::symbol_registry::SymbolRegistry::new();
        let did_initialize_scan_results = initialize_snapshot_scan_results_if_empty(&mut snapshot, &symbol_registry, &[], MemoryAlignment::Alignment4);

        assert!(!did_initialize_scan_results);
        assert_eq!(snapshot.get_number_of_results(), 0);
    }

    #[test]
    fn initialize_snapshot_scan_results_if_empty_preserves_existing_results() {
        let mut snapshot = create_snapshot_with_region(0x10);
        let symbol_registry = squalr_engine_api::registries::symbols::symbol_registry::SymbolRegistry::new();
        let existing_scan_results = SnapshotRegionScanResults::new(vec![SnapshotRegionFilterCollection::new(
            &symbol_registry,
            vec![vec![SnapshotRegionFilter::new(0x1000, 2)]],
            DataTypeRef::new("u8"),
            MemoryAlignment::Alignment1,
        )]);
        snapshot
            .get_snapshot_regions_mut()
            .first_mut()
            .expect("Expected a snapshot region for the test.")
            .set_scan_results(existing_scan_results);

        let did_initialize_scan_results =
            initialize_snapshot_scan_results_if_empty(&mut snapshot, &symbol_registry, &[DataTypeRef::new("u32")], MemoryAlignment::Alignment4);
        let scan_result = snapshot
            .get_scan_result(&symbol_registry, 0)
            .expect("Expected the existing scan result to remain.");

        assert!(!did_initialize_scan_results);
        assert_eq!(snapshot.get_number_of_results(), 2);
        assert_eq!(scan_result.get_data_type_ref(), &DataTypeRef::new("u8"));
    }
}
