use crate::scanners::parameters::scan_compare_type::ScanCompareType;
use squalr_engine_common::values::data_type::DataType;
use std::ops::{Add, Sub};
use std::simd::cmp::{SimdPartialEq, SimdPartialOrd};
use std::simd::{i16x8, i32x4, i64x2, i8x16, f32x4, f64x2, u16x8, u32x4, u64x2, u8x16};
use std::mem;

macro_rules! impl_scanner_vector_comparer {
    ($bit_width:expr) => {
        impl ScannerVectorComparer<$bit_width> {
            pub fn get_instance() -> &'static ScannerVectorComparer<$bit_width> {
                static mut INSTANCE: Option<ScannerVectorComparer<$bit_width>> = None;
                static INIT: std::sync::Once = std::sync::Once::new();

                unsafe {
                    INIT.call_once(|| {
                        let instance = ScannerVectorComparer::<$bit_width>::new();
                        INSTANCE = Some(instance);
                    });

                    INSTANCE.as_ref().unwrap_unchecked()
                }
            }
        }
    };
}

/// Defines a compare function that operates on an immediate (ie all inequalities)
type VectorCompareFnImmediate = unsafe fn(
    // Current v1lue buffer
    current_v1lue_pointer: *const u8,
    // Immediate v1lue
    immediate_v1lue_pointer: *const u8,
) -> u8x16;

/// Defines a compare function that operates on current and previous values (ie changed, unchanged, increased, decreased)
type VectorCompareFnRelative = unsafe fn(
    // Current v1lue buffer
    current_v1lue_pointer: *const u8,
    // Previous v1lue buffer
    previous_v1lue_pointer: *const u8,
) -> u8x16;

/// Defines a compare function that operates on current and previous values, with a delta arg (ie +x, -x)
type VectorCompareFnDelta = unsafe fn(
    // Current v1lue buffer
    current_v1lue_pointer: *const u8,
    // Previous v1lue buffer
    previous_v1lue_pointer: *const u8,
    // Delta v1lue buffer
    delta_v1lue_pointer: *const u8,
) -> u8x16;

pub struct ScannerVectorComparer<const BIT_WIDTH: usize>;

impl_scanner_vector_comparer!(128);
impl_scanner_vector_comparer!(256);
impl_scanner_vector_comparer!(512);

/// Implements a set of scalar (ie CPU bound, non-SIMD) boolean comparison operations to be used by more complex scanners.
impl<const BIT_WIDTH: usize> ScannerVectorComparer<BIT_WIDTH> {
    fn new() -> Self {
        Self { }
    }
    
    pub fn get_immediate_compare_func(
        &self,
        scan_compare_type: ScanCompareType,
        data_type: &DataType,
    ) -> VectorCompareFnImmediate {
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
    ) -> VectorCompareFnRelative {
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
    ) -> VectorCompareFnDelta {
        match scan_compare_type {
            ScanCompareType::IncreasedByX => self.get_compare_increased_by(data_type),
            ScanCompareType::DecreasedByX => self.get_compare_decreased_by(data_type),
            _ => panic!("Unsupported type passed to get_relative_delta_compare_func"),
        }
    }

    fn get_compare_equal(&self, data_type: &DataType) -> VectorCompareFnImmediate {
        unsafe {
            match data_type {
                DataType::U8() => |current_values_ptr, immediate_ptr: *const u8| {
                    let immediate_value = u8x16::splat(*(immediate_ptr as *const u8));
                    let current_values = u8x16::from_array(*(current_values_ptr as *const [u8; 16]));
                    return mem::transmute(current_values.simd_eq(immediate_value));
                },
                DataType::I8() => |current_values_ptr, immediate_ptr| {
                    let immediate_value = i8x16::splat(*(immediate_ptr as *const i8));
                    let current_values = i8x16::from_array(*(current_values_ptr as *const [i8; 16]));
                    return mem::transmute(current_values.simd_eq(immediate_value));
                },
                DataType::U16(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = u16x8::splat(*(immediate_ptr as *const u16));
                    let current_values = u16x8::from_array(*(current_values_ptr as *const [u16; 8]));
                    return mem::transmute(current_values.simd_eq(immediate_value));
                },
                DataType::I16(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = i16x8::splat(*(immediate_ptr as *const i16));
                    let current_values = i16x8::from_array(*(current_values_ptr as *const [i16; 8]));
                    return mem::transmute(current_values.simd_eq(immediate_value));
                },
                DataType::U32(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = u32x4::splat(*(immediate_ptr as *const u32));
                    let current_values = u32x4::from_array(*(current_values_ptr as *const [u32; 4]));
                    return mem::transmute(current_values.simd_eq(immediate_value));
                },
                DataType::I32(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = i32x4::splat(*(immediate_ptr as *const i32));
                    let current_values = i32x4::from_array(*(current_values_ptr as *const [i32; 4]));
                    return mem::transmute(current_values.simd_eq(immediate_value));
                },
                DataType::U64(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = u64x2::splat(*(immediate_ptr as *const u64));
                    let current_values = u64x2::from_array(*(current_values_ptr as *const [u64; 2]));
                    return mem::transmute(current_values.simd_eq(immediate_value));
                },
                DataType::I64(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = i64x2::splat(*(immediate_ptr as *const i64));
                    let current_values = i64x2::from_array(*(current_values_ptr as *const [i64; 2]));
                    return mem::transmute(current_values.simd_eq(immediate_value));
                },
                DataType::F32(_) => |current_values_ptr, immediate_ptr| {
                    // TODO: Support floating point tolerance
                    let immediate_value = f32x4::splat(*(immediate_ptr as *const f32));
                    let current_values = f32x4::from_array(*(current_values_ptr as *const [f32; 4]));
                    return mem::transmute(current_values.simd_eq(immediate_value));
                },
                DataType::F64(_) => |current_values_ptr, immediate_ptr| {
                    // TODO: Support floating point tolerance
                    let immediate_value = f64x2::splat(*(immediate_ptr as *const f64));
                    let current_values = f64x2::from_array(*(current_values_ptr as *const [f64; 2]));
                    return mem::transmute(current_values.simd_eq(immediate_value));
                },
                _ => panic!("unsupported data type"),
            }
        }
    }

    fn get_compare_not_equal(&self, data_type: &DataType) -> VectorCompareFnImmediate {
        unsafe {
            match data_type {
                DataType::U8() => |current_values_ptr, immediate_ptr: *const u8| {
                    let immediate_value = u8x16::splat(*(immediate_ptr as *const u8));
                    let current_values = u8x16::from_array(*(current_values_ptr as *const [u8; 16]));
                    return mem::transmute(current_values.simd_ne(immediate_value));
                },
                DataType::I8() => |current_values_ptr, immediate_ptr| {
                    let immediate_value = i8x16::splat(*(immediate_ptr as *const i8));
                    let current_values = i8x16::from_array(*(current_values_ptr as *const [i8; 16]));
                    return mem::transmute(current_values.simd_ne(immediate_value));
                },
                DataType::U16(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = u16x8::splat(*(immediate_ptr as *const u16));
                    let current_values = u16x8::from_array(*(current_values_ptr as *const [u16; 8]));
                    return mem::transmute(current_values.simd_ne(immediate_value));
                },
                DataType::I16(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = i16x8::splat(*(immediate_ptr as *const i16));
                    let current_values = i16x8::from_array(*(current_values_ptr as *const [i16; 8]));
                    return mem::transmute(current_values.simd_ne(immediate_value));
                },
                DataType::U32(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = u32x4::splat(*(immediate_ptr as *const u32));
                    let current_values = u32x4::from_array(*(current_values_ptr as *const [u32; 4]));
                    return mem::transmute(current_values.simd_ne(immediate_value));
                },
                DataType::I32(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = i32x4::splat(*(immediate_ptr as *const i32));
                    let current_values = i32x4::from_array(*(current_values_ptr as *const [i32; 4]));
                    return mem::transmute(current_values.simd_ne(immediate_value));
                },
                DataType::U64(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = u64x2::splat(*(immediate_ptr as *const u64));
                    let current_values = u64x2::from_array(*(current_values_ptr as *const [u64; 2]));
                    return mem::transmute(current_values.simd_ne(immediate_value));
                },
                DataType::I64(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = i64x2::splat(*(immediate_ptr as *const i64));
                    let current_values = i64x2::from_array(*(current_values_ptr as *const [i64; 2]));
                    return mem::transmute(current_values.simd_ne(immediate_value));
                },
                DataType::F32(_) => |current_values_ptr, immediate_ptr| {
                    // TODO: Support floating point tolerance
                    let immediate_value = f32x4::splat(*(immediate_ptr as *const f32));
                    let current_values = f32x4::from_array(*(current_values_ptr as *const [f32; 4]));
                    return mem::transmute(current_values.simd_ne(immediate_value));
                },
                DataType::F64(_) => |current_values_ptr, immediate_ptr| {
                    // TODO: Support floating point tolerance
                    let immediate_value = f64x2::splat(*(immediate_ptr as *const f64));
                    let current_values = f64x2::from_array(*(current_values_ptr as *const [f64; 2]));
                    return mem::transmute(current_values.simd_ne(immediate_value));
                },
                _ => panic!("unsupported data type"),
            }
        }
    }

    fn get_compare_greater_than(&self, data_type: &DataType) -> VectorCompareFnImmediate {
        unsafe {
            match data_type {
                DataType::U8() => |current_values_ptr, immediate_ptr: *const u8| {
                    let immediate_value = u8x16::splat(*(immediate_ptr as *const u8));
                    let current_values = u8x16::from_array(*(current_values_ptr as *const [u8; 16]));
                    return mem::transmute(current_values.simd_gt(immediate_value));
                },
                DataType::I8() => |current_values_ptr, immediate_ptr| {
                    let immediate_value = i8x16::splat(*(immediate_ptr as *const i8));
                    let current_values = i8x16::from_array(*(current_values_ptr as *const [i8; 16]));
                    return mem::transmute(current_values.simd_gt(immediate_value));
                },
                DataType::U16(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = u16x8::splat(*(immediate_ptr as *const u16));
                    let current_values = u16x8::from_array(*(current_values_ptr as *const [u16; 8]));
                    return mem::transmute(current_values.simd_gt(immediate_value));
                },
                DataType::I16(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = i16x8::splat(*(immediate_ptr as *const i16));
                    let current_values = i16x8::from_array(*(current_values_ptr as *const [i16; 8]));
                    return mem::transmute(current_values.simd_gt(immediate_value));
                },
                DataType::U32(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = u32x4::splat(*(immediate_ptr as *const u32));
                    let current_values = u32x4::from_array(*(current_values_ptr as *const [u32; 4]));
                    return mem::transmute(current_values.simd_gt(immediate_value));
                },
                DataType::I32(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = i32x4::splat(*(immediate_ptr as *const i32));
                    let current_values = i32x4::from_array(*(current_values_ptr as *const [i32; 4]));
                    return mem::transmute(current_values.simd_gt(immediate_value));
                },
                DataType::U64(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = u64x2::splat(*(immediate_ptr as *const u64));
                    let current_values = u64x2::from_array(*(current_values_ptr as *const [u64; 2]));
                    return mem::transmute(current_values.simd_gt(immediate_value));
                },
                DataType::I64(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = i64x2::splat(*(immediate_ptr as *const i64));
                    let current_values = i64x2::from_array(*(current_values_ptr as *const [i64; 2]));
                    return mem::transmute(current_values.simd_gt(immediate_value));
                },
                DataType::F32(_) => |current_values_ptr, immediate_ptr| {
                    // TODO: Support floating point tolerance
                    let immediate_value = f32x4::splat(*(immediate_ptr as *const f32));
                    let current_values = f32x4::from_array(*(current_values_ptr as *const [f32; 4]));
                    return mem::transmute(current_values.simd_gt(immediate_value));
                },
                DataType::F64(_) => |current_values_ptr, immediate_ptr| {
                    // TODO: Support floating point tolerance
                    let immediate_value = f64x2::splat(*(immediate_ptr as *const f64));
                    let current_values = f64x2::from_array(*(current_values_ptr as *const [f64; 2]));
                    return mem::transmute(current_values.simd_gt(immediate_value));
                },
                _ => panic!("unsupported data type"),
            }
        }
    }

    fn get_compare_greater_than_or_equal(&self, data_type: &DataType) -> VectorCompareFnImmediate {
        unsafe {
            match data_type {
                DataType::U8() => |current_values_ptr, immediate_ptr: *const u8| {
                    let immediate_value = u8x16::splat(*(immediate_ptr as *const u8));
                    let current_values = u8x16::from_array(*(current_values_ptr as *const [u8; 16]));
                    return mem::transmute(current_values.simd_ge(immediate_value));
                },
                DataType::I8() => |current_values_ptr, immediate_ptr| {
                    let immediate_value = i8x16::splat(*(immediate_ptr as *const i8));
                    let current_values = i8x16::from_array(*(current_values_ptr as *const [i8; 16]));
                    return mem::transmute(current_values.simd_ge(immediate_value));
                },
                DataType::U16(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = u16x8::splat(*(immediate_ptr as *const u16));
                    let current_values = u16x8::from_array(*(current_values_ptr as *const [u16; 8]));
                    return mem::transmute(current_values.simd_ge(immediate_value));
                },
                DataType::I16(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = i16x8::splat(*(immediate_ptr as *const i16));
                    let current_values = i16x8::from_array(*(current_values_ptr as *const [i16; 8]));
                    return mem::transmute(current_values.simd_ge(immediate_value));
                },
                DataType::U32(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = u32x4::splat(*(immediate_ptr as *const u32));
                    let current_values = u32x4::from_array(*(current_values_ptr as *const [u32; 4]));
                    return mem::transmute(current_values.simd_ge(immediate_value));
                },
                DataType::I32(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = i32x4::splat(*(immediate_ptr as *const i32));
                    let current_values = i32x4::from_array(*(current_values_ptr as *const [i32; 4]));
                    return mem::transmute(current_values.simd_ge(immediate_value));
                },
                DataType::U64(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = u64x2::splat(*(immediate_ptr as *const u64));
                    let current_values = u64x2::from_array(*(current_values_ptr as *const [u64; 2]));
                    return mem::transmute(current_values.simd_ge(immediate_value));
                },
                DataType::I64(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = i64x2::splat(*(immediate_ptr as *const i64));
                    let current_values = i64x2::from_array(*(current_values_ptr as *const [i64; 2]));
                    return mem::transmute(current_values.simd_ge(immediate_value));
                },
                DataType::F32(_) => |current_values_ptr, immediate_ptr| {
                    // TODO: Support floating point tolerance
                    let immediate_value = f32x4::splat(*(immediate_ptr as *const f32));
                    let current_values = f32x4::from_array(*(current_values_ptr as *const [f32; 4]));
                    return mem::transmute(current_values.simd_ge(immediate_value));
                },
                DataType::F64(_) => |current_values_ptr, immediate_ptr| {
                    // TODO: Support floating point tolerance
                    let immediate_value = f64x2::splat(*(immediate_ptr as *const f64));
                    let current_values = f64x2::from_array(*(current_values_ptr as *const [f64; 2]));
                    return mem::transmute(current_values.simd_ge(immediate_value));
                },
                _ => panic!("unsupported data type"),
            }
        }
    }

    fn get_compare_less_than(&self, data_type: &DataType) -> VectorCompareFnImmediate {
        unsafe {
            match data_type {
                DataType::U8() => |current_values_ptr, immediate_ptr: *const u8| {
                    let immediate_value = u8x16::splat(*(immediate_ptr as *const u8));
                    let current_values = u8x16::from_array(*(current_values_ptr as *const [u8; 16]));
                    return mem::transmute(current_values.simd_lt(immediate_value));
                },
                DataType::I8() => |current_values_ptr, immediate_ptr| {
                    let immediate_value = i8x16::splat(*(immediate_ptr as *const i8));
                    let current_values = i8x16::from_array(*(current_values_ptr as *const [i8; 16]));
                    return mem::transmute(current_values.simd_lt(immediate_value));
                },
                DataType::U16(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = u16x8::splat(*(immediate_ptr as *const u16));
                    let current_values = u16x8::from_array(*(current_values_ptr as *const [u16; 8]));
                    return mem::transmute(current_values.simd_lt(immediate_value));
                },
                DataType::I16(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = i16x8::splat(*(immediate_ptr as *const i16));
                    let current_values = i16x8::from_array(*(current_values_ptr as *const [i16; 8]));
                    return mem::transmute(current_values.simd_lt(immediate_value));
                },
                DataType::U32(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = u32x4::splat(*(immediate_ptr as *const u32));
                    let current_values = u32x4::from_array(*(current_values_ptr as *const [u32; 4]));
                    return mem::transmute(current_values.simd_lt(immediate_value));
                },
                DataType::I32(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = i32x4::splat(*(immediate_ptr as *const i32));
                    let current_values = i32x4::from_array(*(current_values_ptr as *const [i32; 4]));
                    return mem::transmute(current_values.simd_lt(immediate_value));
                },
                DataType::U64(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = u64x2::splat(*(immediate_ptr as *const u64));
                    let current_values = u64x2::from_array(*(current_values_ptr as *const [u64; 2]));
                    return mem::transmute(current_values.simd_lt(immediate_value));
                },
                DataType::I64(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = i64x2::splat(*(immediate_ptr as *const i64));
                    let current_values = i64x2::from_array(*(current_values_ptr as *const [i64; 2]));
                    return mem::transmute(current_values.simd_lt(immediate_value));
                },
                DataType::F32(_) => |current_values_ptr, immediate_ptr| {
                    // TODO: Support floating point tolerance
                    let immediate_value = f32x4::splat(*(immediate_ptr as *const f32));
                    let current_values = f32x4::from_array(*(current_values_ptr as *const [f32; 4]));
                    return mem::transmute(current_values.simd_lt(immediate_value));
                },
                DataType::F64(_) => |current_values_ptr, immediate_ptr| {
                    // TODO: Support floating point tolerance
                    let immediate_value = f64x2::splat(*(immediate_ptr as *const f64));
                    let current_values = f64x2::from_array(*(current_values_ptr as *const [f64; 2]));
                    return mem::transmute(current_values.simd_lt(immediate_value));
                },
                _ => panic!("unsupported data type"),
            }
        }
    }

    fn get_compare_less_than_or_equal(&self, data_type: &DataType) -> VectorCompareFnImmediate {
        unsafe {
            match data_type {
                DataType::U8() => |current_values_ptr, immediate_ptr: *const u8| {
                    let immediate_value = u8x16::splat(*(immediate_ptr as *const u8));
                    let current_values = u8x16::from_array(*(current_values_ptr as *const [u8; 16]));
                    return mem::transmute(current_values.simd_le(immediate_value));
                },
                DataType::I8() => |current_values_ptr, immediate_ptr| {
                    let immediate_value = i8x16::splat(*(immediate_ptr as *const i8));
                    let current_values = i8x16::from_array(*(current_values_ptr as *const [i8; 16]));
                    return mem::transmute(current_values.simd_le(immediate_value));
                },
                DataType::U16(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = u16x8::splat(*(immediate_ptr as *const u16));
                    let current_values = u16x8::from_array(*(current_values_ptr as *const [u16; 8]));
                    return mem::transmute(current_values.simd_le(immediate_value));
                },
                DataType::I16(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = i16x8::splat(*(immediate_ptr as *const i16));
                    let current_values = i16x8::from_array(*(current_values_ptr as *const [i16; 8]));
                    return mem::transmute(current_values.simd_le(immediate_value));
                },
                DataType::U32(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = u32x4::splat(*(immediate_ptr as *const u32));
                    let current_values = u32x4::from_array(*(current_values_ptr as *const [u32; 4]));
                    return mem::transmute(current_values.simd_le(immediate_value));
                },
                DataType::I32(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = i32x4::splat(*(immediate_ptr as *const i32));
                    let current_values = i32x4::from_array(*(current_values_ptr as *const [i32; 4]));
                    return mem::transmute(current_values.simd_le(immediate_value));
                },
                DataType::U64(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = u64x2::splat(*(immediate_ptr as *const u64));
                    let current_values = u64x2::from_array(*(current_values_ptr as *const [u64; 2]));
                    return mem::transmute(current_values.simd_le(immediate_value));
                },
                DataType::I64(_) => |current_values_ptr, immediate_ptr| {
                    let immediate_value = i64x2::splat(*(immediate_ptr as *const i64));
                    let current_values = i64x2::from_array(*(current_values_ptr as *const [i64; 2]));
                    return mem::transmute(current_values.simd_le(immediate_value));
                },
                DataType::F32(_) => |current_values_ptr, immediate_ptr| {
                    // TODO: Support floating point tolerance
                    let immediate_value = f32x4::splat(*(immediate_ptr as *const f32));
                    let current_values = f32x4::from_array(*(current_values_ptr as *const [f32; 4]));
                    return mem::transmute(current_values.simd_le(immediate_value));
                },
                DataType::F64(_) => |current_values_ptr, immediate_ptr| {
                    // TODO: Support floating point tolerance
                    let immediate_value = f64x2::splat(*(immediate_ptr as *const f64));
                    let current_values = f64x2::from_array(*(current_values_ptr as *const [f64; 2]));
                    return mem::transmute(current_values.simd_le(immediate_value));
                },
                _ => panic!("unsupported data type"),
            }
        }
    }

    fn get_compare_changed(&self, data_type: &DataType) -> VectorCompareFnRelative {
        unsafe {
            match data_type {
                DataType::U8() => |current_values_ptr, previous_values_ptr| {
                    let current_values = u8x16::from_array(*(current_values_ptr as *const [u8; 16]));
                    let previous_values = u8x16::from_array(*(previous_values_ptr as *const [u8; 16]));
                    return mem::transmute(current_values.simd_ne(previous_values));
                },
                DataType::I8() => |current_values_ptr, previous_values_ptr| {
                    let current_values = i8x16::from_array(*(current_values_ptr as *const [i8; 16]));
                    let previous_values = i8x16::from_array(*(previous_values_ptr as *const [i8; 16]));
                    return mem::transmute(current_values.simd_ne(previous_values));
                },
                DataType::U16(_) => |current_values_ptr, previous_values_ptr| {
                    let current_values = u16x8::from_array(*(current_values_ptr as *const [u16; 8]));
                    let previous_values = u16x8::from_array(*(previous_values_ptr as *const [u16; 8]));
                    return mem::transmute(current_values.simd_ne(previous_values));
                },
                DataType::I16(_) => |current_values_ptr, previous_values_ptr| {
                    let current_values = i16x8::from_array(*(current_values_ptr as *const [i16; 8]));
                    let previous_values = i16x8::from_array(*(previous_values_ptr as *const [i16; 8]));
                    return mem::transmute(current_values.simd_ne(previous_values));
                },
                DataType::U32(_) => |current_values_ptr, previous_values_ptr| {
                    let current_values = u32x4::from_array(*(current_values_ptr as *const [u32; 4]));
                    let previous_values = u32x4::from_array(*(previous_values_ptr as *const [u32; 4]));
                    return mem::transmute(current_values.simd_ne(previous_values));
                },
                DataType::I32(_) => |current_values_ptr, previous_values_ptr| {
                    let current_values = i32x4::from_array(*(current_values_ptr as *const [i32; 4]));
                    let previous_values = i32x4::from_array(*(previous_values_ptr as *const [i32; 4]));
                    return mem::transmute(current_values.simd_ne(previous_values));
                },
                DataType::U64(_) => |current_values_ptr, previous_values_ptr| {
                    let current_values = u64x2::from_array(*(current_values_ptr as *const [u64; 2]));
                    let previous_values = u64x2::from_array(*(previous_values_ptr as *const [u64; 2]));
                    return mem::transmute(current_values.simd_ne(previous_values));
                },
                DataType::I64(_) => |current_values_ptr, previous_values_ptr| {
                    let current_values = i64x2::from_array(*(current_values_ptr as *const [i64; 2]));
                    let previous_values = i64x2::from_array(*(previous_values_ptr as *const [i64; 2]));
                    return mem::transmute(current_values.simd_ne(previous_values));
                },
                DataType::F32(_) => |current_values_ptr, previous_values_ptr| {
                    // TODO: Support floating point tolerance
                    let current_values = f32x4::from_array(*(current_values_ptr as *const [f32; 4]));
                    let previous_values = f32x4::from_array(*(previous_values_ptr as *const [f32; 4]));
                    return mem::transmute(current_values.simd_ne(previous_values));
                },
                DataType::F64(_) => |current_values_ptr, previous_values_ptr| {
                    // TODO: Support floating point tolerance
                    let current_values = f64x2::from_array(*(current_values_ptr as *const [f64; 2]));
                    let previous_values = f64x2::from_array(*(previous_values_ptr as *const [f64; 2]));
                    return mem::transmute(current_values.simd_ne(previous_values));
                },
                _ => panic!("unsupported data type"),
            }
        }
    }

    fn get_compare_unchanged(&self, data_type: &DataType) -> VectorCompareFnRelative {
        unsafe {
            match data_type {
                DataType::U8() => |current_values_ptr, previous_values_ptr| {
                    let current_values = u8x16::from_array(*(current_values_ptr as *const [u8; 16]));
                    let previous_values = u8x16::from_array(*(previous_values_ptr as *const [u8; 16]));
                    return mem::transmute(current_values.simd_ne(previous_values));
                },
                DataType::I8() => |current_values_ptr, previous_values_ptr| {
                    let current_values = i8x16::from_array(*(current_values_ptr as *const [i8; 16]));
                    let previous_values = i8x16::from_array(*(previous_values_ptr as *const [i8; 16]));
                    return mem::transmute(current_values.simd_ne(previous_values));
                },
                DataType::U16(_) => |current_values_ptr, previous_values_ptr| {
                    let current_values = u16x8::from_array(*(current_values_ptr as *const [u16; 8]));
                    let previous_values = u16x8::from_array(*(previous_values_ptr as *const [u16; 8]));
                    return mem::transmute(current_values.simd_ne(previous_values));
                },
                DataType::I16(_) => |current_values_ptr, previous_values_ptr| {
                    let current_values = i16x8::from_array(*(current_values_ptr as *const [i16; 8]));
                    let previous_values = i16x8::from_array(*(previous_values_ptr as *const [i16; 8]));
                    return mem::transmute(current_values.simd_ne(previous_values));
                },
                DataType::U32(_) => |current_values_ptr, previous_values_ptr| {
                    let current_values = u32x4::from_array(*(current_values_ptr as *const [u32; 4]));
                    let previous_values = u32x4::from_array(*(previous_values_ptr as *const [u32; 4]));
                    return mem::transmute(current_values.simd_ne(previous_values));
                },
                DataType::I32(_) => |current_values_ptr, previous_values_ptr| {
                    let current_values = i32x4::from_array(*(current_values_ptr as *const [i32; 4]));
                    let previous_values = i32x4::from_array(*(previous_values_ptr as *const [i32; 4]));
                    return mem::transmute(current_values.simd_ne(previous_values));
                },
                DataType::U64(_) => |current_values_ptr, previous_values_ptr| {
                    let current_values = u64x2::from_array(*(current_values_ptr as *const [u64; 2]));
                    let previous_values = u64x2::from_array(*(previous_values_ptr as *const [u64; 2]));
                    return mem::transmute(current_values.simd_ne(previous_values));
                },
                DataType::I64(_) => |current_values_ptr, previous_values_ptr| {
                    let current_values = i64x2::from_array(*(current_values_ptr as *const [i64; 2]));
                    let previous_values = i64x2::from_array(*(previous_values_ptr as *const [i64; 2]));
                    return mem::transmute(current_values.simd_ne(previous_values));
                },
                DataType::F32(_) => |current_values_ptr, previous_values_ptr| {
                    let current_values = f32x4::from_array(*(current_values_ptr as *const [f32; 4]));
                    let previous_values = f32x4::from_array(*(previous_values_ptr as *const [f32; 4]));
                    return mem::transmute(current_values.simd_ne(previous_values));
                },
                DataType::F64(_) => |current_values_ptr, previous_values_ptr| {
                    let current_values = f64x2::from_array(*(current_values_ptr as *const [f64; 2]));
                    let previous_values = f64x2::from_array(*(previous_values_ptr as *const [f64; 2]));
                    return mem::transmute(current_values.simd_ne(previous_values));
                },
                _ => panic!("unsupported data type"),
            }
        }
    }

    fn get_compare_increased(&self, data_type: &DataType) -> VectorCompareFnRelative {
        unsafe {
            match data_type {
                DataType::U8() => |current_values_ptr, previous_values_ptr| {
                    let current_values = u8x16::from_array(*(current_values_ptr as *const [u8; 16]));
                    let previous_values = u8x16::from_array(*(previous_values_ptr as *const [u8; 16]));
                    return mem::transmute(current_values.simd_gt(previous_values));
                },
                DataType::I8() => |current_values_ptr, previous_values_ptr| {
                    let current_values = i8x16::from_array(*(current_values_ptr as *const [i8; 16]));
                    let previous_values = i8x16::from_array(*(previous_values_ptr as *const [i8; 16]));
                    return mem::transmute(current_values.simd_gt(previous_values));
                },
                DataType::U16(_) => |current_values_ptr, previous_values_ptr| {
                    let current_values = u16x8::from_array(*(current_values_ptr as *const [u16; 8]));
                    let previous_values = u16x8::from_array(*(previous_values_ptr as *const [u16; 8]));
                    return mem::transmute(current_values.simd_gt(previous_values));
                },
                DataType::I16(_) => |current_values_ptr, previous_values_ptr| {
                    let current_values = i16x8::from_array(*(current_values_ptr as *const [i16; 8]));
                    let previous_values = i16x8::from_array(*(previous_values_ptr as *const [i16; 8]));
                    return mem::transmute(current_values.simd_gt(previous_values));
                },
                DataType::U32(_) => |current_values_ptr, previous_values_ptr| {
                    let current_values = u32x4::from_array(*(current_values_ptr as *const [u32; 4]));
                    let previous_values = u32x4::from_array(*(previous_values_ptr as *const [u32; 4]));
                    return mem::transmute(current_values.simd_gt(previous_values));
                },
                DataType::I32(_) => |current_values_ptr, previous_values_ptr| {
                    let current_values = i32x4::from_array(*(current_values_ptr as *const [i32; 4]));
                    let previous_values = i32x4::from_array(*(previous_values_ptr as *const [i32; 4]));
                    return mem::transmute(current_values.simd_gt(previous_values));
                },
                DataType::U64(_) => |current_values_ptr, previous_values_ptr| {
                    let current_values = u64x2::from_array(*(current_values_ptr as *const [u64; 2]));
                    let previous_values = u64x2::from_array(*(previous_values_ptr as *const [u64; 2]));
                    return mem::transmute(current_values.simd_gt(previous_values));
                },
                DataType::I64(_) => |current_values_ptr, previous_values_ptr| {
                    let current_values = i64x2::from_array(*(current_values_ptr as *const [i64; 2]));
                    let previous_values = i64x2::from_array(*(previous_values_ptr as *const [i64; 2]));
                    return mem::transmute(current_values.simd_gt(previous_values));
                },
                DataType::F32(_) => |current_values_ptr, previous_values_ptr| {
                    let current_values = f32x4::from_array(*(current_values_ptr as *const [f32; 4]));
                    let previous_values = f32x4::from_array(*(previous_values_ptr as *const [f32; 4]));
                    return mem::transmute(current_values.simd_gt(previous_values));
                },
                DataType::F64(_) => |current_values_ptr, previous_values_ptr| {
                    let current_values = f64x2::from_array(*(current_values_ptr as *const [f64; 2]));
                    let previous_values = f64x2::from_array(*(previous_values_ptr as *const [f64; 2]));
                    return mem::transmute(current_values.simd_gt(previous_values));
                },
                _ => panic!("unsupported data type"),
            }
        }
    }

    fn get_compare_decreased(&self, data_type: &DataType) -> VectorCompareFnRelative {
        unsafe {
            match data_type {
                DataType::U8() => |current_values_ptr, previous_values_ptr| {
                    let current_values = u8x16::from_array(*(current_values_ptr as *const [u8; 16]));
                    let previous_values = u8x16::from_array(*(previous_values_ptr as *const [u8; 16]));
                    return mem::transmute(current_values.simd_lt(previous_values));
                },
                DataType::I8() => |current_values_ptr, previous_values_ptr| {
                    let current_values = i8x16::from_array(*(current_values_ptr as *const [i8; 16]));
                    let previous_values = i8x16::from_array(*(previous_values_ptr as *const [i8; 16]));
                    return mem::transmute(current_values.simd_lt(previous_values));
                },
                DataType::U16(_) => |current_values_ptr, previous_values_ptr| {
                    let current_values = u16x8::from_array(*(current_values_ptr as *const [u16; 8]));
                    let previous_values = u16x8::from_array(*(previous_values_ptr as *const [u16; 8]));
                    return mem::transmute(current_values.simd_lt(previous_values));
                },
                DataType::I16(_) => |current_values_ptr, previous_values_ptr| {
                    let current_values = i16x8::from_array(*(current_values_ptr as *const [i16; 8]));
                    let previous_values = i16x8::from_array(*(previous_values_ptr as *const [i16; 8]));
                    return mem::transmute(current_values.simd_lt(previous_values));
                },
                DataType::U32(_) => |current_values_ptr, previous_values_ptr| {
                    let current_values = u32x4::from_array(*(current_values_ptr as *const [u32; 4]));
                    let previous_values = u32x4::from_array(*(previous_values_ptr as *const [u32; 4]));
                    return mem::transmute(current_values.simd_lt(previous_values));
                },
                DataType::I32(_) => |current_values_ptr, previous_values_ptr| {
                    let current_values = i32x4::from_array(*(current_values_ptr as *const [i32; 4]));
                    let previous_values = i32x4::from_array(*(previous_values_ptr as *const [i32; 4]));
                    return mem::transmute(current_values.simd_lt(previous_values));
                },
                DataType::U64(_) => |current_values_ptr, previous_values_ptr| {
                    let current_values = u64x2::from_array(*(current_values_ptr as *const [u64; 2]));
                    let previous_values = u64x2::from_array(*(previous_values_ptr as *const [u64; 2]));
                    return mem::transmute(current_values.simd_lt(previous_values));
                },
                DataType::I64(_) => |current_values_ptr, previous_values_ptr| {
                    let current_values = i64x2::from_array(*(current_values_ptr as *const [i64; 2]));
                    let previous_values = i64x2::from_array(*(previous_values_ptr as *const [i64; 2]));
                    return mem::transmute(current_values.simd_lt(previous_values));
                },
                DataType::F32(_) => |current_values_ptr, previous_values_ptr| {
                    let current_values = f32x4::from_array(*(current_values_ptr as *const [f32; 4]));
                    let previous_values = f32x4::from_array(*(previous_values_ptr as *const [f32; 4]));
                    return mem::transmute(current_values.simd_lt(previous_values));
                },
                DataType::F64(_) => |current_values_ptr, previous_values_ptr| {
                    let current_values = f64x2::from_array(*(current_values_ptr as *const [f64; 2]));
                    let previous_values = f64x2::from_array(*(previous_values_ptr as *const [f64; 2]));
                    return mem::transmute(current_values.simd_lt(previous_values));
                },
                _ => panic!("unsupported data type"),
            }
        }
    }

    fn get_compare_increased_by(&self, data_type: &DataType) -> VectorCompareFnDelta {
        unsafe {
            match data_type {
                DataType::U8() => |current_values_ptr, previous_values_ptr, immediate_ptr| {
                    let current_values = u8x16::from_array(*(current_values_ptr as *const [u8; 16]));
                    let previous_values = u8x16::from_array(*(previous_values_ptr as *const [u8; 16]));
                    let immediate_value = u8x16::splat(*(immediate_ptr as *const u8));
                    return mem::transmute(current_values.simd_eq(previous_values.add(immediate_value)));
                },
                DataType::I8() => |current_values_ptr, previous_values_ptr, immediate_ptr| {
                    let current_values = i8x16::from_array(*(current_values_ptr as *const [i8; 16]));
                    let previous_values = i8x16::from_array(*(previous_values_ptr as *const [i8; 16]));
                    let immediate_value = i8x16::splat(*(immediate_ptr as *const i8));
                    return mem::transmute(current_values.simd_eq(previous_values.add(immediate_value)));
                },
                DataType::U16(_) => |current_values_ptr, previous_values_ptr, immediate_ptr| {
                    let current_values = u16x8::from_array(*(current_values_ptr as *const [u16; 8]));
                    let previous_values = u16x8::from_array(*(previous_values_ptr as *const [u16; 8]));
                    let immediate_value = u16x8::splat(*(immediate_ptr as *const u16));
                    return mem::transmute(current_values.simd_eq(previous_values.add(immediate_value)));
                },
                DataType::I16(_) => |current_values_ptr, previous_values_ptr, immediate_ptr| {
                    let current_values = i16x8::from_array(*(current_values_ptr as *const [i16; 8]));
                    let previous_values = i16x8::from_array(*(previous_values_ptr as *const [i16; 8]));
                    let immediate_value = i16x8::splat(*(immediate_ptr as *const i16));
                    return mem::transmute(current_values.simd_eq(previous_values.add(immediate_value)));
                },
                DataType::U32(_) => |current_values_ptr, previous_values_ptr, immediate_ptr| {
                    let current_values = u32x4::from_array(*(current_values_ptr as *const [u32; 4]));
                    let previous_values = u32x4::from_array(*(previous_values_ptr as *const [u32; 4]));
                    let immediate_value = u32x4::splat(*(immediate_ptr as *const u32));
                    return mem::transmute(current_values.simd_eq(previous_values.add(immediate_value)));
                },
                DataType::I32(_) => |current_values_ptr, previous_values_ptr, immediate_ptr| {
                    let current_values = i32x4::from_array(*(current_values_ptr as *const [i32; 4]));
                    let previous_values = i32x4::from_array(*(previous_values_ptr as *const [i32; 4]));
                    let immediate_value = i32x4::splat(*(immediate_ptr as *const i32));
                    return mem::transmute(current_values.simd_eq(previous_values.add(immediate_value)));
                },
                DataType::U64(_) => |current_values_ptr, previous_values_ptr, immediate_ptr| {
                    let current_values = u64x2::from_array(*(current_values_ptr as *const [u64; 2]));
                    let previous_values = u64x2::from_array(*(previous_values_ptr as *const [u64; 2]));
                    let immediate_value = u64x2::splat(*(immediate_ptr as *const u64));
                    return mem::transmute(current_values.simd_eq(previous_values.add(immediate_value)));
                },
                DataType::I64(_) => |current_values_ptr, previous_values_ptr, immediate_ptr| {
                    let current_values = i64x2::from_array(*(current_values_ptr as *const [i64; 2]));
                    let previous_values = i64x2::from_array(*(previous_values_ptr as *const [i64; 2]));
                    let immediate_value = i64x2::splat(*(immediate_ptr as *const i64));
                    return mem::transmute(current_values.simd_eq(previous_values.add(immediate_value)));
                },
                DataType::F32(_) => |current_values_ptr, previous_values_ptr, immediate_ptr| {
                    let current_values = f32x4::from_array(*(current_values_ptr as *const [f32; 4]));
                    let previous_values = f32x4::from_array(*(previous_values_ptr as *const [f32; 4]));
                    let immediate_value = f32x4::splat(*(immediate_ptr as *const f32));
                    return mem::transmute(current_values.simd_eq(previous_values.add(immediate_value)));
                },
                DataType::F64(_) => |current_values_ptr, previous_values_ptr, immediate_ptr| {
                    let current_values = f64x2::from_array(*(current_values_ptr as *const [f64; 2]));
                    let previous_values = f64x2::from_array(*(previous_values_ptr as *const [f64; 2]));
                    let immediate_value = f64x2::splat(*(immediate_ptr as *const f64));
                    return mem::transmute(current_values.simd_eq(previous_values.add(immediate_value)));
                },
                _ => panic!("unsupported data type"),
            }
        }
    }

    fn get_compare_decreased_by(&self, data_type: &DataType) -> VectorCompareFnDelta {
        unsafe {
            match data_type {
                DataType::U8() => |current_values_ptr, previous_values_ptr, immediate_ptr| {
                    let current_values = u8x16::from_array(*(current_values_ptr as *const [u8; 16]));
                    let previous_values = u8x16::from_array(*(previous_values_ptr as *const [u8; 16]));
                    let immediate_value = u8x16::splat(*(immediate_ptr as *const u8));
                    return mem::transmute(current_values.simd_eq(previous_values.sub(immediate_value)));
                },
                DataType::I8() => |current_values_ptr, previous_values_ptr, immediate_ptr| {
                    let current_values = i8x16::from_array(*(current_values_ptr as *const [i8; 16]));
                    let previous_values = i8x16::from_array(*(previous_values_ptr as *const [i8; 16]));
                    let immediate_value = i8x16::splat(*(immediate_ptr as *const i8));
                    return mem::transmute(current_values.simd_eq(previous_values.sub(immediate_value)));
                },
                DataType::U16(_) => |current_values_ptr, previous_values_ptr, immediate_ptr| {
                    let current_values = u16x8::from_array(*(current_values_ptr as *const [u16; 8]));
                    let previous_values = u16x8::from_array(*(previous_values_ptr as *const [u16; 8]));
                    let immediate_value = u16x8::splat(*(immediate_ptr as *const u16));
                    return mem::transmute(current_values.simd_eq(previous_values.sub(immediate_value)));
                },
                DataType::I16(_) => |current_values_ptr, previous_values_ptr, immediate_ptr| {
                    let current_values = i16x8::from_array(*(current_values_ptr as *const [i16; 8]));
                    let previous_values = i16x8::from_array(*(previous_values_ptr as *const [i16; 8]));
                    let immediate_value = i16x8::splat(*(immediate_ptr as *const i16));
                    return mem::transmute(current_values.simd_eq(previous_values.sub(immediate_value)));
                },
                DataType::U32(_) => |current_values_ptr, previous_values_ptr, immediate_ptr| {
                    let current_values = u32x4::from_array(*(current_values_ptr as *const [u32; 4]));
                    let previous_values = u32x4::from_array(*(previous_values_ptr as *const [u32; 4]));
                    let immediate_value = u32x4::splat(*(immediate_ptr as *const u32));
                    return mem::transmute(current_values.simd_eq(previous_values.sub(immediate_value)));
                },
                DataType::I32(_) => |current_values_ptr, previous_values_ptr, immediate_ptr| {
                    let current_values = i32x4::from_array(*(current_values_ptr as *const [i32; 4]));
                    let previous_values = i32x4::from_array(*(previous_values_ptr as *const [i32; 4]));
                    let immediate_value = i32x4::splat(*(immediate_ptr as *const i32));
                    return mem::transmute(current_values.simd_eq(previous_values.sub(immediate_value)));
                },
                DataType::U64(_) => |current_values_ptr, previous_values_ptr, immediate_ptr| {
                    let current_values = u64x2::from_array(*(current_values_ptr as *const [u64; 2]));
                    let previous_values = u64x2::from_array(*(previous_values_ptr as *const [u64; 2]));
                    let immediate_value = u64x2::splat(*(immediate_ptr as *const u64));
                    return mem::transmute(current_values.simd_eq(previous_values.sub(immediate_value)));
                },
                DataType::I64(_) => |current_values_ptr, previous_values_ptr, immediate_ptr| {
                    let current_values = i64x2::from_array(*(current_values_ptr as *const [i64; 2]));
                    let previous_values = i64x2::from_array(*(previous_values_ptr as *const [i64; 2]));
                    let immediate_value = i64x2::splat(*(immediate_ptr as *const i64));
                    return mem::transmute(current_values.simd_eq(previous_values.sub(immediate_value)));
                },
                DataType::F32(_) => |current_values_ptr, previous_values_ptr, immediate_ptr| {
                    let current_values = f32x4::from_array(*(current_values_ptr as *const [f32; 4]));
                    let previous_values = f32x4::from_array(*(previous_values_ptr as *const [f32; 4]));
                    let immediate_value = f32x4::splat(*(immediate_ptr as *const f32));
                    return mem::transmute(current_values.simd_eq(previous_values.sub(immediate_value)));
                },
                DataType::F64(_) => |current_values_ptr, previous_values_ptr, immediate_ptr| {
                    let current_values = f64x2::from_array(*(current_values_ptr as *const [f64; 2]));
                    let previous_values = f64x2::from_array(*(previous_values_ptr as *const [f64; 2]));
                    let immediate_value = f64x2::splat(*(immediate_ptr as *const f64));
                    return mem::transmute(current_values.simd_eq(previous_values.sub(immediate_value)));
                },
                _ => panic!("unsupported data type"),
            }
        }
    }   
}
