use crate::structures::scanning::scan_compare_type_delta::ScanCompareTypeDelta;
use crate::structures::scanning::scan_compare_type_immediate::ScanCompareTypeImmediate;
use crate::structures::scanning::scan_compare_type_relative::ScanCompareTypeRelative;
use std::{
    mem, ptr,
    simd::{LaneCount, Simd, SimdElement, SupportedLaneCount, cmp::SimdPartialEq},
};

pub trait VectorComparable {
    fn get_vector_compare_equal<const N: usize>(&self) -> fn(*const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount;
    fn get_vector_compare_not_equal<const N: usize>(&self) -> fn(*const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount;
    fn get_vector_compare_greater_than<const N: usize>(&self) -> fn(*const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount;
    fn get_vector_compare_greater_than_or_equal<const N: usize>(&self) -> fn(*const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount;
    fn get_vector_compare_less_than<const N: usize>(&self) -> fn(*const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount;
    fn get_vector_compare_less_than_or_equal<const N: usize>(&self) -> fn(*const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount;

    fn get_vector_compare_changed<const N: usize>(&self) -> fn(*const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount;
    fn get_vector_compare_unchanged<const N: usize>(&self) -> fn(*const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount;
    fn get_vector_compare_increased<const N: usize>(&self) -> fn(*const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount;
    fn get_vector_compare_decreased<const N: usize>(&self) -> fn(*const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount;

    fn get_vector_compare_increased_by<const N: usize>(&self) -> fn(*const u8, *const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount;
    fn get_vector_compare_decreased_by<const N: usize>(&self) -> fn(*const u8, *const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount;

    fn get_vector_compare_function_immediate<const N: usize>(
        &self,
        scan_compare_type: ScanCompareTypeImmediate,
    ) -> impl Fn(*const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        match scan_compare_type {
            ScanCompareTypeImmediate::Equal => self.get_vector_compare_equal(),
            ScanCompareTypeImmediate::NotEqual => self.get_vector_compare_not_equal(),
            ScanCompareTypeImmediate::GreaterThan => self.get_vector_compare_greater_than(),
            ScanCompareTypeImmediate::GreaterThanOrEqual => self.get_vector_compare_greater_than_or_equal(),
            ScanCompareTypeImmediate::LessThan => self.get_vector_compare_less_than(),
            ScanCompareTypeImmediate::LessThanOrEqual => self.get_vector_compare_less_than_or_equal(),
        }
    }

    fn get_vector_relative_compare_func<const N: usize>(
        &self,
        scan_compare_type: ScanCompareTypeRelative,
    ) -> impl Fn(*const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        match scan_compare_type {
            ScanCompareTypeRelative::Changed => self.get_vector_compare_changed(),
            ScanCompareTypeRelative::Unchanged => self.get_vector_compare_unchanged(),
            ScanCompareTypeRelative::Increased => self.get_vector_compare_increased(),
            ScanCompareTypeRelative::Decreased => self.get_vector_compare_decreased(),
        }
    }

    fn get_vector_compare_function_delta<const N: usize>(
        &self,
        scan_compare_type: ScanCompareTypeDelta,
    ) -> impl Fn(*const u8, *const u8, *const u8) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        match scan_compare_type {
            ScanCompareTypeDelta::IncreasedByX => self.get_vector_compare_increased_by(),
            ScanCompareTypeDelta::DecreasedByX => self.get_vector_compare_decreased_by(),
        }
    }

    fn safe_transmute<PrimitiveType, const N: usize>(value: &<Simd<PrimitiveType, N> as SimdPartialEq>::Mask) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
        PrimitiveType: SimdElement + PartialEq,
        Simd<PrimitiveType, N>: SimdPartialEq,
    {
        let mut result_array = [0u8; N];
        let value_ptr = value as *const _ as *const u8;
        unsafe {
            ptr::copy_nonoverlapping(value_ptr, result_array.as_mut_ptr(), mem::size_of_val(value));
        }

        Simd::<u8, N>::from_array(result_array)
    }
}
