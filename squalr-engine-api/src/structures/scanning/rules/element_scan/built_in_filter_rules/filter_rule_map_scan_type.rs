use crate::registries::symbols::symbol_registry::SymbolRegistry;
use crate::structures::data_types::generics::vector_generics::VectorGenerics;
use crate::structures::scanning::constraints::scan_constraint_finalized::ScanConstraintFinalized;
use crate::structures::scanning::rules::element_scan_filter_rule::ElementScanFilterRule;
use crate::structures::scanning::{
    comparisons::{scan_compare_type::ScanCompareType, scan_compare_type_immediate::ScanCompareTypeImmediate},
    filters::{snapshot_region_filter::SnapshotRegionFilter, snapshot_region_filter_collection::SnapshotRegionFilterCollection},
    plans::{
        element_scan::snapshot_filter_element_scan_plan::SnapshotFilterElementScanPlan,
        plan_types::{
            planned_scan_type::PlannedScanType, planned_scan_type_byte_array::PlannedScanTypeByteArray, planned_scan_type_scalar::PlannedScanTypeScalar,
            planned_scan_type_vector::PlannedScanTypeVector, planned_scan_vectorization_size::PlannedScanVectorizationSize,
        },
    },
};
use crate::structures::snapshots::snapshot_region::SnapshotRegion;

pub struct RuleMapScanType {}

impl RuleMapScanType {
    pub const RULE_ID: &str = "map_scan_type";
}

impl ElementScanFilterRule for RuleMapScanType {
    fn get_id(&self) -> &str {
        &Self::RULE_ID
    }

    fn map_parameters(
        &self,
        snapshot_region: &SnapshotRegion,
        _snapshot_region_filter_collection: &SnapshotRegionFilterCollection,
        snapshot_region_filter: &SnapshotRegionFilter,
        _scan_constraint_finalized: &ScanConstraintFinalized,
        snapshot_filter_element_scan_plan: &mut SnapshotFilterElementScanPlan,
    ) {
        let is_valid_for_snapshot_region = if snapshot_region.has_current_values() {
            match snapshot_filter_element_scan_plan.get_compare_type() {
                ScanCompareType::Immediate(_) => true,
                ScanCompareType::Relative(_) | ScanCompareType::Delta(_) => snapshot_region.has_previous_values(),
            }
        } else {
            false
        };

        if !is_valid_for_snapshot_region {
            snapshot_filter_element_scan_plan.set_planned_scan_type(PlannedScanType::Invalid());

            return;
        }

        let region_size = snapshot_region_filter.get_region_size();

        // Early check as to whether we are smaller than the smallest possible vector.
        // Saves some computation to check this now, as this is a very frequent case.
        if region_size < 16 {
            snapshot_filter_element_scan_plan.set_planned_scan_type(PlannedScanType::Scalar(PlannedScanTypeScalar::ScalarIterative));

            return;
        }

        // Rather than using the snapshot_region_filter.get_region_size() directly, we try to be smart about ensuring
        // There is enough space to actually read a full vector of elements.
        // For example, if scanning for i32, 1-byte aligned, a single region of 64 bytes is not actually very helpful.
        // This is because we would actually want to overlap based on alignment, and thus would need at least 67 bytes.
        // This is derived from scanning for four i32 values at alignments 0, 1, 2, and 3.
        let symbol_registry = SymbolRegistry::get_instance();
        let data_type_ref = snapshot_filter_element_scan_plan.get_data_type_ref();
        let data_type_size_bytes = symbol_registry.get_unit_size_in_bytes(data_type_ref);
        let is_floating_point = symbol_registry.is_floating_point(data_type_ref);
        let memory_alignment_size = snapshot_filter_element_scan_plan.get_memory_alignment() as u64;

        // Decide whether to use a scalar or SIMD scan based on filter region size.
        let vectorization_size = if VectorGenerics::plan_vector_scan::<64>(region_size, data_type_size_bytes, memory_alignment_size).is_valid() {
            PlannedScanVectorizationSize::Vector64
        } else if VectorGenerics::plan_vector_scan::<32>(region_size, data_type_size_bytes, memory_alignment_size).is_valid() {
            PlannedScanVectorizationSize::Vector32
        } else if VectorGenerics::plan_vector_scan::<16>(region_size, data_type_size_bytes, memory_alignment_size).is_valid() {
            PlannedScanVectorizationSize::Vector16
        } else {
            // The filter cannot fit into a vector! Revert to scalar scan.
            snapshot_filter_element_scan_plan.set_planned_scan_type(PlannedScanType::Scalar(PlannedScanTypeScalar::ScalarIterative));

            return;
        };

        if data_type_size_bytes > memory_alignment_size {
            // Check if we can leverage periodicity, which is calculated in the `RuleMapPeriodicScans` rule.
            // See that particular rule for additional information on the concept of periodicity.
            match snapshot_filter_element_scan_plan.get_periodicity() {
                1 => {
                    // Better for debug mode.
                    // snapshot_filter_element_scan_plan.set_planned_scan_type(PlannedScanType::Vector(PlannedScanTypeVector::OverlappingBytewisePeriodic));

                    // Better for release mode.
                    snapshot_filter_element_scan_plan
                        .set_planned_scan_type(PlannedScanType::Vector(PlannedScanTypeVector::OverlappingBytewiseStaggered, vectorization_size));
                }
                2 | 4 | 8 => {
                    snapshot_filter_element_scan_plan
                        .set_planned_scan_type(PlannedScanType::Vector(PlannedScanTypeVector::OverlappingBytewiseStaggered, vectorization_size));
                }
                _ => {
                    snapshot_filter_element_scan_plan.set_planned_scan_type(PlannedScanType::Vector(PlannedScanTypeVector::Overlapping, vectorization_size));
                }
            }
        } else if data_type_size_bytes < memory_alignment_size {
            snapshot_filter_element_scan_plan.set_planned_scan_type(PlannedScanType::Vector(PlannedScanTypeVector::Sparse, vectorization_size));
        } else {
            snapshot_filter_element_scan_plan.set_planned_scan_type(PlannedScanType::Vector(PlannedScanTypeVector::Aligned, vectorization_size));
        }

        match snapshot_filter_element_scan_plan.get_compare_type() {
            ScanCompareType::Relative(_) | ScanCompareType::Delta(_) => {}
            ScanCompareType::Immediate(scan_compare_type_immediate) => {
                match scan_compare_type_immediate {
                    ScanCompareTypeImmediate::Equal | ScanCompareTypeImmediate::NotEqual => {
                        if !is_floating_point {
                            // Perform a byte array scan, since we were unable to map the byte array to a primitive type.
                            // These are the only acceptable options, either the type is a primitive, or its a byte array.
                            snapshot_filter_element_scan_plan.set_planned_scan_type(PlannedScanType::ByteArray(PlannedScanTypeByteArray::ByteArrayBooyerMoore));
                        }
                    }
                    _ => {}
                }
            }
        };
    }
}
