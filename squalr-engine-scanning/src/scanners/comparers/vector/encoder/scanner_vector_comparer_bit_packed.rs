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

macro_rules! pack_1bit {
    ($bitmasks:ident, $packed:ident) => {
        seq!(N in 0..16 {
            $packed[N] = 
                ($bitmasks[N * 8 + 0] << 0) |
                ($bitmasks[N * 8 + 1] << 1) |
                ($bitmasks[N * 8 + 2] << 2) |
                ($bitmasks[N * 8 + 3] << 3) |
                ($bitmasks[N * 8 + 4] << 4) |
                ($bitmasks[N * 8 + 5] << 5) |
                ($bitmasks[N * 8 + 6] << 6) |
                ($bitmasks[N * 8 + 7] << 7);
        });
    };
}

macro_rules! pack_2bit {
    ($bitmasks:ident, $packed:ident) => {
        seq!(N in 0..16 {
            $packed[N] = 
                ($bitmasks[N * 4 + 0] << 0) |
                ($bitmasks[N * 4 + 1] << 2) |
                ($bitmasks[N * 4 + 2] << 4) |
                ($bitmasks[N * 4 + 3] << 6);
        });
    };
}

macro_rules! pack_4bit {
    ($bitmasks:ident, $packed:ident) => {
        seq!(N in 0..16 {
            $packed[N] = 
                ($bitmasks[N * 2 + 0] << 0) |
                ($bitmasks[N * 2 + 1] << 4);
        });
    };
}

macro_rules! pack_8bit {
    ($bitmasks:ident, $packed:ident) => {
        seq!(N in 0..16 {
            $packed[N] = $bitmasks[N];
        });
    };
}

macro_rules! simd_compare {
    ($data_type:ty, $data_size:expr, $simd_type:ident, $simd_load_fn:ident, $simd_op:ident, $current_values_ptr:ident, $immediate_ptr:ident, $packing:ident) => {{
        unsafe {
            let immediate_value = $simd_type::splat(*($immediate_ptr as *const $data_type));

            let mut bitmasks = [0u8; $data_size];

            seq!(N in 0..$data_size {
                let current_values = $simd_type::$simd_load_fn(*($current_values_ptr.add(N * 16) as *const [$data_type; 128 / $data_size]));
                let result~N = current_values.$simd_op(immediate_value);
                bitmasks[N] = result~N.to_bitmask() as u8;
            });

            let mut packed = [0u8; 16];

            $packing!(bitmasks, packed);

            u8x16::from_array(packed)
        }
    }};
}

macro_rules! unroll_blocks_generic {(
    $num_blocks:expr,
    $simd_type:ty,
    $load_intrinsic:ident,
    $cmp_intrinsic:ident,
    $movemask_intrinsic:ident,
    $cast_intrinsic:ident,
    $immediate:expr,
    $current_values_ptr:expr,
    $element_size:expr
    ) => {{
        unsafe {
            let immediate = $immediate;
            let mut bitmask_accum = 0i32;

            seq!(N in 0..$num_blocks {
                let values_~N = $load_intrinsic($current_values_ptr.add(N * 16) as *const $simd_type);
                let cmp_mask_~N = $cmp_intrinsic(values_~N, immediate);
                let bitmask_~N = $movemask_intrinsic($cast_intrinsic(cmp_mask_~N)) as i32;
                let shift_~N = ($element_size * N) & 31;
                bitmask_accum |= bitmask_~N << shift_~N;
            })

            u8x16::from(_mm_cvtsi32_si128(bitmask_accum))
        }
    }};
}

pub struct ScannerVectorComparerBitPacked {
}

/// Implements a set of scalar (ie CPU bound, non-SIMD) boolean comparison operations to be used by more complex scanners.
impl ScannerVectorComparerBitPacked {
    fn new() -> Self {
        Self { }
    }
    
    pub fn get_instance() -> &'static ScannerVectorComparerBitPacked {
        static mut INSTANCE: Option<ScannerVectorComparerBitPacked> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = ScannerVectorComparerBitPacked::new();
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
        match data_type {
            DataType::U8() => |current_values_ptr, immediate_ptr: *const u8| {
                panic!("not implemented");
                return simd_compare!(u8, 8, u8x16, from_array, simd_eq, current_values_ptr, immediate_ptr, pack_1bit);
            },
            DataType::I8() => |current_values_ptr: *const u8, immediate_ptr| {
                panic!("not implemented");
                return simd_compare!(i8, 8, i8x16, from_array, simd_eq, current_values_ptr, immediate_ptr, pack_1bit);
            },
            DataType::U16(_) => |current_values_ptr, immediate_ptr| {
                panic!("not implemented");
                return simd_compare!(u16, 16, u16x8, from_array, simd_eq, current_values_ptr, immediate_ptr, pack_2bit);
            },
            DataType::I16(_) => |current_values_ptr, immediate_ptr| {
                panic!("not implemented");
                return simd_compare!(i16, 16, i16x8, from_array, simd_eq, current_values_ptr, immediate_ptr, pack_2bit);
            },
            DataType::U32(_) => |current_values_ptr, immediate_ptr| {
                // return simd_compare!(u32, 32, u32x4, from_array, simd_eq, current_values_ptr, immediate_ptr, pack_4bit);
                unsafe {
                    let immediate_value = u32x4::splat(*(immediate_ptr as *const u32));
                    let current_values = u32x4::from_array(*(current_values_ptr.add(0 * 16) as *const [u32; 128 / 32]));
                    let result0 = current_values.simd_eq(immediate_value);
                    let current_values = u32x4::from_array(*(current_values_ptr.add(1 * 16) as *const [u32; 128 / 32]));
                    let result1 = current_values.simd_eq(immediate_value);
                    let current_values = u32x4::from_array(*(current_values_ptr.add(2 * 16) as *const [u32; 128 / 32]));
                    let result2 = current_values.simd_eq(immediate_value);
                    let current_values = u32x4::from_array(*(current_values_ptr.add(3 * 16) as *const [u32; 128 / 32]));
                    let result3 = current_values.simd_eq(immediate_value);
                    let current_values = u32x4::from_array(*(current_values_ptr.add(4 * 16) as *const [u32; 128 / 32]));
                    let result4 = current_values.simd_eq(immediate_value);
                    let current_values =  u32x4::from_array(*(current_values_ptr.add(5 * 16) as *const [u32; 128 / 32]));
                    let result5 = current_values.simd_eq(immediate_value);
                    let current_values = u32x4::from_array(*(current_values_ptr.add(6 * 16) as *const [u32; 128 / 32]));
                    let result6 = current_values.simd_eq(immediate_value);
                    let current_values = u32x4::from_array(*(current_values_ptr.add(7 * 16) as *const [u32; 128 / 32]));
                    let result7 = current_values.simd_eq(immediate_value);
                    let current_values = u32x4::from_array(*(current_values_ptr.add(8 * 16) as *const [u32; 128 / 32]));
                    let result8 = current_values.simd_eq(immediate_value);
                    let current_values = u32x4::from_array(*(current_values_ptr.add(9 * 16) as *const [u32; 128 / 32]));
                    let result9 = current_values.simd_eq(immediate_value);
                    let current_values = u32x4::from_array(*(current_values_ptr.add(10 * 16) as *const [u32; 128 / 32]));
                    let result10 = current_values.simd_eq(immediate_value);
                    let current_values = u32x4::from_array(*(current_values_ptr.add(11 * 16) as *const [u32; 128 / 32]));
                    let result11 = current_values.simd_eq(immediate_value);
                    let current_values = u32x4::from_array(*(current_values_ptr.add(12 * 16) as *const [u32; 128 / 32]));
                    let result12 = current_values.simd_eq(immediate_value);
                    let current_values = u32x4::from_array(*(current_values_ptr.add(13 * 16) as *const [u32; 128 / 32]));
                    let result13 = current_values.simd_eq(immediate_value);
                    let current_values = u32x4::from_array(*(current_values_ptr.add(14 * 16) as *const [u32; 128 / 32]));
                    let result14 = current_values.simd_eq(immediate_value);
                    let current_values = u32x4::from_array(*(current_values_ptr.add(15 * 16) as *const [u32; 128 / 32]));
                    let result15 = current_values.simd_eq(immediate_value);
                    let current_values = u32x4::from_array(*(current_values_ptr.add(16 * 16) as *const [u32; 128 / 32]));
                    let result16 = current_values.simd_eq(immediate_value);
                    let current_values = u32x4::from_array(*(current_values_ptr.add(17 * 16) as *const [u32; 128 / 32]));
                    let result17 = current_values.simd_eq(immediate_value);
                    let current_values = u32x4::from_array(*(current_values_ptr.add(18 * 16) as *const [u32; 128 / 32]));
                    let result18 = current_values.simd_eq(immediate_value);
                    let current_values = u32x4::from_array(*(current_values_ptr.add(19 * 16) as *const [u32; 128 / 32]));
                    let result19 = current_values.simd_eq(immediate_value);
                    let current_values = u32x4::from_array(*(current_values_ptr.add(20 * 16) as *const [u32; 128 / 32]));
                    let result20 = current_values.simd_eq(immediate_value);
                    let current_values = u32x4::from_array(*(current_values_ptr.add(21 * 16) as *const [u32; 128 / 32]));
                    let result21 = current_values.simd_eq(immediate_value);
                    let current_values = u32x4::from_array(*(current_values_ptr.add(22 * 16) as *const [u32; 128 / 32]));
                    let result22 = current_values.simd_eq(immediate_value);
                    let current_values = u32x4::from_array(*(current_values_ptr.add(23 * 16) as *const [u32; 128 / 32]));
                    let result23 = current_values.simd_eq(immediate_value);
                    let current_values = u32x4::from_array(*(current_values_ptr.add(24 * 16) as *const [u32; 128 / 32]));
                    let result24 = current_values.simd_eq(immediate_value);
                    let current_values = u32x4::from_array(*(current_values_ptr.add(25 * 16) as *const [u32; 128 / 32]));
                    let result25 = current_values.simd_eq(immediate_value);
                    let current_values =  u32x4::from_array(*(current_values_ptr.add(26 * 16) as *const [u32; 128 / 32]));
                    let result26 = current_values.simd_eq(immediate_value);
                    let current_values = u32x4::from_array(*(current_values_ptr.add(27 * 16) as *const [u32; 128 / 32]));
                    let result27 = current_values.simd_eq(immediate_value);
                    let current_values = u32x4::from_array(*(current_values_ptr.add(28 * 16) as *const [u32; 128 / 32]));
                    let result28 = current_values.simd_eq(immediate_value);
                    let current_values = u32x4::from_array(*(current_values_ptr.add(29 * 16) as *const [u32; 128 / 32]));
                    let result29 = current_values.simd_eq(immediate_value);
                    let current_values = u32x4::from_array(*(current_values_ptr.add(30 * 16) as *const [u32; 128 / 32]));
                    let result30 = current_values.simd_eq(immediate_value);
                    let current_values = u32x4::from_array(*(current_values_ptr.add(31 * 16) as *const [u32; 128 / 32]));
                    let result31 = current_values.simd_eq(immediate_value);

                    let mut packed = [0u8; 16];
                    /*
                    packed[0] = (!result0.test_unchecked(0) as u8) << 0 | (!result0.test_unchecked(1) as u8) << 1 | (!result0.test_unchecked(2) as u8) << 2 | (!result0.test_unchecked(3) as u8) << 3
                        | (!result1.test_unchecked(0) as u8) << 4 | (!result1.test_unchecked(1) as u8) << 5 | (!result1.test_unchecked(2) as u8) << 6 | (!result1.test_unchecked(3) as u8) << 7;
                    
                    packed[1] = (!result2.test_unchecked(0) as u8) << 0 | (!result2.test_unchecked(1) as u8) << 1 | (!result2.test_unchecked(2) as u8) << 2 | (!result2.test_unchecked(3) as u8) << 3
                        | (!result3.test_unchecked(0) as u8) << 4 | (!result3.test_unchecked(1) as u8) << 5 | (!result3.test_unchecked(2) as u8) << 6 | (!result3.test_unchecked(3) as u8) << 7;
                    
                    packed[2] = (!result4.test_unchecked(0) as u8) << 0 | (!result4.test_unchecked(1) as u8) << 1 | (!result4.test_unchecked(2) as u8) << 2 | (!result4.test_unchecked(3) as u8) << 3
                        | (!result5.test_unchecked(0) as u8) << 4 | (!result5.test_unchecked(1) as u8) << 5 | (!result5.test_unchecked(2) as u8) << 6 | (!result5.test_unchecked(3) as u8) << 7;
                    
                    packed[3] = (!result6.test_unchecked(0) as u8) << 0 | (!result6.test_unchecked(1) as u8) << 1 | (!result6.test_unchecked(2) as u8) << 2 | (!result6.test_unchecked(3) as u8) << 3
                        | (!result7.test_unchecked(0) as u8) << 4 | (!result7.test_unchecked(1) as u8) << 5 | (!result7.test_unchecked(2) as u8) << 6 | (!result7.test_unchecked(3) as u8) << 7;
                    
                    packed[4] = (!result8.test_unchecked(0) as u8) << 0 | (!result8.test_unchecked(1) as u8) << 1 | (!result8.test_unchecked(2) as u8) << 2 | (!result8.test_unchecked(3) as u8) << 3
                        | (!result9.test_unchecked(0) as u8) << 4 | (!result9.test_unchecked(1) as u8) << 5 | (!result9.test_unchecked(2) as u8) << 6 | (!result9.test_unchecked(3) as u8) << 7;
                    
                    packed[5] = (!result10.test_unchecked(0) as u8) << 0 | (!result10.test_unchecked(1) as u8) << 1 | (!result10.test_unchecked(2) as u8) << 2 | (!result10.test_unchecked(3) as u8) << 3
                        | (!result11.test_unchecked(0) as u8) << 4 | (!result11.test_unchecked(1) as u8) << 5 | (!result11.test_unchecked(2) as u8) << 6 | (!result11.test_unchecked(3) as u8) << 7;
                    
                    packed[6] = (!result12.test_unchecked(0) as u8) << 0 | (!result12.test_unchecked(1) as u8) << 1 | (!result12.test_unchecked(2) as u8) << 2 | (!result12.test_unchecked(3) as u8) << 3
                        | (!result13.test_unchecked(0) as u8) << 4 | (!result13.test_unchecked(1) as u8) << 5 | (!result13.test_unchecked(2) as u8) << 6 | (!result13.test_unchecked(3) as u8) << 7;
                    
                    packed[7] = (!result14.test_unchecked(0) as u8) << 0 | (!result14.test_unchecked(1) as u8) << 1 | (!result14.test_unchecked(2) as u8) << 2 | (!result14.test_unchecked(3) as u8) << 3
                        | (!result15.test_unchecked(0) as u8) << 4 | (!result15.test_unchecked(1) as u8) << 5 | (!result15.test_unchecked(2) as u8) << 6 | (!result15.test_unchecked(3) as u8) << 7;
                    
                    packed[8] = (!result16.test_unchecked(0) as u8) << 0 | (!result16.test_unchecked(1) as u8) << 1 | (!result16.test_unchecked(2) as u8) << 2 | (!result16.test_unchecked(3) as u8) << 3
                        | (!result17.test_unchecked(0) as u8) << 4 | (!result17.test_unchecked(1) as u8) << 5 | (!result17.test_unchecked(2) as u8) << 6 | (!result17.test_unchecked(3) as u8) << 7;
                    
                    packed[9] = (!result18.test_unchecked(0) as u8) << 0 | (!result18.test_unchecked(1) as u8) << 1 | (!result18.test_unchecked(2) as u8) << 2 | (!result18.test_unchecked(3) as u8) << 3
                        | (!result19.test_unchecked(0) as u8) << 4 | (!result19.test_unchecked(1) as u8) << 5 | (!result19.test_unchecked(2) as u8) << 6 | (!result19.test_unchecked(3) as u8) << 7;
                    
                    packed[10] = (!result20.test_unchecked(0) as u8) << 0 | (!result20.test_unchecked(1) as u8) << 1 | (!result20.test_unchecked(2) as u8) << 2 | (!result20.test_unchecked(3) as u8) << 3
                        | (!result21.test_unchecked(0) as u8) << 4 | (!result21.test_unchecked(1) as u8) << 5 | (!result21.test_unchecked(2) as u8) << 6 | (!result21.test_unchecked(3) as u8) << 7;
                    
                    packed[11] = (!result22.test_unchecked(0) as u8) << 0 | (!result22.test_unchecked(1) as u8) << 1 | (!result22.test_unchecked(2) as u8) << 2 | (!result22.test_unchecked(3) as u8) << 3
                        | (!result23.test_unchecked(0) as u8) << 4 | (!result23.test_unchecked(1) as u8) << 5 | (!result23.test_unchecked(2) as u8) << 6 | (!result23.test_unchecked(3) as u8) << 7;
                    
                    packed[12] = (!result24.test_unchecked(0) as u8) << 0 | (!result24.test_unchecked(1) as u8) << 1 | (!result24.test_unchecked(2) as u8) << 2 | (!result24.test_unchecked(3) as u8) << 3
                        | (!result25.test_unchecked(0) as u8) << 4 | (!result25.test_unchecked(1) as u8) << 5 | (!result25.test_unchecked(2) as u8) << 6 | (!result25.test_unchecked(3) as u8) << 7;
                    
                    packed[13] = (!result26.test_unchecked(0) as u8) << 0 | (!result26.test_unchecked(1) as u8) << 1 | (!result26.test_unchecked(2) as u8) << 2 | (!result26.test_unchecked(3) as u8) << 3
                        | (!result27.test_unchecked(0) as u8) << 4 | (!result27.test_unchecked(1) as u8) << 5 | (!result27.test_unchecked(2) as u8) << 6 | (!result27.test_unchecked(3) as u8) << 7;
                    
                    packed[14] = (!result28.test_unchecked(0) as u8) << 0 | (!result28.test_unchecked(1) as u8) << 1 | (!result28.test_unchecked(2) as u8) << 2 | (!result28.test_unchecked(3) as u8) << 3
                        | (!result29.test_unchecked(0) as u8) << 4 | (!result29.test_unchecked(1) as u8) << 5 | (!result29.test_unchecked(2) as u8) << 6 | (!result29.test_unchecked(3) as u8) << 7;
                    
                    packed[15] = (!result30.test_unchecked(0) as u8) << 0 | (!result30.test_unchecked(1) as u8) << 1 | (!result30.test_unchecked(2) as u8) << 2 | (!result30.test_unchecked(3) as u8) << 3
                        | (!result31.test_unchecked(0) as u8) << 4 | (!result31.test_unchecked(1) as u8) << 5 | (!result31.test_unchecked(2) as u8) << 6 | (!result31.test_unchecked(3) as u8) << 7;
                

                    return u8x16::from_array(packed); */
                    
                    let bitmask_0 = result0.to_bitmask();
                    let bitmask_1 = result1.to_bitmask();
                    let bitmask_2 = result2.to_bitmask();
                    let bitmask_3 = result3.to_bitmask();
                    let bitmask_4 = result4.to_bitmask();
                    let bitmask_5 = result5.to_bitmask();
                    let bitmask_6 = result6.to_bitmask();
                    let bitmask_7 = result7.to_bitmask();
                    let bitmask_8 = result8.to_bitmask();
                    let bitmask_9 = result9.to_bitmask();
                    let bitmask_10 = result10.to_bitmask();
                    let bitmask_11 = result11.to_bitmask();
                    let bitmask_12 = result12.to_bitmask();
                    let bitmask_13 = result13.to_bitmask();
                    let bitmask_14 = result14.to_bitmask();
                    let bitmask_15 = result15.to_bitmask();
                    
                    let bitmask_16 = result16.to_bitmask();
                    let bitmask_17 = result17.to_bitmask();
                    let bitmask_18 = result18.to_bitmask();
                    let bitmask_19 = result19.to_bitmask();
                    let bitmask_20 = result20.to_bitmask();
                    let bitmask_21 = result21.to_bitmask();
                    let bitmask_22 = result22.to_bitmask();
                    let bitmask_23 = result23.to_bitmask();
                    let bitmask_24 = result24.to_bitmask();
                    let bitmask_25 = result25.to_bitmask();
                    let bitmask_26 = result26.to_bitmask();
                    let bitmask_27 = result27.to_bitmask();
                    let bitmask_28 = result28.to_bitmask();
                    let bitmask_29 = result29.to_bitmask();
                    let bitmask_30 = result30.to_bitmask();
                    let bitmask_31 = result31.to_bitmask();

                    let packed_0 = ((bitmask_0 as u8) << 0) | ((bitmask_1 as u8) << 4);
                    let packed_1 = ((bitmask_2 as u8) << 0) | ((bitmask_3 as u8) << 4);
                    let packed_2 = ((bitmask_4 as u8) << 0) | ((bitmask_5 as u8) << 4);
                    let packed_3 = ((bitmask_6 as u8) << 0) | ((bitmask_7 as u8) << 4);
                    let packed_4 = ((bitmask_8 as u8) << 0) | ((bitmask_9 as u8) << 4);
                    let packed_5 = ((bitmask_10 as u8) << 0) | ((bitmask_11 as u8) << 4);
                    let packed_6 = ((bitmask_12 as u8) << 0) | ((bitmask_13 as u8) << 4);
                    let packed_7 = ((bitmask_14 as u8) << 0) | ((bitmask_15 as u8) << 4);
                    
                    let packed_8 = ((bitmask_16 as u8) << 0) | ((bitmask_17 as u8) << 4);
                    let packed_9 = ((bitmask_18 as u8) << 0) | ((bitmask_19 as u8) << 4);
                    let packed_10 = ((bitmask_20 as u8) << 0) | ((bitmask_21 as u8) << 4);
                    let packed_11 = ((bitmask_22 as u8) << 0) | ((bitmask_23 as u8) << 4);
                    let packed_12 = ((bitmask_24 as u8) << 0) | ((bitmask_25 as u8) << 4);
                    let packed_13 = ((bitmask_26 as u8) << 0) | ((bitmask_27 as u8) << 4);
                    let packed_14 = ((bitmask_28 as u8) << 0) | ((bitmask_29 as u8) << 4);
                    let packed_15 = ((bitmask_30 as u8) << 0) | ((bitmask_31 as u8) << 4);

                    u8x16::from_array([
                        packed_0, packed_1, packed_2, packed_3,
                        packed_4, packed_5, packed_6, packed_7,
                        packed_8, packed_9, packed_10, packed_11,
                        packed_12, packed_13, packed_14, packed_15,
                    ])
                }
            },
            DataType::I32(_) => |current_values_ptr, immediate_ptr| {
                return simd_compare!(i32, 32, i32x4, from_array, simd_eq, current_values_ptr, immediate_ptr, pack_4bit);
            },
            DataType::U64(_) => |current_values_ptr, immediate_ptr| {
                return simd_compare!(u64, 64, u64x2, from_array, simd_eq, current_values_ptr, immediate_ptr, pack_8bit);
            },
            DataType::I64(_) => |current_values_ptr, immediate_ptr| {
                return simd_compare!(i64, 64, i64x2, from_array, simd_eq, current_values_ptr, immediate_ptr, pack_8bit);
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

    fn get_compare_changed(&self, _data_type: &DataType) -> VectorCompareFnRelative {
        panic!("get_compare_changed not implemented")
    }

    fn get_compare_unchanged(&self, _data_type: &DataType) -> VectorCompareFnRelative {
        panic!("get_compare_unchanged not implemented")
    }

    fn get_compare_increased(&self, _data_type: &DataType) -> VectorCompareFnRelative {
        panic!("get_compare_increased not implemented")
    }

    fn get_compare_decreased(&self, _data_type: &DataType) -> VectorCompareFnRelative {
        panic!("get_compare_decreased not implemented")
    }

    fn get_compare_increased_by(&self, _data_type: &DataType) -> VectorCompareFnDelta {
        panic!("get_compare_increased_by not implemented")
    }

    fn get_compare_decreased_by(&self, _data_type: &DataType) -> VectorCompareFnDelta {
        panic!("get_compare_decreased_by not implemented")
    }   
}
