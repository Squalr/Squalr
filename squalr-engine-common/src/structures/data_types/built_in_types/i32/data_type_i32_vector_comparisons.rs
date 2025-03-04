use crate::structures::data_types::comparisons::vector_comparable::VectorComparable;
use std::ptr;
use std::simd::{LaneCount, Simd, SupportedLaneCount, cmp::SimdPartialEq, cmp::SimdPartialOrd};

type PrimitiveType = i32;

pub struct DataTypeI32 {}

impl VectorComparable for DataTypeI32 {
    fn get_vector_compare_equal<const N: usize>(&self) -> fn(*const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        |current_values_ptr: *const u8, immediate_ptr: *const u8| unsafe {
            let current_values: Simd<PrimitiveType, N> =
                Simd::<PrimitiveType, N>::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; N]));
            let immediate_value = Simd::<PrimitiveType, N>::splat(ptr::read_unaligned(immediate_ptr as *const PrimitiveType));
            Self::safe_transmute::<PrimitiveType, N>(&current_values.simd_eq(immediate_value))
        }
    }

    fn get_vector_compare_not_equal<const N: usize>(&self) -> fn(*const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        |current_values_ptr: *const u8, immediate_ptr: *const u8| unsafe {
            let current_values = Simd::<PrimitiveType, N>::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; N]));
            let immediate_value = Simd::<PrimitiveType, N>::splat(ptr::read_unaligned(immediate_ptr as *const PrimitiveType));
            Self::safe_transmute::<PrimitiveType, N>(&current_values.simd_ne(immediate_value))
        }
    }

    fn get_vector_compare_greater_than<const N: usize>(&self) -> fn(*const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        |current_values_ptr: *const u8, immediate_ptr: *const u8| unsafe {
            let current_values = Simd::<PrimitiveType, N>::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; N]));
            let immediate_value = Simd::<PrimitiveType, N>::splat(ptr::read_unaligned(immediate_ptr as *const PrimitiveType));
            Self::safe_transmute::<PrimitiveType, N>(&current_values.simd_gt(immediate_value))
        }
    }

    fn get_vector_compare_greater_than_or_equal<const N: usize>(&self) -> fn(*const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        |current_values_ptr: *const u8, immediate_ptr: *const u8| unsafe {
            let current_values = Simd::<PrimitiveType, N>::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; N]));
            let immediate_value = Simd::<PrimitiveType, N>::splat(ptr::read_unaligned(immediate_ptr as *const PrimitiveType));
            Self::safe_transmute::<PrimitiveType, N>(&current_values.simd_ge(immediate_value))
        }
    }

    fn get_vector_compare_less_than<const N: usize>(&self) -> fn(*const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        |current_values_ptr: *const u8, immediate_ptr: *const u8| unsafe {
            let current_values = Simd::<PrimitiveType, N>::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; N]));
            let immediate_value = Simd::<PrimitiveType, N>::splat(ptr::read_unaligned(immediate_ptr as *const PrimitiveType));
            Self::safe_transmute::<PrimitiveType, N>(&current_values.simd_lt(immediate_value))
        }
    }

    fn get_vector_compare_less_than_or_equal<const N: usize>(&self) -> fn(*const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        |current_values_ptr: *const u8, immediate_ptr: *const u8| unsafe {
            let current_values = Simd::<PrimitiveType, N>::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; N]));
            let immediate_value = Simd::<PrimitiveType, N>::splat(ptr::read_unaligned(immediate_ptr as *const PrimitiveType));
            Self::safe_transmute::<PrimitiveType, N>(&current_values.simd_le(immediate_value))
        }
    }

    fn get_vector_compare_changed<const N: usize>(&self) -> fn(*const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        |current_values_ptr: *const u8, previous_values_ptr: *const u8| unsafe {
            let current_values = Simd::<PrimitiveType, N>::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; N]));
            let previous_values = Simd::<PrimitiveType, N>::from_array(ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; N]));
            Self::safe_transmute::<PrimitiveType, N>(&current_values.simd_ne(previous_values))
        }
    }

    fn get_vector_compare_unchanged<const N: usize>(&self) -> fn(*const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        |current_values_ptr: *const u8, previous_values_ptr: *const u8| unsafe {
            let current_values = Simd::<PrimitiveType, N>::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; N]));
            let previous_values = Simd::<PrimitiveType, N>::from_array(ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; N]));
            Self::safe_transmute::<PrimitiveType, N>(&current_values.simd_eq(previous_values))
        }
    }

    fn get_vector_compare_increased<const N: usize>(&self) -> fn(*const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        |current_values_ptr: *const u8, previous_values_ptr: *const u8| unsafe {
            let current_values = Simd::<PrimitiveType, N>::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; N]));
            let previous_values = Simd::<PrimitiveType, N>::from_array(ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; N]));
            Self::safe_transmute::<PrimitiveType, N>(&current_values.simd_gt(previous_values))
        }
    }

    fn get_vector_compare_decreased<const N: usize>(&self) -> fn(*const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        |current_values_ptr: *const u8, previous_values_ptr: *const u8| unsafe {
            let current_values = Simd::<PrimitiveType, N>::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; N]));
            let previous_values = Simd::<PrimitiveType, N>::from_array(ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; N]));
            Self::safe_transmute::<PrimitiveType, N>(&current_values.simd_lt(previous_values))
        }
    }

    fn get_vector_compare_increased_by<const N: usize>(&self) -> fn(*const u8, *const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        |current_values_ptr: *const u8, previous_values_ptr: *const u8, delta_ptr: *const u8| unsafe {
            let current_values = Simd::<PrimitiveType, N>::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; N]));
            let previous_values = Simd::<PrimitiveType, N>::from_array(ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; N]));
            let delta_value = Simd::<PrimitiveType, N>::splat(ptr::read_unaligned(delta_ptr as *const PrimitiveType));
            Self::safe_transmute::<PrimitiveType, N>(&current_values.simd_eq(previous_values + delta_value))
        }
    }

    fn get_vector_compare_decreased_by<const N: usize>(&self) -> fn(*const u8, *const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        |current_values_ptr: *const u8, previous_values_ptr: *const u8, delta_ptr: *const u8| unsafe {
            let current_values = Simd::<PrimitiveType, N>::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; N]));
            let previous_values = Simd::<PrimitiveType, N>::from_array(ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; N]));
            let delta_value = Simd::<PrimitiveType, N>::splat(ptr::read_unaligned(delta_ptr as *const PrimitiveType));
            Self::safe_transmute::<PrimitiveType, N>(&current_values.simd_eq(previous_values - delta_value))
        }
    }
}
