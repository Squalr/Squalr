use std::simd::SimdElement;
use std::simd::{Simd, cmp::SimdPartialEq};
use std::{mem, ptr};

use crate::structures::data_types::generics::vectorization_plan::VectorizationPlan;

pub struct VectorGenerics {}

impl VectorGenerics {
    /// Reinterprets a `Simd` vector as a different type.
    pub fn transmute<SourceType, TargetType, const N: usize>(value: Simd<SourceType, N>) -> Simd<TargetType, N>
    where
        SourceType: SimdElement,
        TargetType: SimdElement,
        Simd<TargetType, N>: SimdPartialEq,
    {
        unsafe {
            let mut result = mem::MaybeUninit::<Simd<TargetType, N>>::uninit();
            ptr::copy_nonoverlapping(
                &value as *const _ as *const u8,
                result.as_mut_ptr() as *mut u8,
                mem::size_of::<Simd<TargetType, N>>(),
            );
            result.assume_init()
        }
    }

    /// Reinterprets a `Mask` type as a `Simd` vector of raw bytes.
    pub fn transmute_mask<PrimitiveType, const N: usize, const E: usize>(mask: <Simd<PrimitiveType, E> as SimdPartialEq>::Mask) -> Simd<u8, N>
    where
        PrimitiveType: SimdElement,
        Simd<PrimitiveType, E>: SimdPartialEq,
    {
        let mut result_array = [0u8; N];
        let value_ptr = &mask as *const _ as *const u8;
        unsafe {
            std::ptr::copy_nonoverlapping(value_ptr, result_array.as_mut_ptr(), std::mem::size_of_val(&mask));
        }

        Simd::<u8, N>::from_array(result_array)
    }
    /// Rotates left and sets the last `OFFSET` elements to 0.
    pub fn rotate_left_with_discard<const N: usize, const OFFSET: usize>(vector: Simd<u8, N>) -> Simd<u8, N> {
        let mut rotated = vector.rotate_elements_left::<OFFSET>();
        let zero_start = N.saturating_sub(OFFSET.min(N));

        for index in zero_start..N {
            rotated[index] = 0;
        }

        rotated
    }

    /// Rotates right and sets the first `OFFSET` elements to 0.
    pub fn rotate_right_with_discard<const N: usize, const OFFSET: usize>(vector: Simd<u8, N>) -> Simd<u8, N> {
        let mut rotated = vector.rotate_elements_right::<OFFSET>();

        for index in 0..OFFSET.min(N) {
            rotated[index] = 0;
        }

        rotated
    }

    /// Rotates left and sets the last `OFFSET` elements to 0, up to 8 rotations.
    pub fn rotate_left_with_discard_max_8<const N: usize>(
        vector: Simd<u8, N>,
        rotation: u64,
    ) -> Simd<u8, N> {
        let mut rotated = match rotation {
            1 => vector.rotate_elements_left::<1>(),
            2 => vector.rotate_elements_left::<2>(),
            3 => vector.rotate_elements_left::<3>(),
            4 => vector.rotate_elements_left::<4>(),
            5 => vector.rotate_elements_left::<5>(),
            6 => vector.rotate_elements_left::<6>(),
            7 => vector.rotate_elements_left::<7>(),
            8 => vector.rotate_elements_left::<8>(),
            _ => vector,
        };
        let zero_start = N.saturating_sub((rotation as usize).min(N));

        for index in zero_start..N {
            rotated[index] = 0;
        }

        rotated
    }

    /// Rotates right and sets the last `OFFSET` elements to 0, up to 8 rotations.
    pub fn rotate_right_with_discard_max_8<const N: usize>(
        vector: Simd<u8, N>,
        rotation: u64,
    ) -> Simd<u8, N> {
        let mut rotated = match rotation {
            1 => vector.rotate_elements_right::<1>(),
            2 => vector.rotate_elements_right::<2>(),
            3 => vector.rotate_elements_right::<3>(),
            4 => vector.rotate_elements_right::<4>(),
            5 => vector.rotate_elements_right::<5>(),
            6 => vector.rotate_elements_right::<6>(),
            7 => vector.rotate_elements_right::<7>(),
            8 => vector.rotate_elements_right::<8>(),
            _ => vector,
        };

        for index in 0..(rotation as usize).min(N) {
            rotated[index] = 0;
        }

        rotated
    }

    /// Creates a vectorization plan that determines how many full hardware vectors can iterate over the given region.
    pub fn plan_vector_scan<const N: usize>(
        region_size: u64,
        data_type_size_bytes: u64,
        element_stride_bytes: u64,
    ) -> VectorizationPlan {
        let vector_size_in_bytes = N as u64;
        let stride = element_stride_bytes.max(1);

        // Same semantics as your get_element_count():
        let trailing_bytes = data_type_size_bytes.saturating_sub(stride);
        let valid_bytes = region_size.saturating_sub(trailing_bytes);
        let element_count = valid_bytes / stride;

        let elements_per_vector = vector_size_in_bytes / stride;

        VectorizationPlan {
            vector_size_in_bytes,
            element_stride_bytes: stride,
            valid_bytes,
            element_count,
            elements_per_vector,
        }
    }
}
