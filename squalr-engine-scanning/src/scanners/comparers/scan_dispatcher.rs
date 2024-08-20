use crate::{filters::snapshot_region_filter::SnapshotRegionFilter, scanners::comparers::scalar::scanner_scalar_iterative_chunked::ScannerScalarIterativeChunked};
use crate::scanners::comparers::scalar::scanner_scalar_single_element::ScannerScalarSingleElement;
use crate::scanners::comparers::snapshot_scanner::Scanner;
use crate::scanners::constraints::scan_constraint::ScanConstraint;
use crate::snapshots::snapshot_region::SnapshotRegion;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use squalr_engine_architecture::vectors::vectors;
use squalr_engine_common::dynamic_struct::data_type::DataType;
use std::sync::Once;

pub struct ScanDispatcher {
}

impl ScanDispatcher {
    fn new(
    ) -> Self {
        Self { }
    }
    
    pub fn get_instance(
    ) -> &'static ScanDispatcher {
        static mut INSTANCE: Option<ScanDispatcher> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = ScanDispatcher::new();
                INSTANCE = Some(instance);
            });

            return INSTANCE.as_ref().unwrap_unchecked();
        }
    }

    pub fn dispatch_scan(
        &self,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filters: &Vec<SnapshotRegionFilter>,
        constraint: &ScanConstraint,
        data_type: &DataType,
    ) -> Vec<SnapshotRegionFilter> {
        let mut results = vec![];
    
        for snapshot_region_filter in snapshot_region_filters {
            let scanner_instance = self.acquire_scanner_instance(
                &snapshot_region_filter,
                &constraint,
                data_type,
            );
    
            let result_sub_regions = scanner_instance.scan_region(
                &snapshot_region,
                &snapshot_region_filter,
                &constraint,
                data_type,
            );
            
            for result_sub_region in result_sub_regions {
                results.push(result_sub_region);
            }
        }
    
        return results;
    }

    pub fn dispatch_scan_parallel(
        &self,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filters: &Vec<SnapshotRegionFilter>,
        constraint: &ScanConstraint,
        data_type: &DataType,
    ) -> Vec<SnapshotRegionFilter> {
        snapshot_region_filters
            // Convert the iterator to a parallel iterator
            .par_iter()
            .flat_map(|snapshot_region_filter| {
                let scanner_instance = self.acquire_scanner_instance(snapshot_region_filter, &constraint, data_type);
    
                return scanner_instance.scan_region(
                    &snapshot_region,
                    &snapshot_region_filter,
                    &constraint,
                    data_type,
                );
            })
            .collect()
    }

    fn acquire_scanner_instance(
        &self,
        snapshot_region_filter: &SnapshotRegionFilter,
        constraint: &ScanConstraint,
        data_type: &DataType,
    ) -> &dyn Scanner {
        let alignment = constraint.get_alignment();
        let data_type_size = data_type.size_in_bytes();

        if snapshot_region_filter.get_element_count(alignment, data_type_size) == 1 {
            // Single element scanner
            return ScannerScalarSingleElement::get_instance();
        } else if vectors::has_vector_support() && snapshot_region_filter.is_vector_friendly_size(alignment) {
            match data_type {
                DataType::Bytes(_) => {
                    // Vector array of bytes scanner
                    // return ScannerVectorArrayOfBytes::get_instance();
                }
                _ => {
                    /*
                    if alignment_size == element_size as i32 {
                        // Fast vector scanner
                        // return ScannerVectorFast::get_instance();
                    } else if alignment_size > element_size as i32 {
                        // Sparse vector scanner
                        // return ScannerVectorSparse::get_instance();
                    } else {
                        // Staggered vector scanner
                        // return ScannerVectorStaggered::get_instance();
                    } */
                }
            }
        } else {
            // Iterative scanner
            return ScannerScalarIterativeChunked::get_instance();
        }

        return ScannerScalarIterativeChunked::get_instance();
    }
}
