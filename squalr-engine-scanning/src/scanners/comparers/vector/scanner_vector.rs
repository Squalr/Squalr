use crate::scanners::comparers::snapshot_sub_region_scanner::Scanner;
use crate::scanners::constraints::operation_constraint::{OperationConstraint, OperationType};
use crate::scanners::constraints::scan_constraint::{ScanConstraint,ConstraintType};
use crate::scanners::constraints::scan_constraints::ScanConstraints;
use crate::snapshots::snapshot_sub_region::SnapshotSubRegion;
use squalr_engine_common::dynamic_struct::field_value::FieldValue;
use std::ops::{BitAnd, BitOr, BitXor};
use std::simd::cmp::{SimdOrd, SimdPartialEq, SimdPartialOrd};
use std::simd::{u8x16, u16x8, u32x4, u64x2, i8x16, i16x8, i32x4, i64x2, f32x4, f64x2};

pub struct SnapshotElementScannerVector<'a> {
    pub base_scanner: Scanner<'a>,
    pub vector_read_offset: usize,
    pub vector_read_base: usize,
    pub first_scan_vector_misalignment: usize,
    pub last_scan_vector_overread: usize,
    pub vector_compare_func: Option<Box<dyn Fn() -> u8x16 + 'a>>,
    pub custom_vector_compare: Option<Box<dyn Fn() -> u8x16 + 'a>>,
}

impl<'a> SnapshotElementScannerVector<'a> {
    pub fn new() -> Self {
        Self {
            base_scanner: Scanner::new(),
            vector_read_offset: 0,
            vector_read_base: 0,
            first_scan_vector_misalignment: 0,
            last_scan_vector_overread: 0,
            vector_compare_func: None,
            custom_vector_compare: None,
        }
    }

    pub fn initialize(&mut self, snapshot_sub_region: &'a SnapshotSubRegion, constraints: &'a ScanConstraints) {
        self.base_scanner.initialize(snapshot_sub_region, constraints);
        self.vector_read_offset = 0;
        self.vector_compare_func = Some(self.build_compare_actions(constraints));
        self.first_scan_vector_misalignment = self.calculate_first_scan_vector_misalignment();
        self.vector_read_base = snapshot_sub_region.region_offset - self.first_scan_vector_misalignment;
        self.last_scan_vector_overread = self.calculate_last_scan_vector_overread();
    }

    pub fn set_custom_compare_action(&mut self, custom_compare: Box<dyn Fn() -> u8x16 + 'a>) {
        self.custom_vector_compare = Some(custom_compare);
    }

    pub fn perform_vector_scan(
        &mut self,
        false_mask: u8x16,
        vector_increment_size: usize,
        vector_comparer: Box<dyn Fn() -> u8x16 + 'a>,
    ) -> Vec<SnapshotSubRegion<'a>> {
        // This algorithm has three stages:
        // 1) Scan the first vector of memory, which may contain elements outside of the intended range. For example, in <x, x, x, x ... y, y, y, y>,
        //      where x is data outside the element range (but within the snapshot region), and y is within the region we are scanning.
        //      to solve this, we mask out the x values such that these will always be considered false by our scan.
        // 2) Scan the middle parts of. These will all fit perfectly into vectors.
        // 3) Scan the final vector, if it exists. This may spill outside of the element range (but within the snapshot region).
        //      This works exactly like the first scan, but reversed. ie <y, y, y, y, ... x, x, x, x>, where x values are masked to be false.
        //      Note: This mask may also be applied to the first scan, if it is also the last scan (ie only 1 scan total for this region).
        
        let mut scan_count = (self.base_scanner.get_snapshot_sub_region().as_ref().unwrap().range / 16)
            + if self.last_scan_vector_overread > 0 { 1 } else { 0 };
        let misalignment_mask = self.build_vector_misalignment_mask();
        let overread_mask = self.build_vector_overread_mask();
        let mut run_length_encoded_scan_result;

        // Perform the first scan
        {
            run_length_encoded_scan_result = misalignment_mask.bitand(vector_comparer());
            run_length_encoded_scan_result =
                run_length_encoded_scan_result.bitand(if scan_count == 1 {
                    overread_mask
                } else {
                    u8x16::splat(0xFF)
                });
            self.base_scanner
                .get_run_length_encoder()
                .adjust_for_misalignment(self.first_scan_vector_misalignment);
            self.encode_scan_results(run_length_encoded_scan_result);
            self.vector_read_offset += vector_increment_size;
        }

        // Perform middle scans
        while self.vector_read_offset < self.base_scanner.get_snapshot_sub_region().as_ref().unwrap().range - 16 {
            run_length_encoded_scan_result = vector_comparer();
            self.encode_scan_results(run_length_encoded_scan_result);
            self.vector_read_offset += vector_increment_size;
        }

        // Perform final scan
        // TODO: Didn't the algorithm comments say that this could still be applied to a single scan? Revisit this.
        if scan_count > 1 {
            run_length_encoded_scan_result = overread_mask.bitand(vector_comparer());
            self.encode_scan_results_with_mask(run_length_encoded_scan_result, false_mask);
            self.vector_read_offset += vector_increment_size;
        }

        self.base_scanner.get_run_length_encoder().finalize_current_encode_unchecked(0);
        
        return self.base_scanner.get_run_length_encoder().get_collected_regions().clone();
    }

    fn build_vector_misalignment_mask(&self) -> u8x16 {
        let mut misalignment_mask = [0u8; 16];
        for i in self.first_scan_vector_misalignment..16 {
            misalignment_mask[i] = 0xFF;
        }
        u8x16::from_array(misalignment_mask)
    }
    
    fn build_vector_overread_mask(&self) -> u8x16 {
        let mut overread_mask = [0u8; 16];
        let fill_up_to = 16 - self.last_scan_vector_overread;
        for i in 0..fill_up_to {
            overread_mask[i] = 0xFF;
        }
        u8x16::from_array(overread_mask)
    }

    fn encode_scan_results(&mut self, scan_results: u8x16) {
        self.encode_scan_results_with_mask(scan_results, u8x16::splat(0));
    }

    fn encode_scan_results_with_mask(
        &mut self,
        scan_results: u8x16,
        false_mask: u8x16,
    ) {
        // Collect necessary information with immutable borrows
        let all_true = scan_results.simd_gt(false_mask).all(); //.to_array().iter().all(|&x| x);
        let all_false = scan_results.simd_eq(false_mask).all(); //.to_array().iter().all(|&x| x);
        let alignment = self.base_scanner.get_byte_alignment() as usize;
    
        // Perform encoding with mutable borrows
        let encoder = self.base_scanner.get_run_length_encoder();
    
        if all_true {
            encoder.encode_range(16);
        } else if all_false {
            encoder.finalize_current_encode_unchecked(16);
        } else {
            for i in (0..16).step_by(alignment) {
                if scan_results[i] != false_mask[i] {
                    encoder.encode_range(alignment);
                } else {
                    encoder.finalize_current_encode_unchecked(alignment);
                }
            }
        }
    }
    

    fn calculate_first_scan_vector_misalignment(&self) -> usize {
        let parent_region_size = self.base_scanner.get_snapshot_sub_region().as_ref().unwrap().parent_region.borrow().get_region_size();
        let range_region_offset = self.base_scanner.get_snapshot_sub_region().as_ref().unwrap().region_offset;
        let available_byte_count = parent_region_size - range_region_offset;
        let vector_spill_over = available_byte_count % 16;
        if vector_spill_over == 0 || range_region_offset + self.base_scanner.get_snapshot_sub_region().as_ref().unwrap().range + vector_spill_over < parent_region_size {
            0
        } else {
            16 - vector_spill_over
        }
    }

    fn calculate_last_scan_vector_overread(&self) -> usize {
        let remaining_bytes = self.base_scanner.get_snapshot_sub_region().as_ref().unwrap().range % 16;
        if remaining_bytes == 0 {
            0
        } else {
            16 - remaining_bytes
        }
    }

    fn build_compare_actions(&self, constraint: &ScanConstraints) -> Box<dyn Fn() -> u8x16> {
        if let Some(custom_vector_compare) = &self.custom_vector_compare {
            return custom_vector_compare.clone();
        }

        match constraint {
            ScanConstraints::Operation(operation_constraint) => self.build_operation_compare_actions(operation_constraint),
            ScanConstraints::Value(scan_constraint) => self.build_value_compare_actions(scan_constraint),
            _ => panic!("Invalid constraint type"),
        }
    }

    fn build_operation_compare_actions(&self, operation_constraint: &OperationConstraint) -> Box<dyn Fn() -> u8x16> {
        let left = self.build_compare_actions(&operation_constraint.get_left());
        let right = self.build_compare_actions(&operation_constraint.get_right());

        match operation_constraint.binary_operation {
            OperationType::And => Box::new(move || left().bitand(right())),
            OperationType::Or => Box::new(move || left().bitor(right())),
            OperationType::Xor => Box::new(move || left().bitxor(right())),
        }
    }

    fn build_value_compare_actions(&self, scan_constraint: &ScanConstraint) -> Box<dyn Fn() -> u8x16> {
        match scan_constraint.get_constraint_type() {
            ConstraintType::Unchanged => self.get_comparison_unchanged(scan_constraint.get_constraint_args()),
            ConstraintType::Changed => self.get_comparison_changed(scan_constraint.get_constraint_args()),
            ConstraintType::Increased => self.get_comparison_increased(),
            ConstraintType::Decreased => self.get_comparison_decreased(),
            ConstraintType::IncreasedByX => self.get_comparison_increased_by(scan_constraint.constraint_value, scan_constraint.get_constraint_args()),
            ConstraintType::DecreasedByX => self.get_comparison_decreased_by(scan_constraint.constraint_value, scan_constraint.get_constraint_args()),
            ConstraintType::Equal => self.get_comparison_equal(scan_constraint.constraint_value, scan_constraint.get_constraint_args()),
            ConstraintType::NotEqual => self.get_comparison_not_equal(scan_constraint.constraint_value, scan_constraint.get_constraint_args()),
            ConstraintType::GreaterThan => self.get_comparison_greater_than(scan_constraint.get_constraint_value()),
            ConstraintType::GreaterThanOrEqual => self.get_comparison_greater_than_or_equal(scan_constraint.get_constraint_value()),
            ConstraintType::LessThan => self.get_comparison_less_than(scan_constraint.get_constraint_value()),
            ConstraintType::LessThanOrEqual => self.get_comparison_less_than_or_equal(scan_constraint.get_constraint_value()),
        }
    }

    // Unchanged comparison
    fn get_comparison_unchanged(&self, _args: Option<FieldValue>) -> Box<dyn Fn() -> u8x16> {
        match self.data_type {
            ScannableType::U8 => {
                Box::new(|| self.current_values.eq(self.previous_values))
            },
            ScannableType::U16 => {
                Box::new(|| self.current_values_as_u16().eq(self.previous_values_as_u16()))
            },
            ScannableType::U32 => {
                Box::new(|| self.current_values_as_u32().eq(self.previous_values_as_u32()))
            },
            ScannableType::U64 => {
                Box::new(|| self.current_values_as_u64().eq(self.previous_values_as_u64()))
            },
            ScannableType::I8 => {
                Box::new(|| self.current_values_as_i8().eq(self.previous_values_as_i8()))
            },
            ScannableType::I16 => {
                Box::new(|| self.current_values_as_i16().eq(self.previous_values_as_i16()))
            },
            ScannableType::I32 => {
                Box::new(|| self.current_values_as_i32().eq(self.previous_values_as_i32()))
            },
            ScannableType::I64 => {
                Box::new(|| self.current_values_as_i64().eq(self.previous_values_as_i64()))
            },
            ScannableType::F32 => {
                Box::new(|| self.current_values_as_f32().eq(self.previous_values_as_f32()))
            },
            ScannableType::F64 => {
                Box::new(|| self.current_values_as_f64().eq(self.previous_values_as_f64()))
            },
            _ => unimplemented!("Unchanged comparison is not supported for the provided type."),
        }
    }

    // Changed comparison
    fn get_comparison_changed(&self, _args: Option<FieldValue>) -> Box<dyn Fn() -> u8x16> {
        match self.data_type {
            ScannableType::U8 => {
                Box::new(|| !self.current_values.eq(self.previous_values))
            },
            ScannableType::U16 => {
                Box::new(|| !self.current_values_as_u16().eq(self.previous_values_as_u16()))
            },
            ScannableType::U32 => {
                Box::new(|| !self.current_values_as_u32().eq(self.previous_values_as_u32()))
            },
            ScannableType::U64 => {
                Box::new(|| !self.current_values_as_u64().eq(self.previous_values_as_u64()))
            },
            ScannableType::I8 => {
                Box::new(|| !self.current_values_as_i8().eq(self.previous_values_as_i8()))
            },
            ScannableType::I16 => {
                Box::new(|| !self.current_values_as_i16().eq(self.previous_values_as_i16()))
            },
            ScannableType::I32 => {
                Box::new(|| !self.current_values_as_i32().eq(self.previous_values_as_i32()))
            },
            ScannableType::I64 => {
                Box::new(|| !self.current_values_as_i64().eq(self.previous_values_as_i64()))
            },
            ScannableType::F32 => {
                Box::new(|| !self.current_values_as_f32().eq(self.previous_values_as_f32()))
            },
            ScannableType::F64 => {
                Box::new(|| !self.current_values_as_f64().eq(self.previous_values_as_f64()))
            },
            _ => unimplemented!("Changed comparison is not supported for the provided type."),
        }
    }

    // Increased comparison
    fn get_comparison_increased(&self) -> Box<dyn Fn() -> u8x16> {
        match self.data_type {
            ScannableType::U8 => {
                Box::new(|| self.current_values.gt(self.previous_values))
            },
            ScannableType::U16 => {
                Box::new(|| self.current_values_as_u16().gt(self.previous_values_as_u16()))
            },
            ScannableType::U32 => {
                Box::new(|| self.current_values_as_u32().gt(self.previous_values_as_u32()))
            },
            ScannableType::U64 => {
                Box::new(|| self.current_values_as_u64().gt(self.previous_values_as_u64()))
            },
            ScannableType::I8 => {
                Box::new(|| self.current_values_as_i8().gt(self.previous_values_as_i8()))
            },
            ScannableType::I16 => {
                Box::new(|| self.current_values_as_i16().gt(self.previous_values_as_i16()))
            },
            ScannableType::I32 => {
                Box::new(|| self.current_values_as_i32().gt(self.previous_values_as_i32()))
            },
            ScannableType::I64 => {
                Box::new(|| self.current_values_as_i64().gt(self.previous_values_as_i64()))
            },
            ScannableType::F32 => {
                Box::new(|| self.current_values_as_f32().gt(self.previous_values_as_f32()))
            },
            ScannableType::F64 => {
                Box::new(|| self.current_values_as_f64().gt(self.previous_values_as_f64()))
            },
            _ => unimplemented!("Increased comparison is not supported for the provided type."),
        }
    }

    // Decreased comparison
    fn get_comparison_decreased(&self) -> Box<dyn Fn() -> u8x16> {
        match self.data_type {
            ScannableType::U8 => {
                Box::new(|| self.current_values.lt(self.previous_values))
            },
            ScannableType::U16 => {
                Box::new(|| self.current_values_as_u16().lt(self.previous_values_as_u16()))
            },
            ScannableType::U32 => {
                Box::new(|| self.current_values_as_u32().lt(self.previous_values_as_u32()))
            },
            ScannableType::U64 => {
                Box::new(|| self.current_values_as_u64().lt(self.previous_values_as_u64()))
            },
            ScannableType::I8 => {
                Box::new(|| self.current_values_as_i8().lt(self.previous_values_as_i8()))
            },
            ScannableType::I16 => {
                Box::new(|| self.current_values_as_i16().lt(self.previous_values_as_i16()))
            },
            ScannableType::I32 => {
                Box::new(|| self.current_values_as_i32().lt(self.previous_values_as_i32()))
            },
            ScannableType::I64 => {
                Box::new(|| self.current_values_as_i64().lt(self.previous_values_as_i64()))
            },
            ScannableType::F32 => {
                Box::new(|| self.current_values_as_f32().lt(self.previous_values_as_f32()))
            },
            ScannableType::F64 => {
                Box::new(|| self.current_values_as_f64().lt(self.previous_values_as_f64()))
            },
            _ => unimplemented!("Decreased comparison is not supported for the provided type."),
        }
    }

    // IncreasedByX comparison
    fn get_comparison_increased_by(&self, value: FieldValue, _args: Option<FieldValue>) -> Box<dyn Fn() -> u8x16> {
        match value {
            FieldValue::U8(v) => {
                let comparison_value = self.previous_values + u8x16::splat(v);
                Box::new(move || self.current_values.eq(comparison_value))
            },
            FieldValue::U16(v, _) => {
                let comparison_value = self.previous_values_as_u16() + u16x8::splat(v);
                Box::new(move || self.current_values_as_u16().eq(comparison_value))
            },
            FieldValue::U32(v, _) => {
                let comparison_value = self.previous_values_as_u32() + u32x4::splat(v);
                Box::new(move || self.current_values_as_u32().eq(comparison_value))
            },
            FieldValue::U64(v, _) => {
                let comparison_value = self.previous_values_as_u64() + u64x2::splat(v);
                Box::new(move || self.current_values_as_u64().eq(comparison_value))
            },
            FieldValue::I8(v) => {
                let comparison_value = self.previous_values_as_i8() + i8x16::splat(v);
                Box::new(move || self.current_values_as_i8().eq(comparison_value))
            },
            FieldValue::I16(v, _) => {
                let comparison_value = self.previous_values_as_i16() + i16x8::splat(v);
                Box::new(move || self.current_values_as_i16().eq(comparison_value))
            },
            FieldValue::I32(v, _) => {
                let comparison_value = self.previous_values_as_i32() + i32x4::splat(v);
                Box::new(move || self.current_values_as_i32().eq(comparison_value))
            },
            FieldValue::I64(v, _) => {
                let comparison_value = self.previous_values_as_i64() + i64x2::splat(v);
                Box::new(move || self.current_values_as_i64().eq(comparison_value))
            },
            FieldValue::F32(v, _) => {
                let comparison_value = self.previous_values_as_f32() + f32x4::splat(v);
                Box::new(move || self.current_values_as_f32().eq(comparison_value))
            },
            FieldValue::F64(v, _) => {
                let comparison_value = self.previous_values_as_f64() + f64x2::splat(v);
                Box::new(move || self.current_values_as_f64().eq(comparison_value))
            },
            _ => unimplemented!("IncreasedByX comparison is not supported for the provided type."),
        }
    }

    // DecreasedByX comparison
    fn get_comparison_decreased_by(&self, value: FieldValue, _args: Option<FieldValue>) -> Box<dyn Fn() -> u8x16> {
        match value {
            FieldValue::U8(v) => {
                let comparison_value = self.previous_values - u8x16::splat(v);
                Box::new(move || self.current_values.eq(comparison_value))
            },
            FieldValue::U16(v, _) => {
                let comparison_value = self.previous_values_as_u16() - u16x8::splat(v);
                Box::new(move || self.current_values_as_u16().eq(comparison_value))
            },
            FieldValue::U32(v, _) => {
                let comparison_value = self.previous_values_as_u32() - u32x4::splat(v);
                Box::new(move || self.current_values_as_u32().eq(comparison_value))
            },
            FieldValue::U64(v, _) => {
                let comparison_value = self.previous_values_as_u64() - u64x2::splat(v);
                Box::new(move || self.current_values_as_u64().eq(comparison_value))
            },
            FieldValue::I8(v) => {
                let comparison_value = self.previous_values_as_i8() - i8x16::splat(v);
                Box::new(move || self.current_values_as_i8().eq(comparison_value))
            },
            FieldValue::I16(v, _) => {
                let comparison_value = self.previous_values_as_i16() - i16x8::splat(v);
                Box::new(move || self.current_values_as_i16().eq(comparison_value))
            },
            FieldValue::I32(v, _) => {
                let comparison_value = self.previous_values_as_i32() - i32x4::splat(v);
                Box::new(move || self.current_values_as_i32().eq(comparison_value))
            },
            FieldValue::I64(v, _) => {
                let comparison_value = self.previous_values_as_i64() - i64x2::splat(v);
                Box::new(move || self.current_values_as_i64().eq(comparison_value))
            },
            FieldValue::F32(v, _) => {
                let comparison_value = self.previous_values_as_f32() - f32x4::splat(v);
                Box::new(move || self.current_values_as_f32().eq(comparison_value))
            },
            FieldValue::F64(v, _) => {
                let comparison_value = self.previous_values_as_f64() - f64x2::splat(v);
                Box::new(move || self.current_values_as_f64().eq(comparison_value))
            },
            _ => unimplemented!("DecreasedByX comparison is not supported for the provided type."),
        }
    }

    // Equal comparison
    fn get_comparison_equal(&self, value: FieldValue, _args: Option<FieldValue>) -> Box<dyn Fn() -> u8x16> {
        match value {
            FieldValue::U8(v) => {
                let comparison_value = u8x16::splat(v);
                Box::new(move || self.current_values.eq(comparison_value))
            },
            FieldValue::U16(v, _) => {
                let comparison_value = u16x8::splat(v);
                Box::new(move || self.current_values_as_u16().eq(comparison_value))
            },
            FieldValue::U32(v, _) => {
                let comparison_value = u32x4::splat(v);
                Box::new(move || self.current_values_as_u32().eq(comparison_value))
            },
            FieldValue::U64(v, _) => {
                let comparison_value = u64x2::splat(v);
                Box::new(move || self.current_values_as_u64().eq(comparison_value))
            },
            FieldValue::I8(v) => {
                let comparison_value = i8x16::splat(v);
                Box::new(move || self.current_values_as_i8().eq(comparison_value))
            },
            FieldValue::I16(v, _) => {
                let comparison_value = i16x8::splat(v);
                Box::new(move || self.current_values_as_i16().eq(comparison_value))
            },
            FieldValue::I32(v, _) => {
                let comparison_value = i32x4::splat(v);
                Box::new(move || self.current_values_as_i32().eq(comparison_value))
            },
            FieldValue::I64(v, _) => {
                let comparison_value = i64x2::splat(v);
                Box::new(move || self.current_values_as_i64().eq(comparison_value))
            },
            FieldValue::F32(v, _) => {
                let comparison_value = f32x4::splat(v);
                Box::new(move || self.current_values_as_f32().eq(comparison_value))
            },
            FieldValue::F64(v, _) => {
                let comparison_value = f64x2::splat(v);
                Box::new(move || self.current_values_as_f64().eq(comparison_value))
            },
            _ => unimplemented!("Equal comparison is not supported for the provided type."),
        }
    }

    // NotEqual comparison
    fn get_comparison_not_equal(&self, value: FieldValue, _args: Option<FieldValue>) -> Box<dyn Fn() -> u8x16> {
        match value {
            FieldValue::U8(v) => {
                let comparison_value = u8x16::splat(v);
                Box::new(move || !self.current_values.eq(comparison_value))
            },
            FieldValue::U16(v, _) => {
                let comparison_value = u16x8::splat(v);
                Box::new(move || !self.current_values_as_u16().eq(comparison_value))
            },
            FieldValue::U32(v, _) => {
                let comparison_value = u32x4::splat(v);
                Box::new(move || !self.current_values_as_u32().eq(comparison_value))
            },
            FieldValue::U64(v, _) => {
                let comparison_value = u64x2::splat(v);
                Box::new(move || !self.current_values_as_u64().eq(comparison_value))
            },
            FieldValue::I8(v) => {
                let comparison_value = i8x16::splat(v);
                Box::new(move || !self.current_values_as_i8().eq(comparison_value))
            },
            FieldValue::I16(v, _) => {
                let comparison_value = i16x8::splat(v);
                Box::new(move || !self.current_values_as_i16().eq(comparison_value))
            },
            FieldValue::I32(v, _) => {
                let comparison_value = i32x4::splat(v);
                Box::new(move || !self.current_values_as_i32().eq(comparison_value))
            },
            FieldValue::I64(v, _) => {
                let comparison_value = i64x2::splat(v);
                Box::new(move || !self.current_values_as_i64().eq(comparison_value))
            },
            FieldValue::F32(v, _) => {
                let comparison_value = f32x4::splat(v);
                Box::new(move || !self.current_values_as_f32().eq(comparison_value))
            },
            FieldValue::F64(v, _) => {
                let comparison_value = f64x2::splat(v);
                Box::new(move || !self.current_values_as_f64().eq(comparison_value))
            },
            _ => unimplemented!("NotEqual comparison is not supported for the provided type."),
        }
    }

    // GreaterThan comparison
    fn get_comparison_greater_than(&self, value: FieldValue) -> Box<dyn Fn() -> u8x16> {
        match value {
            FieldValue::U8(v) => {
                let comparison_value = u8x16::splat(v);
                Box::new(move || self.current_values.gt(comparison_value))
            },
            FieldValue::U16(v, _) => {
                let comparison_value = u16x8::splat(v);
                Box::new(move || self.current_values_as_u16().gt(comparison_value))
            },
            FieldValue::U32(v, _) => {
                let comparison_value = u32x4::splat(v);
                Box::new(move || self.current_values_as_u32().gt(comparison_value))
            },
            FieldValue::U64(v, _) => {
                let comparison_value = u64x2::splat(v);
                Box::new(move || self.current_values_as_u64().gt(comparison_value))
            },
            FieldValue::I8(v) => {
                let comparison_value = i8x16::splat(v);
                Box::new(move || self.current_values_as_i8().gt(comparison_value))
            },
            FieldValue::I16(v, _) => {
                let comparison_value = i16x8::splat(v);
                Box::new(move || self.current_values_as_i16().gt(comparison_value))
            },
            FieldValue::I32(v, _) => {
                let comparison_value = i32x4::splat(v);
                Box::new(move || self.current_values_as_i32().gt(comparison_value))
            },
            FieldValue::I64(v, _) => {
                let comparison_value = i64x2::splat(v);
                Box::new(move || self.current_values_as_i64().gt(comparison_value))
            },
            FieldValue::F32(v, _) => {
                let comparison_value = f32x4::splat(v);
                Box::new(move || self.current_values_as_f32().gt(comparison_value))
            },
            FieldValue::F64(v, _) => {
                let comparison_value = f64x2::splat(v);
                Box::new(move || self.current_values_as_f64().gt(comparison_value))
            },
            _ => unimplemented!("GreaterThan comparison is not supported for the provided type."),
        }
    }

    // GreaterThanOrEqual comparison
    fn get_comparison_greater_than_or_equal(&self, value: FieldValue) -> Box<dyn Fn() -> u8x16> {
        match value {
            FieldValue::U8(v) => {
                let comparison_value = u8x16::splat(v);
                Box::new(move || self.current_values.ge(comparison_value))
            },
            FieldValue::U16(v, _) => {
                let comparison_value = u16x8::splat(v);
                Box::new(move || self.current_values_as_u16().ge(comparison_value))
            },
            FieldValue::U32(v, _) => {
                let comparison_value = u32x4::splat(v);
                Box::new(move || self.current_values_as_u32().ge(comparison_value))
            },
            FieldValue::U64(v, _) => {
                let comparison_value = u64x2::splat(v);
                Box::new(move || self.current_values_as_u64().ge(comparison_value))
            },
            FieldValue::I8(v) => {
                let comparison_value = i8x16::splat(v);
                Box::new(move || self.current_values_as_i8().ge(comparison_value))
            },
            FieldValue::I16(v, _) => {
                let comparison_value = i16x8::splat(v);
                Box::new(move || self.current_values_as_i16().ge(comparison_value))
            },
            FieldValue::I32(v, _) => {
                let comparison_value = i32x4::splat(v);
                Box::new(move || self.current_values_as_i32().ge(comparison_value))
            },
            FieldValue::I64(v, _) => {
                let comparison_value = i64x2::splat(v);
                Box::new(move || self.current_values_as_i64().ge(comparison_value))
            },
            FieldValue::F32(v, _) => {
                let comparison_value = f32x4::splat(v);
                Box::new(move || self.current_values_as_f32().ge(comparison_value))
            },
            FieldValue::F64(v, _) => {
                let comparison_value = f64x2::splat(v);
                Box::new(move || self.current_values_as_f64().ge(comparison_value))
            },
            _ => unimplemented!("GreaterThanOrEqual comparison is not supported for the provided type."),
        }
    }

    // LessThan comparison
    fn get_comparison_less_than(&self, value: FieldValue) -> Box<dyn Fn() -> u8x16> {
        match value {
            FieldValue::U8(v) => {
                let comparison_value = u8x16::splat(v);
                Box::new(move || self.current_values.lt(comparison_value))
            },
            FieldValue::U16(v, _) => {
                let comparison_value = u16x8::splat(v);
                Box::new(move || self.current_values_as_u16().lt(comparison_value))
            },
            FieldValue::U32(v, _) => {
                let comparison_value = u32x4::splat(v);
                Box::new(move || self.current_values_as_u32().lt(comparison_value))
            },
            FieldValue::U64(v, _) => {
                let comparison_value = u64x2::splat(v);
                Box::new(move || self.current_values_as_u64().lt(comparison_value))
            },
            FieldValue::I8(v) => {
                let comparison_value = i8x16::splat(v);
                Box::new(move || self.current_values_as_i8().lt(comparison_value))
            },
            FieldValue::I16(v, _) => {
                let comparison_value = i16x8::splat(v);
                Box::new(move || self.current_values_as_i16().lt(comparison_value))
            },
            FieldValue::I32(v, _) => {
                let comparison_value = i32x4::splat(v);
                Box::new(move || self.current_values_as_i32().lt(comparison_value))
            },
            FieldValue::I64(v, _) => {
                let comparison_value = i64x2::splat(v);
                Box::new(move || self.current_values_as_i64().lt(comparison_value))
            },
            FieldValue::F32(v, _) => {
                let comparison_value = f32x4::splat(v);
                Box::new(move || self.current_values_as_f32().lt(comparison_value))
            },
            FieldValue::F64(v, _) => {
                let comparison_value = f64x2::splat(v);
                Box::new(move || self.current_values_as_f64().lt(comparison_value))
            },
            _ => unimplemented!("LessThan comparison is not supported for the provided type."),
        }
    }

    // LessThanOrEqual comparison
    fn get_comparison_less_than_or_equal(&self, value: FieldValue) -> Box<dyn Fn() -> u8x16> {
        match value {
            FieldValue::U8(v) => {
                let comparison_value = u8x16::splat(v);
                Box::new(move || self.current_values.le(comparison_value))
            },
            FieldValue::U16(v, _) => {
                let comparison_value = u16x8::splat(v);
                Box::new(move || self.current_values_as_u16().le(comparison_value))
            },
            FieldValue::U32(v, _) => {
                let comparison_value = u32x4::splat(v);
                Box::new(move || self.current_values_as_u32().le(comparison_value))
            },
            FieldValue::U64(v, _) => {
                let comparison_value = u64x2::splat(v);
                Box::new(move || self.current_values_as_u64().le(comparison_value))
            },
            FieldValue::I8(v) => {
                let comparison_value = i8x16::splat(v);
                Box::new(move || self.current_values_as_i8().le(comparison_value))
            },
            FieldValue::I16(v, _) => {
                let comparison_value = i16x8::splat(v);
                Box::new(move || self.current_values_as_i16().le(comparison_value))
            },
            FieldValue::I32(v, _) => {
                let comparison_value = i32x4::splat(v);
                Box::new(move || self.current_values_as_i32().le(comparison_value))
            },
            FieldValue::I64(v, _) => {
                let comparison_value = i64x2::splat(v);
                Box::new(move || self.current_values_as_i64().le(comparison_value))
            },
            FieldValue::F32(v, _) => {
                let comparison_value = f32x4::splat(v);
                Box::new(move || self.current_values_as_f32().le(comparison_value))
            },
            FieldValue::F64(v, _) => {
                let comparison_value = f64x2::splat(v);
                Box::new(move || self.current_values_as_f64().le(comparison_value))
            },
            _ => unimplemented!("LessThanOrEqual comparison is not supported for the provided type."),
        }
    }
}
