use crate::structures::data_types::generics::vector_generics::VectorGenerics;
use crate::structures::scanning::comparisons::scan_function_vector::{VectorCompareFnDelta, VectorCompareFnImmediate, VectorCompareFnRelative};
use crate::structures::scanning::constraints::scan_constraint::ScanConstraint;
use std::ops::{Add, BitAnd, BitOr, BitXor, Mul, Rem, Shl, Shr, Sub};
use std::ptr;
use std::simd::cmp::{SimdPartialEq, SimdPartialOrd};
use std::simd::{Simd, SimdElement};
use std::sync::Arc;

pub struct VectorComparisonsInteger {}

impl VectorComparisonsInteger {
    pub fn get_vector_compare_equal<const N: usize, const E: usize, PrimitiveType: SimdElement + PartialEq + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnImmediate<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq,
    {
        let immediate_value = scan_constraint.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = Simd::splat(unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) });

        Some(Arc::new(move |current_values_ptr| {
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });

            VectorGenerics::transmute_mask(current_values.simd_eq(immediate_value))
        }))
    }

    pub fn get_vector_compare_not_equal<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnImmediate<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq,
    {
        let immediate_value = scan_constraint.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = Simd::splat(unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) });

        Some(Arc::new(move |current_values_ptr| {
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });

            VectorGenerics::transmute_mask(current_values.simd_ne(immediate_value))
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

            VectorGenerics::transmute_mask(current_values.simd_le(immediate_value))
        }))
    }

    pub fn get_vector_compare_changed<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        _scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnRelative<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq,
    {
        Some(Arc::new(move |current_values_ptr, previous_values_ptr| unsafe {
            let current_values = Simd::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]));
            let previous_values = Simd::from_array(ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E]));

            VectorGenerics::transmute_mask(current_values.simd_ne(previous_values))
        }))
    }

    pub fn get_vector_compare_unchanged<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        _scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnRelative<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq,
    {
        Some(Arc::new(move |current_values_ptr, previous_values_ptr| unsafe {
            let current_values = Simd::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]));
            let previous_values = Simd::from_array(ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E]));

            VectorGenerics::transmute_mask(current_values.simd_eq(previous_values))
        }))
    }

    pub fn get_vector_compare_increased<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        _scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnRelative<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialOrd,
    {
        Some(Arc::new(move |current_values_ptr, previous_values_ptr| unsafe {
            let current_values = Simd::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]));
            let previous_values = Simd::from_array(ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E]));

            VectorGenerics::transmute_mask(current_values.simd_gt(previous_values))
        }))
    }

    pub fn get_vector_compare_decreased<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        _scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnRelative<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialOrd,
    {
        Some(Arc::new(move |current_values_ptr, previous_values_ptr| unsafe {
            let current_values = Simd::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]));
            let previous_values = Simd::from_array(ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E]));

            VectorGenerics::transmute_mask(current_values.simd_lt(previous_values))
        }))
    }

    pub fn get_vector_compare_increased_by<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + Add<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = Simd::splat(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) });

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });
            let previous_values = Simd::from_array(unsafe { ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E]) });
            let target_values = previous_values.add(delta_value);

            VectorGenerics::transmute_mask(current_values.simd_eq(target_values))
        }))
    }

    pub fn get_vector_compare_decreased_by<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + Sub<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = Simd::splat(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) });

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });
            let previous_values = Simd::from_array(unsafe { ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E]) });
            let target_values = previous_values.sub(delta_value);

            VectorGenerics::transmute_mask(current_values.simd_eq(target_values))
        }))
    }

    pub fn get_vector_compare_multiplied_by<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + Mul<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = Simd::splat(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) });

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });
            let previous_values = Simd::from_array(unsafe { ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E]) });
            let target_values = previous_values.mul(delta_value);

            VectorGenerics::transmute_mask(current_values.simd_eq(target_values))
        }))
    }

    pub fn get_vector_compare_divided_by<const N: usize, const E: usize, PrimitiveType: SimdElement + PartialEq + Default + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + Mul<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let primitive_value = unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) };

        // Disallow divide by zero.
        if primitive_value == PrimitiveType::default() {
            return None;
        }

        let delta_value = Simd::splat(primitive_value);

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });
            let previous_values = Simd::from_array(unsafe { ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E]) });
            let target_values = previous_values.mul(delta_value);

            VectorGenerics::transmute_mask(current_values.simd_eq(target_values))
        }))
    }

    pub fn get_vector_compare_modulo_by<const N: usize, const E: usize, PrimitiveType: SimdElement + PartialEq + Default + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + Rem<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let primitive_value = unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) };

        // Disallow divide by zero.
        if primitive_value == PrimitiveType::default() {
            return None;
        }

        let delta_value = Simd::splat(primitive_value);

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });
            let previous_values = Simd::from_array(unsafe { ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E]) });
            let target_values = previous_values.rem(delta_value);

            VectorGenerics::transmute_mask(current_values.simd_eq(target_values))
        }))
    }

    pub fn get_vector_compare_shift_left_by<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + Shl<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = Simd::splat(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) });

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });
            let previous_values = Simd::from_array(unsafe { ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E]) });
            let target_values = previous_values.shl(delta_value);

            VectorGenerics::transmute_mask(current_values.simd_eq(target_values))
        }))
    }

    pub fn get_vector_compare_shift_right_by<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + Shr<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = Simd::splat(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) });

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });
            let previous_values = Simd::from_array(unsafe { ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E]) });
            let target_values = previous_values.shr(delta_value);

            VectorGenerics::transmute_mask(current_values.simd_eq(target_values))
        }))
    }

    pub fn get_vector_compare_logical_and_by<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + BitAnd<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = Simd::splat(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) });

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });
            let previous_values = Simd::from_array(unsafe { ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E]) });
            let target_values = previous_values.bitand(delta_value);

            VectorGenerics::transmute_mask(current_values.simd_eq(target_values))
        }))
    }

    pub fn get_vector_compare_logical_or_by<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + BitOr<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = Simd::splat(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) });

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });
            let previous_values = Simd::from_array(unsafe { ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E]) });
            let target_values = previous_values.bitor(delta_value);

            VectorGenerics::transmute_mask(current_values.simd_eq(target_values))
        }))
    }

    pub fn get_vector_compare_logical_xor_by<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + BitXor<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = Simd::splat(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) });

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });
            let previous_values = Simd::from_array(unsafe { ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E]) });
            let target_values = previous_values.bitxor(delta_value);

            VectorGenerics::transmute_mask(current_values.simd_eq(target_values))
        }))
    }
}
