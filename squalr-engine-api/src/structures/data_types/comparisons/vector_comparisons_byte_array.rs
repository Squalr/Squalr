use crate::structures::scanning::comparisons::scan_function_vector::{VectorCompareFnDelta, VectorCompareFnImmediate, VectorCompareFnRelative};
use crate::structures::scanning::constraints::scan_constraint::ScanConstraint;
use std::ops::{Add, BitAnd, BitOr, BitXor, Mul, Rem, Shl, Shr, Sub};
use std::simd::cmp::{SimdPartialEq, SimdPartialOrd};
use std::simd::{Simd, SimdElement};

pub struct VectorComparisonsByteArray {}

/// Deliberately not implemented. Vector based byte array comparisons are implemented elsewhere in specialized scan routines.
impl VectorComparisonsByteArray {
    pub fn get_vector_compare_equal<const N: usize, const E: usize, PrimitiveType: SimdElement + PartialEq + Send + Sync + 'static>(
        _scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnImmediate<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq,
    {
        None
    }

    pub fn get_vector_compare_not_equal<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        _scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnImmediate<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq,
    {
        None
    }

    pub fn get_vector_compare_greater_than<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        _scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnImmediate<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialOrd,
    {
        None
    }

    pub fn get_vector_compare_greater_than_or_equal<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        _scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnImmediate<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialOrd,
    {
        None
    }

    pub fn get_vector_compare_less_than<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        _scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnImmediate<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialOrd,
    {
        None
    }

    pub fn get_vector_compare_less_than_or_equal<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        _scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnImmediate<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialOrd,
    {
        None
    }

    pub fn get_vector_compare_changed<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        __scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnRelative<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq,
    {
        None
    }

    pub fn get_vector_compare_unchanged<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        __scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnRelative<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq,
    {
        None
    }

    pub fn get_vector_compare_increased<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        __scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnRelative<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialOrd,
    {
        None
    }

    pub fn get_vector_compare_decreased<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        __scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnRelative<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialOrd,
    {
        None
    }

    pub fn get_vector_compare_increased_by<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        _scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + Add<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        None
    }

    pub fn get_vector_compare_decreased_by<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        _scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + Sub<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        None
    }

    pub fn get_vector_compare_multiplied_by<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        _scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + Mul<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        None
    }

    pub fn get_vector_compare_divided_by<const N: usize, const E: usize, PrimitiveType: SimdElement + PartialEq + Default + Send + Sync + 'static>(
        _scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + Mul<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        None
    }

    pub fn get_vector_compare_modulo_by<const N: usize, const E: usize, PrimitiveType: SimdElement + PartialEq + Default + Send + Sync + 'static>(
        _scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + Rem<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        None
    }

    pub fn get_vector_compare_shift_left_by<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        _scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + Shl<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        None
    }

    pub fn get_vector_compare_shift_right_by<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        _scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + Shr<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        None
    }

    pub fn get_vector_compare_logical_and_by<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        _scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + BitAnd<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        None
    }

    pub fn get_vector_compare_logical_or_by<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        _scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + BitOr<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        None
    }

    pub fn get_vector_compare_logical_xor_by<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        _scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + BitXor<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        None
    }
}
