use crate::{filters::snapshot_region_filter::SnapshotRegionFilter, scanners::comparers::scalar::scanner_scalar_iterative_chunked::ScannerScalarIterativeChunked};
use crate::scanners::comparers::scalar::scanner_scalar_single_element::ScannerScalarSingleElement;
use crate::scanners::comparers::snapshot_scanner::Scanner;
use crate::scanners::parameters::scan_parameters::ScanParameters;
use crate::scanners::parameters::scan_filter_parameters::ScanFilterParameters;
use crate::snapshots::snapshot_region::SnapshotRegion;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use squalr_engine_architecture::vectors::vectors;
use squalr_engine_common::values::data_type::DataType;
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
        scan_parameters: &ScanParameters,
        scan_filter_parameters: &ScanFilterParameters,
    ) -> Vec<SnapshotRegionFilter> {
        let mut results = vec![];
    
        for snapshot_region_filter in snapshot_region_filters {
            let scanner_instance = self.acquire_scanner_instance(
                snapshot_region_filter,
                scan_filter_parameters
            );
    
            let result_sub_regions = scanner_instance.scan_region(
                snapshot_region,
                snapshot_region_filter,
                scan_parameters,
                scan_filter_parameters,
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
        scan_parameters: &ScanParameters,
        scan_filter_parameters: &ScanFilterParameters,
    ) -> Vec<SnapshotRegionFilter> {
        snapshot_region_filters
            // Convert the iterator to a parallel iterator
            .par_iter()
            .flat_map(|snapshot_region_filter| {
                let scanner_instance = self.acquire_scanner_instance(
                    snapshot_region_filter,
                    scan_filter_parameters
                );
    
                return scanner_instance.scan_region(
                    snapshot_region,
                    snapshot_region_filter,
                    scan_parameters,
                    scan_filter_parameters,
                );
            })
            .collect()
    }

    fn acquire_scanner_instance(
        &self,
        snapshot_region_filter: &SnapshotRegionFilter,
        scan_filter_parameters: &ScanFilterParameters,
    ) -> &dyn Scanner {
        let data_type = scan_filter_parameters.get_data_type();
        let data_type_size = data_type.size_in_bytes();
        let memory_alignment = scan_filter_parameters.get_memory_alignment_or_default(data_type);

        if snapshot_region_filter.get_element_count(memory_alignment, data_type_size) == 1 {
            // Single element scanner
            return ScannerScalarSingleElement::get_instance();
        } else if vectors::has_vector_support() && snapshot_region_filter.is_vector_friendly_size(memory_alignment) {
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
