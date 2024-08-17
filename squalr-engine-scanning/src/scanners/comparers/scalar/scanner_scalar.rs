use core::panic;

use crate::scanners::constraints::scan_constraint_type::ScanConstraintType;
use squalr_engine_common::dynamic_struct::field_value::FieldMemoryLoadFunc;
use squalr_engine_common::dynamic_struct::field_value::FieldValue;

/// Defines a compare function that operates on an immediate (ie all inequalities)
type ScalarCompareFnImmediate = unsafe fn(
    // The function used to load new values from memory
    &FieldMemoryLoadFunc,
    // Current values pointer
    *const u8,
    // Current value struct ref
    &mut FieldValue,
    // Immediate value
    &FieldValue,
) -> bool;

/// Defines a compare function that operates on current and previous values (ie changed, unchanged, increased, decreased)
type ScalarCompareFnRelative = unsafe fn(
    // The function used to load new values from memory
    &FieldMemoryLoadFunc,
    // Current values pointer
    *const u8,
    // Previous values pointer
    *const u8,
    // Current value struct ref
    &mut FieldValue,
    // Previous value struct ref
    &mut FieldValue,
) -> bool;

/// Defines a compare function that operates on current and previous values, with a delta arg (ie +x, -x)
type ScalarCompareFnDelta = unsafe fn(
    // The function used to load new values from memory
    &FieldMemoryLoadFunc,
    // Current values pointer
    *const u8,
    // Previous values pointer
    *const u8,
    // Current value struct ref
    &mut FieldValue,
    // Previous value struct ref
    &mut FieldValue,
    // Delta value
    &FieldValue,
) -> bool;

pub struct ScannerScalar {
}

/// Implements a scalar (ie CPU bound, non-SIMD) scanner which contains all boolean comparison operations to be used by more complex scanners,
/// in addition to handling common functionality like reading values and structures from snapshot memory given a pointer.
impl ScannerScalar {
    // Intentionally stateless
    pub fn new() -> Self { Self {} }

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
        memory_load_func: &FieldMemoryLoadFunc,
        current_value_ptr: *const u8,
        current_value_ref: &mut FieldValue,
        compare_value: &FieldValue
    ) -> bool {
        memory_load_func(current_value_ref, current_value_ptr);

        return current_value_ref == compare_value;
    }

    unsafe fn compare_not_equal(
        memory_load_func: &FieldMemoryLoadFunc,
        current_value_ptr: *const u8,
        current_value_ref: &mut FieldValue,
        compare_value: &FieldValue
    ) -> bool {
        memory_load_func(current_value_ref, current_value_ptr);

        return current_value_ref != compare_value;
    }
    
    unsafe fn compare_changed(
        memory_load_func: &FieldMemoryLoadFunc,
        current_value_ptr: *const u8,
        compare_value_ptr: *const u8,
        current_value_ref: &mut FieldValue,
        compare_value_ref: &mut FieldValue
    ) -> bool {
        memory_load_func(current_value_ref, current_value_ptr);
        memory_load_func(compare_value_ref, compare_value_ptr);
        current_value_ref != compare_value_ref
    }

    unsafe fn compare_unchanged(
        memory_load_func: &FieldMemoryLoadFunc,
        current_value_ptr: *const u8,
        compare_value_ptr: *const u8,
        current_value_ref: &mut FieldValue,
        compare_value_ref: &mut FieldValue
    ) -> bool {
        memory_load_func(current_value_ref, current_value_ptr);
        memory_load_func(compare_value_ref, compare_value_ptr);
        current_value_ref == compare_value_ref
    }

    unsafe fn compare_increased(
        memory_load_func: &FieldMemoryLoadFunc,
        current_value_ptr: *const u8,
        compare_value_ptr: *const u8,
        current_value_ref: &mut FieldValue,
        compare_value_ref: &mut FieldValue
    ) -> bool {
        memory_load_func(current_value_ref, current_value_ptr);
        memory_load_func(compare_value_ref, compare_value_ptr);

        return current_value_ref > compare_value_ref;
    }

    unsafe fn compare_decreased(
        memory_load_func: &FieldMemoryLoadFunc,
        current_value_ptr: *const u8,
        compare_value_ptr: *const u8,
        current_value_ref: &mut FieldValue,
        compare_value_ref: &mut FieldValue
    ) -> bool {
        memory_load_func(current_value_ref, current_value_ptr);
        memory_load_func(compare_value_ref, compare_value_ptr);

        return current_value_ref < compare_value_ref;
    }

    unsafe fn compare_greater_than(
        memory_load_func: &FieldMemoryLoadFunc,
        current_value_ptr: *const u8,
        current_value_ref: &mut FieldValue,
        compare_value: &FieldValue
    ) -> bool {
        memory_load_func(current_value_ref, current_value_ptr);

        return *current_value_ref > *compare_value;
    }

    unsafe fn compare_greater_than_or_equal(
        memory_load_func: &FieldMemoryLoadFunc,
        current_value_ptr: *const u8,
        current_value_ref: &mut FieldValue,
        compare_value: &FieldValue
    ) -> bool {
        memory_load_func(current_value_ref, current_value_ptr);

        return *current_value_ref >= *compare_value;
    }

    unsafe fn compare_less_than(
        memory_load_func: &FieldMemoryLoadFunc,
        current_value_ptr: *const u8,
        current_value_ref: &mut FieldValue,
        compare_value: &FieldValue
    ) -> bool {
        memory_load_func(current_value_ref, current_value_ptr);

        return *current_value_ref < *compare_value;
    }

    unsafe fn compare_less_than_or_equal(
        memory_load_func: &FieldMemoryLoadFunc,
        current_value_ptr: *const u8,
        current_value_ref: &mut FieldValue,
        compare_value: &FieldValue
    ) -> bool {
        memory_load_func(current_value_ref, current_value_ptr);

        return *current_value_ref <= *compare_value;
    }

    unsafe fn compare_increased_by(
        memory_load_func: &FieldMemoryLoadFunc,
        current_value_ptr: *const u8,
        compare_value_ptr: *const u8,
        current_value_ref: &mut FieldValue,
        compare_value_ref: &mut FieldValue,
        compare_delta_value: &FieldValue
    )-> bool {
        memory_load_func(current_value_ref, current_value_ptr);
        memory_load_func(compare_value_ref, compare_value_ptr);

        match (current_value_ref, compare_value_ref) {
            (FieldValue::U8(a), FieldValue::U8(b)) => *a == b.wrapping_add(compare_delta_value.as_u8().unwrap()),
            (FieldValue::I8(a), FieldValue::I8(b)) => *a == b.wrapping_add(compare_delta_value.as_i8().unwrap()),
            (FieldValue::U16(a, _), FieldValue::U16(b, _)) => *a == b.wrapping_add(compare_delta_value.as_u16().unwrap()),
            (FieldValue::I16(a, _), FieldValue::I16(b, _)) => *a == b.wrapping_add(compare_delta_value.as_i16().unwrap()),
            (FieldValue::U32(a, _), FieldValue::U32(b, _)) => *a == b.wrapping_add(compare_delta_value.as_u32().unwrap()),
            (FieldValue::I32(a, _), FieldValue::I32(b, _)) => *a == b.wrapping_add(compare_delta_value.as_i32().unwrap()),
            (FieldValue::U64(a, _), FieldValue::U64(b, _)) => *a == b.wrapping_add(compare_delta_value.as_u64().unwrap()),
            (FieldValue::I64(a, _), FieldValue::I64(b, _)) => *a == b.wrapping_add(compare_delta_value.as_i64().unwrap()),
            _ => false,
        }
    }

    unsafe fn compare_decreased_by(
        memory_load_func: &FieldMemoryLoadFunc,
        current_value_ptr: *const u8,
        compare_value_ptr: *const u8,
        current_value_ref: &mut FieldValue,
        compare_value_ref: &mut FieldValue,
        compare_delta_value: &FieldValue
    )-> bool {
        memory_load_func(current_value_ref, current_value_ptr);
        memory_load_func(compare_value_ref, compare_value_ptr);

        match (current_value_ref, compare_value_ref) {
            (FieldValue::U8(a), FieldValue::U8(b)) => *a == b.wrapping_sub(compare_delta_value.as_u8().unwrap()),
            (FieldValue::I8(a), FieldValue::I8(b)) => *a == b.wrapping_sub(compare_delta_value.as_i8().unwrap()),
            (FieldValue::U16(a, _), FieldValue::U16(b, _)) => *a == b.wrapping_sub(compare_delta_value.as_u16().unwrap()),
            (FieldValue::I16(a, _), FieldValue::I16(b, _)) => *a == b.wrapping_sub(compare_delta_value.as_i16().unwrap()),
            (FieldValue::U32(a, _), FieldValue::U32(b, _)) => *a == b.wrapping_sub(compare_delta_value.as_u32().unwrap()),
            (FieldValue::I32(a, _), FieldValue::I32(b, _)) => *a == b.wrapping_sub(compare_delta_value.as_i32().unwrap()),
            (FieldValue::U64(a, _), FieldValue::U64(b, _)) => *a == b.wrapping_sub(compare_delta_value.as_u64().unwrap()),
            (FieldValue::I64(a, _), FieldValue::I64(b, _)) => *a == b.wrapping_sub(compare_delta_value.as_i64().unwrap()),
            _ => false,
        }
    }
}
