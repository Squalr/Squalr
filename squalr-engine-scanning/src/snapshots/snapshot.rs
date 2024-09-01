use std::sync::Arc;

use crate::results::scan_result::ScanResult;
use crate::results::snapshot_scan_results::SnapshotScanResults;
use crate::scanners::parameters::scan_filter_parameters::ScanFilterParameters;
use crate::snapshots::snapshot_region::SnapshotRegion;
use dashmap::DashMap;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::values::data_type::DataType;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use squalr_engine_memory::memory_queryer::memory_queryer::MemoryQueryer;
use squalr_engine_memory::memory_queryer::memory_queryer::PageRetrievalMode;
use squalr_engine_processes::process_info::ProcessInfo;

#[derive(Debug)]
pub struct Snapshot {
    snapshot_regions: Vec<SnapshotRegion>,
    scan_results_by_data_type: Arc<DashMap<DataType, SnapshotScanResults>>,
}

/// Represents a snapshot of memory in an external process that contains current and previous values of memory pages.
impl Snapshot {
    pub fn new(mut snapshot_regions: Vec<SnapshotRegion>) -> Self {
        // Remove empty regions and sort them ascending
        snapshot_regions.retain(|region| region.get_region_size() > 0);
        snapshot_regions.sort_by_key(|region| region.get_base_address());

        Self {
            snapshot_regions,
            scan_results_by_data_type: Arc::new(DashMap::new()),
        }
    }

    pub fn new_scan(
        &mut self,
        process_info: &ProcessInfo,
        scan_filter_parameters: Vec<ScanFilterParameters>,
    ) {
        self.create_initial_snapshot_regions(process_info);
        self.scan_results_by_data_type.clear();

        for scan_filter_parameter in scan_filter_parameters {
            self.scan_results_by_data_type.insert(
                scan_filter_parameter.get_data_type().clone(),
                SnapshotScanResults::new(
                    scan_filter_parameter.get_data_type().clone(),
                    scan_filter_parameter.get_memory_alignment_or_default(),
                ),
            );
        }

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

    pub fn get_scan_result(
        &self,
        index: u64,
        data_type: &DataType,
    ) -> Option<ScanResult> {
        if let Some(scan_results) = self.scan_results_by_data_type.get(data_type) {
            return scan_results.get_scan_result(index, &self.snapshot_regions);
        }
        return None;
    }

    pub fn get_memory_alignment_or_default_for_data_type(
        &self,
        data_type: &DataType,
    ) -> MemoryAlignment {
        if let Some(scan_results) = self.scan_results_by_data_type.get(data_type) {
            return scan_results.get_memory_alignment();
        }
        return MemoryAlignment::Alignment1;
    }

    pub fn get_scan_results_by_data_type(&self) -> &DashMap<DataType, SnapshotScanResults> {
        return &self.scan_results_by_data_type;
    }

    pub fn get_data_types_and_alignments(&self) -> Vec<(DataType, MemoryAlignment)> {
        let result: Vec<(DataType, MemoryAlignment)> = self
            .scan_results_by_data_type
            .iter()
            .map(|entry| {
                let data_type = entry.key().clone();
                let scan_result = entry.value();
                let alignment = scan_result.get_memory_alignment();
                (data_type, alignment)
            })
            .collect();

        result
    }

    pub fn build_scan_results(&mut self) {
        for mut scan_results in self.scan_results_by_data_type.iter_mut() {
            return scan_results
                .value_mut()
                .build_scan_results(&self.snapshot_regions);
        }
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
