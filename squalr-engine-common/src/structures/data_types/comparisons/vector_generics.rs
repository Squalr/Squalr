use std::ops::{Add, Sub};
use std::simd::SimdElement;
use std::simd::{LaneCount, Simd, SupportedLaneCount, cmp::SimdPartialEq, cmp::SimdPartialOrd};
use std::{mem, ptr};

pub struct VectorGenerics {}

impl VectorGenerics {
    pub fn get_vector_compare_equal<T, const N: usize>() -> fn(*const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
        T: SimdElement,
        Simd<T, N>: SimdPartialEq,
    {
        |current_values_ptr: *const u8, immediate_ptr: *const u8| unsafe {
            let current_values: Simd<T, N> = Simd::<T, N>::from_array(ptr::read_unaligned(current_values_ptr as *const [T; N]));
            let immediate_value = Simd::<T, N>::splat(ptr::read_unaligned(immediate_ptr as *const T));
            Self::safe_transmute::<T, N>(&current_values.simd_eq(immediate_value))
        }
    }

    pub fn get_vector_compare_not_equal<T, const N: usize>() -> fn(*const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
        T: SimdElement,
        Simd<T, N>: SimdPartialEq,
    {
        |current_values_ptr: *const u8, immediate_ptr: *const u8| unsafe {
            let current_values = Simd::<T, N>::from_array(ptr::read_unaligned(current_values_ptr as *const [T; N]));
            let immediate_value = Simd::<T, N>::splat(ptr::read_unaligned(immediate_ptr as *const T));
            Self::safe_transmute::<T, N>(&current_values.simd_ne(immediate_value))
        }
    }

    pub fn get_vector_compare_greater_than<T, const N: usize>() -> fn(*const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
        T: SimdElement,
        Simd<T, N>: SimdPartialEq + SimdPartialOrd,
    {
        |current_values_ptr: *const u8, immediate_ptr: *const u8| unsafe {
            let current_values = Simd::<T, N>::from_array(ptr::read_unaligned(current_values_ptr as *const [T; N]));
            let immediate_value = Simd::<T, N>::splat(ptr::read_unaligned(immediate_ptr as *const T));
            Self::safe_transmute::<T, N>(&current_values.simd_gt(immediate_value))
        }
    }

    pub fn get_vector_compare_greater_than_or_equal<T, const N: usize>() -> fn(*const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
        T: SimdElement,
        Simd<T, N>: SimdPartialEq + SimdPartialOrd,
    {
        |current_values_ptr: *const u8, immediate_ptr: *const u8| unsafe {
            let current_values = Simd::<T, N>::from_array(ptr::read_unaligned(current_values_ptr as *const [T; N]));
            let immediate_value = Simd::<T, N>::splat(ptr::read_unaligned(immediate_ptr as *const T));
            Self::safe_transmute::<T, N>(&current_values.simd_ge(immediate_value))
        }
    }

    pub fn get_vector_compare_less_than<T, const N: usize>() -> fn(*const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
        T: SimdElement,
        Simd<T, N>: SimdPartialEq + SimdPartialOrd,
    {
        |current_values_ptr: *const u8, immediate_ptr: *const u8| unsafe {
            let current_values = Simd::<T, N>::from_array(ptr::read_unaligned(current_values_ptr as *const [T; N]));
            let immediate_value = Simd::<T, N>::splat(ptr::read_unaligned(immediate_ptr as *const T));
            Self::safe_transmute::<T, N>(&current_values.simd_lt(immediate_value))
        }
    }

    pub fn get_vector_compare_less_than_or_equal<T, const N: usize>() -> fn(*const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
        T: SimdElement,
        Simd<T, N>: SimdPartialEq + SimdPartialOrd,
    {
        |current_values_ptr: *const u8, immediate_ptr: *const u8| unsafe {
            let current_values = Simd::<T, N>::from_array(ptr::read_unaligned(current_values_ptr as *const [T; N]));
            let immediate_value = Simd::<T, N>::splat(ptr::read_unaligned(immediate_ptr as *const T));
            Self::safe_transmute::<T, N>(&current_values.simd_le(immediate_value))
        }
    }

    pub fn get_vector_compare_changed<T, const N: usize>() -> fn(*const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
        T: SimdElement,
        Simd<T, N>: SimdPartialEq,
    {
        |current_values_ptr: *const u8, previous_values_ptr: *const u8| unsafe {
            let current_values = Simd::<T, N>::from_array(ptr::read_unaligned(current_values_ptr as *const [T; N]));
            let previous_values = Simd::<T, N>::from_array(ptr::read_unaligned(previous_values_ptr as *const [T; N]));
            Self::safe_transmute::<T, N>(&current_values.simd_ne(previous_values))
        }
    }

    pub fn get_vector_compare_unchanged<T, const N: usize>() -> fn(*const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
        T: SimdElement,
        Simd<T, N>: SimdPartialEq,
    {
        |current_values_ptr: *const u8, previous_values_ptr: *const u8| unsafe {
            let current_values = Simd::<T, N>::from_array(ptr::read_unaligned(current_values_ptr as *const [T; N]));
            let previous_values = Simd::<T, N>::from_array(ptr::read_unaligned(previous_values_ptr as *const [T; N]));
            Self::safe_transmute::<T, N>(&current_values.simd_eq(previous_values))
        }
    }

    pub fn get_vector_compare_increased<T, const N: usize>() -> fn(*const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
        T: SimdElement,
        Simd<T, N>: SimdPartialEq + SimdPartialOrd,
    {
        |current_values_ptr: *const u8, previous_values_ptr: *const u8| unsafe {
            let current_values = Simd::<T, N>::from_array(ptr::read_unaligned(current_values_ptr as *const [T; N]));
            let previous_values = Simd::<T, N>::from_array(ptr::read_unaligned(previous_values_ptr as *const [T; N]));
            Self::safe_transmute::<T, N>(&current_values.simd_gt(previous_values))
        }
    }

    pub fn get_vector_compare_decreased<T, const N: usize>() -> fn(*const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
        T: SimdElement,
        Simd<T, N>: SimdPartialEq + SimdPartialOrd,
    {
        |current_values_ptr: *const u8, previous_values_ptr: *const u8| unsafe {
            let current_values = Simd::<T, N>::from_array(ptr::read_unaligned(current_values_ptr as *const [T; N]));
            let previous_values = Simd::<T, N>::from_array(ptr::read_unaligned(previous_values_ptr as *const [T; N]));
            Self::safe_transmute::<T, N>(&current_values.simd_lt(previous_values))
        }
    }

    pub fn get_vector_compare_increased_by<T, const N: usize>() -> fn(*const u8, *const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
        T: SimdElement,
        Simd<T, N>: SimdPartialEq + Add<Output = Simd<T, N>>,
    {
        |current_values_ptr: *const u8, previous_values_ptr: *const u8, delta_ptr: *const u8| unsafe {
            let current_values = Simd::<T, N>::from_array(ptr::read_unaligned(current_values_ptr as *const [T; N]));
            let previous_values = Simd::<T, N>::from_array(ptr::read_unaligned(previous_values_ptr as *const [T; N]));
            let delta_value = Simd::<T, N>::splat(ptr::read_unaligned(delta_ptr as *const T));
            Self::safe_transmute::<T, N>(&current_values.simd_eq(previous_values + delta_value))
        }
    }

    pub fn get_vector_compare_decreased_by<T, const N: usize>() -> fn(*const u8, *const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
        T: SimdElement,
        Simd<T, N>: SimdPartialEq + Sub<Output = Simd<T, N>>,
    {
        |current_values_ptr: *const u8, previous_values_ptr: *const u8, delta_ptr: *const u8| unsafe {
            let current_values = Simd::<T, N>::from_array(ptr::read_unaligned(current_values_ptr as *const [T; N]));
            let previous_values = Simd::<T, N>::from_array(ptr::read_unaligned(previous_values_ptr as *const [T; N]));
            let delta_value = Simd::<T, N>::splat(ptr::read_unaligned(delta_ptr as *const T));
            Self::safe_transmute::<T, N>(&current_values.simd_eq(previous_values - delta_value))
        }
    }

    pub fn safe_transmute<T, const N: usize>(value: &<Simd<T, N> as SimdPartialEq>::Mask) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
        T: SimdElement,
        Simd<T, N>: SimdPartialEq,
    {
        let mut result_array = [0u8; N];
        let value_ptr = value as *const _ as *const u8;
        unsafe {
            ptr::copy_nonoverlapping(value_ptr, result_array.as_mut_ptr(), mem::size_of_val(value));
        }

        Simd::<u8, N>::from_array(result_array)
    }
}
