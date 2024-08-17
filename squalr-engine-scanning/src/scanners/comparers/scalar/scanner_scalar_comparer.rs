use core::panic;

use crate::scanners::constraints::scan_constraint_type::ScanConstraintType;
use squalr_engine_common::dynamic_struct::data_type::DataLoadFunc;
use squalr_engine_common::dynamic_struct::data_value::DataValue;
use std::sync::Once;

/// Defines a compare function that operates on an immediate (ie all inequalities)
type ScalarCompareFnImmediate = unsafe fn(
    // The function used to load new values from memory
    &DataLoadFunc,
    // Current values pointer
    *const u8,
    // Current value struct ref
    &mut DataValue,
    // Immediate value
    &DataValue,
) -> bool;

/// Defines a compare function that operates on current and previous values (ie changed, unchanged, increased, decreased)
type ScalarCompareFnRelative = unsafe fn(
    // The function used to load new values from memory
    &DataLoadFunc,
    // Current values pointer
    *const u8,
    // Previous values pointer
    *const u8,
    // Current value struct ref
    &mut DataValue,
    // Previous value struct ref
    &mut DataValue,
) -> bool;

/// Defines a compare function that operates on current and previous values, with a delta arg (ie +x, -x)
type ScalarCompareFnDelta = unsafe fn(
    // The function used to load new values from memory
    &DataLoadFunc,
    // Current values pointer
    *const u8,
    // Previous values pointer
    *const u8,
    // Current value struct ref
    &mut DataValue,
    // Previous value struct ref
    &mut DataValue,
    // Delta value
    &DataValue,
) -> bool;

pub struct ScannerScalarComparer {
}

/// Implements a set of scalar (ie CPU bound, non-SIMD) boolean comparison operations to be used by more complex scanners.
impl ScannerScalarComparer {
    fn new() -> Self {
        Self { }
    }
    
    pub fn get_instance() -> &'static ScannerScalarComparer {
        static mut INSTANCE: Option<ScannerScalarComparer> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = ScannerScalarComparer::new();
                INSTANCE = Some(instance);
            });

            return INSTANCE.as_ref().unwrap_unchecked();
        }
    }

    pub fn get_immediate_compare_func(&self, scan_constraint_type: ScanConstraintType) -> ScalarCompareFnImmediate {
        match scan_constraint_type {
            ScanConstraintType::Equal => Self::compare_equal,
            ScanConstraintType::NotEqual => Self::compare_not_equal,
            ScanConstraintType::GreaterThan => Self::compare_greater_than,
            ScanConstraintType::GreaterThanOrEqual => Self::compare_greater_than_or_equal,
            ScanConstraintType::LessThan => Self::compare_less_than,
            ScanConstraintType::LessThanOrEqual => Self::compare_less_than_or_equal,
            _ => panic!("Unsupported type passed to get_immediate_compare_func"),
        }
    }

    pub fn get_relative_compare_func(&self, scan_constraint_type: ScanConstraintType) -> ScalarCompareFnRelative {
        match scan_constraint_type {
            ScanConstraintType::Changed => Self::compare_changed,
            ScanConstraintType::Unchanged => Self::compare_unchanged,
            ScanConstraintType::Increased => Self::compare_increased,
            ScanConstraintType::Decreased => Self::compare_decreased,
            _ => panic!("Unsupported type passed to get_relative_compare_func"),
        }
    }

    pub fn get_relative_delta_compare_func(&self, scan_constraint_type: ScanConstraintType) -> ScalarCompareFnDelta {
        match scan_constraint_type {
            ScanConstraintType::IncreasedByX => Self::compare_increased_by,
            ScanConstraintType::DecreasedByX => Self::compare_decreased_by,
            _ => panic!("Unsupported type passed to get_relative_delta_compare_func"),
        }
    }

    unsafe fn compare_equal(
        memory_load_func: &DataLoadFunc,
        current_value_ptr: *const u8,
        current_value_ref: &mut DataValue,
        compare_value: &DataValue
    ) -> bool {
        memory_load_func(current_value_ref, current_value_ptr);

        return current_value_ref == compare_value;
    }

    unsafe fn compare_not_equal(
        memory_load_func: &DataLoadFunc,
        current_value_ptr: *const u8,
        current_value_ref: &mut DataValue,
        compare_value: &DataValue
    ) -> bool {
        memory_load_func(current_value_ref, current_value_ptr);

        return current_value_ref != compare_value;
    }
    
    unsafe fn compare_changed(
        memory_load_func: &DataLoadFunc,
        current_value_ptr: *const u8,
        compare_value_ptr: *const u8,
        current_value_ref: &mut DataValue,
        compare_value_ref: &mut DataValue
    ) -> bool {
        memory_load_func(current_value_ref, current_value_ptr);
        memory_load_func(compare_value_ref, compare_value_ptr);
        current_value_ref != compare_value_ref
    }

    unsafe fn compare_unchanged(
        memory_load_func: &DataLoadFunc,
        current_value_ptr: *const u8,
        compare_value_ptr: *const u8,
        current_value_ref: &mut DataValue,
        compare_value_ref: &mut DataValue
    ) -> bool {
        memory_load_func(current_value_ref, current_value_ptr);
        memory_load_func(compare_value_ref, compare_value_ptr);
        current_value_ref == compare_value_ref
    }

    unsafe fn compare_increased(
        memory_load_func: &DataLoadFunc,
        current_value_ptr: *const u8,
        compare_value_ptr: *const u8,
        current_value_ref: &mut DataValue,
        compare_value_ref: &mut DataValue
    ) -> bool {
        memory_load_func(current_value_ref, current_value_ptr);
        memory_load_func(compare_value_ref, compare_value_ptr);

        return current_value_ref > compare_value_ref;
    }

    unsafe fn compare_decreased(
        memory_load_func: &DataLoadFunc,
        current_value_ptr: *const u8,
        compare_value_ptr: *const u8,
        current_value_ref: &mut DataValue,
        compare_value_ref: &mut DataValue
    ) -> bool {
        memory_load_func(current_value_ref, current_value_ptr);
        memory_load_func(compare_value_ref, compare_value_ptr);

        return current_value_ref < compare_value_ref;
    }

    unsafe fn compare_greater_than(
        memory_load_func: &DataLoadFunc,
        current_value_ptr: *const u8,
        current_value_ref: &mut DataValue,
        compare_value: &DataValue
    ) -> bool {
        memory_load_func(current_value_ref, current_value_ptr);

        return *current_value_ref > *compare_value;
    }

    unsafe fn compare_greater_than_or_equal(
        memory_load_func: &DataLoadFunc,
        current_value_ptr: *const u8,
        current_value_ref: &mut DataValue,
        compare_value: &DataValue
    ) -> bool {
        memory_load_func(current_value_ref, current_value_ptr);

        return *current_value_ref >= *compare_value;
    }

    unsafe fn compare_less_than(
        memory_load_func: &DataLoadFunc,
        current_value_ptr: *const u8,
        current_value_ref: &mut DataValue,
        compare_value: &DataValue
    ) -> bool {
        memory_load_func(current_value_ref, current_value_ptr);

        return *current_value_ref < *compare_value;
    }

    unsafe fn compare_less_than_or_equal(
        memory_load_func: &DataLoadFunc,
        current_value_ptr: *const u8,
        current_value_ref: &mut DataValue,
        compare_value: &DataValue
    ) -> bool {
        memory_load_func(current_value_ref, current_value_ptr);

        return *current_value_ref <= *compare_value;
    }

    unsafe fn compare_increased_by(
        memory_load_func: &DataLoadFunc,
        current_value_ptr: *const u8,
        compare_value_ptr: *const u8,
        current_value_ref: &mut DataValue,
        compare_value_ref: &mut DataValue,
        compare_delta_value: &DataValue
    )-> bool {
        memory_load_func(current_value_ref, current_value_ptr);
        memory_load_func(compare_value_ref, compare_value_ptr);

        match (current_value_ref, compare_value_ref) {
            (DataValue::U8(a), DataValue::U8(b)) => *a == b.wrapping_add(compare_delta_value.as_u8().unwrap()),
            (DataValue::I8(a), DataValue::I8(b)) => *a == b.wrapping_add(compare_delta_value.as_i8().unwrap()),
            (DataValue::U16(a), DataValue::U16(b)) => *a == b.wrapping_add(compare_delta_value.as_u16().unwrap()),
            (DataValue::I16(a), DataValue::I16(b)) => *a == b.wrapping_add(compare_delta_value.as_i16().unwrap()),
            (DataValue::U32(a), DataValue::U32(b)) => *a == b.wrapping_add(compare_delta_value.as_u32().unwrap()),
            (DataValue::I32(a), DataValue::I32(b)) => *a == b.wrapping_add(compare_delta_value.as_i32().unwrap()),
            (DataValue::U64(a), DataValue::U64(b)) => *a == b.wrapping_add(compare_delta_value.as_u64().unwrap()),
            (DataValue::I64(a), DataValue::I64(b)) => *a == b.wrapping_add(compare_delta_value.as_i64().unwrap()),
            _ => false,
        }
    }

    unsafe fn compare_decreased_by(
        memory_load_func: &DataLoadFunc,
        current_value_ptr: *const u8,
        compare_value_ptr: *const u8,
        current_value_ref: &mut DataValue,
        compare_value_ref: &mut DataValue,
        compare_delta_value: &DataValue
    )-> bool {
        memory_load_func(current_value_ref, current_value_ptr);
        memory_load_func(compare_value_ref, compare_value_ptr);

        match (current_value_ref, compare_value_ref) {
            (DataValue::U8(a), DataValue::U8(b)) => *a == b.wrapping_sub(compare_delta_value.as_u8().unwrap()),
            (DataValue::I8(a), DataValue::I8(b)) => *a == b.wrapping_sub(compare_delta_value.as_i8().unwrap()),
            (DataValue::U16(a), DataValue::U16(b)) => *a == b.wrapping_sub(compare_delta_value.as_u16().unwrap()),
            (DataValue::I16(a), DataValue::I16(b)) => *a == b.wrapping_sub(compare_delta_value.as_i16().unwrap()),
            (DataValue::U32(a), DataValue::U32(b)) => *a == b.wrapping_sub(compare_delta_value.as_u32().unwrap()),
            (DataValue::I32(a), DataValue::I32(b)) => *a == b.wrapping_sub(compare_delta_value.as_i32().unwrap()),
            (DataValue::U64(a), DataValue::U64(b)) => *a == b.wrapping_sub(compare_delta_value.as_u64().unwrap()),
            (DataValue::I64(a), DataValue::I64(b)) => *a == b.wrapping_sub(compare_delta_value.as_i64().unwrap()),
            _ => false,
        }
    }
}
