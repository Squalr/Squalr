use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::filters::snapshot_region_filter_collection::SnapshotRegionFilterCollection;
use crate::scanners::parameters::scan_parameters::ScanParameters;
use crate::scanners::scalar::scanner_scalar_iterative::ScannerScalarIterative;
use crate::scanners::snapshot_scanner::Scanner;
use crate::snapshots::snapshot_region::SnapshotRegion;
use rayon::iter::ParallelIterator;
use squalr_engine_common::structures::data_types::data_type::DataType;
use squalr_engine_common::structures::memory_alignment::MemoryAlignment;

pub struct ScanDispatcher {}

/// Implements a scan dispatcher, which picks the best scanner based on the scan constraints and the region being scanned.
/// Choosing the best scanner is critical to maintaining high performance scans.
impl ScanDispatcher {
    /// Performs a scan over a provided filter collection, returning a new filter collection with the results.
    pub fn dispatch_scan(
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter_collection: &SnapshotRegionFilterCollection,
        scan_parameters: &ScanParameters,
    ) -> SnapshotRegionFilterCollection {
        let data_type = snapshot_region_filter_collection.get_data_type();
        let memory_alignment = snapshot_region_filter_collection.get_memory_alignment();

        let result_snapshot_region_filters = snapshot_region_filter_collection
            .iter()
            .filter_map(|snapshot_region_filter| {
                let scanner_instance = Self::acquire_scanner_instance(snapshot_region_filter, data_type, memory_alignment);
                let filters = scanner_instance.scan_region(snapshot_region, snapshot_region_filter, scan_parameters, data_type, memory_alignment);

                if filters.len() > 0 { Some(filters) } else { None }
            })
            .collect();

        SnapshotRegionFilterCollection::new(result_snapshot_region_filters, data_type.clone(), memory_alignment)
    }

    /// Performs a parallelized scan over a provided filter collection, returning a new filter collection with the results.
    pub fn dispatch_scan_parallel(
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter_collection: &SnapshotRegionFilterCollection,
        scan_parameters: &ScanParameters,
    ) -> SnapshotRegionFilterCollection {
        let data_type = snapshot_region_filter_collection.get_data_type();
        let memory_alignment = snapshot_region_filter_collection.get_memory_alignment();

        let result_snapshot_region_filters = snapshot_region_filter_collection
            .par_iter()
            .filter_map(|snapshot_region_filter| {
                let scanner_instance = Self::acquire_scanner_instance(snapshot_region_filter, data_type, memory_alignment);
                let filters = scanner_instance.scan_region(snapshot_region, snapshot_region_filter, scan_parameters, data_type, memory_alignment);

                if filters.len() > 0 { Some(filters) } else { None }
            })
            .collect();

        SnapshotRegionFilterCollection::new(result_snapshot_region_filters, data_type.clone(), memory_alignment)
    }

    fn acquire_scanner_instance(
        snapshot_region_filter: &SnapshotRegionFilter,
        data_type: &Box<dyn DataType>,
        memory_alignment: MemoryAlignment,
    ) -> &'static dyn Scanner {
        /*
        let data_type_size = data_type.get_size_in_bytes();
        let memory_alignment_size = memory_alignment as u64;
        let element_count = snapshot_region_filter.get_element_count(data_type_size, memory_alignment);
        let region_size = snapshot_region_filter.get_region_size();

        if element_count == 1 {
            // Single element scanner
            return ScannerScalarSingleElement::get_instance();
        }

        // Use parallel scanners when the region size is >= 64MB
        if region_size >= 1024 * 1024 * 64 {
            match data_type {
                DataType::Bytes(_) => {
                    return ScannerScalarIterativeByteArray::get_instance();
                }
                // Check if a vector (SIMD) scan can be applied
                _ => {
                    // We actually don't really care whether the processor supports AVX-512, AVX2, etc, PortableSimd is smart enough to
                    // abstract this. It is actually more performant to greedily try to use AVX-512 even if its not available. PortableSimd
                    // seems to fall back to unrolled AVX2 or SSE2 code, and it ends up being faster than the AVX2/SSE-first implementations.
                    if region_size >= 64 {
                        if memory_alignment_size < data_type_size {
                            return &self.scanner_cascading_chunked_64;
                        } else if memory_alignment_size == data_type_size {
                            return &self.scanner_aligned_chunked_64;
                        } else if memory_alignment_size > data_type_size {
                            return &self.scanner_sparse_64;
                        }
                    } else if region_size >= 32 {
                        if memory_alignment_size < data_type_size {
                            return &self.scanner_cascading_chunked_32;
                        } else if memory_alignment_size == data_type_size {
                            return &self.scanner_aligned_chunked_32;
                        } else if memory_alignment_size > data_type_size {
                            return &self.scanner_sparse_32;
                        }
                    } else if region_size >= 16 {
                        if memory_alignment_size < data_type_size {
                            return &self.scanner_cascading_chunked_16;
                        } else if memory_alignment_size == data_type_size {
                            return &self.scanner_aligned_chunked_16;
                        } else if memory_alignment_size > data_type_size {
                            return &self.scanner_sparse_16;
                        }
                    }
                }
            }
        }

        // Prioritize vector scans for small to large regions.
        match data_type {
            DataType::Bytes(_) => {
                return ScannerScalarIterativeByteArray::get_instance();
            }
            // Check if a vector (SIMD) scan can be applied
            _ => {
                // We actually don't really care whether the processor supports AVX-512, AVX2, etc, PortableSimd is smart enough to
                // abstract this. It is actually more performant to greedily try to use AVX-512 even if its not available. PortableSimd
                // seems to fall back to unrolled AVX2 or SSE2 code, and it ends up being faster than the AVX2/SSE-first implementations.
                if region_size >= 64 {
                    if memory_alignment_size < data_type_size {
                        return &self.scanner_cascading_64;
                    } else if memory_alignment_size == data_type_size {
                        return &self.scanner_aligned_64;
                    } else if memory_alignment_size > data_type_size {
                        return &self.scanner_sparse_64;
                    }
                } else if region_size >= 32 {
                    if memory_alignment_size < data_type_size {
                        return &self.scanner_cascading_32;
                    } else if memory_alignment_size == data_type_size {
                        return &self.scanner_aligned_32;
                    } else if memory_alignment_size > data_type_size {
                        return &self.scanner_sparse_32;
                    }
                } else if region_size >= 16 {
                    if memory_alignment_size < data_type_size {
                        return &self.scanner_cascading_16;
                    } else if memory_alignment_size == data_type_size {
                        return &self.scanner_aligned_16;
                    } else if memory_alignment_size > data_type_size {
                        return &self.scanner_sparse_16;
                    }
                }
            }
        } */

        // Default to scalar iterative
        &ScannerScalarIterative {}
    }
}
