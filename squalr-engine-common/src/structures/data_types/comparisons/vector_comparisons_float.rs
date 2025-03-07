use crate::structures::data_types::generics::vector_generics::VectorGenerics;
use crate::structures::scanning::scan_parameters_global::ScanParametersGlobal;
use crate::structures::scanning::scan_parameters_local::ScanParametersLocal;
use num_traits::Float;
use std::ops::{Add, Sub};
use std::ptr;
use std::simd::cmp::{SimdPartialEq, SimdPartialOrd};
use std::simd::num::SimdFloat;
use std::simd::{LaneCount, Simd, SimdElement, SupportedLaneCount};

pub struct VectorComparisonsFloat {}

impl VectorComparisonsFloat {
    pub fn get_vector_compare_equal<const N: usize, const E: usize, PrimitiveType: SimdElement + Float + 'static>(
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<Box<dyn Fn(*const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
        LaneCount<E>: SupportedLaneCount,
        Simd<PrimitiveType, E>: SimdFloat + SimdPartialOrd + Sub<Output = Simd<PrimitiveType, E>>,
    {
        if let Some(immediate_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            unsafe {
                let tolerance: Simd<PrimitiveType, E> = Simd::splat(
                    scan_parameters_global
                        .get_floating_point_tolerance()
                        .get_value(),
                );
                let immediate_value_ptr = immediate_value.as_ptr();
                let immediate_value = Simd::splat(ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType));

                Some(Box::new(move |current_values_ptr| {
                    let current_values = Simd::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]));

                    // Equality between the current and immediate value is determined by being within the given tolerance.
                    VectorGenerics::transmute_mask(current_values.sub(immediate_value).abs().simd_le(tolerance))
                }))
            }
        } else {
            None
        }
    }

    pub fn get_vector_compare_not_equal<const N: usize, const E: usize, PrimitiveType: SimdElement + Float + 'static>(
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<Box<dyn Fn(*const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
        LaneCount<E>: SupportedLaneCount,
        Simd<PrimitiveType, E>: SimdFloat + SimdPartialOrd + Sub<Output = Simd<PrimitiveType, E>>,
    {
        if let Some(immediate_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            unsafe {
                let tolerance: Simd<PrimitiveType, E> = Simd::splat(
                    scan_parameters_global
                        .get_floating_point_tolerance()
                        .get_value(),
                );
                let immediate_value_ptr = immediate_value.as_ptr();
                let immediate_value = Simd::splat(ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType));

                Some(Box::new(move |current_values_ptr| {
                    let current_values = Simd::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]));

                    // Inequality between the current and immediate value is determined by being outside the given tolerance.
                    VectorGenerics::transmute_mask(current_values.sub(immediate_value).abs().simd_gt(tolerance))
                }))
            }
        } else {
            None
        }
    }

    pub fn get_vector_compare_greater_than<const N: usize, const E: usize, PrimitiveType: SimdElement + 'static>(
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<Box<dyn Fn(*const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
        LaneCount<E>: SupportedLaneCount,
        Simd<PrimitiveType, E>: SimdPartialOrd,
    {
        if let Some(immediate_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            unsafe {
                let immediate_value_ptr = immediate_value.as_ptr();
                let immediate_value = Simd::splat(ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType));

                Some(Box::new(move |current_values_ptr| {
                    let current_values = Simd::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]));

                    // No checks tolerance required.
                    VectorGenerics::transmute_mask(current_values.simd_gt(immediate_value))
                }))
            }
        } else {
            None
        }
    }

    pub fn get_vector_compare_greater_than_or_equal<const N: usize, const E: usize, PrimitiveType: SimdElement + 'static>(
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<Box<dyn Fn(*const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
        LaneCount<E>: SupportedLaneCount,
        Simd<PrimitiveType, E>: SimdPartialOrd,
    {
        if let Some(immediate_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            unsafe {
                let immediate_value_ptr = immediate_value.as_ptr();
                let immediate_value = Simd::splat(ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType));

                Some(Box::new(move |current_values_ptr| {
                    let current_values = Simd::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]));

                    // No checks tolerance required.
                    VectorGenerics::transmute_mask(current_values.simd_ge(immediate_value))
                }))
            }
        } else {
            None
        }
    }

    pub fn get_vector_compare_less_than<const N: usize, const E: usize, PrimitiveType: SimdElement + 'static>(
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<Box<dyn Fn(*const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
        LaneCount<E>: SupportedLaneCount,
        Simd<PrimitiveType, E>: SimdPartialOrd,
    {
        if let Some(immediate_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            unsafe {
                let immediate_value_ptr = immediate_value.as_ptr();
                let immediate_value = Simd::splat(ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType));

                Some(Box::new(move |current_values_ptr| {
                    let current_values = Simd::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]));

                    // No checks tolerance required.
                    VectorGenerics::transmute_mask(current_values.simd_lt(immediate_value))
                }))
            }
        } else {
            None
        }
    }

    pub fn get_vector_compare_less_than_or_equal<const N: usize, const E: usize, PrimitiveType: SimdElement + 'static>(
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<Box<dyn Fn(*const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
        LaneCount<E>: SupportedLaneCount,
        Simd<PrimitiveType, E>: SimdPartialOrd,
    {
        if let Some(immediate_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            unsafe {
                let immediate_value_ptr = immediate_value.as_ptr();
                let immediate_value = Simd::splat(ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType));

                Some(Box::new(move |current_values_ptr| {
                    let current_values = Simd::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]));

                    // No checks tolerance required.
                    VectorGenerics::transmute_mask(current_values.simd_le(immediate_value))
                }))
            }
        } else {
            None
        }
    }

    pub fn get_vector_compare_changed<const N: usize, const E: usize, PrimitiveType: SimdElement + 'static>(
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
        LaneCount<E>: SupportedLaneCount,
        Simd<PrimitiveType, E>: SimdPartialEq,
    {
        Some(Box::new(move |current_values_ptr, previous_values_ptr| unsafe {
            let current_values = Simd::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]));
            let previous_values = Simd::from_array(ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E]));

            // No checks tolerance required.
            VectorGenerics::transmute_mask(current_values.simd_ne(previous_values))
        }))
    }

    pub fn get_vector_compare_unchanged<const N: usize, const E: usize, PrimitiveType: SimdElement + 'static>(
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
        LaneCount<E>: SupportedLaneCount,
        Simd<PrimitiveType, E>: SimdPartialEq,
    {
        Some(Box::new(move |current_values_ptr, previous_values_ptr| unsafe {
            let current_values = Simd::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]));
            let previous_values = Simd::from_array(ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E]));

            // No checks tolerance required.
            VectorGenerics::transmute_mask(current_values.simd_eq(previous_values))
        }))
    }

    pub fn get_vector_compare_increased<const N: usize, const E: usize, PrimitiveType: SimdElement + 'static>(
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
        LaneCount<E>: SupportedLaneCount,
        Simd<PrimitiveType, E>: SimdPartialOrd,
    {
        Some(Box::new(move |current_values_ptr, previous_values_ptr| unsafe {
            let current_values = Simd::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]));
            let previous_values = Simd::from_array(ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E]));

            // No checks tolerance required.
            VectorGenerics::transmute_mask(current_values.simd_gt(previous_values))
        }))
    }

    pub fn get_vector_compare_decreased<const N: usize, const E: usize, PrimitiveType: SimdElement + 'static>(
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
        LaneCount<E>: SupportedLaneCount,
        Simd<PrimitiveType, E>: SimdPartialOrd,
    {
        Some(Box::new(move |current_values_ptr, previous_values_ptr| unsafe {
            let current_values = Simd::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]));
            let previous_values = Simd::from_array(ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E]));

            // No checks tolerance required.
            VectorGenerics::transmute_mask(current_values.simd_lt(previous_values))
        }))
    }

    pub fn get_vector_compare_increased_by<const N: usize, const E: usize, PrimitiveType: SimdElement + Float + 'static>(
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
        LaneCount<E>: SupportedLaneCount,
        Simd<PrimitiveType, E>: SimdFloat
            + SimdPartialOrd
            + Add<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>
            + Sub<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        if let Some(delta_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            unsafe {
                let tolerance: Simd<PrimitiveType, E> = Simd::splat(
                    scan_parameters_global
                        .get_floating_point_tolerance()
                        .get_value(),
                );
                let delta_value_ptr = delta_value.as_ptr();
                let delta_value = Simd::splat(ptr::read_unaligned(delta_value_ptr as *const PrimitiveType));

                Some(Box::new(move |current_values_ptr, previous_values_ptr| {
                    let current_values = Simd::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]));
                    let previous_values = Simd::from_array(ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E]));
                    let target_values = previous_values.add(delta_value);

                    // Equality between the current and target value is determined by being within the given tolerance.
                    VectorGenerics::transmute_mask(current_values.sub(target_values).abs().simd_le(tolerance))
                }))
            }
        } else {
            None
        }
    }

    pub fn get_vector_compare_decreased_by<const N: usize, const E: usize, PrimitiveType: SimdElement + Float + 'static>(
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
        LaneCount<E>: SupportedLaneCount,
        Simd<PrimitiveType, E>: SimdFloat + SimdPartialOrd + Sub<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        if let Some(delta_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            unsafe {
                let tolerance: Simd<PrimitiveType, E> = Simd::splat(
                    scan_parameters_global
                        .get_floating_point_tolerance()
                        .get_value(),
                );
                let delta_value_ptr = delta_value.as_ptr();
                let delta_value = Simd::splat(ptr::read_unaligned(delta_value_ptr as *const PrimitiveType));

                Some(Box::new(move |current_values_ptr, previous_values_ptr| {
                    let current_values = Simd::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]));
                    let previous_values = Simd::from_array(ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E]));
                    let target_values = previous_values.sub(delta_value);

                    // Equality between the current and target value is determined by being within the given tolerance.
                    VectorGenerics::transmute_mask(current_values.sub(target_values).abs().simd_le(tolerance))
                }))
            }
        } else {
            None
        }
    }
}
