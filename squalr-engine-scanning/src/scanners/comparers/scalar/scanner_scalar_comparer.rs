use crate::scanners::parameters::scan_compare_type::ScanCompareType;
use squalr_engine_common::values::data_value::DataValue;
use std::sync::Once;

/// Defines a compare function that operates on an immediate (ie all inequalities)
type ScalarCompareFnImmediate = unsafe fn(
    // Current value struct ref
    &DataValue,
    // Immediate value
    &DataValue,
) -> bool;

/// Defines a compare function that operates on current and previous values (ie changed, unchanged, increased, decreased)
type ScalarCompareFnRelative = unsafe fn(
    // Current value struct ref
    &DataValue,
    // Previous value struct ref
    &DataValue,
) -> bool;

/// Defines a compare function that operates on current and previous values, with a delta arg (ie +x, -x)
type ScalarCompareFnDelta = unsafe fn(
    // Current value struct ref
    &DataValue,
    // Previous value struct ref
    &DataValue,
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

    pub fn get_immediate_compare_func(&self, scan_compare_type: ScanCompareType) -> ScalarCompareFnImmediate {
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

    pub fn get_relative_compare_func(&self, scan_compare_type: ScanCompareType) -> ScalarCompareFnRelative {
        match scan_compare_type {
            ScanCompareType::Changed => Self::compare_changed,
            ScanCompareType::Unchanged => Self::compare_unchanged,
            ScanCompareType::Increased => Self::compare_increased,
            ScanCompareType::Decreased => Self::compare_decreased,
            _ => panic!("Unsupported type passed to get_relative_compare_func"),
        }
    }

    pub fn get_relative_delta_compare_func(&self, scan_compare_type: ScanCompareType) -> ScalarCompareFnDelta {
        match scan_compare_type {
            ScanCompareType::IncreasedByX => Self::compare_increased_by,
            ScanCompareType::DecreasedByX => Self::compare_decreased_by,
            _ => panic!("Unsupported type passed to get_relative_delta_compare_func"),
        }
    }

    unsafe fn compare_equal(
        current_value_ref: &DataValue,
        compare_value: &DataValue
    ) -> bool {
        return current_value_ref == compare_value;
    }

    unsafe fn compare_not_equal(
        current_value_ref: &DataValue,
        compare_value: &DataValue
    ) -> bool {
        return current_value_ref != compare_value;
    }
    
    unsafe fn compare_changed(
        current_value_ref: &DataValue,
        compare_value_ref: &DataValue
    ) -> bool {
        current_value_ref != compare_value_ref
    }

    unsafe fn compare_unchanged(
        current_value_ref: &DataValue,
        compare_value_ref: &DataValue
    ) -> bool {
        current_value_ref == compare_value_ref
    }

    unsafe fn compare_increased(
        current_value_ref: &DataValue,
        compare_value_ref: &DataValue
    ) -> bool {
        return current_value_ref > compare_value_ref;
    }

    unsafe fn compare_decreased(
        current_value_ref: &DataValue,
        compare_value_ref: &DataValue
    ) -> bool {
        return current_value_ref < compare_value_ref;
    }

    unsafe fn compare_greater_than(
        current_value_ref: &DataValue,
        compare_value: &DataValue
    ) -> bool {
        return *current_value_ref > *compare_value;
    }

    unsafe fn compare_greater_than_or_equal(
        current_value_ref: &DataValue,
        compare_value: &DataValue
    ) -> bool {
        return *current_value_ref >= *compare_value;
    }

    unsafe fn compare_less_than(
        current_value_ref: &DataValue,
        compare_value: &DataValue
    ) -> bool {
        return *current_value_ref < *compare_value;
    }

    unsafe fn compare_less_than_or_equal(
        current_value_ref: &DataValue,
        compare_value: &DataValue
    ) -> bool {
        return *current_value_ref <= *compare_value;
    }

    unsafe fn compare_increased_by(
        current_value_ref: &DataValue,
        compare_value_ref: &DataValue,
        compare_delta_value: &DataValue
    )-> bool {
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
        current_value_ref: &DataValue,
        compare_value_ref: &DataValue,
        compare_delta_value: &DataValue
    )-> bool {
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
