use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::filters::snapshot_region_filter_collection::SnapshotRegionFilterCollection;
use crate::scanners::scalar::scanner_scalar_byte_array_booyer_moore::ScannerScalarByteArrayBooyerMoore;
use crate::scanners::scalar::scanner_scalar_iterative::ScannerScalarIterative;
use crate::scanners::scalar::scanner_scalar_single_element::ScannerScalarSingleElement;
use crate::scanners::snapshot_scanner::Scanner;
use crate::scanners::vector::scanner_vector_aligned::ScannerVectorAligned;
use crate::scanners::vector::scanner_vector_overlapping::ScannerVectorOverlapping;
use crate::scanners::vector::scanner_vector_overlapping_1_periodic::ScannerVectorOverlapping1Periodic;
use crate::scanners::vector::scanner_vector_sparse::ScannerVectorSparse;
use crate::snapshots::snapshot_region::SnapshotRegion;
use rayon::iter::ParallelIterator;
use squalr_engine_api::structures::data_types::built_in_types::byte_array::data_type_byte_array::DataTypeByteArray;
use squalr_engine_api::structures::scanning::parameters::scan_parameter_optimizations::ScanParameterOptimizations;
use squalr_engine_api::structures::scanning::parameters::scan_parameters::ScanParameters;
use squalr_engine_api::structures::scanning::parameters::scan_parameters_global::ScanParametersGlobal;
use squalr_engine_api::structures::scanning::parameters::scan_parameters_local::ScanParametersLocal;

use super::vector::scanner_vector_overlapping_2_periodic::ScannerVectorOverlapping2Periodic;

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
        // The main body of the scan.
        let snapshot_region_scanner = |snapshot_region_filter| {
            let scan_parameters_local = snapshot_region_filter_collection.get_scan_parameters_local();

            // Before selecting a scanner, attempt to gather information that can be used to optimize scanner selections and implementations.
            let scan_parameter_optimizations = ScanParameterOptimizations::new(scan_parameters_global, &scan_parameters_local);

            // Combine the global, local, and optimization parameters into a single container type.
            let mut scan_parameters = ScanParameters::new(scan_parameters_global, scan_parameters_local, &scan_parameter_optimizations);

            // Choose the best scanner given the provided parameters.
            let scanner_instance = Self::acquire_scanner_instance(snapshot_region_filter, scan_parameters_local, &scan_parameter_optimizations);

            // Finally do the actual scan.
            let filters = scanner_instance.scan_region(snapshot_region, snapshot_region_filter, &mut scan_parameters);

            if filters.len() > 0 { Some(filters) } else { None }
        };

        // Run the scan either single-threaded or parallel based on settings. Single-thread is not advised unless debugging.
        let result_snapshot_region_filters = if scan_parameters_global.is_single_thread_scan() {
            snapshot_region_filter_collection
                .iter()
                .filter_map(snapshot_region_scanner)
                .collect()
        } else {
            snapshot_region_filter_collection
                .par_iter()
                .filter_map(snapshot_region_scanner)
                .collect()
        };

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
        scan_parameter_optimizations: &ScanParameterOptimizations,
    ) -> &'static dyn Scanner {
        // First try a single element scanner. This is valid even for cases like array of byte scans, as all data types support basic equality checks.
        let element_count =
            snapshot_region_filter.get_element_count(scan_parameters_local.get_data_type(), scan_parameters_local.get_memory_alignment_or_default());

        if element_count == 1 {
            return &ScannerScalarSingleElement {};
        }

        // Select the data type used to select the best scanner implementation, based on the applied scan parameter optimizations.
        let data_type_override = scan_parameter_optimizations.get_data_type_override();
        let original_data_type = scan_parameters_local.get_data_type();
        let data_type = if let Some(data_type) = data_type_override {
            data_type
        } else {
            original_data_type
        };

        let memory_alignment = scan_parameters_local.get_memory_alignment_or_default();
        let data_type_size = data_type.get_size_in_bytes();
        let memory_alignment_size = memory_alignment as u64;
        let region_size = snapshot_region_filter.get_region_size();

        // Byte array scanners. Note that it's only worth using the specialized byte array scan if the byte arrays overlap.
        // If the byte arrays are sequential (back-to-back), or sparse (spaced out), then there is no need for an advanced algorithm.
        if data_type.get_data_type_id() == DataTypeByteArray::get_data_type_id() {
            if memory_alignment_size < data_type_size {
                return &ScannerScalarByteArrayBooyerMoore {};
            } else {
                // The arrays are spaced so far apart that they cannot possibly overlap, making the fancy algorithms useless.
                // Just use a normal iterative scalar scan for aligned and sparse scalar scans.
                return &ScannerScalarIterative {};
            }
        }

        // For overlapping periodic scans, we use custom scanners to greatly optimize scan performance.
        if let Some(periodicity) = scan_parameter_optimizations.get_periodicity() {
            /* return &ScannerScalarIterative {}; */
            if periodicity == 1 {
                if region_size >= 64 {
                    return &ScannerVectorOverlapping1Periodic::<64> {};
                } else if region_size >= 32 {
                    return &ScannerVectorOverlapping1Periodic::<32> {};
                } else if region_size >= 16 {
                    return &ScannerVectorOverlapping1Periodic::<16> {};
                }
            } else if periodicity == 2 {
                if region_size >= 64 {
                    return &ScannerVectorOverlapping2Periodic::<64> {};
                } else if region_size >= 32 {
                    return &ScannerVectorOverlapping2Periodic::<32> {};
                } else if region_size >= 16 {
                    return &ScannerVectorOverlapping2Periodic::<16> {};
                }
            }
        }

        // We actually don't really care whether the processor supports AVX-512, AVX2, etc, PortableSimd is smart enough to
        // abstract this. It is actually more performant to greedily try to use AVX-512 even if its not available. PortableSimd
        // seems to fall back to unrolled AVX2 or SSE2 code, and it ends up being faster than the AVX2/SSE-first implementations.
        if memory_alignment_size < data_type_size {
            if region_size >= 64 {
                return &ScannerVectorOverlapping::<64> {};
            } else if region_size >= 32 {
                return &ScannerVectorOverlapping::<32> {};
            } else if region_size >= 16 {
                return &ScannerVectorOverlapping::<16> {};
            }
        } else if memory_alignment_size == data_type_size {
            if region_size >= 64 {
                return &ScannerVectorAligned::<64> {};
            } else if region_size >= 32 {
                return &ScannerVectorAligned::<32> {};
            } else if region_size >= 16 {
                return &ScannerVectorAligned::<16> {};
            }
        } else if memory_alignment_size > data_type_size {
            if region_size >= 64 {
                return &ScannerVectorSparse::<64> {};
            } else if region_size >= 32 {
                return &ScannerVectorSparse::<32> {};
            } else if region_size >= 16 {
                return &ScannerVectorSparse::<16> {};
            }
        }

        // Default to scalar iterative.
        &ScannerScalarIterative {}
    }
}
