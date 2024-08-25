use crate::scanners::comparers::vector::types::simd_type::SimdType;
use crate::scanners::comparers::vector::types::simd_wrapper::SimdWrapper;
use crate::scanners::parameters::scan_compare_type::ScanCompareType;
use squalr_engine_common::values::data_type::DataType;
use std::simd::cmp::{SimdPartialEq, SimdPartialOrd};
use std::simd::{LaneCount, Simd, SimdElement, SupportedLaneCount};

pub struct ScannerVectorComparer<T: SimdElement + SimdType, const N: usize>
where
    LaneCount<N>: SupportedLaneCount,
{
    _marker: std::marker::PhantomData<T>,
}

impl<T: SimdElement + SimdType, const N: usize> ScannerVectorComparer<T, N>
where
    LaneCount<N>: SupportedLaneCount,
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
    ) -> impl Fn(*const u8, *const u8) -> Simd<u8, N> {
        match scan_compare_type {
            ScanCompareType::Equal => self.get_compare_equal(data_type),
            ScanCompareType::NotEqual => self.get_compare_not_equal(data_type),
            ScanCompareType::GreaterThan => self.get_compare_greater_than(data_type),
            ScanCompareType::GreaterThanOrEqual => self.get_compare_greater_than_or_equal(data_type),
            ScanCompareType::LessThan => self.get_compare_less_than(data_type),
            ScanCompareType::LessThanOrEqual => self.get_compare_less_than_or_equal(data_type),
            _ => panic!("Unsupported type passed to get_immediate_compare_func"),
        }
    }

    pub fn get_relative_compare_func(
        &self,
        scan_compare_type: ScanCompareType,
        data_type: &DataType,
    ) -> impl Fn(*const u8, *const u8) -> Simd<u8, N> {
        match scan_compare_type {
            ScanCompareType::Changed => self.get_compare_changed(data_type),
            ScanCompareType::Unchanged => self.get_compare_unchanged(data_type),
            ScanCompareType::Increased => self.get_compare_increased(data_type),
            ScanCompareType::Decreased => self.get_compare_decreased(data_type),
            _ => panic!("Unsupported type passed to get_relative_compare_func"),
        }
    }

    pub fn get_relative_delta_compare_func(
        &self,
        scan_compare_type: ScanCompareType,
        data_type: &DataType,
    ) -> impl Fn(*const u8, *const u8, *const u8) -> Simd<u8, N> {
        match scan_compare_type {
            ScanCompareType::IncreasedByX => self.get_compare_increased_by(data_type),
            ScanCompareType::DecreasedByX => self.get_compare_decreased_by(data_type),
            _ => panic!("Unsupported type passed to get_relative_delta_compare_func"),
        }
    }

    fn immediate_simd_compare<U: SimdElement>(
        current_values_ptr: *const u8,
        immediate_ptr: *const u8,
        simd_fn: fn(Simd<U, N>, Simd<U, N>) -> Simd<u8, N>,
    ) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        unsafe {
            let immediate_value = SimdWrapper::<U, N>::splat(*(immediate_ptr as *const U));
            let current_values = SimdWrapper::<U, N>::from_array(*(current_values_ptr as *const [U; N]));
            simd_fn(current_values, immediate_value)
        }
    }

    fn relative_simd_compare(
        current_values_ptr: *const u8,
        previous_values_ptr: *const u8,
        simd_fn: fn(Simd<T, N>, Simd<T, N>) -> Simd<u8, N>,
    ) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        unsafe {
            let current_values = SimdWrapper::<T, N>::from_array(*(current_values_ptr as *const [T; N]));
            let previous_values = SimdWrapper::<T, N>::from_array(*(previous_values_ptr as *const [T; N]));
            simd_fn(current_values, previous_values)
        }
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
        unsafe {
            let current_values = SimdWrapper::<T, N>::from_array(*(current_values_ptr as *const [T; N]));
            let previous_values = SimdWrapper::<T, N>::from_array(*(previous_values_ptr as *const [T; N]));
            let delta_value = SimdWrapper::<T, N>::splat(*(delta_ptr as *const T));
            simd_fn(current_values, simd_op(previous_values, delta_value))
        }
    }

    pub fn get_compare_equal(
        &self,
        data_type: &DataType,
    ) -> impl Fn(*const u8, *const u8) -> Simd<u8, N> {
        match data_type {
            DataType::U8() => {
                let compare_fn = SimdWrapper::<u8, N>::eq as fn(Simd<u8, N>, Simd<u8, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<u8>(a, b, compare_fn)
            }
            DataType::I8() => {
                let compare_fn = SimdWrapper::<i8, N>::eq as fn(Simd<i8, N>, Simd<i8, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<i8>(a, b, compare_fn)
            }
            DataType::U16(_) => {
                let compare_fn = SimdWrapper::<u16, N>::eq as fn(Simd<u16, N>, Simd<u16, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<u16>(a, b, compare_fn)
            }
            DataType::I16(_) => {
                let compare_fn = SimdWrapper::<i16, N>::eq as fn(Simd<i16, N>, Simd<i16, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<i16>(a, b, compare_fn)
            }
            DataType::U32(_) => {
                let compare_fn = SimdWrapper::<u32, N>::eq as fn(Simd<u32, N>, Simd<u32, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<u32>(a, b, compare_fn)
            }
            DataType::I32(_) => {
                let compare_fn = SimdWrapper::<i32, N>::eq as fn(Simd<i32, N>, Simd<i32, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<i32>(a, b, compare_fn)
            }
            DataType::U64(_) => {
                let compare_fn = SimdWrapper::<u64, N>::eq as fn(Simd<u64, N>, Simd<u64, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<u64>(a, b, compare_fn)
            }
            DataType::I64(_) => {
                let compare_fn = SimdWrapper::<i64, N>::eq as fn(Simd<i64, N>, Simd<i64, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<i64>(a, b, compare_fn)
            }
            DataType::F32(_) => {
                let compare_fn = SimdWrapper::<f32, N>::eq as fn(Simd<f32, N>, Simd<f32, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<f32>(a, b, compare_fn)
            }
            DataType::F64(_) => {
                let compare_fn = SimdWrapper::<f64, N>::eq as fn(Simd<f64, N>, Simd<f64, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<f64>(a, b, compare_fn)
            }
            _ => panic!("Unsupported data type"),
        }
    }

    pub fn get_compare_not_equal(
        &self,
        data_type: &DataType,
    ) -> impl Fn(*const u8, *const u8) -> Simd<u8, N> {
        match data_type {
            DataType::U8() => {
                let compare_fn = SimdWrapper::<u8, N>::ne as fn(Simd<u8, N>, Simd<u8, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<u8>(a, b, compare_fn)
            }
            DataType::I8() => {
                let compare_fn = SimdWrapper::<i8, N>::ne as fn(Simd<i8, N>, Simd<i8, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<i8>(a, b, compare_fn)
            }
            DataType::U16(_) => {
                let compare_fn = SimdWrapper::<u16, N>::ne as fn(Simd<u16, N>, Simd<u16, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<u16>(a, b, compare_fn)
            }
            DataType::I16(_) => {
                let compare_fn = SimdWrapper::<i16, N>::ne as fn(Simd<i16, N>, Simd<i16, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<i16>(a, b, compare_fn)
            }
            DataType::U32(_) => {
                let compare_fn = SimdWrapper::<u32, N>::ne as fn(Simd<u32, N>, Simd<u32, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<u32>(a, b, compare_fn)
            }
            DataType::I32(_) => {
                let compare_fn = SimdWrapper::<i32, N>::ne as fn(Simd<i32, N>, Simd<i32, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<i32>(a, b, compare_fn)
            }
            DataType::U64(_) => {
                let compare_fn = SimdWrapper::<u64, N>::ne as fn(Simd<u64, N>, Simd<u64, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<u64>(a, b, compare_fn)
            }
            DataType::I64(_) => {
                let compare_fn = SimdWrapper::<i64, N>::ne as fn(Simd<i64, N>, Simd<i64, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<i64>(a, b, compare_fn)
            }
            DataType::F32(_) => {
                let compare_fn = SimdWrapper::<f32, N>::ne as fn(Simd<f32, N>, Simd<f32, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<f32>(a, b, compare_fn)
            }
            DataType::F64(_) => {
                let compare_fn = SimdWrapper::<f64, N>::ne as fn(Simd<f64, N>, Simd<f64, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<f64>(a, b, compare_fn)
            }
            _ => panic!("Unsupported data type"),
        }
    }

    pub fn get_compare_greater_than(
        &self,
        data_type: &DataType,
    ) -> impl Fn(*const u8, *const u8) -> Simd<u8, N> {
        match data_type {
            DataType::U8() => {
                let compare_fn = SimdWrapper::<u8, N>::gt as fn(Simd<u8, N>, Simd<u8, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<u8>(a, b, compare_fn)
            }
            DataType::I8() => {
                let compare_fn = SimdWrapper::<i8, N>::gt as fn(Simd<i8, N>, Simd<i8, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<i8>(a, b, compare_fn)
            }
            DataType::U16(_) => {
                let compare_fn = SimdWrapper::<u16, N>::gt as fn(Simd<u16, N>, Simd<u16, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<u16>(a, b, compare_fn)
            }
            DataType::I16(_) => {
                let compare_fn = SimdWrapper::<i16, N>::gt as fn(Simd<i16, N>, Simd<i16, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<i16>(a, b, compare_fn)
            }
            DataType::U32(_) => {
                let compare_fn = SimdWrapper::<u32, N>::gt as fn(Simd<u32, N>, Simd<u32, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<u32>(a, b, compare_fn)
            }
            DataType::I32(_) => {
                let compare_fn = SimdWrapper::<i32, N>::gt as fn(Simd<i32, N>, Simd<i32, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<i32>(a, b, compare_fn)
            }
            DataType::U64(_) => {
                let compare_fn = SimdWrapper::<u64, N>::gt as fn(Simd<u64, N>, Simd<u64, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<u64>(a, b, compare_fn)
            }
            DataType::I64(_) => {
                let compare_fn = SimdWrapper::<i64, N>::gt as fn(Simd<i64, N>, Simd<i64, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<i64>(a, b, compare_fn)
            }
            DataType::F32(_) => {
                let compare_fn = SimdWrapper::<f32, N>::gt as fn(Simd<f32, N>, Simd<f32, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<f32>(a, b, compare_fn)
            }
            DataType::F64(_) => {
                let compare_fn = SimdWrapper::<f64, N>::gt as fn(Simd<f64, N>, Simd<f64, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<f64>(a, b, compare_fn)
            }
            _ => panic!("Unsupported data type"),
        }
    }

    pub fn get_compare_greater_than_or_equal(
        &self,
        data_type: &DataType,
    ) -> impl Fn(*const u8, *const u8) -> Simd<u8, N> {
        match data_type {
            DataType::U8() => {
                let compare_fn = SimdWrapper::<u8, N>::ge as fn(Simd<u8, N>, Simd<u8, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<u8>(a, b, compare_fn)
            }
            DataType::I8() => {
                let compare_fn = SimdWrapper::<i8, N>::ge as fn(Simd<i8, N>, Simd<i8, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<i8>(a, b, compare_fn)
            }
            DataType::U16(_) => {
                let compare_fn = SimdWrapper::<u16, N>::ge as fn(Simd<u16, N>, Simd<u16, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<u16>(a, b, compare_fn)
            }
            DataType::I16(_) => {
                let compare_fn = SimdWrapper::<i16, N>::ge as fn(Simd<i16, N>, Simd<i16, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<i16>(a, b, compare_fn)
            }
            DataType::U32(_) => {
                let compare_fn = SimdWrapper::<u32, N>::ge as fn(Simd<u32, N>, Simd<u32, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<u32>(a, b, compare_fn)
            }
            DataType::I32(_) => {
                let compare_fn = SimdWrapper::<i32, N>::ge as fn(Simd<i32, N>, Simd<i32, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<i32>(a, b, compare_fn)
            }
            DataType::U64(_) => {
                let compare_fn = SimdWrapper::<u64, N>::ge as fn(Simd<u64, N>, Simd<u64, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<u64>(a, b, compare_fn)
            }
            DataType::I64(_) => {
                let compare_fn = SimdWrapper::<i64, N>::ge as fn(Simd<i64, N>, Simd<i64, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<i64>(a, b, compare_fn)
            }
            DataType::F32(_) => {
                let compare_fn = SimdWrapper::<f32, N>::ge as fn(Simd<f32, N>, Simd<f32, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<f32>(a, b, compare_fn)
            }
            DataType::F64(_) => {
                let compare_fn = SimdWrapper::<f64, N>::ge as fn(Simd<f64, N>, Simd<f64, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<f64>(a, b, compare_fn)
            }
            _ => panic!("Unsupported data type"),
        }
    }

    pub fn get_compare_less_than(
        &self,
        data_type: &DataType,
    ) -> impl Fn(*const u8, *const u8) -> Simd<u8, N> {
        match data_type {
            DataType::U8() => {
                let compare_fn = SimdWrapper::<u8, N>::lt as fn(Simd<u8, N>, Simd<u8, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<u8>(a, b, compare_fn)
            }
            DataType::I8() => {
                let compare_fn = SimdWrapper::<i8, N>::lt as fn(Simd<i8, N>, Simd<i8, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<i8>(a, b, compare_fn)
            }
            DataType::U16(_) => {
                let compare_fn = SimdWrapper::<u16, N>::lt as fn(Simd<u16, N>, Simd<u16, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<u16>(a, b, compare_fn)
            }
            DataType::I16(_) => {
                let compare_fn = SimdWrapper::<i16, N>::lt as fn(Simd<i16, N>, Simd<i16, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<i16>(a, b, compare_fn)
            }
            DataType::U32(_) => {
                let compare_fn = SimdWrapper::<u32, N>::lt as fn(Simd<u32, N>, Simd<u32, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<u32>(a, b, compare_fn)
            }
            DataType::I32(_) => {
                let compare_fn = SimdWrapper::<i32, N>::lt as fn(Simd<i32, N>, Simd<i32, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<i32>(a, b, compare_fn)
            }
            DataType::U64(_) => {
                let compare_fn = SimdWrapper::<u64, N>::lt as fn(Simd<u64, N>, Simd<u64, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<u64>(a, b, compare_fn)
            }
            DataType::I64(_) => {
                let compare_fn = SimdWrapper::<i64, N>::lt as fn(Simd<i64, N>, Simd<i64, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<i64>(a, b, compare_fn)
            }
            DataType::F32(_) => {
                let compare_fn = SimdWrapper::<f32, N>::lt as fn(Simd<f32, N>, Simd<f32, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<f32>(a, b, compare_fn)
            }
            DataType::F64(_) => {
                let compare_fn = SimdWrapper::<f64, N>::lt as fn(Simd<f64, N>, Simd<f64, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<f64>(a, b, compare_fn)
            }
            _ => panic!("Unsupported data type"),
        }
    }

    pub fn get_compare_less_than_or_equal(
        &self,
        data_type: &DataType,
    ) -> impl Fn(*const u8, *const u8) -> Simd<u8, N> {
        match data_type {
            DataType::U8() => {
                let compare_fn = SimdWrapper::<u8, N>::le as fn(Simd<u8, N>, Simd<u8, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<u8>(a, b, compare_fn)
            }
            DataType::I8() => {
                let compare_fn = SimdWrapper::<i8, N>::le as fn(Simd<i8, N>, Simd<i8, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<i8>(a, b, compare_fn)
            }
            DataType::U16(_) => {
                let compare_fn = SimdWrapper::<u16, N>::le as fn(Simd<u16, N>, Simd<u16, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<u16>(a, b, compare_fn)
            }
            DataType::I16(_) => {
                let compare_fn = SimdWrapper::<i16, N>::le as fn(Simd<i16, N>, Simd<i16, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<i16>(a, b, compare_fn)
            }
            DataType::U32(_) => {
                let compare_fn = SimdWrapper::<u32, N>::le as fn(Simd<u32, N>, Simd<u32, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<u32>(a, b, compare_fn)
            }
            DataType::I32(_) => {
                let compare_fn = SimdWrapper::<i32, N>::le as fn(Simd<i32, N>, Simd<i32, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<i32>(a, b, compare_fn)
            }
            DataType::U64(_) => {
                let compare_fn = SimdWrapper::<u64, N>::le as fn(Simd<u64, N>, Simd<u64, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<u64>(a, b, compare_fn)
            }
            DataType::I64(_) => {
                let compare_fn = SimdWrapper::<i64, N>::le as fn(Simd<i64, N>, Simd<i64, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<i64>(a, b, compare_fn)
            }
            DataType::F32(_) => {
                let compare_fn = SimdWrapper::<f32, N>::le as fn(Simd<f32, N>, Simd<f32, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<f32>(a, b, compare_fn)
            }
            DataType::F64(_) => {
                let compare_fn = SimdWrapper::<f64, N>::le as fn(Simd<f64, N>, Simd<f64, N>) -> Simd<u8, N>;
                move |a, b| Self::immediate_simd_compare::<f64>(a, b, compare_fn)
            }
            _ => panic!("Unsupported data type"),
        }
    }

    fn get_compare_changed(&self, data_type: &DataType) -> impl Fn(*const u8, *const u8) -> Simd<u8, N> {
        match data_type {
            DataType::U8() => {
                let compare_fn = SimdWrapper::<u8, N>::ne as fn(Simd<u8, N>, Simd<u8, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::I8() => {
                let compare_fn = SimdWrapper::<i8, N>::ne as fn(Simd<i8, N>, Simd<i8, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::U16(_) => {
                let compare_fn = SimdWrapper::<u16, N>::ne as fn(Simd<u16, N>, Simd<u16, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::I16(_) => {
                let compare_fn = SimdWrapper::<i16, N>::ne as fn(Simd<i16, N>, Simd<i16, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::U32(_) => {
                let compare_fn = SimdWrapper::<u32, N>::ne as fn(Simd<u32, N>, Simd<u32, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::I32(_) => {
                let compare_fn = SimdWrapper::<i32, N>::ne as fn(Simd<i32, N>, Simd<i32, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::U64(_) => {
                let compare_fn = SimdWrapper::<u64, N>::ne as fn(Simd<u64, N>, Simd<u64, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::I64(_) => {
                let compare_fn = SimdWrapper::<i64, N>::ne as fn(Simd<i64, N>, Simd<i64, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::F32(_) => {
                let compare_fn = SimdWrapper::<f32, N>::ne as fn(Simd<f32, N>, Simd<f32, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::F64(_) => {
                let compare_fn = SimdWrapper::<f64, N>::ne as fn(Simd<f64, N>, Simd<f64, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            _ => panic!("Unsupported data type"),
        }
    }

    fn get_compare_unchanged(&self, data_type: &DataType) -> impl Fn(*const u8, *const u8) -> Simd<u8, N> {
        match data_type {
            DataType::U8() => {
                let compare_fn = SimdWrapper::<u8, N>::eq as fn(Simd<u8, N>, Simd<u8, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::I8() => {
                let compare_fn = SimdWrapper::<i8, N>::eq as fn(Simd<i8, N>, Simd<i8, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::U16(_) => {
                let compare_fn = SimdWrapper::<u16, N>::eq as fn(Simd<u16, N>, Simd<u16, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::I16(_) => {
                let compare_fn = SimdWrapper::<i16, N>::eq as fn(Simd<i16, N>, Simd<i16, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::U32(_) => {
                let compare_fn = SimdWrapper::<u32, N>::eq as fn(Simd<u32, N>, Simd<u32, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::I32(_) => {
                let compare_fn = SimdWrapper::<i32, N>::eq as fn(Simd<i32, N>, Simd<i32, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::U64(_) => {
                let compare_fn = SimdWrapper::<u64, N>::eq as fn(Simd<u64, N>, Simd<u64, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::I64(_) => {
                let compare_fn = SimdWrapper::<i64, N>::eq as fn(Simd<i64, N>, Simd<i64, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::F32(_) => {
                let compare_fn = SimdWrapper::<f32, N>::eq as fn(Simd<f32, N>, Simd<f32, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::F64(_) => {
                let compare_fn = SimdWrapper::<f64, N>::eq as fn(Simd<f64, N>, Simd<f64, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            _ => panic!("Unsupported data type"),
        }
    }

    fn get_compare_increased(&self, data_type: &DataType) -> impl Fn(*const u8, *const u8) -> Simd<u8, N> {
        match data_type {
            DataType::U8() => {
                let compare_fn = SimdWrapper::<u8, N>::gt as fn(Simd<u8, N>, Simd<u8, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::I8() => {
                let compare_fn = SimdWrapper::<i8, N>::gt as fn(Simd<i8, N>, Simd<i8, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::U16(_) => {
                let compare_fn = SimdWrapper::<u16, N>::gt as fn(Simd<u16, N>, Simd<u16, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::I16(_) => {
                let compare_fn = SimdWrapper::<i16, N>::gt as fn(Simd<i16, N>, Simd<i16, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::U32(_) => {
                let compare_fn = SimdWrapper::<u32, N>::gt as fn(Simd<u32, N>, Simd<u32, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::I32(_) => {
                let compare_fn = SimdWrapper::<i32, N>::gt as fn(Simd<i32, N>, Simd<i32, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::U64(_) => {
                let compare_fn = SimdWrapper::<u64, N>::gt as fn(Simd<u64, N>, Simd<u64, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::I64(_) => {
                let compare_fn = SimdWrapper::<i64, N>::gt as fn(Simd<i64, N>, Simd<i64, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::F32(_) => {
                let compare_fn = SimdWrapper::<f32, N>::gt as fn(Simd<f32, N>, Simd<f32, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::F64(_) => {
                let compare_fn = SimdWrapper::<f64, N>::gt as fn(Simd<f64, N>, Simd<f64, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            _ => panic!("Unsupported data type"),
        }
    }

    fn get_compare_decreased(&self, data_type: &DataType) -> impl Fn(*const u8, *const u8) -> Simd<u8, N> {
        match data_type {
            DataType::U8() => {
                let compare_fn = SimdWrapper::<u8, N>::lt as fn(Simd<u8, N>, Simd<u8, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::I8() => {
                let compare_fn = SimdWrapper::<i8, N>::lt as fn(Simd<i8, N>, Simd<i8, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::U16(_) => {
                let compare_fn = SimdWrapper::<u16, N>::lt as fn(Simd<u16, N>, Simd<u16, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::I16(_) => {
                let compare_fn = SimdWrapper::<i16, N>::lt as fn(Simd<i16, N>, Simd<i16, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::U32(_) => {
                let compare_fn = SimdWrapper::<u32, N>::lt as fn(Simd<u32, N>, Simd<u32, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::I32(_) => {
                let compare_fn = SimdWrapper::<i32, N>::lt as fn(Simd<i32, N>, Simd<i32, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::U64(_) => {
                let compare_fn = SimdWrapper::<u64, N>::lt as fn(Simd<u64, N>, Simd<u64, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::I64(_) => {
                let compare_fn = SimdWrapper::<i64, N>::lt as fn(Simd<i64, N>, Simd<i64, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::F32(_) => {
                let compare_fn = SimdWrapper::<f32, N>::lt as fn(Simd<f32, N>, Simd<f32, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            DataType::F64(_) => {
                let compare_fn = SimdWrapper::<f64, N>::lt as fn(Simd<f64, N>, Simd<f64, N>) -> Simd<u8, N>;
                move |a, b| Self::relative_simd_compare(a, b, compare_fn)
            }
            _ => panic!("Unsupported data type"),
        }
    }

    fn get_compare_increased_by(&self, data_type: &DataType) -> impl Fn(*const u8, *const u8, *const u8) -> Simd<u8, N> {
        match data_type {
            DataType::U8() => {
                let simd_op = SimdWrapper::<u8, N>::add as fn(Simd<u8, N>, Simd<u8, N>) -> Simd<u8, N>;
                let compare_fn = SimdWrapper::<u8, N>::eq as fn(Simd<u8, N>, Simd<u8, N>) -> Simd<u8, N>;
                move |a, b, c| Self::relative_delta_simd_compare(a, b, c, simd_op, compare_fn)
            }
            DataType::I8() => {
                let simd_op = SimdWrapper::<i8, N>::add as fn(Simd<i8, N>, Simd<i8, N>) -> Simd<i8, N>;
                let compare_fn = SimdWrapper::<i8, N>::eq as fn(Simd<i8, N>, Simd<i8, N>) -> Simd<u8, N>;
                move |a, b, c| Self::relative_delta_simd_compare(a, b, c, simd_op, compare_fn)
            }
            DataType::U16(_) => {
                let simd_op = SimdWrapper::<u16, N>::add as fn(Simd<u16, N>, Simd<u16, N>) -> Simd<u16, N>;
                let compare_fn = SimdWrapper::<u16, N>::eq as fn(Simd<u16, N>, Simd<u16, N>) -> Simd<u8, N>;
                move |a, b, c| Self::relative_delta_simd_compare(a, b, c, simd_op, compare_fn)
            }
            DataType::I16(_) => {
                let simd_op = SimdWrapper::<i16, N>::add as fn(Simd<i16, N>, Simd<i16, N>) -> Simd<i16, N>;
                let compare_fn = SimdWrapper::<i16, N>::eq as fn(Simd<i16, N>, Simd<i16, N>) -> Simd<u8, N>;
                move |a, b, c| Self::relative_delta_simd_compare(a, b, c, simd_op, compare_fn)
            }
            DataType::U32(_) => {
                let simd_op = SimdWrapper::<u32, N>::add as fn(Simd<u32, N>, Simd<u32, N>) -> Simd<u32, N>;
                let compare_fn = SimdWrapper::<u32, N>::eq as fn(Simd<u32, N>, Simd<u32, N>) -> Simd<u8, N>;
                move |a, b, c| Self::relative_delta_simd_compare(a, b, c, simd_op, compare_fn)
            }
            DataType::I32(_) => {
                let simd_op = SimdWrapper::<i32, N>::add as fn(Simd<i32, N>, Simd<i32, N>) -> Simd<i32, N>;
                let compare_fn = SimdWrapper::<i32, N>::eq as fn(Simd<i32, N>, Simd<i32, N>) -> Simd<u8, N>;
                move |a, b, c| Self::relative_delta_simd_compare(a, b, c, simd_op, compare_fn)
            }
            DataType::U64(_) => {
                let simd_op = SimdWrapper::<u64, N>::add as fn(Simd<u64, N>, Simd<u64, N>) -> Simd<u64, N>;
                let compare_fn = SimdWrapper::<u64, N>::eq as fn(Simd<u64, N>, Simd<u64, N>) -> Simd<u8, N>;
                move |a, b, c| Self::relative_delta_simd_compare(a, b, c, simd_op, compare_fn)
            }
            DataType::I64(_) => {
                let simd_op = SimdWrapper::<i64, N>::add as fn(Simd<i64, N>, Simd<i64, N>) -> Simd<i64, N>;
                let compare_fn = SimdWrapper::<i64, N>::eq as fn(Simd<i64, N>, Simd<i64, N>) -> Simd<u8, N>;
                move |a, b, c| Self::relative_delta_simd_compare(a, b, c, simd_op, compare_fn)
            }
            DataType::F32(_) => {
                let simd_op = SimdWrapper::<f32, N>::add as fn(Simd<f32, N>, Simd<f32, N>) -> Simd<f32, N>;
                let compare_fn = SimdWrapper::<f32, N>::eq as fn(Simd<f32, N>, Simd<f32, N>) -> Simd<u8, N>;
                move |a, b, c| Self::relative_delta_simd_compare(a, b, c, simd_op, compare_fn)
            }
            DataType::F64(_) => {
                let simd_op = SimdWrapper::<f64, N>::add as fn(Simd<f64, N>, Simd<f64, N>) -> Simd<f64, N>;
                let compare_fn = SimdWrapper::<f64, N>::eq as fn(Simd<f64, N>, Simd<f64, N>) -> Simd<u8, N>;
                move |a, b, c| Self::relative_delta_simd_compare(a, b, c, simd_op, compare_fn)
            }
            _ => panic!("Unsupported data type"),
        }
    }
    
    fn get_compare_decreased_by(&self, data_type: &DataType) -> impl Fn(*const u8, *const u8, *const u8) -> Simd<u8, N> {
        match data_type {
            DataType::U8() => {
                let simd_op = SimdWrapper::<u8, N>::sub as fn(Simd<u8, N>, Simd<u8, N>) -> Simd<u8, N>;
                let compare_fn = SimdWrapper::<u8, N>::eq as fn(Simd<u8, N>, Simd<u8, N>) -> Simd<u8, N>;
                move |a, b, c| Self::relative_delta_simd_compare(a, b, c, simd_op, compare_fn)
            }
            DataType::I8() => {
                let simd_op = SimdWrapper::<i8, N>::sub as fn(Simd<i8, N>, Simd<i8, N>) -> Simd<i8, N>;
                let compare_fn = SimdWrapper::<i8, N>::eq as fn(Simd<i8, N>, Simd<i8, N>) -> Simd<u8, N>;
                move |a, b, c| Self::relative_delta_simd_compare(a, b, c, simd_op, compare_fn)
            }
            DataType::U16(_) => {
                let simd_op = SimdWrapper::<u16, N>::sub as fn(Simd<u16, N>, Simd<u16, N>) -> Simd<u16, N>;
                let compare_fn = SimdWrapper::<u16, N>::eq as fn(Simd<u16, N>, Simd<u16, N>) -> Simd<u8, N>;
                move |a, b, c| Self::relative_delta_simd_compare(a, b, c, simd_op, compare_fn)
            }
            DataType::I16(_) => {
                let simd_op = SimdWrapper::<i16, N>::sub as fn(Simd<i16, N>, Simd<i16, N>) -> Simd<i16, N>;
                let compare_fn = SimdWrapper::<i16, N>::eq as fn(Simd<i16, N>, Simd<i16, N>) -> Simd<u8, N>;
                move |a, b, c| Self::relative_delta_simd_compare(a, b, c, simd_op, compare_fn)
            }
            DataType::U32(_) => {
                let simd_op = SimdWrapper::<u32, N>::sub as fn(Simd<u32, N>, Simd<u32, N>) -> Simd<u32, N>;
                let compare_fn = SimdWrapper::<u32, N>::eq as fn(Simd<u32, N>, Simd<u32, N>) -> Simd<u8, N>;
                move |a, b, c| Self::relative_delta_simd_compare(a, b, c, simd_op, compare_fn)
            }
            DataType::I32(_) => {
                let simd_op = SimdWrapper::<i32, N>::sub as fn(Simd<i32, N>, Simd<i32, N>) -> Simd<i32, N>;
                let compare_fn = SimdWrapper::<i32, N>::eq as fn(Simd<i32, N>, Simd<i32, N>) -> Simd<u8, N>;
                move |a, b, c| Self::relative_delta_simd_compare(a, b, c, simd_op, compare_fn)
            }
            DataType::U64(_) => {
                let simd_op = SimdWrapper::<u64, N>::sub as fn(Simd<u64, N>, Simd<u64, N>) -> Simd<u64, N>;
                let compare_fn = SimdWrapper::<u64, N>::eq as fn(Simd<u64, N>, Simd<u64, N>) -> Simd<u8, N>;
                move |a, b, c| Self::relative_delta_simd_compare(a, b, c, simd_op, compare_fn)
            }
            DataType::I64(_) => {
                let simd_op = SimdWrapper::<i64, N>::sub as fn(Simd<i64, N>, Simd<i64, N>) -> Simd<i64, N>;
                let compare_fn = SimdWrapper::<i64, N>::eq as fn(Simd<i64, N>, Simd<i64, N>) -> Simd<u8, N>;
                move |a, b, c| Self::relative_delta_simd_compare(a, b, c, simd_op, compare_fn)
            }
            DataType::F32(_) => {
                let simd_op = SimdWrapper::<f32, N>::sub as fn(Simd<f32, N>, Simd<f32, N>) -> Simd<f32, N>;
                let compare_fn = SimdWrapper::<f32, N>::eq as fn(Simd<f32, N>, Simd<f32, N>) -> Simd<u8, N>;
                move |a, b, c| Self::relative_delta_simd_compare(a, b, c, simd_op, compare_fn)
            }
            DataType::F64(_) => {
                let simd_op = SimdWrapper::<f64, N>::sub as fn(Simd<f64, N>, Simd<f64, N>) -> Simd<f64, N>;
                let compare_fn = SimdWrapper::<f64, N>::eq as fn(Simd<f64, N>, Simd<f64, N>) -> Simd<u8, N>;
                move |a, b, c| Self::relative_delta_simd_compare(a, b, c, simd_op, compare_fn)
            }
            _ => panic!("Unsupported data type"),
        }
    }
}
