use squalr_engine_common::values::data_type::DataType;

use crate::scanners::parameters::scan_compare_type::ScanCompareType;
use std::{arch::x86_64::{__m128i, _mm_castsi128_ps, _mm_cmpeq_epi32, _mm_loadu_si128, _mm_loadu_si32, _mm_movemask_ps, _mm_or_si128, _mm_packs_epi16, _mm_packs_epi32, _mm_set1_epi32, _mm_set1_epi8, _mm_slli_epi32, _mm_storeu_si128}, simd::{cmp::SimdPartialEq, f32x4, i16x8, i32x4, i64x2, i8x16, mask8x16, u16x8, u32x16, u32x4, u32x8, u64x2, u8x16, Mask, Simd}, sync::Once};
use std::arch::x86_64::_mm_movemask_epi8;
use std::arch::x86_64::_mm_setzero_si128;

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
                let immediate = unsafe { u8x16::splat(*immediate_ptr) };
                let v1 = unsafe { u8x16::from_slice(std::slice::from_raw_parts(current_values_ptr, 16)) };
                // v1.simd_eq(immediate)
                panic!("stupid");
            },
            DataType::I8() => |current_values_ptr: *const u8, immediate_ptr| {
                let immediate = unsafe { i8x16::splat(*(immediate_ptr as *const i8)) };
                let v1 = unsafe { i8x16::from_slice(std::slice::from_raw_parts(current_values_ptr as *const i8, 16)) };
                // v1.simd_eq(immediate).cast()
                panic!("stupid");
            },
            DataType::U16(_) => |current_values_ptr, immediate_ptr| {
                let immediate = unsafe { u16x8::splat(*(immediate_ptr as *const u16)) };
                let v1 = unsafe { u16x8::from_slice(std::slice::from_raw_parts(current_values_ptr as *const u16, 8)) };
                let v2 = unsafe { u16x8::from_slice(std::slice::from_raw_parts((current_values_ptr.add(16)) as *const u16, 8)) };
                let v1_res = v1.simd_eq(immediate).cast::<i8>();
                let v2_res = v2.simd_eq(immediate).cast::<i8>();

                let mut combined_array = [false; 16];
                combined_array[..8].copy_from_slice(&v1_res.to_array());
                combined_array[8..].copy_from_slice(&v2_res.to_array());

                panic!("stupid");
            },
            DataType::I16(_) => |current_values_ptr, immediate_ptr| {
                let immediate = unsafe { i16x8::splat(*(immediate_ptr as *const i16)) };
                let v1 = unsafe { i16x8::from_slice(std::slice::from_raw_parts(current_values_ptr as *const i16, 8)) };
                let v2 = unsafe { i16x8::from_slice(std::slice::from_raw_parts((current_values_ptr.add(16)) as *const i16, 8)) };
                let v1_res = v1.simd_eq(immediate).cast::<i8>();
                let v2_res = v2.simd_eq(immediate).cast::<i8>();

                let mut combined_array = [false; 16];
                combined_array[..8].copy_from_slice(&v1_res.to_array());
                combined_array[8..].copy_from_slice(&v2_res.to_array());

                // Mask::from_array(combined_array)
                panic!("stupid");
            },
            DataType::U32(_) => |current_values_ptr, immediate_ptr| {
                unsafe {
                    // Load the immediate value and broadcast it to all lanes of a 128-bit register
                    let immediate = *(immediate_ptr as *const u32);
                    let immediate_simd = _mm_set1_epi32(immediate as i32);
            
                    // Initialize a zeroed result array of 128 bits (16 bytes)
                    let mut result_array = [0u8; 16];

                    // Iterate over chunks of 16 bytes (4 u32 values)
                    for i in 0..32 {
                        // Load 16 bytes (4 u32 values) from current_values_ptr into an __m128i register
                        let values_simd = _mm_loadu_si128(current_values_ptr.add(i * 16) as *const __m128i);

                        // Compare the 4 u32 values with the immediate value in parallel
                        let cmp_mask = _mm_cmpeq_epi32(values_simd, immediate_simd);

                        // Convert the comparison result to a bitmask
                        let bitmask = _mm_movemask_ps(_mm_castsi128_ps(cmp_mask));

                        // Set the corresponding bits in the result array based on the bitmask
                        for j in 0..4 {
                            if (bitmask & (1 << j)) != 0 {
                                let bit_index = i * 4 + j;
                                let byte = bit_index / 8;
                                let bit = bit_index % 8;
                                result_array[byte] |= 1 << bit;
                            }
                        }
                    }

                    // Return the final result as a __m128i (u8x16)
                    u8x16::from(result_array)
                }
                /*
                let immediate = unsafe { *(immediate_ptr as *const u32) };
                let immediate_simd = u32x4::splat(immediate);
                let mut result_array = [0u8; 16];
                
                unsafe {
                let immediate = unsafe { *(immediate_ptr as *const u32) };
                let mut result_array = [0u8; 16];
            
                unsafe {
                    // Iterate through all u32 values (128 integers)
                    for i in 0..128 {
                        let value = *(current_values_ptr.add(i * 4) as *const u32);
            
                        if value == immediate {
                            let byte = i / 8;  // Determine which byte to update
                            let bit = i % 8;   // Determine which bit in that byte
                            result_array[byte] |= 1 << bit;  // Set the appropriate bit
                        }
                    }
                }
            
                u8x16::from(result_array) */
            },
            DataType::I32(_) => |current_values_ptr, immediate_ptr| {
                let immediate = unsafe { i32x4::splat(*(immediate_ptr as *const i32)) };
                let v1 = unsafe { i32x4::from_slice(std::slice::from_raw_parts(current_values_ptr as *const i32, 4)) };
                let v2 = unsafe { i32x4::from_slice(std::slice::from_raw_parts((current_values_ptr.add(16)) as *const i32, 4)) };
                let v3 = unsafe { i32x4::from_slice(std::slice::from_raw_parts((current_values_ptr.add(32)) as *const i32, 4)) };
                let v4 = unsafe { i32x4::from_slice(std::slice::from_raw_parts((current_values_ptr.add(48)) as *const i32, 4)) };
                let v1_res = v1.simd_eq(immediate).cast::<i8>();
                let v2_res = v2.simd_eq(immediate).cast::<i8>();
                let v3_res = v3.simd_eq(immediate).cast::<i8>();
                let v4_res = v4.simd_eq(immediate).cast::<i8>();
    
                let mut combined_array = [false; 16];
                combined_array[..4].copy_from_slice(&v1_res.to_array());
                combined_array[4..8].copy_from_slice(&v2_res.to_array());
                combined_array[8..12].copy_from_slice(&v3_res.to_array());
                combined_array[12..].copy_from_slice(&v4_res.to_array());
    
                // Mask::from_array(combined_array)
                panic!("stupid");
            },
            DataType::U64(_) => |current_values_ptr, immediate_ptr| {
                let immediate = unsafe { u64x2::splat(*(immediate_ptr as *const u64)) };
                let v1 = unsafe { u64x2::from_slice(std::slice::from_raw_parts(current_values_ptr as *const u64, 2)) };
                let v2 = unsafe { u64x2::from_slice(std::slice::from_raw_parts((current_values_ptr.add(16)) as *const u64, 2)) };
                let v3 = unsafe { u64x2::from_slice(std::slice::from_raw_parts((current_values_ptr.add(32)) as *const u64, 2)) };
                let v4 = unsafe { u64x2::from_slice(std::slice::from_raw_parts((current_values_ptr.add(48)) as *const u64, 2)) };
                let v5 = unsafe { u64x2::from_slice(std::slice::from_raw_parts((current_values_ptr.add(64)) as *const u64, 2)) };
                let v6 = unsafe { u64x2::from_slice(std::slice::from_raw_parts((current_values_ptr.add(80)) as *const u64, 2)) };
                let v7 = unsafe { u64x2::from_slice(std::slice::from_raw_parts((current_values_ptr.add(96)) as *const u64, 2)) };
                let v8 = unsafe { u64x2::from_slice(std::slice::from_raw_parts((current_values_ptr.add(112)) as *const u64, 2)) };
            
                let v1_res = v1.simd_eq(immediate).cast::<i8>();
                let v2_res = v2.simd_eq(immediate).cast::<i8>();
                let v3_res = v3.simd_eq(immediate).cast::<i8>();
                let v4_res = v4.simd_eq(immediate).cast::<i8>();
                let v5_res = v5.simd_eq(immediate).cast::<i8>();
                let v6_res = v6.simd_eq(immediate).cast::<i8>();
                let v7_res = v7.simd_eq(immediate).cast::<i8>();
                let v8_res = v8.simd_eq(immediate).cast::<i8>();
            
                let mut combined_array = [false; 16];
                combined_array[..2].copy_from_slice(&v1_res.to_array());
                combined_array[2..4].copy_from_slice(&v2_res.to_array());
                combined_array[4..6].copy_from_slice(&v3_res.to_array());
                combined_array[6..8].copy_from_slice(&v4_res.to_array());
                combined_array[8..10].copy_from_slice(&v5_res.to_array());
                combined_array[10..12].copy_from_slice(&v6_res.to_array());
                combined_array[12..14].copy_from_slice(&v7_res.to_array());
                combined_array[14..16].copy_from_slice(&v8_res.to_array());
            
                // Mask::from_array(combined_array)
                panic!("stupid");
            },
            DataType::I64(_) => |current_values_ptr, immediate_ptr| {
                let immediate = unsafe { i64x2::splat(*(immediate_ptr as *const i64)) };
                let v1 = unsafe { i64x2::from_slice(std::slice::from_raw_parts(current_values_ptr as *const i64, 2)) };
                let v2 = unsafe { i64x2::from_slice(std::slice::from_raw_parts((current_values_ptr.add(16)) as *const i64, 2)) };
                let v3 = unsafe { i64x2::from_slice(std::slice::from_raw_parts((current_values_ptr.add(32)) as *const i64, 2)) };
                let v4 = unsafe { i64x2::from_slice(std::slice::from_raw_parts((current_values_ptr.add(48)) as *const i64, 2)) };
                let v5 = unsafe { i64x2::from_slice(std::slice::from_raw_parts((current_values_ptr.add(64)) as *const i64, 2)) };
                let v6 = unsafe { i64x2::from_slice(std::slice::from_raw_parts((current_values_ptr.add(80)) as *const i64, 2)) };
                let v7 = unsafe { i64x2::from_slice(std::slice::from_raw_parts((current_values_ptr.add(96)) as *const i64, 2)) };
                let v8 = unsafe { i64x2::from_slice(std::slice::from_raw_parts((current_values_ptr.add(112)) as *const i64, 2)) };

                let v1_res = v1.simd_eq(immediate).cast::<i8>();
                let v2_res = v2.simd_eq(immediate).cast::<i8>();
                let v3_res = v3.simd_eq(immediate).cast::<i8>();
                let v4_res = v4.simd_eq(immediate).cast::<i8>();
                let v5_res = v5.simd_eq(immediate).cast::<i8>();
                let v6_res = v6.simd_eq(immediate).cast::<i8>();
                let v7_res = v7.simd_eq(immediate).cast::<i8>();
                let v8_res = v8.simd_eq(immediate).cast::<i8>();
            
                let mut combined_array = [false; 16];
                combined_array[..2].copy_from_slice(&v1_res.to_array());
                combined_array[2..4].copy_from_slice(&v2_res.to_array());
                combined_array[4..6].copy_from_slice(&v3_res.to_array());
                combined_array[6..8].copy_from_slice(&v4_res.to_array());
                combined_array[8..10].copy_from_slice(&v5_res.to_array());
                combined_array[10..12].copy_from_slice(&v6_res.to_array());
                combined_array[12..14].copy_from_slice(&v7_res.to_array());
                combined_array[14..16].copy_from_slice(&v8_res.to_array());
            
                // Mask::from_array(combined_array)
                panic!("stupid");
            },
            DataType::F32(endian) => {
                panic!("unsupported data type")
            }
            DataType::F64(endian) => {
                panic!("unsupported data type")
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
