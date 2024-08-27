use crate::scanners::comparers::vector::types::simd_type::SimdType;
use crate::scanners::parameters::scan_compare_type::ScanCompareType;
use squalr_engine_common::values::data_type::DataType;
use squalr_engine_common::values::endian::Endian;
use std::marker::PhantomData;
use std::mem::transmute;
use std::ops::{
    Add,
    Sub,
};
use std::simd::cmp::{
    SimdPartialEq,
    SimdPartialOrd,
};
use std::simd::num::{
    SimdInt,
    SimdUint,
};
use std::simd::{
    LaneCount,
    Mask,
    MaskElement,
    Simd,
    SimdElement,
    SupportedLaneCount,
};

pub struct ScannerVectorComparer<T: SimdElement + SimdType + PartialEq, const N: usize>
where
    T: SimdElement + SimdType + PartialEq,
    LaneCount<N>: SupportedLaneCount,
    LaneCount<{ N / 2 }>: SupportedLaneCount,
    LaneCount<{ N / 4 }>: SupportedLaneCount,
    LaneCount<{ N / 8 }>: SupportedLaneCount,
    Simd<T, N>: SimdPartialEq,
{
    _marker: std::marker::PhantomData<T>,
}

pub enum CompareFunc<T, const N: usize>
where
    T: SimdElement + SimdType + PartialEq,
    LaneCount<N>: SupportedLaneCount,
    LaneCount<{ N / 2 }>: SupportedLaneCount,
    LaneCount<{ N / 4 }>: SupportedLaneCount,
    LaneCount<{ N / 8 }>: SupportedLaneCount,
    Simd<T, N>: SimdPartialEq,
{
    Immediate(fn(*const u8, *const u8) -> Simd<u8, N>),
    Relative(fn(*const u8, *const u8) -> Simd<u8, N>),
    RelativeDelta(fn(*const u8, *const u8, *const u8) -> Simd<u8, N>),
    _Marker(PhantomData<T>),
}

impl<T: SimdElement + SimdType, const N: usize> ScannerVectorComparer<T, N>
where
    T: SimdElement + SimdType + PartialEq,
    LaneCount<N>: SupportedLaneCount,
    LaneCount<{ N / 2 }>: SupportedLaneCount,
    LaneCount<{ N / 4 }>: SupportedLaneCount,
    LaneCount<{ N / 8 }>: SupportedLaneCount,
    Simd<T, N>: SimdPartialEq,
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
            ScanCompareType::Equal => CompareFunc::Immediate(Self::get_compare_equal_func(data_type)),
            ScanCompareType::NotEqual => CompareFunc::Immediate(Self::get_compare_not_equal_func(data_type)),
            /*
            ScanCompareType::GreaterThan => {
                // CompareFunc::Immediate(Self::get_compare_greater_than(data_type))
            }
            ScanCompareType::GreaterThanOrEqual => {
                // CompareFunc::Immediate(Self::get_compare_greater_than_or_equal(data_type))
            }
            ScanCompareType::LessThan => {
                // CompareFunc::Immediate(Self::get_compare_less_than(data_type))
            }
            ScanCompareType::LessThanOrEqual => {
                // CompareFunc::Immediate(Self::get_compare_less_than_or_equal(data_type))
            } */
            _ => panic!("Unsupported type passed to get_immediate_compare_func"),
        }
    }

    pub fn get_relative_compare_func(
        &self,
        scan_compare_type: ScanCompareType,
        data_type: &DataType,
    ) -> CompareFunc<T, N> {
        match scan_compare_type {
            // ScanCompareType::Changed => CompareFunc::Relative(Self::get_compare_changed(data_type)),
            // ScanCompareType::Unchanged => CompareFunc::Relative(Self::get_compare_unchanged(data_type)),
            // ScanCompareType::Increased => CompareFunc::Relative(Self::get_compare_increased(data_type)),
            // ScanCompareType::Decreased => CompareFunc::Relative(Self::get_compare_decreased(data_type)),
            _ => panic!("Unsupported type passed to get_relative_compare_func"),
        }
    }

    pub fn get_relative_delta_compare_func(
        &self,
        scan_compare_type: ScanCompareType,
        data_type: &DataType,
    ) -> CompareFunc<T, N> {
        match scan_compare_type {
            // ScanCompareType::IncreasedByX => CompareFunc::RelativeDelta(Self::get_compare_increased_by(data_type)),
            // ScanCompareType::DecreasedByX => CompareFunc::RelativeDelta(Self::get_compare_decreased_by(data_type)),
            _ => panic!("Unsupported type passed to get_relative_delta_compare_func"),
        }
    }

    fn check_endian(
        &self,
        endian: &Endian,
    ) -> bool {
        cfg!(target_endian = "little") == (*endian == Endian::Little)
    }

    fn get_compare_equal_func(data_type: &DataType) -> fn(*const u8, *const u8) -> Simd<u8, N> {
        match data_type {
            DataType::U8() => Self::compare_equal::<u8, N>,
            DataType::I8() => Self::compare_equal::<i8, N>,
            DataType::U16(_) => Self::compare_equal::<u16, { N / 2 }>,
            DataType::I16(_) => Self::compare_equal::<i16, { N / 2 }>,
            DataType::U32(_) => Self::compare_equal::<u32, { N / 4 }>,
            DataType::I32(_) => Self::compare_equal::<i32, { N / 4 }>,
            DataType::U64(_) => Self::compare_equal::<u64, { N / 8 }>,
            DataType::I64(_) => Self::compare_equal::<i64, { N / 8 }>,
            // TODO: Support floating point tolerance
            DataType::F32(_) => Self::compare_equal::<f32, { N / 8 }>,
            DataType::F64(_) => Self::compare_equal::<f64, { N / 8 }>,
            _ => panic!("Unsupported data type"),
        }
    }

    fn get_compare_not_equal_func(data_type: &DataType) -> fn(*const u8, *const u8) -> Simd<u8, N> {
        match data_type {
            DataType::U8() => Self::compare_not_equal::<u8, N>,
            DataType::I8() => Self::compare_not_equal::<i8, N>,
            DataType::U16(_) => Self::compare_not_equal::<u16, { N / 2 }>,
            DataType::I16(_) => Self::compare_not_equal::<i16, { N / 2 }>,
            DataType::U32(_) => Self::compare_not_equal::<u32, { N / 4 }>,
            DataType::I32(_) => Self::compare_not_equal::<i32, { N / 4 }>,
            DataType::U64(_) => Self::compare_not_equal::<u64, { N / 8 }>,
            DataType::I64(_) => Self::compare_not_equal::<i64, { N / 8 }>,
            // TODO: Support floating point tolerance
            DataType::F32(_) => Self::compare_not_equal::<f32, { N / 8 }>,
            DataType::F64(_) => Self::compare_not_equal::<f64, { N / 8 }>,
            _ => panic!("Unsupported data type"),
        }
    }

    #[inline(always)]
    unsafe fn unsafe_transmute<M, const M_LANES: usize>(value: &<Simd<M, M_LANES> as SimdPartialEq>::Mask) -> Simd<u8, N>
    where
        M: SimdElement + PartialEq,
        LaneCount<M_LANES>: SupportedLaneCount,
        Simd<M, M_LANES>: SimdPartialEq,
    {
        // debug_assert_eq!(
        //     std::mem::size_of::<<Simd<M, M_LANES> as SimdPartialEq>::Mask>(),
        //     std::mem::size_of::<Simd<u8, N>>(),
        //     "Size mismatch between Mask and Simd<u8, N>"
        // );

        // These are guaranteed to be the same size, but std::mem::transmute is not passing Rust's compile checks
        // Perhaps Rust is not smart enough to realize that the resulting sizes are the exact same.
        return *(&*value as *const _ as *const Simd<u8, N>);
    }

    fn compare_equal<M, const M_LANES: usize>(
        current_values_ptr: *const u8,
        immediate_ptr: *const u8,
    ) -> Simd<u8, N>
    where
        M: SimdElement + PartialEq,
        LaneCount<M_LANES>: SupportedLaneCount,
        Simd<M, M_LANES>: SimdPartialEq,
    {
        unsafe {
            let immediate_value = Simd::<M, M_LANES>::splat(*(immediate_ptr as *const M));
            let current_values = Simd::<M, M_LANES>::from_array(*(current_values_ptr as *const [M; M_LANES]));
            let compare_result = current_values.simd_eq(immediate_value);
            return Self::unsafe_transmute::<M, M_LANES>(&compare_result);
        }
    }

    fn compare_not_equal<M, const M_LANES: usize>(
        current_values_ptr: *const u8,
        immediate_ptr: *const u8,
    ) -> Simd<u8, N>
    where
        M: SimdElement + PartialEq,
        LaneCount<M_LANES>: SupportedLaneCount,
        Simd<M, M_LANES>: SimdPartialEq,
    {
        unsafe {
            let immediate_value = Simd::<M, M_LANES>::splat(*(immediate_ptr as *const M));
            let current_values = Simd::<M, M_LANES>::from_array(*(current_values_ptr as *const [M; M_LANES]));
            let compare_result = current_values.simd_eq(immediate_value);
            return Self::unsafe_transmute::<M, M_LANES>(&compare_result);
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
}
