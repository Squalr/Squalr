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
use squalr_engine_api::structures::data_types::built_in_types::u16::data_type_u16::DataTypeU16;
use squalr_engine_api::structures::data_types::built_in_types::u16be::data_type_u16be::DataTypeU16be;
use squalr_engine_api::structures::data_types::built_in_types::u32::data_type_u32::DataTypeU32;
use squalr_engine_api::structures::data_types::built_in_types::u32be::data_type_u32be::DataTypeU32be;
use squalr_engine_api::structures::data_types::built_in_types::u64::data_type_u64::DataTypeU64;
use squalr_engine_api::structures::data_types::built_in_types::u64be::data_type_u64be::DataTypeU64be;
use squalr_engine_api::structures::data_types::data_type_meta_data::DataTypeMetaData;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use squalr_engine_api::structures::scanning::scan_parameters_global::ScanParametersGlobal;
use squalr_engine_api::structures::scanning::scan_parameters_local::ScanParametersLocal;

use super::vector::scanner_vector_overlapping_1_periodic::ScannerVectorOverlapping1Periodic;

struct OptimizedScanParameters {
    scan_parameters_global: ScanParametersGlobal,
    scan_parameters_local: ScanParametersLocal,
    periodicity: Option<u64>,
}

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
            let scanner_instance = Self::acquire_scanner_instance(snapshot_region_filter, &scan_parameters_global, &mut scan_parameters_local);
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
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &mut ScanParametersLocal,
    ) -> &'static dyn Scanner {
        // First try a single element scanner. This is valid even for cases like array of byte scans, as all data types support basic equality checks.
        let element_count =
            snapshot_region_filter.get_element_count(scan_parameters_local.get_data_type(), scan_parameters_local.get_memory_alignment_or_default());

        if element_count == 1 {
            return &ScannerScalarSingleElement {};
        }

        // Before selecting a scanner, attempt to remap the scan parameters onto a new data type if applicable for performance.
        let optimized_scan_parameters = Self::optimize_scan_parameters(scan_parameters_global.clone(), scan_parameters_local.clone());

        let scan_parameters_local = optimized_scan_parameters.scan_parameters_local;
        let periodicity = optimized_scan_parameters.periodicity;
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

        // For overlapping periodic scans, we use custom scanners to greatly optimize scan performance.
        if let Some(periodicity) = periodicity {
            if periodicity == 1 {
                if region_size >= 64 {
                    return &ScannerVectorOverlapping1Periodic::<64> {};
                } else if region_size >= 32 {
                    return &ScannerVectorOverlapping1Periodic::<32> {};
                } else if region_size >= 16 {
                    return &ScannerVectorOverlapping1Periodic::<16> {};
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

    /// Calculates the length of repeating byte patterns within a given data type and value combination.
    /// If there are no repeating patterns, the periodicity will be equal to the data type size.
    /// For example, 7C 01 7C 01 has a data typze size of 4, but a periodicity of 2.
    fn calculate_periodicity_from_immediate(
        immediate_value_bytes: &[u8],
        data_type_size_bytes: u64,
    ) -> u64 {
        // Assume optimal periodicity to begin with
        let mut period = 1;

        // Loop through all remaining bytes, and increase the periodicity when we encounter a byte that violates the current assumption.
        for byte_index in 1..data_type_size_bytes as usize {
            if immediate_value_bytes[byte_index] != immediate_value_bytes[byte_index % period] {
                period = byte_index + 1;
            }
        }

        period as u64
    }

    /// Remaps scan parameters into "functionally equivalent" paramters for performance gains.
    /// For example, an array of byte scan for 00 00 00 00 is better treated as a u32 scan of 0, as this is easily vectorized.
    fn optimize_scan_parameters(
        scan_parameters_global: ScanParametersGlobal,
        scan_parameters_local: ScanParametersLocal,
    ) -> OptimizedScanParameters {
        let original_data_type_size = scan_parameters_local.get_data_type().get_size_in_bytes();
        let mut optimized_scan_parameters = OptimizedScanParameters {
            scan_parameters_global,
            scan_parameters_local,
            periodicity: None,
        };

        let scan_parameters_global = &optimized_scan_parameters.scan_parameters_global;
        let scan_parameters_local = &mut optimized_scan_parameters.scan_parameters_local;

        if scan_parameters_local.get_data_type().get_data_type_id() == DataTypeByteArray::get_data_type_id() {
            // If applicable, try to map array of byte scans to any primitive types of the same size.
            // These are much more efficient than array of byte scans, so for scans of these sizes performance will be improved greatly.
            if let Some(new_data_type) = match original_data_type_size {
                8 => Some(DataTypeRef::new(DataTypeU64be::get_data_type_id(), DataTypeMetaData::None)),
                4 => Some(DataTypeRef::new(DataTypeU32be::get_data_type_id(), DataTypeMetaData::None)),
                2 => Some(DataTypeRef::new(DataTypeU16be::get_data_type_id(), DataTypeMetaData::None)),
                1 => Some(DataTypeRef::new(DataTypeU8::get_data_type_id(), DataTypeMetaData::None)),
                // If we can't map onto a primitive, continue selecting the best array of byte scan algorithm.
                _ => None,
            } {
                scan_parameters_local.set_data_type(new_data_type);
            }
        }

        // Grab the potentially updated data type / size now that we have finished remapping.
        let data_type = scan_parameters_local.get_data_type();
        let data_type_size = data_type.get_size_in_bytes();

        optimized_scan_parameters.periodicity = match optimized_scan_parameters
            .scan_parameters_global
            .get_compare_type()
        {
            ScanCompareType::Immediate(_scan_compare_type_immediate) => {
                if let Some(immediate_value) = scan_parameters_global.deanonymize_immediate(data_type) {
                    Some(Self::calculate_periodicity_from_immediate(immediate_value.get_value_bytes(), data_type_size))
                } else {
                    None
                }
            }
            ScanCompareType::Relative(_scan_compare_type_immediate) => None,
            ScanCompareType::Delta(_scan_compare_type_immediate) => {
                if let Some(immediate_value) = scan_parameters_global.deanonymize_immediate(data_type) {
                    Some(Self::calculate_periodicity_from_immediate(immediate_value.get_value_bytes(), data_type_size))
                } else {
                    None
                }
            }
        };

        if let Some(periodicity) = optimized_scan_parameters.periodicity {
            match periodicity {
                1 => {
                    scan_parameters_local.set_data_type(DataTypeRef::new(DataTypeU8::get_data_type_id(), DataTypeMetaData::None));
                }
                2 => {
                    scan_parameters_local.set_data_type(DataTypeRef::new(DataTypeU16::get_data_type_id(), DataTypeMetaData::None));
                }
                4 => {
                    scan_parameters_local.set_data_type(DataTypeRef::new(DataTypeU32::get_data_type_id(), DataTypeMetaData::None));
                }
                8 => {
                    scan_parameters_local.set_data_type(DataTypeRef::new(DataTypeU64::get_data_type_id(), DataTypeMetaData::None));
                }
                _ => {
                    log::warn!("Unsupported periodicity: {}", periodicity);
                }
            }
        }

        optimized_scan_parameters
    }
}
