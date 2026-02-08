use crate::structures::data_types::floating_point_tolerance::FloatingPointTolerance;
use crate::structures::data_types::generics::vector_comparer::VectorComparer;
use crate::structures::data_types::generics::vector_function::GetVectorFunction;
use crate::structures::data_types::generics::vector_lane_count::VectorLaneCount;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::memory::memory_alignment::MemoryAlignment;
use crate::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use crate::structures::scanning::comparisons::scan_function_scalar::ScanFunctionScalar;
use crate::structures::scanning::comparisons::scan_function_vector::ScanFunctionVector;
use crate::structures::scanning::constraints::scan_constraint::ScanConstraint;
use crate::structures::scanning::plans::plan_types::planned_scan_type::PlannedScanType;
use crate::structures::{data_types::data_type_ref::DataTypeRef, scanning::constraints::scan_constraint_finalized::ScanConstraintFinalized};

/// Represents the scan plan for scanning an individual filter within a larger element scan.
pub struct SnapshotFilterElementScanPlan<'lifetime> {
    scan_constraint_finalized: &'lifetime ScanConstraintFinalized,
    memory_alignment: MemoryAlignment,
    floating_point_tolerance: FloatingPointTolerance,
    planned_scan_type: PlannedScanType,
}

impl<'lifetime> SnapshotFilterElementScanPlan<'lifetime> {
    /// Creates optimized scan paramaters for a given snapshot region filter, given user provided scan parameters.
    /// Internally, the user parameters are processed into more optimal parameters that help select the most optimal scan implementation.
    pub fn new(
        scan_constraint_finalized: &'lifetime ScanConstraintFinalized,
        memory_alignment: MemoryAlignment,
        floating_point_tolerance: FloatingPointTolerance,
    ) -> Self {
        Self {
            scan_constraint_finalized,
            memory_alignment,
            floating_point_tolerance,
            planned_scan_type: PlannedScanType::Invalid(),
        }
    }

    pub fn get_scan_constraint_finalized(&self) -> &ScanConstraintFinalized {
        &self.scan_constraint_finalized
    }

    pub fn get_scan_constraint(&self) -> &ScanConstraint {
        self.scan_constraint_finalized.get_scan_constraint()
    }

    pub fn get_data_value(&self) -> &DataValue {
        &self.scan_constraint_finalized.get_data_value()
    }

    pub fn get_data_type_ref(&self) -> &DataTypeRef {
        &self.get_data_value().get_data_type_ref()
    }

    pub fn get_memory_alignment(&self) -> MemoryAlignment {
        self.memory_alignment
    }

    pub fn get_compare_type(&self) -> ScanCompareType {
        self.scan_constraint_finalized.get_scan_compare_type()
    }

    pub fn get_floating_point_tolerance(&self) -> FloatingPointTolerance {
        self.floating_point_tolerance
    }

    pub fn get_periodicity(&self) -> u64 {
        self.scan_constraint_finalized.get_periodicity()
    }

    pub fn get_unit_size_in_bytes(&self) -> u64 {
        self.scan_constraint_finalized.get_unit_size_in_bytes()
    }

    pub fn get_planned_scan_type(&self) -> &PlannedScanType {
        &self.planned_scan_type
    }

    pub fn set_planned_scan_type(
        &mut self,
        planned_scan_type: PlannedScanType,
    ) {
        self.planned_scan_type = planned_scan_type;
    }

    pub fn get_scan_function_scalar(&self) -> &Option<ScanFunctionScalar> {
        self.scan_constraint_finalized.get_scan_function_scalar()
    }

    pub fn get_scan_function_vector<const N: usize>(&self) -> &Option<ScanFunctionVector<N>>
    where
        VectorLaneCount<N>: VectorComparer<N> + GetVectorFunction<N>,
    {
        self.scan_constraint_finalized.get_scan_function_vector()
    }
}
