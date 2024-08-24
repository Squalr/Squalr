use crate::scanners::parameters::scan_compare_type::ScanCompareType;
use squalr_engine_common::values::data_type::DataType;
use std::ops::{Add, Sub};
use std::simd::cmp::{SimdPartialEq, SimdPartialOrd};
use std::simd::Simd;
use std::sync::Once;

type VectorCompareFnImmediate128 = unsafe fn(current_value_pointer: *const u8, immediate_value_pointer: *const u8) -> Simd<u8, 16>;
type VectorCompareFnImmediate256 = unsafe fn(current_value_pointer: *const u8, immediate_value_pointer: *const u8) -> Simd<u8, 32>;
type VectorCompareFnImmediate512 = unsafe fn(current_value_pointer: *const u8, immediate_value_pointer: *const u8) -> Simd<u8, 64>;

type VectorCompareFnRelative128 = unsafe fn(current_value_pointer: *const u8, previous_value_pointer: *const u8) -> Simd<u8, 16>;
type VectorCompareFnRelative256 = unsafe fn(current_value_pointer: *const u8, previous_value_pointer: *const u8) -> Simd<u8, 32>;
type VectorCompareFnRelative512 = unsafe fn(current_value_pointer: *const u8, previous_value_pointer: *const u8) -> Simd<u8, 64>;

type VectorCompareFnDelta128 = unsafe fn(current_value_pointer: *const u8, previous_value_pointer: *const u8, delta_value_pointer: *const u8) -> Simd<u8, 16>;
type VectorCompareFnDelta256 = unsafe fn(current_value_pointer: *const u8, previous_value_pointer: *const u8, delta_value_pointer: *const u8) -> Simd<u8, 32>;
type VectorCompareFnDelta512 = unsafe fn(current_value_pointer: *const u8, previous_value_pointer: *const u8, delta_value_pointer: *const u8) -> Simd<u8, 64>;

pub struct ScannerVectorComparer<const VECTOR_SIZE_BITS: usize>;

macro_rules! define_simd_type {
    ($bit_width:tt, $data_type:ident) => { Simd::<$data_type, { $bit_width / (std::mem::size_of::<$data_type>() * 8) }> };
}

// Primary SIMD logic for relative comparisons
macro_rules! immediate_simd_compare {
    ($simd_fn:ident, $bit_width:tt, $data_type:ident) => {{
        type SimdType = define_simd_type!($bit_width, $data_type);
        |current_values_ptr, immediate_ptr| {
            // Load the immediate value into a SIMD register.
            let immediate_value = SimdType::splat(*(immediate_ptr as *const $data_type));
            // Load the current values from a pointer into the SIMD register.
            let current_values = SimdType::from_array(*(current_values_ptr as *const [$data_type; $bit_width / (std::mem::size_of::<$data_type>() * 8)]));
            // Compare and cast from element-wise SIMD vector to Byte-wise SIMD vector.
            std::mem::transmute(current_values.$simd_fn(immediate_value))
        }
    }};
}

macro_rules! relative_simd_compare {
    ($simd_fn:ident, $bit_width:tt, $data_type:ident) => {{
        type SimdType = define_simd_type!($bit_width, $data_type);
        |current_values_ptr, previous_values_ptr| {
            // Load the current values from a pointer into the SIMD register.
            let current_values = SimdType::from_array(*(current_values_ptr as *const [$data_type; $bit_width / (std::mem::size_of::<$data_type>() * 8)]));
            // Load the previous values from a pointer into the SIMD register.
            let previous_values = SimdType::from_array(*(previous_values_ptr as *const [$data_type; $bit_width / (std::mem::size_of::<$data_type>() * 8)]));
            // Compare and cast from element-wise SIMD vector to Byte-wise SIMD vector.
            std::mem::transmute(current_values.$simd_fn(previous_values))
        }
    }};
}

macro_rules! relative_delta_simd_compare {
    ($simd_op:ident, $bit_width:tt, $data_type:ident) => {{
        type SimdType = define_simd_type!($bit_width, $data_type);
        |current_values_ptr, previous_values_ptr, delta_ptr| {
            // Load the current values from a pointer into the SIMD register.
            let current_values = SimdType::from_array(*(current_values_ptr as *const [$data_type; $bit_width / (std::mem::size_of::<$data_type>() * 8)]));
            // Load the previous values from a pointer into the SIMD register.
            let previous_values = SimdType::from_array(*(previous_values_ptr as *const [$data_type; $bit_width / (std::mem::size_of::<$data_type>() * 8)]));
            // Load the delta value into a SIMD register.
            let delta_value = SimdType::splat(*(delta_ptr as *const $data_type));
            // Compare and cast from element-wise SIMD vector to Byte-wise SIMD vector.
            std::mem::transmute(current_values.simd_eq(previous_values.$simd_op(delta_value)))
        }
    }};
}

macro_rules! define_compare_fn {
    ($simd_fn:ident, $bit_width:tt, $fn_name:ident, $fn_type:ident) => {
        fn $fn_name(&self, data_type: &DataType) -> $fn_type {
            unsafe {
                match data_type {
                    DataType::U8() => immediate_simd_compare!($simd_fn, $bit_width, u8),
                    DataType::I8() => immediate_simd_compare!($simd_fn, $bit_width, i8),
                    DataType::U16(_) => immediate_simd_compare!($simd_fn, $bit_width, u16),
                    DataType::I16(_) => immediate_simd_compare!($simd_fn, $bit_width, i16),
                    DataType::U32(_) => immediate_simd_compare!($simd_fn, $bit_width, u32),
                    DataType::I32(_) => immediate_simd_compare!($simd_fn, $bit_width, i32),
                    DataType::U64(_) => immediate_simd_compare!($simd_fn, $bit_width, u64),
                    DataType::I64(_) => immediate_simd_compare!($simd_fn, $bit_width, i64),
                    DataType::F32(_) => immediate_simd_compare!($simd_fn, $bit_width, f32),
                    DataType::F64(_) => immediate_simd_compare!($simd_fn, $bit_width, f64),
                    _ => panic!("Unsupported data type"),
                }
            }
        }
    };
}

macro_rules! define_relative_compare_fn {
    ($simd_fn:ident, $bit_width:tt, $fn_name:ident, $fn_type:ident) => {
        fn $fn_name(&self, data_type: &DataType) -> $fn_type {
            unsafe {
                match data_type {
                    DataType::U8() => relative_simd_compare!($simd_fn, $bit_width, u8),
                    DataType::I8() => relative_simd_compare!($simd_fn, $bit_width, i8),
                    DataType::U16(_) => relative_simd_compare!($simd_fn, $bit_width, u16),
                    DataType::I16(_) => relative_simd_compare!($simd_fn, $bit_width, i16),
                    DataType::U32(_) => relative_simd_compare!($simd_fn, $bit_width, u32),
                    DataType::I32(_) => relative_simd_compare!($simd_fn, $bit_width, i32),
                    DataType::U64(_) => relative_simd_compare!($simd_fn, $bit_width, u64),
                    DataType::I64(_) => relative_simd_compare!($simd_fn, $bit_width, i64),
                    DataType::F32(_) => relative_simd_compare!($simd_fn, $bit_width, f32),
                    DataType::F64(_) => relative_simd_compare!($simd_fn, $bit_width, f64),
                    _ => panic!("Unsupported data type"),
                }
            }
        }
    };
}

macro_rules! define_delta_compare_fn {
    ($simd_op:ident, $bit_width:tt, $fn_name:ident, $fn_type:ident) => {
        fn $fn_name(&self, data_type: &DataType) -> $fn_type {
            unsafe {
                match data_type {
                    DataType::U8() => relative_delta_simd_compare!($simd_op, $bit_width, u8),
                    DataType::I8() => relative_delta_simd_compare!($simd_op, $bit_width, i8),
                    DataType::U16(_) => relative_delta_simd_compare!($simd_op, $bit_width, u16),
                    DataType::I16(_) => relative_delta_simd_compare!($simd_op, $bit_width, i16),
                    DataType::U32(_) => relative_delta_simd_compare!($simd_op, $bit_width, u32),
                    DataType::I32(_) => relative_delta_simd_compare!($simd_op, $bit_width, i32),
                    DataType::U64(_) => relative_delta_simd_compare!($simd_op, $bit_width, u64),
                    DataType::I64(_) => relative_delta_simd_compare!($simd_op, $bit_width, i64),
                    DataType::F32(_) => relative_delta_simd_compare!($simd_op, $bit_width, f32),
                    DataType::F64(_) => relative_delta_simd_compare!($simd_op, $bit_width, f64),
                    _ => panic!("Unsupported data type"),
                }
            }
        }
    };
}

macro_rules! impl_scanner_vector_comparer {
    ($bit_width:tt, $fn_type_immediate:ident, $fn_type_relative:ident, $fn_type_delta:ident) => {
        impl ScannerVectorComparer<$bit_width> {
            pub fn get_instance() -> &'static ScannerVectorComparer<$bit_width> {
                static mut INSTANCE: Option<ScannerVectorComparer<$bit_width>> = None;
                static INIT: Once = Once::new();

                unsafe {
                    INIT.call_once(|| {
                        let instance = ScannerVectorComparer::<$bit_width>::new();
                        INSTANCE = Some(instance);
                    });

                    INSTANCE.as_ref().unwrap_unchecked()
                }
            }

            fn new() -> Self {
                Self {}
            }

            define_compare_fn!(simd_eq, $bit_width, get_compare_equal, $fn_type_immediate);
            define_compare_fn!(simd_ne, $bit_width, get_compare_not_equal, $fn_type_immediate);
            define_compare_fn!(simd_gt, $bit_width, get_compare_greater_than, $fn_type_immediate);
            define_compare_fn!(simd_ge, $bit_width, get_compare_greater_than_or_equal, $fn_type_immediate);
            define_compare_fn!(simd_lt, $bit_width, get_compare_less_than, $fn_type_immediate);
            define_compare_fn!(simd_le, $bit_width, get_compare_less_than_or_equal, $fn_type_immediate);

            define_relative_compare_fn!(simd_ne, $bit_width, get_compare_changed, $fn_type_relative);
            define_relative_compare_fn!(simd_eq, $bit_width, get_compare_unchanged, $fn_type_relative);
            define_relative_compare_fn!(simd_gt, $bit_width, get_compare_increased, $fn_type_relative);
            define_relative_compare_fn!(simd_lt, $bit_width, get_compare_decreased, $fn_type_relative);

            define_delta_compare_fn!(add, $bit_width, get_compare_increased_by, $fn_type_delta);
            define_delta_compare_fn!(sub, $bit_width, get_compare_decreased_by, $fn_type_delta);
            
            pub fn get_immediate_compare_func(
                &self,
                scan_compare_type: ScanCompareType,
                data_type: &DataType,
            ) -> $fn_type_immediate {
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
            ) -> $fn_type_relative {
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
            ) -> $fn_type_delta {
                match scan_compare_type {
                    ScanCompareType::IncreasedByX => self.get_compare_increased_by(data_type),
                    ScanCompareType::DecreasedByX => self.get_compare_decreased_by(data_type),
                    _ => panic!("Unsupported type passed to get_relative_delta_compare_func"),
                }
            }
        }
    };
}

impl_scanner_vector_comparer!(128, VectorCompareFnImmediate128, VectorCompareFnRelative128, VectorCompareFnDelta128);
impl_scanner_vector_comparer!(256, VectorCompareFnImmediate256, VectorCompareFnRelative256, VectorCompareFnDelta256);
impl_scanner_vector_comparer!(512, VectorCompareFnImmediate512, VectorCompareFnRelative512, VectorCompareFnDelta512);
