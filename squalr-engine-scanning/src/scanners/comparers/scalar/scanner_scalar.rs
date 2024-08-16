use crate::scanners::constraints::scan_constraint::ScanConstraint;
use crate::scanners::constraints::scan_constraint_type::ScanConstraintType;
use squalr_engine_common::dynamic_struct::field_value::FieldValue;

pub struct ScannerScalar {
}

/// Implements a scalar (ie CPU bound, non-SIMD) scanner which contains all boolean comparison operations to be used by more complex scanners,
/// in addition to handling common functionality like reading values and structures from snapshot memory given a pointer.
impl ScannerScalar {
    pub fn new() -> Self {
        Self {}
    }

    /// This is a highly optimized function that performs any given scalar comparison. The Field Values are passed by ref to minimize cloning,
    /// so the caller is expected to allocate and reuse these frequently.
    pub fn do_compare_action(
        &self,
        // Pointer to the values to load from memory to get the current values for the compare.
        current_value_ptr: *const u8,
        // (Optional) Pointer to the values to load from memory to get the current values for the compare. Only necessary for some compares.
        compare_value_ptr: *const u8,
        // A reference to a struct to which the current values are read.
        current_value_ref: &mut FieldValue,
        // A reference to a struct to which the previous values are read (if they are read). Otherwise, this is expected to already have the compare value.
        compare_value_ref: &mut FieldValue,
        // The type of constraint for this comparison.
        scan_constraint: &ScanConstraint,
    ) -> bool {
        unsafe {
            match scan_constraint.get_constraint_type() {
                ScanConstraintType::Unchanged
                    => self.compare_unchanged(current_value_ptr, compare_value_ptr, current_value_ref, compare_value_ref),
                ScanConstraintType::Changed
                    => self.compare_changed(current_value_ptr, compare_value_ptr, current_value_ref, compare_value_ref),
                ScanConstraintType::Increased
                    => self.compare_increased(current_value_ptr, compare_value_ptr, current_value_ref, compare_value_ref),
                ScanConstraintType::Decreased
                    => self.compare_decreased(current_value_ptr, compare_value_ptr, current_value_ref, compare_value_ref),
                ScanConstraintType::IncreasedByX
                    => self.compare_increased_by(current_value_ptr, compare_value_ptr, current_value_ref, compare_value_ref, scan_constraint.get_constraint_value().unwrap_unchecked()),
                ScanConstraintType::DecreasedByX
                    => self.compare_decreased_by(current_value_ptr, compare_value_ptr, current_value_ref, compare_value_ref, scan_constraint.get_constraint_value().unwrap_unchecked()),
                ScanConstraintType::Equal
                    => self.compare_equal(current_value_ptr, current_value_ref, compare_value_ref),
                ScanConstraintType::NotEqual
                    => self.compare_not_equal(current_value_ptr, current_value_ref, compare_value_ref),
                ScanConstraintType::GreaterThan
                    => self.compare_greater_than(current_value_ptr, current_value_ref, compare_value_ref),
                ScanConstraintType::GreaterThanOrEqual
                    => self.compare_greater_than_or_equal(current_value_ptr, current_value_ref, compare_value_ref),
                ScanConstraintType::LessThan
                    => self.compare_less_than(current_value_ptr, current_value_ref, compare_value_ref),
                ScanConstraintType::LessThanOrEqual
                    => self.compare_less_than_or_equal(current_value_ptr, current_value_ref, compare_value_ref),
            }
        }
    }

    fn load_values(&self, value_ptr: *const u8, value_ref: &mut FieldValue) {
        unsafe {
            self.read_value(value_ptr, value_ref);
        }
    }

    unsafe fn read_value(&self, ptr: *const u8, data_type: &mut FieldValue) {
        match data_type {
            FieldValue::U8(ref mut value) => *value = *ptr,
            FieldValue::I8(ref mut value) => *value = *ptr as i8,
            FieldValue::U16(ref mut value, _) => {
                let bytes = [*ptr, *ptr.add(1)];
                *value = u16::from_ne_bytes(bytes);
            }
            FieldValue::I16(ref mut value, _) => {
                let bytes = [*ptr, *ptr.add(1)];
                *value = i16::from_ne_bytes(bytes);
            }
            FieldValue::U32(ref mut value, _) => {
                let bytes = [*ptr, *ptr.add(1), *ptr.add(2), *ptr.add(3)];
                *value = u32::from_ne_bytes(bytes);
            }
            FieldValue::I32(ref mut value, _) => {
                let bytes = [*ptr, *ptr.add(1), *ptr.add(2), *ptr.add(3)];
                *value = i32::from_ne_bytes(bytes);
            }
            FieldValue::U64(ref mut value, _) => {
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
                *value = u64::from_ne_bytes(bytes);
            }
            FieldValue::I64(ref mut value, _) => {
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
                *value = i64::from_ne_bytes(bytes);
            }
            FieldValue::F32(ref mut value, _) => {
                let bytes = [*ptr, *ptr.add(1), *ptr.add(2), *ptr.add(3)];
                let bits = u32::from_ne_bytes(bytes);
                *value = f32::from_bits(bits);
            }
            FieldValue::F64(ref mut value, _) => {
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
                *value = f64::from_bits(bits);
            }
            _ => unimplemented!("Type not supported"),
        }
    }

    fn compare_changed(&self,
        current_value_ptr: *const u8,
        compare_value_ptr: *const u8,
        current_value_ref: &mut FieldValue,
        compare_value_ref: &mut FieldValue
    ) -> bool {
        self.load_values(current_value_ptr, current_value_ref);
        self.load_values(compare_value_ptr, compare_value_ref);
        current_value_ref != compare_value_ref
    }

    fn compare_unchanged(&self,
        current_value_ptr: *const u8,
        compare_value_ptr: *const u8,
        current_value_ref: &mut FieldValue,
        compare_value_ref: &mut FieldValue
    ) -> bool {
        self.load_values(current_value_ptr, current_value_ref);
        self.load_values(compare_value_ptr, compare_value_ref);
        current_value_ref == compare_value_ref
    }

    fn compare_increased(&self,
        current_value_ptr: *const u8,
        compare_value_ptr: *const u8,
        current_value_ref: &mut FieldValue,
        compare_value_ref: &mut FieldValue
    ) -> bool {
        self.load_values(current_value_ptr, current_value_ref);
        self.load_values(compare_value_ptr, compare_value_ref);

        return current_value_ref > compare_value_ref;
    }

    fn compare_decreased(&self,
        current_value_ptr: *const u8,
        compare_value_ptr: *const u8,
        current_value_ref: &mut FieldValue,
        compare_value_ref: &mut FieldValue
    ) -> bool {
        self.load_values(current_value_ptr, current_value_ref);
        self.load_values(compare_value_ptr, compare_value_ref);

        return current_value_ref < compare_value_ref;
    }

    fn compare_equal(&self,
        current_value_ptr: *const u8,
        current_value_ref: &mut FieldValue,
        compare_value: &mut FieldValue
    ) -> bool {
        self.load_values(current_value_ptr, current_value_ref);

        return current_value_ref == compare_value;
    }

    fn compare_not_equal(&self,
        current_value_ptr: *const u8,
        current_value_ref: &mut FieldValue,
        compare_value: &mut FieldValue
    ) -> bool {
        self.load_values(current_value_ptr, current_value_ref);

        return current_value_ref != compare_value;
    }

    fn compare_greater_than(&self,
        current_value_ptr: *const u8,
        current_value_ref: &mut FieldValue,
        compare_value: &mut FieldValue
    ) -> bool {
        self.load_values(current_value_ptr, current_value_ref);

        return current_value_ref > compare_value;
    }

    fn compare_greater_than_or_equal(&self,
        current_value_ptr: *const u8,
        current_value_ref: &mut FieldValue,
        compare_value: &mut FieldValue
    ) -> bool {
        self.load_values(current_value_ptr, current_value_ref);

        return current_value_ref >= compare_value;
    }

    fn compare_less_than(&self,
        current_value_ptr: *const u8,
        current_value_ref: &mut FieldValue,
        compare_value: &mut FieldValue
    ) -> bool {
        self.load_values(current_value_ptr, current_value_ref);

        return current_value_ref < compare_value;
    }

    fn compare_less_than_or_equal(&self,
        current_value_ptr: *const u8,
        current_value_ref: &mut FieldValue,
        compare_value: &mut FieldValue
    ) -> bool {
        self.load_values(current_value_ptr, current_value_ref);

        return current_value_ref <= compare_value;
    }

    fn compare_increased_by(&self,
        current_value_ptr: *const u8,
        compare_value_ptr: *const u8,
        current_value_ref: &mut FieldValue,
        compare_value_ref: &mut FieldValue,
        compare_delta_value: &FieldValue
    )-> bool {
        self.load_values(current_value_ptr, current_value_ref);
        self.load_values(compare_value_ptr, compare_value_ref);

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

    fn compare_decreased_by(&self,
        current_value_ptr: *const u8,
        compare_value_ptr: *const u8,
        current_value_ref: &mut FieldValue,
        compare_value_ref: &mut FieldValue,
        compare_delta_value: &FieldValue
    )-> bool {
        self.load_values(current_value_ptr, current_value_ref);
        self.load_values(compare_value_ptr, compare_value_ref);

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
