use crate::structures::scanning::{comparisons::scan_function_vector::ScanFunctionVector, constraints::scan_constraint_finalized::ScanConstraintFinalized};
use std::simd::{LaneCount, SupportedLaneCount};

/// Trait for dispatching access to the precalculated vector functions based on lane count.
/// This follows the same pattern as VectorComparer to handle const generic specialization.
pub trait GetVectorFunction<const N: usize>
where
    LaneCount<N>: SupportedLaneCount,
{
    fn get_vector_field<'lifetime>(
        &self,
        scan_constraint_finalized: &'lifetime ScanConstraintFinalized,
    ) -> &'lifetime Option<ScanFunctionVector<N>>;
}
