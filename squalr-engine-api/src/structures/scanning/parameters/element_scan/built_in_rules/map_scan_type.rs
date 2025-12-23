use std::sync::{Arc, RwLock};

use crate::registries::symbols::symbol_registry::SymbolRegistry;
use crate::structures::data_types::generics::vector_generics::VectorGenerics;
use crate::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use crate::structures::scanning::comparisons::scan_compare_type_immediate::ScanCompareTypeImmediate;
use crate::structures::scanning::parameters::mapped::mapped_scan_type::{MappedScanType, ScanParametersByteArray, ScanParametersScalar, ScanParametersVector};
use crate::structures::scanning::parameters::mapped::vectorization_size::VectorizationSize;
use crate::structures::scanning::rules::element_scan_mapping_rule::ElementScanMappingRule;
use crate::structures::scanning::{
    filters::{snapshot_region_filter::SnapshotRegionFilter, snapshot_region_filter_collection::SnapshotRegionFilterCollection},
    parameters::{element_scan::element_scan_parameters::ElementScanParameters, mapped::mapped_scan_parameters::MappedScanParameters},
};

pub struct MapScanType {}

impl MapScanType {
    pub const RULE_ID: &str = "map_scan_type";
}

impl ElementScanMappingRule for MapScanType {
    fn get_id(&self) -> &str {
        &Self::RULE_ID
    }

    fn map_parameters(
        &self,
        symbol_registry: &Arc<RwLock<SymbolRegistry>>,
        _snapshot_region_filter_collection: &SnapshotRegionFilterCollection,
        snapshot_region_filter: &SnapshotRegionFilter,
        _original_scan_parameters: &ElementScanParameters,
        mapped_parameters: &mut MappedScanParameters,
    ) {
        // Rather than using the snapshot_region_filter.get_region_size() directly, we try to be smart about ensuring
        // There is enough space to actually read a full vector of elements.
        // For example, if scanning for i32, 1-byte aligned, a single region of 64 bytes is not actually very helpful.
        // This is because we would actually want to overlap based on alignment, and thus would need at least 67 bytes.
        // This is derived from scanning for four i32 values at alignments 0, 1, 2, and 3.
        let symbol_registry_guard = match symbol_registry.read() {
            Ok(registry) => registry,
            Err(error) => {
                log::error!("Failed to acquire read lock on SymbolRegistry: {}", error);

                return;
            }
        };
        let data_type_ref = mapped_parameters.get_data_type_ref();
        let data_type_size_bytes = symbol_registry_guard.get_unit_size_in_bytes(data_type_ref);
        let is_floating_point = symbol_registry_guard.is_floating_point(data_type_ref);
        let memory_alignment_size = mapped_parameters.get_memory_alignment() as u64;
        let region_size = snapshot_region_filter.get_region_size();
        let vectorization_plan_64 = VectorGenerics::plan_vector_scan::<64>(region_size, data_type_size_bytes, memory_alignment_size);
        let vectorization_plan_32 = VectorGenerics::plan_vector_scan::<32>(region_size, data_type_size_bytes, memory_alignment_size);
        let vectorization_plan_16 = VectorGenerics::plan_vector_scan::<16>(region_size, data_type_size_bytes, memory_alignment_size);

        // Decide whether to use a scalar or SIMD scan based on filter region size.
        let vectorization_size = if vectorization_plan_64.is_valid() {
            VectorizationSize::Vector64
        } else if vectorization_plan_32.is_valid() {
            VectorizationSize::Vector32
        } else if vectorization_plan_16.is_valid() {
            VectorizationSize::Vector16
        } else {
            // The filter cannot fit into a vector! Revert to scalar scan.
            mapped_parameters.set_mapped_scan_type(MappedScanType::Scalar(ScanParametersScalar::ScalarIterative));

            return;
        };

        if data_type_size_bytes > memory_alignment_size {
            // Check if we can leverage periodicity, which is calculated in the `MapPeriodicScans` rule.
            // See that particular rule for additional information on the concept of periodicity.
            match mapped_parameters.get_periodicity() {
                1 => {
                    // Better for debug mode.
                    // mapped_parameters.set_mapped_scan_type(MappedScanType::Vector(ScanParametersVector::OverlappingBytewisePeriodic));

                    // Better for release mode.
                    mapped_parameters.set_mapped_scan_type(MappedScanType::Vector(ScanParametersVector::OverlappingBytewiseStaggered, vectorization_size));
                }
                2 | 4 | 8 => {
                    mapped_parameters.set_mapped_scan_type(MappedScanType::Vector(ScanParametersVector::OverlappingBytewiseStaggered, vectorization_size));
                }
                _ => {
                    mapped_parameters.set_mapped_scan_type(MappedScanType::Vector(ScanParametersVector::Overlapping, vectorization_size));
                }
            }
        } else if data_type_size_bytes < memory_alignment_size {
            mapped_parameters.set_mapped_scan_type(MappedScanType::Vector(ScanParametersVector::Sparse, vectorization_size));
        } else {
            mapped_parameters.set_mapped_scan_type(MappedScanType::Vector(ScanParametersVector::Aligned, vectorization_size));
        }

        match mapped_parameters.get_compare_type() {
            ScanCompareType::Relative(_) | ScanCompareType::Delta(_) => {}
            ScanCompareType::Immediate(scan_compare_type_immediate) => {
                match scan_compare_type_immediate {
                    ScanCompareTypeImmediate::Equal | ScanCompareTypeImmediate::NotEqual => {
                        if !is_floating_point {
                            // Perform a byte array scan, since we were unable to map the byte array to a primitive type.
                            // These are the only acceptable options, either the type is a primitive, or its a byte array.
                            mapped_parameters.set_mapped_scan_type(MappedScanType::ByteArray(ScanParametersByteArray::ByteArrayBooyerMoore));
                        }
                    }
                    _ => {}
                }
            }
        };
    }
}
