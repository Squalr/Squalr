use crate::scanners::comparers::vector::types::simd_type::SimdType;
use crate::scanners::parameters::scan_compare_type::ScanCompareType;
use squalr_engine_common::values::data_type::DataType;
use std::marker::PhantomData;
use std::simd::cmp::{SimdPartialEq, SimdPartialOrd};
use std::simd::{LaneCount, Mask, Simd, SimdElement, SupportedLaneCount};

pub struct ScannerVectorComparer<T: SimdElement + SimdType, const N: usize>
where
    LaneCount<N>: SupportedLaneCount,
{
    _marker: std::marker::PhantomData<T>,
}

enum CompareFunc<T, const N: usize>
where
    LaneCount<N>: SupportedLaneCount,
{
    Immediate(fn(*const u8, *const u8) -> Simd<u8, N>),
    Relative(fn(*const u8, *const u8) -> Simd<u8, N>),
    RelativeDelta(fn(*const u8, *const u8, *const u8) -> Simd<u8, N>),
    _Marker(PhantomData<T>),
}

impl<T: SimdElement + SimdType, const N: usize> ScannerVectorComparer<T, N>
where
    LaneCount<N>: SupportedLaneCount,
    Simd<T, N>: SimdPartialEq<Mask = Mask<i8, N>> + SimdPartialOrd<Mask = Mask<i8, N>>, 
{
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }

    pub fn get_immediate_compare_func(
        &self,
        scan_compare_type: ScanCompareType,
        data_type: &DataType,
    ) -> CompareFunc<T, N> {
        match scan_compare_type {
            ScanCompareType::Equal => CompareFunc::Immediate(Self::compare_equal),
            ScanCompareType::NotEqual => CompareFunc::Immediate(Self::compare_not_equal),
            ScanCompareType::GreaterThan => CompareFunc::Immediate(Self::compare_greater_than),
            ScanCompareType::GreaterThanOrEqual => CompareFunc::Immediate(Self::compare_greater_than_or_equal),
            ScanCompareType::LessThan => CompareFunc::Immediate(Self::compare_less_than),
            ScanCompareType::LessThanOrEqual => CompareFunc::Immediate(Self::compare_less_than_or_equal),
            _ => panic!("Unsupported type passed to get_immediate_compare_func"),
        }
    }

    pub fn get_relative_compare_func(
        &self,
        scan_compare_type: ScanCompareType,
        data_type: &DataType,
    ) -> CompareFunc<T, N> {
        match scan_compare_type {
            ScanCompareType::Changed => CompareFunc::Relative(Self::get_compare_changed),
            ScanCompareType::Unchanged => CompareFunc::Relative(Self::get_compare_unchanged),
            ScanCompareType::Increased => CompareFunc::Relative(Self::get_compare_increased),
            ScanCompareType::Decreased => CompareFunc::Relative(Self::get_compare_decreased),
            _ => panic!("Unsupported type passed to get_relative_compare_func"),
        }
    }

    pub fn get_relative_delta_compare_func(
        &self,
        scan_compare_type: ScanCompareType,
        data_type: &DataType,
    ) -> CompareFunc<T, N> {
        match scan_compare_type {
            ScanCompareType::IncreasedByX => CompareFunc::RelativeDelta(Self::get_compare_increased_by),
            ScanCompareType::DecreasedByX => CompareFunc::RelativeDelta(Self::get_compare_decreased_by),
            _ => panic!("Unsupported type passed to get_relative_delta_compare_func"),
        }
    }

    fn immediate_simd_compare(
        current_values_ptr: *const u8,
        immediate_ptr: *const u8,
        simd_fn: fn(Simd<T, N>, Simd<T, N>) -> Simd<u8, N>,
    ) -> Simd<u8, N> {
        let immediate_value = unsafe { Simd::<T, N>::splat(*(immediate_ptr as *const T)) };
        let current_values = unsafe { Simd::<T, N>::from_array(*(current_values_ptr as *const [T; N])) };
        simd_fn(current_values, immediate_value)
    }

    fn relative_simd_compare(
        current_values_ptr: *const u8,
        previous_values_ptr: *const u8,
        simd_fn: fn(Simd<T, N>, Simd<T, N>) -> Simd<u8, N>,
    ) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        let current_values = unsafe { Simd::<T, N>::from_array(*(current_values_ptr as *const [T; N])) };
        let previous_values = unsafe { Simd::<T, N>::from_array(*(previous_values_ptr as *const [T; N])) };
        simd_fn(current_values, previous_values)
    }

    fn relative_delta_simd_compare(
        current_values_ptr: *const u8,
        previous_values_ptr: *const u8,
        delta_ptr: *const u8,
        simd_op: fn(Simd<T, N>, Simd<T, N>) -> Simd<T, N>,
        simd_fn: fn(Simd<T, N>, Simd<T, N>) -> Simd<u8, N>,
    ) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        let current_values = unsafe { Simd::<T, N>::from_array(*(current_values_ptr as *const [T; N])) };
        let previous_values = unsafe { Simd::<T, N>::from_array(*(previous_values_ptr as *const [T; N])) };
        let delta_value = unsafe { Simd::<T, N>::splat(*(delta_ptr as *const T)) };
        simd_fn(current_values, simd_op(previous_values, delta_value))
    }

    fn mask_to_u8(mask: Mask<i8, N>) -> Simd<u8, N> {
        mask.select(Simd::splat(255), Simd::splat(0))
    }

    fn compare_equal(
        current_values_ptr: *const u8,
        immediate_ptr: *const u8
    ) -> Simd<u8, N> {
        let current_values = unsafe { Simd::<T, N>::from_array(*(current_values_ptr as *const [T; N])) };
        let immediate_value = unsafe { Simd::<T, N>::splat(*(immediate_ptr as *const T)) };
        Self::mask_to_u8(current_values.simd_eq(immediate_value))
    }

    fn compare_not_equal(
        current_values_ptr: *const u8,
        immediate_ptr: *const u8
    ) -> Simd<u8, N> {
        let current_values = unsafe { Simd::<T, N>::from_array(*(current_values_ptr as *const [T; N])) };
        let immediate_value = unsafe { Simd::<T, N>::splat(*(immediate_ptr as *const T)) };
        Self::mask_to_u8(current_values.simd_ne(immediate_value))
    }

    fn compare_greater_than(
        current_values_ptr: *const u8,
        immediate_ptr: *const u8
    ) -> Simd<u8, N> {
        let current_values = unsafe { Simd::<T, N>::from_array(*(current_values_ptr as *const [T; N])) };
        let immediate_value = unsafe { Simd::<T, N>::splat(*(immediate_ptr as *const T)) };
        Self::mask_to_u8(current_values.simd_gt(immediate_value))
    }

    fn compare_greater_than_or_equal(
        current_values_ptr: *const u8,
        immediate_ptr: *const u8
    ) -> Simd<u8, N> {
        let current_values = unsafe { Simd::<T, N>::from_array(*(current_values_ptr as *const [T; N])) };
        let immediate_value = unsafe { Simd::<T, N>::splat(*(immediate_ptr as *const T)) };
        Self::mask_to_u8(current_values.simd_ge(immediate_value))
    }

    fn compare_less_than(
        current_values_ptr: *const u8,
        immediate_ptr: *const u8
    ) -> Simd<u8, N> {
        let current_values = unsafe { Simd::<T, N>::from_array(*(current_values_ptr as *const [T; N])) };
        let immediate_value = unsafe { Simd::<T, N>::splat(*(immediate_ptr as *const T)) };
        Self::mask_to_u8(current_values.simd_lt(immediate_value))
    }

    fn compare_less_than_or_equal(
        current_values_ptr: *const u8,
        immediate_ptr: *const u8
    ) -> Simd<u8, N> {
        let current_values = unsafe { Simd::<T, N>::from_array(*(current_values_ptr as *const [T; N])) };
        let immediate_value = unsafe { Simd::<T, N>::splat(*(immediate_ptr as *const T)) };
        Self::mask_to_u8(current_values.simd_le(immediate_value))
    }

    fn get_compare_changed(
        current_values_ptr: *const u8,
        previous_values_ptr: *const u8
    ) -> Simd<u8, N> {
        let current_values = unsafe { Simd::<T, N>::from_array(*(current_values_ptr as *const [T; N])) };
        let previous_values = unsafe { Simd::<T, N>::from_array(*(previous_values_ptr as *const [T; N])) };
        Self::mask_to_u8(current_values.simd_ne(previous_values))
    }

    fn get_compare_unchanged(
        current_values_ptr: *const u8,
        previous_values_ptr: *const u8
    ) -> Simd<u8, N> {
        let current_values = unsafe { Simd::<T, N>::from_array(*(current_values_ptr as *const [T; N])) };
        let previous_values = unsafe { Simd::<T, N>::from_array(*(previous_values_ptr as *const [T; N])) };
        Self::mask_to_u8(current_values.simd_eq(previous_values))
    }

    fn get_compare_increased(
        current_values_ptr: *const u8,
        previous_values_ptr: *const u8
    ) -> Simd<u8, N> {
        let current_values = unsafe { Simd::<T, N>::from_array(*(current_values_ptr as *const [T; N])) };
        let previous_values = unsafe { Simd::<T, N>::from_array(*(previous_values_ptr as *const [T; N])) };
        Self::mask_to_u8(current_values.simd_gt(previous_values))
    }

    fn get_compare_decreased(
        current_values_ptr: *const u8,
        previous_values_ptr: *const u8
    ) -> Simd<u8, N> {
        let current_values = unsafe { Simd::<T, N>::from_array(*(current_values_ptr as *const [T; N])) };
        let previous_values = unsafe { Simd::<T, N>::from_array(*(previous_values_ptr as *const [T; N])) };
        Self::mask_to_u8(current_values.simd_lt(previous_values))
    }

    fn get_compare_increased_by(
        current_values_ptr: *const u8,
        previous_values_ptr: *const u8,
        delta_ptr: *const u8,
    ) -> Simd<u8, N> {
        let current_values = unsafe { Simd::<T, N>::from_array(*(current_values_ptr as *const [T; N])) };
        let previous_values = unsafe { Simd::<T, N>::from_array(*(previous_values_ptr as *const [T; N])) };
        let delta_value = unsafe { Simd::<T, N>::splat(*(delta_ptr as *const T)) };
        let expected_values = previous_values + delta_value;
        Self::mask_to_u8(current_values.simd_eq(expected_values))
    }

    fn get_compare_decreased_by(
        current_values_ptr: *const u8,
        previous_values_ptr: *const u8,
        delta_ptr: *const u8,
    ) -> Simd<u8, N> {
        let current_values = unsafe { Simd::<T, N>::from_array(*(current_values_ptr as *const [T; N])) };
        let previous_values = unsafe { Simd::<T, N>::from_array(*(previous_values_ptr as *const [T; N])) };
        let delta_value = unsafe { Simd::<T, N>::splat(*(delta_ptr as *const T)) };
        let expected_values = previous_values - delta_value;
        Self::mask_to_u8(current_values.simd_eq(expected_values))
    }
}
