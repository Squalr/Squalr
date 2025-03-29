use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::filters::snapshot_region_filter_collection::SnapshotRegionFilterCollection;
use crate::scanners::scalar::scanner_scalar_byte_array_booyer_moore::ScannerScalarByteArrayBooyerMoore;
use crate::scanners::scalar::scanner_scalar_iterative::ScannerScalarIterative;
use crate::scanners::scalar::scanner_scalar_single_element::ScannerScalarSingleElement;
use crate::scanners::snapshot_scanner::Scanner;
use crate::scanners::vector::scanner_vector_aligned::ScannerVectorAligned;
use crate::scanners::vector::scanner_vector_overlapping::ScannerVectorOverlapping;
use crate::scanners::vector::scanner_vector_sparse::ScannerVectorSparse;
use crate::snapshots::snapshot_region::SnapshotRegion;
use rayon::iter::ParallelIterator;
use squalr_engine_api::structures::data_types::built_in_types::byte_array::data_type_byte_array::DataTypeByteArray;
use squalr_engine_api::structures::data_types::built_in_types::u8::data_type_u8::DataTypeU8;
use squalr_engine_api::structures::data_types::built_in_types::u16be::data_type_u16be::DataTypeU16be;
use squalr_engine_api::structures::data_types::built_in_types::u32be::data_type_u32be::DataTypeU32be;
use squalr_engine_api::structures::data_types::built_in_types::u64be::data_type_u64be::DataTypeU64be;
use squalr_engine_api::structures::data_types::data_type_meta_data::DataTypeMetaData;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
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
        // The main body of the scan.
        let snapshot_region_scanner = |snapshot_region_filter| {
            // Create mutable scan parameters, since we do optimizations to redirect certain scan types.
            // For example, scanning for an array of byte of length 4 is equivalent to scanning for a u32, so we edit the scan parameters.
            let mut scan_parameters_local = snapshot_region_filter_collection
                .get_scan_parameters_local()
                .clone();
            let scanner_instance = Self::acquire_scanner_instance(snapshot_region_filter, &mut scan_parameters_local);
            let filters = scanner_instance.scan_region(snapshot_region, snapshot_region_filter, scan_parameters_global, &scan_parameters_local);

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
        scan_parameters_local: &mut ScanParametersLocal,
    ) -> &'static dyn Scanner {
        // First try a single element scanner. This is valid even for cases like array of byte scans, as all data types support basic equality checks.
        let element_count =
            snapshot_region_filter.get_element_count(scan_parameters_local.get_data_type(), scan_parameters_local.get_memory_alignment_or_default());

        if element_count == 1 {
            return &ScannerScalarSingleElement {};
        }

        // Before selecting a scanner, attempt to remap the scan parameters onto a new data type if applicable for performance.
        if let Some(new_data_type) = Self::optimize_scan_parameters(scan_parameters_local) {
            scan_parameters_local.set_data_type(new_data_type);
        }

        let data_type = scan_parameters_local.get_data_type();
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

        // We actually don't really care whether the processor supports AVX-512, AVX2, etc, PortableSimd is smart enough to
        // abstract this. It is actually more performant to greedily try to use AVX-512 even if its not available. PortableSimd
        // seems to fall back to unrolled AVX2 or SSE2 code, and it ends up being faster than the AVX2/SSE-first implementations.
        if region_size >= 64 {
            if memory_alignment_size < data_type_size {
                return &ScannerVectorOverlapping::<64> {};
            } else if memory_alignment_size == data_type_size {
                return &ScannerVectorAligned::<64> {};
            } else if memory_alignment_size > data_type_size {
                return &ScannerVectorSparse::<64> {};
            }
        } else if region_size >= 32 {
            if memory_alignment_size < data_type_size {
                return &ScannerVectorOverlapping::<32> {};
            } else if memory_alignment_size == data_type_size {
                return &ScannerVectorAligned::<32> {};
            } else if memory_alignment_size > data_type_size {
                return &ScannerVectorSparse::<32> {};
            }
        } else if region_size >= 16 {
            if memory_alignment_size < data_type_size {
                return &ScannerVectorOverlapping::<16> {};
            } else if memory_alignment_size == data_type_size {
                return &ScannerVectorAligned::<16> {};
            } else if memory_alignment_size > data_type_size {
                return &ScannerVectorSparse::<16> {};
            }
        }

        // Default to scalar iterative.
        &ScannerScalarIterative {}
    }

    /// Remaps scan parameters into "functionally equivalent" paramters for performance gains.
    /// For example, an array of byte scan for 00 00 00 00 is better treated as a u32 scan of 0, as this is easily vectorized.
    fn optimize_scan_parameters(scan_parameters_local: &ScanParametersLocal) -> Option<DataTypeRef> {
        let data_type = scan_parameters_local.get_data_type();
        let data_type_size = data_type.get_size_in_bytes();

        if data_type.get_data_type_id() == DataTypeByteArray::get_data_type_id() {
            // If applicable, try to map array of byte scans to any primitive types of the same size.
            // These are much more efficient than array of byte scans, so for scans of these sizes performance will be improved greatly.
            match data_type_size {
                8 => Some(DataTypeRef::new(DataTypeU64be::get_data_type_id(), DataTypeMetaData::None)),
                4 => Some(DataTypeRef::new(DataTypeU32be::get_data_type_id(), DataTypeMetaData::None)),
                2 => Some(DataTypeRef::new(DataTypeU16be::get_data_type_id(), DataTypeMetaData::None)),
                1 => Some(DataTypeRef::new(DataTypeU8::get_data_type_id(), DataTypeMetaData::None)),
                // If we can't map onto a primitive, continue selecting the best array of byte scan algorithm.
                _ => None,
            }
        } else {
            None
        }
    }
}
