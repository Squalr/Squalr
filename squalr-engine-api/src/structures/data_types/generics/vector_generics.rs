use std::simd::SimdElement;
use std::simd::{LaneCount, Simd, SupportedLaneCount, cmp::SimdPartialEq};
use std::{mem, ptr};

pub struct VectorGenerics {}

impl VectorGenerics {
    /// Rotates left and sets the last `OFFSET` elements to 0.
    pub fn rotate_left_with_discard<const N: usize, const OFFSET: usize>(vector: Simd<u8, N>) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        let mut rotated = vector.rotate_elements_left::<OFFSET>();
        let zero_start = N.saturating_sub(OFFSET.min(N));

        for index in zero_start..N {
            rotated[index] = 0;
        }

        rotated
    }

    /// Rotates right and sets the first `OFFSET` elements to 0.
    pub fn rotate_right_with_discard<const N: usize, const OFFSET: usize>(vector: Simd<u8, N>) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        let mut rotated = vector.rotate_elements_right::<OFFSET>();

        for index in 0..OFFSET.min(N) {
            rotated[index] = 0;
        }

        rotated
    }

    /// Reinterprets a `Simd` vector as a different type.
    pub fn transmute<SourceType, TargetType, const N: usize>(value: Simd<SourceType, N>) -> Simd<TargetType, N>
    where
        LaneCount<N>: SupportedLaneCount,
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
        LaneCount<N>: SupportedLaneCount,
        LaneCount<E>: SupportedLaneCount,
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
}
