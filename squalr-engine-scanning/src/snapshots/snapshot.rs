use crate::results::scan_result_lookup_table::ScanResultLookupTable;
use crate::scanners::parameters::scan_filter_parameters::ScanFilterParameters;
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
    pub fn new(mut snapshot_regions: Vec<SnapshotRegion>) -> Self {
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
        scan_filter_parameters: Vec<ScanFilterParameters>,
    ) {
        self.scan_result_lookup_table
            .set_scan_filter_parameters(scan_filter_parameters);
        self.create_initial_snapshot_regions(process_info);
        Logger::get_instance().log(LogLevel::Info, "New scan created.", None);
    }

    pub fn get_snapshot_regions(&self) -> &Vec<SnapshotRegion> {
        return &self.snapshot_regions;
    }

    pub fn get_snapshot_regions_for_update(&mut self) -> &mut Vec<SnapshotRegion> {
        return &mut self.snapshot_regions;
    }

    pub fn discard_empty_regions(&mut self) {
        self.snapshot_regions
            .retain(|region| region.get_region_size() > 0);
    }

    pub fn get_region_count(&self) -> u64 {
        return self.snapshot_regions.len() as u64;
    }

    pub fn get_byte_count(&self) -> u64 {
        return self
            .snapshot_regions
            .iter()
            .map(|region| region.get_region_size())
            .sum();
    }

    pub fn get_scan_parameters_filters(&self) -> &Vec<ScanFilterParameters> {
        return self.scan_result_lookup_table.get_scan_parameters_filters();
    }

    pub fn take_scan_parameters_filters(&mut self) -> Vec<ScanFilterParameters> {
        return take(&mut self.scan_result_lookup_table.take_scan_parameters_filters());
    }

    pub fn create_initial_snapshot_regions(
        &mut self,
        process_info: &ProcessInfo,
    ) {
        // Query all memory pages for the process from the OS
        let memory_pages = MemoryQueryer::get_memory_page_bounds(process_info, PageRetrievalMode::FROM_SETTINGS);

        if memory_pages.is_empty() {
            self.snapshot_regions.clear();
            return;
        }

        // Attempt to merge any adjacent regions, tracking the page boundaries at which the merge took place.
        // This is done since we want to track these such that the SnapshotRegions can avoid reading process memory across
        // page boundaries, as this can be problematic (ie if one of the pages deallocates later, we want to be able to recover).
        let mut merged_snapshot_regions = Vec::new();
        let mut iter = memory_pages.into_iter();
        let mut current_region = iter.next().unwrap();
        let mut page_boundaries = Vec::new();

        loop {
            let Some(region) = iter.next() else {
                break;
            };

            if current_region.get_end_address() == region.get_base_address() {
                current_region.set_end_address(region.get_end_address());
                page_boundaries.push(region.get_base_address());
            } else {
                merged_snapshot_regions.push(SnapshotRegion::new(current_region, std::mem::take(&mut page_boundaries)));
                current_region = region;
            }
        }

        // Push the last region
        merged_snapshot_regions.push(SnapshotRegion::new(current_region, page_boundaries));

        self.snapshot_regions = merged_snapshot_regions;
    }
}
