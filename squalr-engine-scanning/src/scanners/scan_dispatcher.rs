use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::filters::snapshot_region_filter_collection::SnapshotRegionFilterCollection;
use crate::scanners::scalar::scanner_scalar_iterative::ScannerScalarIterative;
use crate::scanners::scalar::scanner_scalar_iterative_byte_array_cascading::ScannerScalarIterativeByteArrayCascading;
use crate::scanners::scalar::scanner_scalar_single_element::ScannerScalarSingleElement;
use crate::scanners::snapshot_scanner::Scanner;
use crate::scanners::vector::scanner_vector_aligned::ScannerVectorAligned;
use crate::scanners::vector::scanner_vector_cascading::ScannerVectorCascading;
use crate::scanners::vector::scanner_vector_sparse::ScannerVectorSparse;
use crate::snapshots::snapshot_region::SnapshotRegion;
use rayon::iter::ParallelIterator;
use squalr_engine_api::structures::data_types::built_in_types::byte_array::data_type_byte_array::DataTypeByteArray;
use squalr_engine_api::structures::scanning::scan_parameters_global::ScanParametersGlobal;
use squalr_engine_api::structures::scanning::scan_parameters_local::ScanParametersLocal;

pub struct ScanDispatcher {}

/// Implements a scan dispatcher, which picks the best scanner based on the scan constraints and the region being scanned.
/// Choosing the best scanner is critical to maintaining high performance scans.
impl ScanDispatcher {
    /// Performs a scan over a provided filter collection, returning a new filter collection with the results.
    pub fn dispatch_scan(
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter_collection: &SnapshotRegionFilterCollection,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> SnapshotRegionFilterCollection {
        let scan_parameters_local = snapshot_region_filter_collection.get_scan_parameters_local();

        let result_snapshot_region_filters = snapshot_region_filter_collection
            .iter()
            .filter_map(|snapshot_region_filter| {
                let scanner_instance = Self::acquire_scanner_instance(snapshot_region_filter, scan_parameters_local);
                let filters = scanner_instance.scan_region(
                    snapshot_region,
                    snapshot_region_filter,
                    scan_parameters_global,
                    snapshot_region_filter_collection.get_scan_parameters_local(),
                );

                if filters.len() > 0 { Some(filters) } else { None }
            })
            .collect();

        SnapshotRegionFilterCollection::new(
            result_snapshot_region_filters,
            snapshot_region_filter_collection
                .get_scan_parameters_local()
                .clone(),
        )
    }

    /// Performs a parallelized scan over a provided filter collection, returning a new filter collection with the results.
    pub fn dispatch_scan_parallel(
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter_collection: &SnapshotRegionFilterCollection,
        scan_parameters_global: &ScanParametersGlobal,
    ) -> SnapshotRegionFilterCollection {
        let scan_parameters_local = snapshot_region_filter_collection.get_scan_parameters_local();

        let result_snapshot_region_filters = snapshot_region_filter_collection
            .par_iter()
            .filter_map(|snapshot_region_filter| {
                let scanner_instance = Self::acquire_scanner_instance(snapshot_region_filter, scan_parameters_local);
                let filters = scanner_instance.scan_region(
                    snapshot_region,
                    snapshot_region_filter,
                    scan_parameters_global,
                    snapshot_region_filter_collection.get_scan_parameters_local(),
                );

                if filters.len() > 0 { Some(filters) } else { None }
            })
            .collect();

        SnapshotRegionFilterCollection::new(
            result_snapshot_region_filters,
            snapshot_region_filter_collection
                .get_scan_parameters_local()
                .clone(),
        )
    }

    fn acquire_scanner_instance(
        snapshot_region_filter: &SnapshotRegionFilter,
        scan_parameters_local: &ScanParametersLocal,
    ) -> &'static dyn Scanner {
        let data_type = scan_parameters_local.get_data_type();
        let memory_alignment = scan_parameters_local.get_memory_alignment_or_default();
        let data_type_size = data_type.get_size_in_bytes();
        let memory_alignment_size = memory_alignment as u64;
        let element_count = snapshot_region_filter.get_element_count(data_type, memory_alignment);
        let region_size = snapshot_region_filter.get_region_size();

        // Single element scanner. This is valid even for cases like array of byte scans, as all data types support basic equality checks.
        if element_count == 1 {
            return &ScannerScalarSingleElement {};
        }

        // Byte array scanners. Note that it's only worth using the specialized byte array scan if the byte arrays overlap.
        // If the byte arrays are sequential (back-to-back), or sparse (spaced out), then there is no need for an advanced algorithm.
        if data_type.get_data_type_id() == DataTypeByteArray::get_data_type_id() {
            if memory_alignment_size < data_type_size {
                return &ScannerScalarIterativeByteArrayCascading {};
            } else {
                // Here's the magic trick, we just use a normal iterative scalar scan for cascading and sparse scalar scans.
                return &ScannerScalarIterative {};
            }

            // JIRA: Switch on sizes for vectorized version
        }

        // We actually don't really care whether the processor supports AVX-512, AVX2, etc, PortableSimd is smart enough to
        // abstract this. It is actually more performant to greedily try to use AVX-512 even if its not available. PortableSimd
        // seems to fall back to unrolled AVX2 or SSE2 code, and it ends up being faster than the AVX2/SSE-first implementations.
        if region_size >= 64 {
            if memory_alignment_size < data_type_size {
                return &ScannerVectorCascading::<64> {};
            } else if memory_alignment_size == data_type_size {
                return &ScannerVectorAligned::<64> {};
            } else if memory_alignment_size > data_type_size {
                return &ScannerVectorSparse::<64> {};
            }
        } else if region_size >= 32 {
            if memory_alignment_size < data_type_size {
                return &ScannerVectorCascading::<32> {};
            } else if memory_alignment_size == data_type_size {
                return &ScannerVectorAligned::<32> {};
            } else if memory_alignment_size > data_type_size {
                return &ScannerVectorSparse::<32> {};
            }
        } else if region_size >= 16 {
            if memory_alignment_size < data_type_size {
                return &ScannerVectorCascading::<16> {};
            } else if memory_alignment_size == data_type_size {
                return &ScannerVectorAligned::<16> {};
            } else if memory_alignment_size > data_type_size {
                return &ScannerVectorSparse::<16> {};
            }
        }

        // Default to scalar iterative.
        &ScannerScalarIterative {}
    }
}
