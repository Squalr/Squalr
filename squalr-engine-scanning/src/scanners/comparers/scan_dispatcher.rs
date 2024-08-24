use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::comparers::scalar::scanner_scalar_iterative::ScannerScalarIterative;
use crate::scanners::comparers::scalar::scanner_scalar_single_element::ScannerScalarSingleElement;
use crate::scanners::comparers::snapshot_scanner::Scanner;
use crate::scanners::comparers::vector::scanner_vector_aligned::ScannerVectorAligned;
use crate::scanners::comparers::vector::scanner_vector_sparse::ScannerVectorSparse;
use crate::scanners::comparers::vector::scanner_vector_staggered::ScannerVectorStaggered;
use crate::scanners::parameters::scan_parameters::ScanParameters;
use crate::scanners::parameters::scan_filter_parameters::ScanFilterParameters;
use crate::snapshots::snapshot_region::SnapshotRegion;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
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
        let memory_alignment = scan_filter_parameters.get_memory_alignment_or_default();
        let element_count = snapshot_region_filter.get_element_count(memory_alignment, data_type_size);
        let memory_alignment = memory_alignment as u64;

        if element_count == 1 {
            // Single element scanner
            return ScannerScalarSingleElement::get_instance();
        } else {
            match data_type {
                DataType::Bytes(_) => {
                    // Vector array of bytes scanner
                    // return ScannerVectorArrayOfBytes::get_instance();
                }
                _ => {
                    // We actually don't really care whether the processor supports AVX-512, AVX2, etc, Rust is smart enough to abstract this.
                    // It is actually more performant to greedily try to use AVX-512 even if its not available, because Rust generates
                    // Essentially unrolled loops of AVX2 or SSE2 code, and it ends up being faster than the AVX2/SSE-first implementations.
                    if element_count >= 64 {
                        if memory_alignment == data_type_size {
                            return ScannerVectorAligned::<512>::get_instance();
                        } else if memory_alignment > data_type_size {
                            return ScannerVectorSparse::<512>::get_instance();
                        } else {
                            // return ScannerVectorStaggered::<512>::get_instance();
                        }
                    }
                    else if element_count >= 32 {
                        if memory_alignment == data_type_size {
                            return ScannerVectorAligned::<256>::get_instance();
                        } else if memory_alignment > data_type_size {
                            return ScannerVectorSparse::<256>::get_instance();
                        } else {
                            // return ScannerVectorStaggered::<256>::get_instance();
                        }
                    }
                    else if element_count >= 16 {
                        if memory_alignment == data_type_size {
                            return ScannerVectorAligned::<128>::get_instance();
                        } else if memory_alignment > data_type_size {
                            return ScannerVectorSparse::<128>::get_instance();
                        } else {
                            // return ScannerVectorStaggered::<128>::get_instance();
                        }
                    }
                }
            }
        }

        // Default to scalar iterative
        return ScannerScalarIterative::get_instance();
    }
}
