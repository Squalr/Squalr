use crate::results::scan_result_lookup_table::ScanResultLookupTable;
use crate::scanners::constraints::scan_filter_constraint::ScanFilterConstraint;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_memory::memory_queryer::memory_queryer::MemoryQueryer;
use squalr_engine_memory::memory_queryer::memory_queryer::PageRetrievalMode;
use squalr_engine_processes::process_info::ProcessInfo;
use std::mem::take;

#[derive(Debug)]
pub struct Snapshot {
    snapshot_regions: Vec<SnapshotRegion>,
    scan_result_lookup_table: ScanResultLookupTable,
}

/// Represents a snapshot of memory in an external process that contains current and previous values of memory pages.
impl Snapshot {
    pub fn new(
        mut snapshot_regions: Vec<SnapshotRegion>
    ) -> Self {
        // Remove empty regions and sort them ascending
        snapshot_regions.retain(|region| region.get_region_size() > 0);
        snapshot_regions.sort_by_key(|region| region.get_base_address());

        Self {
            snapshot_regions,
            scan_result_lookup_table: ScanResultLookupTable::new(256),
        }
    }

    pub fn new_scan(
        &mut self,
        process_info: &ProcessInfo,
        scan_filter_constraints: Vec<ScanFilterConstraint>,
    ) {
        self.scan_result_lookup_table.set_scan_filter_constraints(scan_filter_constraints);
        self.create_initial_snapshot_regions(process_info);
        Logger::get_instance().log(LogLevel::Info, "New scan created.", None);
    }

    pub fn get_snapshot_regions(
        &self
    ) -> &Vec<SnapshotRegion> {
        return &self.snapshot_regions;
    }

    pub fn get_snapshot_regions_for_update(
        &mut self
    ) -> &mut Vec<SnapshotRegion> {
        return &mut self.snapshot_regions;
    }

    pub fn get_region_count(
        &self
    ) -> u64 {
        return self.snapshot_regions.len() as u64;
    }
    
    pub fn get_byte_count(
        &self
    ) -> u64 {
        return self.snapshot_regions.iter().map(|region| region.get_region_size()).sum();
    }

    pub fn get_scan_constraint_filters(
        &self,
    ) -> &Vec<ScanFilterConstraint> {
        return self.scan_result_lookup_table.get_scan_constraint_filters();
    }

    pub fn take_scan_constraint_filters(
        &mut self,
    ) -> Vec<ScanFilterConstraint> {
        return take(&mut self.scan_result_lookup_table.take_scan_constraint_filters());
    }

    pub fn update_scan_results(
        &mut self,
    ) {
        self.scan_result_lookup_table.build_scan_results(
            &self.snapshot_regions,
        );
    }

    pub fn create_initial_snapshot_regions(
        &mut self,
        process_info: &ProcessInfo,
    ) {
        let memory_pages = MemoryQueryer::get_memory_page_bounds(process_info, PageRetrievalMode::FROM_SETTINGS);
        let snapshot_regions = memory_pages.into_iter().map(|region| SnapshotRegion::new(region)).collect();

        self.snapshot_regions = snapshot_regions;
    }
}
