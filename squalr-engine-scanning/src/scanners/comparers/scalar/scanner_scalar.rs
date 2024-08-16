use crate::scanners::constraints::scan_constraint::ScanConstraint;
use crate::scanners::constraints::scan_constraint_type::ScanConstraintType;
use squalr_engine_common::dynamic_struct::field_value::FieldValue;

pub struct ScannerScalar {
}

/// Implements a scalar (ie CPU bound, non-SIMD) scanner which contains all boolean comparison operations to be used by more complex scanners,
/// in addition to handling common functionality like reading values and structures from snapshot memory given a pointer.
impl ScannerScalar {
    pub fn new() -> Self {
        return Self { };
    }

    pub fn do_compare_action(&self, current_value_ptr: *const u8, previous_value_ptr: *const u8, constraint: &ScanConstraint, data_type: &FieldValue) -> bool {
        match constraint.get_constraint_type() {
            ScanConstraintType::Unchanged => self.compare_unchanged(current_value_ptr, previous_value_ptr, data_type),
            ScanConstraintType::Changed => self.compare_changed(current_value_ptr, previous_value_ptr, data_type),
            ScanConstraintType::Increased => self.compare_increased(current_value_ptr, previous_value_ptr, data_type),
            ScanConstraintType::Decreased => self.compare_decreased(current_value_ptr, previous_value_ptr, data_type),
            ScanConstraintType::IncreasedByX => self.compare_increased_by(current_value_ptr, previous_value_ptr, constraint.get_constraint_value().cloned().unwrap_or_default(), data_type),
            ScanConstraintType::DecreasedByX => self.compare_decreased_by(current_value_ptr, previous_value_ptr, constraint.get_constraint_value().cloned().unwrap_or_default(), data_type),
            ScanConstraintType::Equal => self.compare_equal(current_value_ptr, constraint.get_constraint_value().cloned().unwrap_or_default(), data_type),
            ScanConstraintType::NotEqual => self.compare_not_equal(current_value_ptr, constraint.get_constraint_value().cloned().unwrap_or_default(), data_type),
            ScanConstraintType::GreaterThan => self.compare_greater_than(current_value_ptr, constraint.get_constraint_value().cloned().unwrap_or_default(), data_type),
            ScanConstraintType::GreaterThanOrEqual => self.compare_greater_than_or_equal(current_value_ptr, constraint.get_constraint_value().cloned().unwrap_or_default(), data_type),
            ScanConstraintType::LessThan => self.compare_less_than(current_value_ptr, constraint.get_constraint_value().cloned().unwrap_or_default(), data_type),
            ScanConstraintType::LessThanOrEqual => self.compare_less_than_or_equal(current_value_ptr, constraint.get_constraint_value().cloned().unwrap_or_default(), data_type),
        }
    }

    fn get_current_values(&self, current_value_ptr: *const u8, data_type: &FieldValue) -> FieldValue {
        let current_value = unsafe { self.read_value(current_value_ptr, data_type) };

        return current_value;
    }

    fn get_current_previous_values(&self, current_value_ptr: *const u8, previous_value_ptr: *const u8, data_type: &FieldValue) -> (FieldValue, FieldValue) {
        let current_value = unsafe { self.read_value(current_value_ptr, data_type) };
        let previous_value = unsafe { self.read_value(previous_value_ptr, data_type) };

        return (current_value, previous_value);
    }

    unsafe fn read_value(&self, ptr: *const u8, data_type: &FieldValue) -> FieldValue {
        match data_type {
            FieldValue::U8(_) => FieldValue::U8(*ptr),
            FieldValue::I8(_) => FieldValue::I8(*ptr as i8),
            FieldValue::U16(_, endian) => {
                let bytes = [*ptr, *ptr.add(1)];
                FieldValue::U16(u16::from_ne_bytes(bytes), endian.clone())
            }
            FieldValue::I16(_, endian) => {
                let bytes = [*ptr, *ptr.add(1)];
                FieldValue::I16(i16::from_ne_bytes(bytes), endian.clone())
            }
            FieldValue::U32(_, endian) => {
                let bytes = [*ptr, *ptr.add(1), *ptr.add(2), *ptr.add(3)];
                FieldValue::U32(u32::from_ne_bytes(bytes), endian.clone())
            }
            FieldValue::I32(_, endian) => {
                let bytes = [*ptr, *ptr.add(1), *ptr.add(2), *ptr.add(3)];
                FieldValue::I32(i32::from_ne_bytes(bytes), endian.clone())
            }
            FieldValue::U64(_, endian) => {
                let bytes = [
                    *ptr,
                    *ptr.add(1),
                    *ptr.add(2),
                    *ptr.add(3),
                    *ptr.add(4),
                    *ptr.add(5),
                    *ptr.add(6),
                    *ptr.add(7),
                ];
                FieldValue::U64(u64::from_ne_bytes(bytes), endian.clone())
            }
            FieldValue::I64(_, endian) => {
                let bytes = [
                    *ptr,
                    *ptr.add(1),
                    *ptr.add(2),
                    *ptr.add(3),
                    *ptr.add(4),
                    *ptr.add(5),
                    *ptr.add(6),
                    *ptr.add(7),
                ];
                FieldValue::I64(i64::from_ne_bytes(bytes), endian.clone())
            }
            FieldValue::F32(_, endian) => {
                let bytes = [*ptr, *ptr.add(1), *ptr.add(2), *ptr.add(3)];
                let bits = u32::from_ne_bytes(bytes);
                FieldValue::F32(f32::from_bits(bits), endian.clone())
            }
            FieldValue::F64(_, endian) => {
                let bytes = [
                    *ptr,
                    *ptr.add(1),
                    *ptr.add(2),
                    *ptr.add(3),
                    *ptr.add(4),
                    *ptr.add(5),
                    *ptr.add(6),
                    *ptr.add(7),
                ];
                let bits = u64::from_ne_bytes(bytes);
                FieldValue::F64(f64::from_bits(bits), endian.clone())
            }
            _ => unimplemented!("Type not supported"),
        }
    }

    fn compare_changed(&self, current_value_ptr: *const u8, previous_value_ptr: *const u8, data_type: &FieldValue) -> bool {
        let (current_value, previous_value) = self.get_current_previous_values(current_value_ptr, previous_value_ptr, data_type);
        return current_value != previous_value;
    }

    fn compare_unchanged(&self, current_value_ptr: *const u8, previous_value_ptr: *const u8, data_type: &FieldValue) -> bool {
        let (current_value, previous_value) = self.get_current_previous_values(current_value_ptr, previous_value_ptr, data_type);
        return current_value == previous_value;
    }

    fn compare_increased(&self, current_value_ptr: *const u8, previous_value_ptr: *const u8, data_type: &FieldValue) -> bool {
        let (current_value, previous_value) = self.get_current_previous_values(current_value_ptr, previous_value_ptr, data_type);
        return current_value > previous_value;
    }

    fn compare_decreased(&self, current_value_ptr: *const u8, previous_value_ptr: *const u8, data_type: &FieldValue) -> bool {
        let (current_value, previous_value) = self.get_current_previous_values(current_value_ptr, previous_value_ptr, data_type);
        return current_value < previous_value;
    }

    fn compare_equal(&self, current_value_ptr: *const u8, value: FieldValue, data_type: &FieldValue) -> bool {
        let current_value = self.get_current_values(current_value_ptr, data_type);
        return current_value == value;
    }

    fn compare_not_equal(&self, current_value_ptr: *const u8, value: FieldValue, data_type: &FieldValue) -> bool {
        let current_value = self.get_current_values(current_value_ptr, data_type);
        return current_value != value;
    }

    fn compare_greater_than(&self, current_value_ptr: *const u8, value: FieldValue, data_type: &FieldValue) -> bool {
        let current_value = self.get_current_values(current_value_ptr, data_type);
        return current_value > value;
    }

    fn compare_greater_than_or_equal(&self, current_value_ptr: *const u8, value: FieldValue, data_type: &FieldValue) -> bool {
        let current_value = self.get_current_values(current_value_ptr, data_type);
        return current_value >= value;
    }

    fn compare_less_than(&self, current_value_ptr: *const u8, value: FieldValue, data_type: &FieldValue) -> bool {
        let current_value = self.get_current_values(current_value_ptr, data_type);
        return current_value < value;
    }

    fn compare_less_than_or_equal(&self, current_value_ptr: *const u8, value: FieldValue, data_type: &FieldValue) -> bool {
        let current_value = self.get_current_values(current_value_ptr, data_type);
        return current_value <= value;
    }

    fn compare_increased_by(&self, current_value_ptr: *const u8, previous_value_ptr: *const u8, value: FieldValue, data_type: &FieldValue) -> bool {
        let (current_value, previous_value) = self.get_current_previous_values(current_value_ptr, previous_value_ptr, data_type);
        return match (current_value, previous_value) {
            (FieldValue::U8(a), FieldValue::U8(b)) => a == b.wrapping_add(value.as_u8().unwrap()),
            (FieldValue::I8(a), FieldValue::I8(b)) => a == b.wrapping_add(value.as_i8().unwrap()),
            (FieldValue::U16(a, _), FieldValue::U16(b, _)) => a == b.wrapping_add(value.as_u16().unwrap()),
            (FieldValue::I16(a, _), FieldValue::I16(b, _)) => a == b.wrapping_add(value.as_i16().unwrap()),
            (FieldValue::U32(a, _), FieldValue::U32(b, _)) => a == b.wrapping_add(value.as_u32().unwrap()),
            (FieldValue::I32(a, _), FieldValue::I32(b, _)) => a == b.wrapping_add(value.as_i32().unwrap()),
            (FieldValue::U64(a, _), FieldValue::U64(b, _)) => a == b.wrapping_add(value.as_u64().unwrap()),
            (FieldValue::I64(a, _), FieldValue::I64(b, _)) => a == b.wrapping_add(value.as_i64().unwrap()),
            _ => false,
        };
    }

    fn compare_decreased_by(&self, current_value_ptr: *const u8, previous_value_ptr: *const u8, value: FieldValue, data_type: &FieldValue) -> bool {
        let (current_value, previous_value) = self.get_current_previous_values(current_value_ptr, previous_value_ptr, data_type);
        return match (current_value, previous_value) {
            (FieldValue::U8(a), FieldValue::U8(b)) => a == b.wrapping_sub(value.as_u8().unwrap()),
            (FieldValue::I8(a), FieldValue::I8(b)) => a == b.wrapping_sub(value.as_i8().unwrap()),
            (FieldValue::U16(a, _), FieldValue::U16(b, _)) => a == b.wrapping_sub(value.as_u16().unwrap()),
            (FieldValue::I16(a, _), FieldValue::I16(b, _)) => a == b.wrapping_sub(value.as_i16().unwrap()),
            (FieldValue::U32(a, _), FieldValue::U32(b, _)) => a == b.wrapping_sub(value.as_u32().unwrap()),
            (FieldValue::I32(a, _), FieldValue::I32(b, _)) => a == b.wrapping_sub(value.as_i32().unwrap()),
            (FieldValue::U64(a, _), FieldValue::U64(b, _)) => a == b.wrapping_sub(value.as_u64().unwrap()),
            (FieldValue::I64(a, _), FieldValue::I64(b, _)) => a == b.wrapping_sub(value.as_i64().unwrap()),
            _ => false,
        };
    }
}
