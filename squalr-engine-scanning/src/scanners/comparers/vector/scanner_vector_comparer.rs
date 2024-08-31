use crate::scanners::encoders::vector::simd_type::SimdType;
use crate::scanners::parameters::scan_compare_type::ScanCompareType;
use squalr_engine_common::values::data_type::DataType;
use squalr_engine_common::values::endian::Endian;
use std::ops::{Add, Sub};
use std::simd::cmp::{SimdPartialEq, SimdPartialOrd};
use std::simd::{LaneCount, Simd, SimdElement, SupportedLaneCount};

pub trait VectorComparer<T: SimdElement + SimdType + PartialEq, const N: usize>
where
    LaneCount<N>: SupportedLaneCount,
    LaneCount<{ N / 2 }>: SupportedLaneCount,
    LaneCount<{ N / 4 }>: SupportedLaneCount,
    LaneCount<{ N / 8 }>: SupportedLaneCount,
    Simd<T, N>: SimdPartialEq,
{
    fn get_immediate_compare_func(
        &self,
        scan_compare_type: ScanCompareType,
        data_type: &DataType,
    ) -> impl Fn(*const u8, *const u8) -> Simd<u8, N>;

    fn get_relative_compare_func(
        &self,
        scan_compare_type: ScanCompareType,
        data_type: &DataType,
    ) -> impl Fn(*const u8, *const u8) -> Simd<u8, N>;

    fn get_relative_delta_compare_func(
        &self,
        scan_compare_type: ScanCompareType,
        data_type: &DataType,
    ) -> impl Fn(*const u8, *const u8, *const u8) -> Simd<u8, N>;
}

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

impl<T, const N: usize> VectorComparer<T, N> for ScannerVectorComparer<T, N>
where
    T: SimdElement + SimdType + PartialEq,
    LaneCount<N>: SupportedLaneCount,
    LaneCount<{ N / 2 }>: SupportedLaneCount,
    LaneCount<{ N / 4 }>: SupportedLaneCount,
    LaneCount<{ N / 8 }>: SupportedLaneCount,
    Simd<T, N>: SimdPartialEq,
{
    fn get_immediate_compare_func(
        &self,
        scan_compare_type: ScanCompareType,
        data_type: &DataType,
    ) -> impl Fn(*const u8, *const u8) -> Simd<u8, N> {
        self.get_immediate_compare_func(scan_compare_type, data_type)
    }

    fn get_relative_compare_func(
        &self,
        scan_compare_type: ScanCompareType,
        data_type: &DataType,
    ) -> impl Fn(*const u8, *const u8) -> Simd<u8, N> {
        self.get_relative_compare_func(scan_compare_type, data_type)
    }

    fn get_relative_delta_compare_func(
        &self,
        scan_compare_type: ScanCompareType,
        data_type: &DataType,
    ) -> impl Fn(*const u8, *const u8, *const u8) -> Simd<u8, N> {
        self.get_relative_delta_compare_func(scan_compare_type, data_type)
    }
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
    ) -> impl Fn(*const u8, *const u8) -> Simd<u8, N> {
        match scan_compare_type {
            ScanCompareType::Equal => Self::get_compare_equal_func(data_type),
            ScanCompareType::NotEqual => Self::get_compare_not_equal_func(data_type),
            ScanCompareType::GreaterThan => Self::get_compare_greater_than(data_type),
            ScanCompareType::GreaterThanOrEqual => Self::get_compare_greater_than_or_equal(data_type),
            ScanCompareType::LessThan => Self::get_compare_less_than(data_type),
            ScanCompareType::LessThanOrEqual => Self::get_compare_less_than_or_equal(data_type),
            _ => panic!("Unsupported type passed to get_immediate_compare_func"),
        }
    }

    pub fn get_relative_compare_func(
        &self,
        scan_compare_type: ScanCompareType,
        data_type: &DataType,
    ) -> impl Fn(*const u8, *const u8) -> Simd<u8, N> {
        match scan_compare_type {
            ScanCompareType::Changed => Self::get_compare_changed(data_type),
            ScanCompareType::Unchanged => Self::get_compare_unchanged(data_type),
            ScanCompareType::Increased => Self::get_compare_increased(data_type),
            ScanCompareType::Decreased => Self::get_compare_decreased(data_type),
            _ => panic!("Unsupported type passed to get_relative_compare_func"),
        }
    }

    pub fn get_relative_delta_compare_func(
        &self,
        scan_compare_type: ScanCompareType,
        data_type: &DataType,
    ) -> impl Fn(*const u8, *const u8, *const u8) -> Simd<u8, N> {
        match scan_compare_type {
            ScanCompareType::IncreasedByX => Self::get_compare_increased_by(data_type),
            ScanCompareType::DecreasedByX => Self::get_compare_decreased_by(data_type),
            _ => panic!("Unsupported type passed to get_relative_delta_compare_func"),
        }
    }

    fn check_endian(
        &self,
        endian: &Endian,
    ) -> bool {
        cfg!(target_endian = "little") == (*endian == Endian::Little)
    }

    fn safe_transmute<M, const M_LANES: usize>(value: &<Simd<M, M_LANES> as SimdPartialEq>::Mask) -> Simd<u8, N>
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

        let mut result_array = [0u8; N];
        let value_ptr = value as *const _ as *const u8;
        unsafe {
            std::ptr::copy_nonoverlapping(value_ptr, result_array.as_mut_ptr(), std::mem::size_of_val(value));
        }

        // Unsafe variant:
        // These are guaranteed to be the same size, but std::mem::transmute() is not passing Rust's compile checks
        // Perhaps Rust is not smart enough to realize that the resulting sizes are the exact same.
        // return *(&*value as *const _ as *const Simd<u8, N>);

        return Simd::<u8, N>::from_array(result_array);
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
            let immediate_value = Simd::<M, M_LANES>::splat(std::ptr::read_unaligned(immediate_ptr as *const M));
            let current_values = Simd::<M, M_LANES>::from_array(std::ptr::read_unaligned(current_values_ptr as *const [M; M_LANES]));
            return Self::safe_transmute::<M, M_LANES>(&current_values.simd_eq(immediate_value));
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
            let immediate_value = Simd::<M, M_LANES>::splat(std::ptr::read_unaligned(immediate_ptr as *const M));
            let current_values = Simd::<M, M_LANES>::from_array(std::ptr::read_unaligned(current_values_ptr as *const [M; M_LANES]));
            return Self::safe_transmute::<M, M_LANES>(&current_values.simd_ne(immediate_value));
        }
    }

    fn get_compare_greater_than(data_type: &DataType) -> fn(*const u8, *const u8) -> Simd<u8, N> {
        match data_type {
            DataType::U8() => Self::compare_greater_than::<u8, N>,
            DataType::I8() => Self::compare_greater_than::<i8, N>,
            DataType::U16(_) => Self::compare_greater_than::<u16, { N / 2 }>,
            DataType::I16(_) => Self::compare_greater_than::<i16, { N / 2 }>,
            DataType::U32(_) => Self::compare_greater_than::<u32, { N / 4 }>,
            DataType::I32(_) => Self::compare_greater_than::<i32, { N / 4 }>,
            DataType::U64(_) => Self::compare_greater_than::<u64, { N / 8 }>,
            DataType::I64(_) => Self::compare_greater_than::<i64, { N / 8 }>,
            DataType::F32(_) => Self::compare_greater_than::<f32, { N / 8 }>,
            DataType::F64(_) => Self::compare_greater_than::<f64, { N / 8 }>,
            _ => panic!("Unsupported data type"),
        }
    }

    fn compare_greater_than<M, const M_LANES: usize>(
        current_values_ptr: *const u8,
        immediate_ptr: *const u8,
    ) -> Simd<u8, N>
    where
        M: SimdElement + PartialOrd,
        LaneCount<M_LANES>: SupportedLaneCount,
        Simd<M, M_LANES>: SimdPartialOrd,
    {
        unsafe {
            let immediate_value = Simd::<M, M_LANES>::splat(std::ptr::read_unaligned(immediate_ptr as *const M));
            let current_values = Simd::<M, M_LANES>::from_array(std::ptr::read_unaligned(current_values_ptr as *const [M; M_LANES]));
            return Self::safe_transmute::<M, M_LANES>(&current_values.simd_gt(immediate_value));
        }
    }

    fn get_compare_greater_than_or_equal(data_type: &DataType) -> fn(*const u8, *const u8) -> Simd<u8, N> {
        match data_type {
            DataType::U8() => Self::compare_greater_than_or_equal::<u8, N>,
            DataType::I8() => Self::compare_greater_than_or_equal::<i8, N>,
            DataType::U16(_) => Self::compare_greater_than_or_equal::<u16, { N / 2 }>,
            DataType::I16(_) => Self::compare_greater_than_or_equal::<i16, { N / 2 }>,
            DataType::U32(_) => Self::compare_greater_than_or_equal::<u32, { N / 4 }>,
            DataType::I32(_) => Self::compare_greater_than_or_equal::<i32, { N / 4 }>,
            DataType::U64(_) => Self::compare_greater_than_or_equal::<u64, { N / 8 }>,
            DataType::I64(_) => Self::compare_greater_than_or_equal::<i64, { N / 8 }>,
            DataType::F32(_) => Self::compare_greater_than_or_equal::<f32, { N / 8 }>,
            DataType::F64(_) => Self::compare_greater_than_or_equal::<f64, { N / 8 }>,
            _ => panic!("Unsupported data type"),
        }
    }

    fn compare_greater_than_or_equal<M, const M_LANES: usize>(
        current_values_ptr: *const u8,
        immediate_ptr: *const u8,
    ) -> Simd<u8, N>
    where
        M: SimdElement + PartialOrd,
        LaneCount<M_LANES>: SupportedLaneCount,
        Simd<M, M_LANES>: SimdPartialOrd,
    {
        unsafe {
            let immediate_value = Simd::<M, M_LANES>::splat(std::ptr::read_unaligned(immediate_ptr as *const M));
            let current_values = Simd::<M, M_LANES>::from_array(std::ptr::read_unaligned(current_values_ptr as *const [M; M_LANES]));
            return Self::safe_transmute::<M, M_LANES>(&current_values.simd_ge(immediate_value));
        }
    }

    fn get_compare_less_than(data_type: &DataType) -> fn(*const u8, *const u8) -> Simd<u8, N> {
        match data_type {
            DataType::U8() => Self::compare_less_than::<u8, N>,
            DataType::I8() => Self::compare_less_than::<i8, N>,
            DataType::U16(_) => Self::compare_less_than::<u16, { N / 2 }>,
            DataType::I16(_) => Self::compare_less_than::<i16, { N / 2 }>,
            DataType::U32(_) => Self::compare_less_than::<u32, { N / 4 }>,
            DataType::I32(_) => Self::compare_less_than::<i32, { N / 4 }>,
            DataType::U64(_) => Self::compare_less_than::<u64, { N / 8 }>,
            DataType::I64(_) => Self::compare_less_than::<i64, { N / 8 }>,
            DataType::F32(_) => Self::compare_less_than::<f32, { N / 8 }>,
            DataType::F64(_) => Self::compare_less_than::<f64, { N / 8 }>,
            _ => panic!("Unsupported data type"),
        }
    }

    fn compare_less_than<M, const M_LANES: usize>(
        current_values_ptr: *const u8,
        immediate_ptr: *const u8,
    ) -> Simd<u8, N>
    where
        M: SimdElement + PartialOrd,
        LaneCount<M_LANES>: SupportedLaneCount,
        Simd<M, M_LANES>: SimdPartialOrd,
    {
        unsafe {
            let immediate_value = Simd::<M, M_LANES>::splat(std::ptr::read_unaligned(immediate_ptr as *const M));
            let current_values = Simd::<M, M_LANES>::from_array(std::ptr::read_unaligned(current_values_ptr as *const [M; M_LANES]));
            return Self::safe_transmute::<M, M_LANES>(&current_values.simd_lt(immediate_value));
        }
    }

    fn get_compare_less_than_or_equal(data_type: &DataType) -> fn(*const u8, *const u8) -> Simd<u8, N> {
        match data_type {
            DataType::U8() => Self::compare_less_than_or_equal::<u8, N>,
            DataType::I8() => Self::compare_less_than_or_equal::<i8, N>,
            DataType::U16(_) => Self::compare_less_than_or_equal::<u16, { N / 2 }>,
            DataType::I16(_) => Self::compare_less_than_or_equal::<i16, { N / 2 }>,
            DataType::U32(_) => Self::compare_less_than_or_equal::<u32, { N / 4 }>,
            DataType::I32(_) => Self::compare_less_than_or_equal::<i32, { N / 4 }>,
            DataType::U64(_) => Self::compare_less_than_or_equal::<u64, { N / 8 }>,
            DataType::I64(_) => Self::compare_less_than_or_equal::<i64, { N / 8 }>,
            DataType::F32(_) => Self::compare_less_than_or_equal::<f32, { N / 8 }>,
            DataType::F64(_) => Self::compare_less_than_or_equal::<f64, { N / 8 }>,
            _ => panic!("Unsupported data type"),
        }
    }

    fn compare_less_than_or_equal<M, const M_LANES: usize>(
        current_values_ptr: *const u8,
        immediate_ptr: *const u8,
    ) -> Simd<u8, N>
    where
        M: SimdElement + PartialOrd,
        LaneCount<M_LANES>: SupportedLaneCount,
        Simd<M, M_LANES>: SimdPartialOrd,
    {
        unsafe {
            let immediate_value = Simd::<M, M_LANES>::splat(std::ptr::read_unaligned(immediate_ptr as *const M));
            let current_values = Simd::<M, M_LANES>::from_array(std::ptr::read_unaligned(current_values_ptr as *const [M; M_LANES]));
            return Self::safe_transmute::<M, M_LANES>(&current_values.simd_le(immediate_value));
        }
    }

    fn get_compare_changed(data_type: &DataType) -> fn(*const u8, *const u8) -> Simd<u8, N> {
        match data_type {
            DataType::U8() => Self::compare_not_equal::<u8, N>,
            DataType::I8() => Self::compare_not_equal::<i8, N>,
            DataType::U16(_) => Self::compare_not_equal::<u16, { N / 2 }>,
            DataType::I16(_) => Self::compare_not_equal::<i16, { N / 2 }>,
            DataType::U32(_) => Self::compare_not_equal::<u32, { N / 4 }>,
            DataType::I32(_) => Self::compare_not_equal::<i32, { N / 4 }>,
            DataType::U64(_) => Self::compare_not_equal::<u64, { N / 8 }>,
            DataType::I64(_) => Self::compare_not_equal::<i64, { N / 8 }>,
            DataType::F32(_) => Self::compare_not_equal::<f32, { N / 8 }>,
            DataType::F64(_) => Self::compare_not_equal::<f64, { N / 8 }>,
            _ => panic!("Unsupported data type"),
        }
    }

    fn get_compare_unchanged(data_type: &DataType) -> fn(*const u8, *const u8) -> Simd<u8, N> {
        match data_type {
            DataType::U8() => Self::compare_equal::<u8, N>,
            DataType::I8() => Self::compare_equal::<i8, N>,
            DataType::U16(_) => Self::compare_equal::<u16, { N / 2 }>,
            DataType::I16(_) => Self::compare_equal::<i16, { N / 2 }>,
            DataType::U32(_) => Self::compare_equal::<u32, { N / 4 }>,
            DataType::I32(_) => Self::compare_equal::<i32, { N / 4 }>,
            DataType::U64(_) => Self::compare_equal::<u64, { N / 8 }>,
            DataType::I64(_) => Self::compare_equal::<i64, { N / 8 }>,
            DataType::F32(_) => Self::compare_equal::<f32, { N / 8 }>,
            DataType::F64(_) => Self::compare_equal::<f64, { N / 8 }>,
            _ => panic!("Unsupported data type"),
        }
    }

    fn get_compare_increased(data_type: &DataType) -> fn(*const u8, *const u8) -> Simd<u8, N> {
        match data_type {
            DataType::U8() => Self::compare_greater_than::<u8, N>,
            DataType::I8() => Self::compare_greater_than::<i8, N>,
            DataType::U16(_) => Self::compare_greater_than::<u16, { N / 2 }>,
            DataType::I16(_) => Self::compare_greater_than::<i16, { N / 2 }>,
            DataType::U32(_) => Self::compare_greater_than::<u32, { N / 4 }>,
            DataType::I32(_) => Self::compare_greater_than::<i32, { N / 4 }>,
            DataType::U64(_) => Self::compare_greater_than::<u64, { N / 8 }>,
            DataType::I64(_) => Self::compare_greater_than::<i64, { N / 8 }>,
            DataType::F32(_) => Self::compare_greater_than::<f32, { N / 8 }>,
            DataType::F64(_) => Self::compare_greater_than::<f64, { N / 8 }>,
            _ => panic!("Unsupported data type"),
        }
    }

    fn get_compare_decreased(data_type: &DataType) -> fn(*const u8, *const u8) -> Simd<u8, N> {
        match data_type {
            DataType::U8() => Self::compare_less_than::<u8, N>,
            DataType::I8() => Self::compare_less_than::<i8, N>,
            DataType::U16(_) => Self::compare_less_than::<u16, { N / 2 }>,
            DataType::I16(_) => Self::compare_less_than::<i16, { N / 2 }>,
            DataType::U32(_) => Self::compare_less_than::<u32, { N / 4 }>,
            DataType::I32(_) => Self::compare_less_than::<i32, { N / 4 }>,
            DataType::U64(_) => Self::compare_less_than::<u64, { N / 8 }>,
            DataType::I64(_) => Self::compare_less_than::<i64, { N / 8 }>,
            DataType::F32(_) => Self::compare_less_than::<f32, { N / 8 }>,
            DataType::F64(_) => Self::compare_less_than::<f64, { N / 8 }>,
            _ => panic!("Unsupported data type"),
        }
    }

    fn get_compare_increased_by(data_type: &DataType) -> fn(*const u8, *const u8, *const u8) -> Simd<u8, N> {
        match data_type {
            DataType::U8() => Self::compare_increased_by::<u8, N>,
            DataType::I8() => Self::compare_increased_by::<i8, N>,
            DataType::U16(_) => Self::compare_increased_by::<u16, { N / 2 }>,
            DataType::I16(_) => Self::compare_increased_by::<i16, { N / 2 }>,
            DataType::U32(_) => Self::compare_increased_by::<u32, { N / 4 }>,
            DataType::I32(_) => Self::compare_increased_by::<i32, { N / 4 }>,
            DataType::U64(_) => Self::compare_increased_by::<u64, { N / 8 }>,
            DataType::I64(_) => Self::compare_increased_by::<i64, { N / 8 }>,
            DataType::F32(_) => Self::compare_increased_by::<f32, { N / 8 }>,
            DataType::F64(_) => Self::compare_increased_by::<f64, { N / 8 }>,
            _ => panic!("Unsupported data type"),
        }
    }

    fn compare_increased_by<M, const M_LANES: usize>(
        current_values_ptr: *const u8,
        previous_values_ptr: *const u8,
        delta_ptr: *const u8,
    ) -> Simd<u8, N>
    where
        M: SimdElement + Add<Output = M> + PartialOrd,
        LaneCount<M_LANES>: SupportedLaneCount,
        Simd<M, M_LANES>: SimdPartialOrd + Add<Output = Simd<M, M_LANES>>,
    {
        unsafe {
            let current_values = Simd::<M, M_LANES>::from_array(std::ptr::read_unaligned(current_values_ptr as *const [M; M_LANES]));
            let previous_values = Simd::<M, M_LANES>::from_array(std::ptr::read_unaligned(previous_values_ptr as *const [M; M_LANES]));
            let delta_value = Simd::<M, M_LANES>::splat(std::ptr::read_unaligned(delta_ptr as *const M));
            return Self::safe_transmute::<M, M_LANES>(&current_values.simd_gt(previous_values + delta_value));
        }
    }

    fn get_compare_decreased_by(data_type: &DataType) -> fn(*const u8, *const u8, *const u8) -> Simd<u8, N> {
        match data_type {
            DataType::U8() => Self::compare_decreased_by::<u8, N>,
            DataType::I8() => Self::compare_decreased_by::<i8, N>,
            DataType::U16(_) => Self::compare_decreased_by::<u16, { N / 2 }>,
            DataType::I16(_) => Self::compare_decreased_by::<i16, { N / 2 }>,
            DataType::U32(_) => Self::compare_decreased_by::<u32, { N / 4 }>,
            DataType::I32(_) => Self::compare_decreased_by::<i32, { N / 4 }>,
            DataType::U64(_) => Self::compare_decreased_by::<u64, { N / 8 }>,
            DataType::I64(_) => Self::compare_decreased_by::<i64, { N / 8 }>,
            DataType::F32(_) => Self::compare_decreased_by::<f32, { N / 8 }>,
            DataType::F64(_) => Self::compare_decreased_by::<f64, { N / 8 }>,
            _ => panic!("Unsupported data type"),
        }
    }

    fn compare_decreased_by<M, const M_LANES: usize>(
        current_values_ptr: *const u8,
        previous_values_ptr: *const u8,
        delta_ptr: *const u8,
    ) -> Simd<u8, N>
    where
        M: SimdElement + Sub<Output = M> + PartialOrd,
        LaneCount<M_LANES>: SupportedLaneCount,
        Simd<M, M_LANES>: SimdPartialOrd + Sub<Output = Simd<M, M_LANES>>,
    {
        unsafe {
            let current_values = Simd::<M, M_LANES>::from_array(std::ptr::read_unaligned(current_values_ptr as *const [M; M_LANES]));
            let previous_values = Simd::<M, M_LANES>::from_array(std::ptr::read_unaligned(previous_values_ptr as *const [M; M_LANES]));
            let delta_value = Simd::<M, M_LANES>::splat(std::ptr::read_unaligned(delta_ptr as *const M));
            return Self::safe_transmute::<M, M_LANES>(&current_values.simd_lt(previous_values - delta_value));
        }
    }
}
