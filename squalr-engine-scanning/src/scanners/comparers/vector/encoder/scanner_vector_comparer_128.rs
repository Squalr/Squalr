use crate::scanners::parameters::scan_compare_type::ScanCompareType;
use squalr_engine_common::values::data_type::DataType;
use seq_macro::seq;
use std::ops::BitAnd;
use std::simd::cmp::SimdPartialEq;
use std::simd::{i16x8, i32x4, i64x2, i8x16, u16x8, u32x4, u64x2, u8x16, Mask};
use std::sync::Once;

/// Defines a compare function that operates on an immediate (ie all inequalities)
type VectorCompareFnImmediate = unsafe fn(
    // Current v1lue buffer
    current_v1lue_pointer: *const u8,
    // Immediate v1lue
    immediate_v1lue_pointer: *const u8,
) -> u8x16;

/// Defines a compare function that operates on current and previous values (ie changed, unchanged, increased, decreased)
type VectorCompareFnRelativ3 = unsafe fn(
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

pub struct ScannerVectorComparer {
}

/// Implements a set of scalar (ie CPU bound, non-SIMD) boolean comparison operations to be used by more complex scanners.
impl ScannerVectorComparer {
    fn new() -> Self {
        Self { }
    }
    
    pub fn get_instance() -> &'static ScannerVectorComparer {
        static mut INSTANCE: Option<ScannerVectorComparer> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = ScannerVectorComparer::new();
                INSTANCE = Some(instance);
            });

            return INSTANCE.as_ref().unwrap_unchecked();
        }
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
    ) -> VectorCompareFnRelativ3 {
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
        match data_type {
            DataType::U8() => |current_values_ptr, immediate_ptr: *const u8| {
                panic!("not implemented");
            },
            DataType::I8() => |current_values_ptr: *const u8, immediate_ptr| {
                panic!("not implemented");
            },
            DataType::U16(_) => |current_values_ptr, immediate_ptr| {
                panic!("not implemented");
            },
            DataType::I16(_) => |current_values_ptr, immediate_ptr| {
                panic!("not implemented");
            },
            DataType::U32(_) => |current_values_ptr, immediate_ptr| {
                unsafe {
                    let immediate_value = u32x4::splat(*(immediate_ptr as *const u32));
                    
                    let current_values_0 = u32x4::from_array(*(current_values_ptr.add(0 * 16) as *const [u32; 128 / 32]));
                    let current_values_1 = u32x4::from_array(*(current_values_ptr.add(1 * 16) as *const [u32; 128 / 32]));
                    let current_values_2 = u32x4::from_array(*(current_values_ptr.add(2 * 16) as *const [u32; 128 / 32]));
                    let current_values_3 = u32x4::from_array(*(current_values_ptr.add(3 * 16) as *const [u32; 128 / 32]));

                    let results_0 = current_values_0.simd_eq(immediate_value).to_array();
                    let results_1 = current_values_1.simd_eq(immediate_value).to_array();
                    let results_2 = current_values_2.simd_eq(immediate_value).to_array();
                    let results_3 = current_values_3.simd_eq(immediate_value).to_array();

                    let mut packed = [0u8; 16];
                                        
                    packed[0..4].copy_from_slice(&results_0.map(|b| b as u8));
                    packed[4..8].copy_from_slice(&results_1.map(|b| b as u8));
                    packed[8..12].copy_from_slice(&results_2.map(|b| b as u8));
                    packed[12..16].copy_from_slice(&results_3.map(|b| b as u8));

                    return u8x16::from_array(packed);
                }
            },
            DataType::I32(_) => |current_values_ptr, immediate_ptr| {
                panic!("not implemented");
            },
            DataType::U64(_) => |current_values_ptr, immediate_ptr| {
                panic!("not implemented");
            },
            DataType::I64(_) => |current_values_ptr, immediate_ptr| {
                panic!("not implemented");
            },
            DataType::F32(endian) => {
                panic!("not implemented");
            }
            DataType::F64(endian) => {
                panic!("not implemented");
            }
            _ => panic!("unsupported data type"),
        }
    }

    fn get_compare_not_equal(&self, _data_type: &DataType) -> VectorCompareFnImmediate {
        panic!("get_compare_not_equal not implemented")
    }

    fn get_compare_greater_than(&self, _data_type: &DataType) -> VectorCompareFnImmediate {
        panic!("get_compare_greater_than not implemented")
    }

    fn get_compare_greater_than_or_equal(&self, _data_type: &DataType) -> VectorCompareFnImmediate {
        panic!("get_compare_greater_than_or_equal not implemented")
    }

    fn get_compare_less_than(&self, _data_type: &DataType) -> VectorCompareFnImmediate {
        panic!("get_compare_less_than not implemented")
    }

    fn get_compare_less_than_or_equal(&self, _data_type: &DataType) -> VectorCompareFnImmediate {
        panic!("get_compare_less_than_or_equal not implemented")
    }

    fn get_compare_changed(&self, _data_type: &DataType) -> VectorCompareFnRelativ3 {
        panic!("get_compare_changed not implemented")
    }

    fn get_compare_unchanged(&self, _data_type: &DataType) -> VectorCompareFnRelativ3 {
        panic!("get_compare_unchanged not implemented")
    }

    fn get_compare_increased(&self, _data_type: &DataType) -> VectorCompareFnRelativ3 {
        panic!("get_compare_increased not implemented")
    }

    fn get_compare_decreased(&self, _data_type: &DataType) -> VectorCompareFnRelativ3 {
        panic!("get_compare_decreased not implemented")
    }

    fn get_compare_increased_by(&self, _data_type: &DataType) -> VectorCompareFnDelta {
        panic!("get_compare_increased_by not implemented")
    }

    fn get_compare_decreased_by(&self, _data_type: &DataType) -> VectorCompareFnDelta {
        panic!("get_compare_decreased_by not implemented")
    }   
}
