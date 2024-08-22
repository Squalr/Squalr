use squalr_engine_common::values::data_type::DataType;

use crate::scanners::parameters::scan_compare_type::ScanCompareType;
use std::sync::Once;
use std::simd::cmp::{SimdPartialEq, SimdPartialOrd};
use std::simd::{i16x8, i32x4, i8x16, mask16x8, u16x8, u32x4, u8x16};
use std::simd::mask8x16;

/// Defines a compare function that operates on an immediate (ie all inequalities)
type VectorCompareFnImmediate = unsafe fn(
    // Current value buffer
    current_value_pointer: *const u8,
    // Immediate value
    u8x16,
) -> mask8x16;

/// Defines a compare function that operates on current and previous values (ie changed, unchanged, increased, decreased)
type VectorCompareFnRelative = unsafe fn(
    // Current value buffer
    current_value_pointer: *const u8,
    // Previous value buffer
    previous_value_pointer: *const u8,
) -> mask8x16;

/// Defines a compare function that operates on current and previous values, with a delta arg (ie +x, -x)
type VectorCompareFnDelta = unsafe fn(
    // Current value buffer
    current_value_pointer: *const u8,
    // Previous value buffer
    previous_value_pointer: *const u8,
    // Delta value buffer
    u8x16,
) -> mask8x16;

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
    
    fn load_u8x16_from_raw_ptr(ptr: *const u8) -> u8x16 {
        unsafe {
            // Load the 16 bytes from the raw pointer into a u8x16
            u8x16::from_slice(std::slice::from_raw_parts(ptr, 16))
        }
    }

    pub fn get_immediate_compare_func(
        &self,
        scan_compare_type: ScanCompareType
    ) -> VectorCompareFnImmediate {
        match scan_compare_type {
            ScanCompareType::Equal => Self::compare_equal,
            ScanCompareType::NotEqual => Self::compare_not_equal,
            ScanCompareType::GreaterThan => Self::compare_greater_than,
            ScanCompareType::GreaterThanOrEqual => Self::compare_greater_than_or_equal,
            ScanCompareType::LessThan => Self::compare_less_than,
            ScanCompareType::LessThanOrEqual => Self::compare_less_than_or_equal,
            _ => panic!("Unsupported type passed to get_immediate_compare_func"),
        }
    }

    pub fn get_relative_compare_func(
        &self,
        scan_compare_type: ScanCompareType
    ) -> VectorCompareFnRelative {
        match scan_compare_type {
            ScanCompareType::Changed => Self::compare_changed,
            ScanCompareType::Unchanged => Self::compare_unchanged,
            ScanCompareType::Increased => Self::compare_increased,
            ScanCompareType::Decreased => Self::compare_decreased,
            _ => panic!("Unsupported type passed to get_relative_compare_func"),
        }
    }

    pub fn get_relative_delta_compare_func(
        &self,
        scan_compare_type: ScanCompareType
    ) -> VectorCompareFnDelta {
        match scan_compare_type {
            ScanCompareType::IncreasedByX => Self::compare_increased_by,
            ScanCompareType::DecreasedByX => Self::compare_decreased_by,
            _ => panic!("Unsupported type passed to get_relative_delta_compare_func"),
        }
    }

    // Doesn't really work that well since for larger types we would need to somehow cast out the bytes to a larger type idk
    // Maybe if we start packing results into a u8x16 then it doesn't matter
    // Need to think on this more.
    unsafe fn get_compare_equal(
        data_type: DataType,
    ) -> VectorCompareFnImmediate {
        match data_type {
            DataType::U8() => {
                unsafe fn compare(current_value_pointer: *const u8, compare_value_ref: u8x16) -> mask8x16 {
                    let current = ScannerVectorComparer::load_u8x16_from_raw_ptr(current_value_pointer);
                    return current.simd_eq(compare_value_ref);
                }
                compare
            },
            _ => panic!("Unsupported type"),
        }
    }

    unsafe fn compare_equal(
        current_value_pointer: *const u8,
        compare_value_ref: u8x16
    ) -> mask8x16 {
        let current = Self::load_u8x16_from_raw_ptr(current_value_pointer);

        return current.simd_eq(compare_value_ref);
    }

    unsafe fn compare_not_equal(
        current_value_pointer: *const u8,
        compare_value_ref: u8x16
    ) -> mask8x16 {
        let current = Self::load_u8x16_from_raw_ptr(current_value_pointer);

        return current.simd_ne(compare_value_ref);
    }

    unsafe fn compare_greater_than(
        current_value_pointer: *const u8,
        compare_value_ref: u8x16
    ) -> mask8x16 {
        let current = Self::load_u8x16_from_raw_ptr(current_value_pointer);

        return current.simd_gt(compare_value_ref);
    }

    unsafe fn compare_greater_than_or_equal(
        current_value_pointer: *const u8,
        compare_value_ref: u8x16
    ) -> mask8x16 {
        let current = Self::load_u8x16_from_raw_ptr(current_value_pointer);

        return current.simd_ge(compare_value_ref);
    }

    unsafe fn compare_less_than(
        current_value_pointer: *const u8,
        compare_value_ref: u8x16
    ) -> mask8x16 {
        let current = Self::load_u8x16_from_raw_ptr(current_value_pointer);

        return current.simd_lt(compare_value_ref);
    }

    unsafe fn compare_less_than_or_equal(
        current_value_pointer: *const u8,
        compare_value_ref: u8x16
    ) -> mask8x16 {
        let current = Self::load_u8x16_from_raw_ptr(current_value_pointer);
        
        return current.simd_le(compare_value_ref);
    }

    unsafe fn compare_changed(
        current_value_pointer: *const u8,
        compare_value_pointer: *const u8,
    ) -> mask8x16 {
        let current = Self::load_u8x16_from_raw_ptr(current_value_pointer);
        let compare = Self::load_u8x16_from_raw_ptr(compare_value_pointer);

        return current.simd_ne(compare);
    }

    unsafe fn compare_unchanged(
        current_value_pointer: *const u8,
        compare_value_pointer: *const u8,
    ) -> mask8x16 {
        let current = Self::load_u8x16_from_raw_ptr(current_value_pointer);
        let compare = Self::load_u8x16_from_raw_ptr(compare_value_pointer);

        return current.simd_eq(compare);
    }

    unsafe fn compare_increased(
        current_value_pointer: *const u8,
        compare_value_pointer: *const u8,
    ) -> mask8x16 {
        let current = Self::load_u8x16_from_raw_ptr(current_value_pointer);
        let compare = Self::load_u8x16_from_raw_ptr(compare_value_pointer);

        return current.simd_gt(compare);
    }

    unsafe fn compare_decreased(
        current_value_pointer: *const u8,
        compare_value_pointer: *const u8,
    ) -> mask8x16 {
        let current = Self::load_u8x16_from_raw_ptr(current_value_pointer);
        let compare = Self::load_u8x16_from_raw_ptr(compare_value_pointer);

        return current.simd_lt(compare);
    }

    unsafe fn compare_increased_by(
        current_value_pointer: *const u8,
        compare_value_pointer: *const u8,
        compare_delta_value: u8x16
    ) -> mask8x16 {
        let current = Self::load_u8x16_from_raw_ptr(current_value_pointer);
        let compare = Self::load_u8x16_from_raw_ptr(compare_value_pointer);
        
        return current.simd_eq(compare + compare_delta_value);
    }

    unsafe fn compare_decreased_by(
        current_value_pointer: *const u8,
        compare_value_pointer: *const u8,
        compare_delta_value: u8x16
    ) -> mask8x16 {
        let current = Self::load_u8x16_from_raw_ptr(current_value_pointer);
        let compare = Self::load_u8x16_from_raw_ptr(compare_value_pointer);
        
        return current.simd_eq(compare - compare_delta_value);
    }
}
