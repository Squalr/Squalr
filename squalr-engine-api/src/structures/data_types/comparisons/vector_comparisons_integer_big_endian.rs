use crate::structures::data_types::generics::vector_generics::VectorGenerics;
use crate::structures::scanning::parameters::mapped_scan_parameters::ScanParametersCommon;
use std::ops::{Add, Sub};
use std::ptr;
use std::simd::cmp::{SimdPartialEq, SimdPartialOrd};
use std::simd::num::{SimdInt, SimdUint};
use std::simd::{LaneCount, Simd, SimdElement, SupportedLaneCount};

pub struct VectorComparisonsIntegerBigEndian {}

impl VectorComparisonsIntegerBigEndian {
    pub fn get_vector_compare_equal<const N: usize, const E: usize, PrimitiveType: SimdElement + 'static>(
        scan_parameters: &ScanParametersCommon
    ) -> Option<Box<dyn Fn(*const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
        LaneCount<E>: SupportedLaneCount,
        Simd<PrimitiveType, E>: SimdPartialEq,
    {
        // Optimization: no endian byte swap required for immediate or current values.
        let immediate_value = scan_parameters.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = Simd::splat(unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) });

        Some(Box::new(move |current_values_ptr| {
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });

            VectorGenerics::transmute_mask(current_values.simd_eq(immediate_value))
        }))
    }

    pub fn get_vector_compare_not_equal<const N: usize, const E: usize, PrimitiveType: SimdElement + 'static>(
        scan_parameters: &ScanParametersCommon
    ) -> Option<Box<dyn Fn(*const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
        LaneCount<E>: SupportedLaneCount,
        Simd<PrimitiveType, E>: SimdPartialEq,
    {
        // Optimization: no endian byte swap required for immediate or current values.
        let immediate_value = scan_parameters.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = Simd::splat(unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) });

        Some(Box::new(move |current_values_ptr| {
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });

            VectorGenerics::transmute_mask(current_values.simd_ne(immediate_value))
        }))
    }

    pub fn get_vector_compare_greater_than<const N: usize, const E: usize, PrimitiveType: SimdElement + 'static>(
        scan_parameters: &ScanParametersCommon
    ) -> Option<Box<dyn Fn(*const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
        LaneCount<E>: SupportedLaneCount,
        Simd<PrimitiveType, E>: SimdPartialOrd + SimdInt,
    {
        let immediate_value = scan_parameters.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = SimdInt::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) }));

        Some(Box::new(move |current_values_ptr| {
            let current_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));

            VectorGenerics::transmute_mask(current_values.simd_gt(immediate_value))
        }))
    }

    pub fn get_vector_compare_greater_than_unsigned<const N: usize, const E: usize, PrimitiveType: SimdElement + 'static>(
        scan_parameters: &ScanParametersCommon
    ) -> Option<Box<dyn Fn(*const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
        LaneCount<E>: SupportedLaneCount,
        Simd<PrimitiveType, E>: SimdPartialOrd + SimdUint,
    {
        let immediate_value = scan_parameters.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = SimdUint::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) }));

        Some(Box::new(move |current_values_ptr| {
            let current_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));

            VectorGenerics::transmute_mask(current_values.simd_gt(immediate_value))
        }))
    }

    pub fn get_vector_compare_greater_than_or_equal<const N: usize, const E: usize, PrimitiveType: SimdElement + 'static>(
        scan_parameters: &ScanParametersCommon
    ) -> Option<Box<dyn Fn(*const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
        LaneCount<E>: SupportedLaneCount,
        Simd<PrimitiveType, E>: SimdPartialOrd + SimdInt,
    {
        let immediate_value = scan_parameters.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = SimdInt::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) }));

        Some(Box::new(move |current_values_ptr| {
            let current_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));

            VectorGenerics::transmute_mask(current_values.simd_ge(immediate_value))
        }))
    }

    pub fn get_vector_compare_greater_than_or_equal_unsigned<const N: usize, const E: usize, PrimitiveType: SimdElement + 'static>(
        scan_parameters: &ScanParametersCommon
    ) -> Option<Box<dyn Fn(*const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
        LaneCount<E>: SupportedLaneCount,
        Simd<PrimitiveType, E>: SimdPartialOrd + SimdUint,
    {
        let immediate_value = scan_parameters.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = SimdUint::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) }));

        Some(Box::new(move |current_values_ptr| {
            let current_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));

            VectorGenerics::transmute_mask(current_values.simd_ge(immediate_value))
        }))
    }

    pub fn get_vector_compare_less_than<const N: usize, const E: usize, PrimitiveType: SimdElement + 'static>(
        scan_parameters: &ScanParametersCommon
    ) -> Option<Box<dyn Fn(*const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
        LaneCount<E>: SupportedLaneCount,
        Simd<PrimitiveType, E>: SimdPartialOrd + SimdInt,
    {
        let immediate_value = scan_parameters.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = SimdInt::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) }));

        Some(Box::new(move |current_values_ptr| {
            let current_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));

            VectorGenerics::transmute_mask(current_values.simd_lt(immediate_value))
        }))
    }

    pub fn get_vector_compare_less_than_unsigned<const N: usize, const E: usize, PrimitiveType: SimdElement + 'static>(
        scan_parameters: &ScanParametersCommon
    ) -> Option<Box<dyn Fn(*const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
        LaneCount<E>: SupportedLaneCount,
        Simd<PrimitiveType, E>: SimdPartialOrd + SimdUint,
    {
        let immediate_value = scan_parameters.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = SimdUint::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) }));

        Some(Box::new(move |current_values_ptr| {
            let current_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));

            VectorGenerics::transmute_mask(current_values.simd_lt(immediate_value))
        }))
    }

    pub fn get_vector_compare_less_than_or_equal<const N: usize, const E: usize, PrimitiveType: SimdElement + 'static>(
        scan_parameters: &ScanParametersCommon
    ) -> Option<Box<dyn Fn(*const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
        LaneCount<E>: SupportedLaneCount,
        Simd<PrimitiveType, E>: SimdPartialOrd + SimdInt,
    {
        let immediate_value = scan_parameters.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = SimdInt::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) }));

        Some(Box::new(move |current_values_ptr| {
            let current_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));

            VectorGenerics::transmute_mask(current_values.simd_le(immediate_value))
        }))
    }

    pub fn get_vector_compare_less_than_or_equal_unsigned<const N: usize, const E: usize, PrimitiveType: SimdElement + 'static>(
        scan_parameters: &ScanParametersCommon
    ) -> Option<Box<dyn Fn(*const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
        LaneCount<E>: SupportedLaneCount,
        Simd<PrimitiveType, E>: SimdPartialOrd + SimdUint,
    {
        let immediate_value = scan_parameters.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = SimdUint::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) }));

        Some(Box::new(move |current_values_ptr| {
            let current_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));

            VectorGenerics::transmute_mask(current_values.simd_le(immediate_value))
        }))
    }

    pub fn get_vector_compare_changed<const N: usize, const E: usize, PrimitiveType: SimdElement + 'static>(
        _scan_parameters: &ScanParametersCommon
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
        LaneCount<E>: SupportedLaneCount,
        Simd<PrimitiveType, E>: SimdPartialEq,
    {
        Some(Box::new(move |current_values_ptr, previous_values_ptr| {
            // Optimization: no endian byte swap required.
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });
            let previous_values = Simd::from_array(unsafe { ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E]) });

            VectorGenerics::transmute_mask(current_values.simd_ne(previous_values))
        }))
    }

    pub fn get_vector_compare_unchanged<const N: usize, const E: usize, PrimitiveType: SimdElement + 'static>(
        _scan_parameters: &ScanParametersCommon
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
        LaneCount<E>: SupportedLaneCount,
        Simd<PrimitiveType, E>: SimdPartialEq,
    {
        Some(Box::new(move |current_values_ptr, previous_values_ptr| {
            // Optimization: no endian byte swap required.
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });
            let previous_values = Simd::from_array(unsafe { ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E]) });

            VectorGenerics::transmute_mask(current_values.simd_eq(previous_values))
        }))
    }

    pub fn get_vector_compare_increased<const N: usize, const E: usize, PrimitiveType: SimdElement + 'static>(
        _scan_parameters: &ScanParametersCommon
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
        LaneCount<E>: SupportedLaneCount,
        Simd<PrimitiveType, E>: SimdPartialOrd + SimdInt,
    {
        Some(Box::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));
            let previous_values: Simd<PrimitiveType, E> =
                SimdInt::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(previous_values_ptr as *const PrimitiveType) }));

            VectorGenerics::transmute_mask(current_values.simd_gt(previous_values))
        }))
    }

    pub fn get_vector_compare_increased_unsigned<const N: usize, const E: usize, PrimitiveType: SimdElement + 'static>(
        _scan_parameters: &ScanParametersCommon
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
        LaneCount<E>: SupportedLaneCount,
        Simd<PrimitiveType, E>: SimdPartialOrd + SimdUint,
    {
        Some(Box::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));
            let previous_values: Simd<PrimitiveType, E> =
                SimdUint::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(previous_values_ptr as *const PrimitiveType) }));

            VectorGenerics::transmute_mask(current_values.simd_gt(previous_values))
        }))
    }

    pub fn get_vector_compare_decreased<const N: usize, const E: usize, PrimitiveType: SimdElement + 'static>(
        _scan_parameters: &ScanParametersCommon
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
        LaneCount<E>: SupportedLaneCount,
        Simd<PrimitiveType, E>: SimdPartialOrd + SimdInt,
    {
        Some(Box::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));
            let previous_values: Simd<PrimitiveType, E> =
                SimdInt::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(previous_values_ptr as *const PrimitiveType) }));

            VectorGenerics::transmute_mask(current_values.simd_lt(previous_values))
        }))
    }

    pub fn get_vector_compare_decreased_unsigned<const N: usize, const E: usize, PrimitiveType: SimdElement + 'static>(
        _scan_parameters: &ScanParametersCommon
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
        LaneCount<E>: SupportedLaneCount,
        Simd<PrimitiveType, E>: SimdPartialOrd + SimdUint,
    {
        Some(Box::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));
            let previous_values: Simd<PrimitiveType, E> =
                SimdUint::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(previous_values_ptr as *const PrimitiveType) }));

            VectorGenerics::transmute_mask(current_values.simd_lt(previous_values))
        }))
    }

    pub fn get_vector_compare_increased_by<const N: usize, const E: usize, PrimitiveType: SimdElement + 'static>(
        scan_parameters: &ScanParametersCommon
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
        LaneCount<E>: SupportedLaneCount,
        Simd<PrimitiveType, E>: SimdPartialEq + SimdInt + Add<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_parameters.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = SimdInt::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) }));

        Some(Box::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));
            let previous_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E])
            }));

            VectorGenerics::transmute_mask(current_values.simd_eq(previous_values.add(delta_value)))
        }))
    }

    pub fn get_vector_compare_increased_by_unsigned<const N: usize, const E: usize, PrimitiveType: SimdElement + 'static>(
        scan_parameters: &ScanParametersCommon
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
        LaneCount<E>: SupportedLaneCount,
        Simd<PrimitiveType, E>: SimdPartialEq + SimdUint + Add<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_parameters.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = SimdUint::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) }));

        Some(Box::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));
            let previous_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E])
            }));

            VectorGenerics::transmute_mask(current_values.simd_eq(previous_values.add(delta_value)))
        }))
    }

    pub fn get_vector_compare_decreased_by<const N: usize, const E: usize, PrimitiveType: SimdElement + 'static>(
        scan_parameters: &ScanParametersCommon
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
        LaneCount<E>: SupportedLaneCount,
        Simd<PrimitiveType, E>: SimdPartialEq + SimdInt + Sub<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_parameters.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = SimdInt::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) }));

        Some(Box::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));
            let previous_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E])
            }));

            VectorGenerics::transmute_mask(current_values.simd_eq(previous_values.sub(delta_value)))
        }))
    }

    pub fn get_vector_compare_decreased_by_unsigned<const N: usize, const E: usize, PrimitiveType: SimdElement + 'static>(
        scan_parameters: &ScanParametersCommon
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
        LaneCount<E>: SupportedLaneCount,
        Simd<PrimitiveType, E>: SimdPartialEq + SimdUint + Sub<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_parameters.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = SimdUint::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) }));

        Some(Box::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));
            let previous_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E])
            }));

            VectorGenerics::transmute_mask(current_values.simd_eq(previous_values.sub(delta_value)))
        }))
    }
}
