use crate::structures::scanning::{comparisons::scan_function_vector::ScanFunctionVector, constraints::scan_constraint_finalized::ScanConstraintFinalized};

/// Trait for dispatching access to the precalculated vector functions based on lane count.
/// This follows the same pattern as VectorComparer to handle const generic specialization.
pub trait GetVectorFunction<const N: usize> {
    fn get_vector_field<'lifetime>(
        &self,
        scan_constraint_finalized: &'lifetime ScanConstraintFinalized,
    ) -> &'lifetime Option<ScanFunctionVector<N>>;
}
