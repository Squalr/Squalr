use crate::structures::data_types::generics::vector_generics::VectorGenerics;
use crate::structures::scanning::comparisons::scan_function_vector::{VectorCompareFnDelta, VectorCompareFnImmediate, VectorCompareFnRelative};
use crate::structures::scanning::constraints::scan_constraint::ScanConstraint;
use num_traits::Float;
use std::ops::{Add, Div, Mul, Rem, Sub};
use std::ptr;
use std::simd::cmp::{SimdPartialEq, SimdPartialOrd};
use std::simd::num::SimdFloat;
use std::simd::{Simd, SimdElement};
use std::sync::Arc;

pub struct VectorComparisonsFloat {}

impl VectorComparisonsFloat {
    pub fn get_vector_compare_equal<const N: usize, const E: usize, PrimitiveType: SimdElement + Float + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnImmediate<N>>
    where
        Simd<PrimitiveType, E>: SimdFloat + SimdPartialOrd + Sub<Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let tolerance: Simd<PrimitiveType, E> = Simd::splat(scan_constraint.get_floating_point_tolerance().get_value());
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = Simd::splat(unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) });

        Some(Arc::new(move |current_values_ptr| {
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });

            // Equality between the current and immediate value is determined by being within the given tolerance.
            VectorGenerics::transmute_mask(current_values.sub(immediate_value).abs().simd_le(tolerance))
        }))
    }

    pub fn get_vector_compare_not_equal<const N: usize, const E: usize, PrimitiveType: SimdElement + Float + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnImmediate<N>>
    where
        Simd<PrimitiveType, E>: SimdFloat + SimdPartialOrd + Sub<Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let tolerance: Simd<PrimitiveType, E> = Simd::splat(scan_constraint.get_floating_point_tolerance().get_value());
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = Simd::splat(unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) });

        Some(Arc::new(move |current_values_ptr| {
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });

            // Inequality between the current and immediate value is determined by being outside the given tolerance.
            VectorGenerics::transmute_mask(current_values.sub(immediate_value).abs().simd_gt(tolerance))
        }))
    }

    pub fn get_vector_compare_greater_than<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnImmediate<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialOrd,
    {
        let immediate_value = scan_constraint.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = Simd::splat(unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) });

        Some(Arc::new(move |current_values_ptr| {
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });

            // No checks tolerance required.
            VectorGenerics::transmute_mask(current_values.simd_gt(immediate_value))
        }))
    }

    pub fn get_vector_compare_greater_than_or_equal<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnImmediate<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialOrd,
    {
        let immediate_value = scan_constraint.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = Simd::splat(unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) });

        Some(Arc::new(move |current_values_ptr| {
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });

            // No checks tolerance required.
            VectorGenerics::transmute_mask(current_values.simd_ge(immediate_value))
        }))
    }

    pub fn get_vector_compare_less_than<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnImmediate<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialOrd,
    {
        let immediate_value = scan_constraint.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = Simd::splat(unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) });

        Some(Arc::new(move |current_values_ptr| {
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });

            // No checks tolerance required.
            VectorGenerics::transmute_mask(current_values.simd_lt(immediate_value))
        }))
    }

    pub fn get_vector_compare_less_than_or_equal<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnImmediate<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialOrd,
    {
        let immediate_value = scan_constraint.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = Simd::splat(unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) });

        Some(Arc::new(move |current_values_ptr| {
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });

            // No checks tolerance required.
            VectorGenerics::transmute_mask(current_values.simd_le(immediate_value))
        }))
    }

    pub fn get_vector_compare_changed<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        _scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnRelative<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq,
    {
        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });
            let previous_values = Simd::from_array(unsafe { ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E]) });

            // No checks tolerance required.
            VectorGenerics::transmute_mask(current_values.simd_ne(previous_values))
        }))
    }

    pub fn get_vector_compare_unchanged<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        _scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnRelative<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq,
    {
        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });
            let previous_values = Simd::from_array(unsafe { ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E]) });

            // No checks tolerance required.
            VectorGenerics::transmute_mask(current_values.simd_eq(previous_values))
        }))
    }

    pub fn get_vector_compare_increased<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        _scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnRelative<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialOrd,
    {
        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });
            let previous_values = Simd::from_array(unsafe { ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E]) });

            // No checks tolerance required.
            VectorGenerics::transmute_mask(current_values.simd_gt(previous_values))
        }))
    }

    pub fn get_vector_compare_decreased<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        _scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnRelative<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialOrd,
    {
        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });
            let previous_values = Simd::from_array(unsafe { ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E]) });

            // No checks tolerance required.
            VectorGenerics::transmute_mask(current_values.simd_lt(previous_values))
        }))
    }

    pub fn get_vector_compare_increased_by<const N: usize, const E: usize, PrimitiveType: SimdElement + Float + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdFloat
            + SimdPartialOrd
            + Add<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>
            + Sub<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let tolerance: Simd<PrimitiveType, E> = Simd::splat(scan_constraint.get_floating_point_tolerance().get_value());
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = Simd::splat(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) });

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });
            let previous_values = Simd::from_array(unsafe { ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E]) });
            let target_values = previous_values.add(delta_value);

            // Equality between the current and target value is determined by being within the given tolerance.
            VectorGenerics::transmute_mask(current_values.sub(target_values).abs().simd_le(tolerance))
        }))
    }

    pub fn get_vector_compare_decreased_by<const N: usize, const E: usize, PrimitiveType: SimdElement + Float + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdFloat + SimdPartialOrd + Sub<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let tolerance: Simd<PrimitiveType, E> = Simd::splat(scan_constraint.get_floating_point_tolerance().get_value());
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = Simd::splat(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) });

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });
            let previous_values = Simd::from_array(unsafe { ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E]) });
            let target_values = previous_values.sub(delta_value);

            // Equality between the current and target value is determined by being within the given tolerance.
            VectorGenerics::transmute_mask(current_values.sub(target_values).abs().simd_le(tolerance))
        }))
    }

    pub fn get_vector_compare_multiplied_by<const N: usize, const E: usize, PrimitiveType: SimdElement + Float + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdFloat
            + SimdPartialOrd
            + Sub<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>
            + Mul<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let tolerance: Simd<PrimitiveType, E> = Simd::splat(scan_constraint.get_floating_point_tolerance().get_value());
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = Simd::splat(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) });

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });
            let previous_values = Simd::from_array(unsafe { ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E]) });
            let target_values = previous_values.mul(delta_value);

            // Equality between the current and target value is determined by being within the given tolerance.
            VectorGenerics::transmute_mask(current_values.sub(target_values).abs().simd_le(tolerance))
        }))
    }

    pub fn get_vector_compare_divided_by<const N: usize, const E: usize, PrimitiveType: SimdElement + Float + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdFloat
            + SimdPartialOrd
            + Sub<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>
            + Div<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let tolerance: Simd<PrimitiveType, E> = Simd::splat(scan_constraint.get_floating_point_tolerance().get_value());
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = Simd::splat(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) });

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });
            let previous_values = Simd::from_array(unsafe { ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E]) });
            let target_values = previous_values.div(delta_value);

            // Equality between the current and target value is determined by being within the given tolerance.
            VectorGenerics::transmute_mask(current_values.sub(target_values).abs().simd_le(tolerance))
        }))
    }

    pub fn get_vector_compare_modulo_by<const N: usize, const E: usize, PrimitiveType: SimdElement + Float + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdFloat
            + SimdPartialOrd
            + Sub<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>
            + Rem<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let tolerance: Simd<PrimitiveType, E> = Simd::splat(scan_constraint.get_floating_point_tolerance().get_value());
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = Simd::splat(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) });

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });
            let previous_values = Simd::from_array(unsafe { ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E]) });
            let target_values = previous_values.rem(delta_value);

            // Equality between the current and target value is determined by being within the given tolerance.
            VectorGenerics::transmute_mask(current_values.sub(target_values).abs().simd_le(tolerance))
        }))
    }
}
