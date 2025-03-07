use std::simd::SimdElement;
use std::simd::{LaneCount, Simd, SupportedLaneCount, cmp::SimdPartialEq};
use std::{mem, ptr};

pub struct VectorGenerics {}

impl VectorGenerics {
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

    pub fn mask_to_simd<PrimitiveType, const N: usize>(mask: <Simd<PrimitiveType, N> as SimdPartialEq>::Mask) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
        PrimitiveType: SimdElement,
        Simd<PrimitiveType, N>: SimdPartialEq,
    {
        let mut result_array = [0u8; N];
        let value_ptr = &mask as *const _ as *const u8;
        unsafe {
            std::ptr::copy_nonoverlapping(value_ptr, result_array.as_mut_ptr(), std::mem::size_of_val(&mask));
        }

        Simd::<u8, N>::from_array(result_array)

        /*
        unsafe {
            let mut result = mem::MaybeUninit::<Simd<u8, N>>::uninit();
            ptr::copy_nonoverlapping(&mask as *const _ as *const u8, result.as_mut_ptr() as *mut u8, mem::size_of::<Simd<u8, N>>());
            result.assume_init()
        } */

        /*
        // Unsafe variant:
        // These are guaranteed to be the same size, but std::mem::transmute() is not passing Rust's compile checks
        // Perhaps Rust is not smart enough to realize that the resulting sizes are the exact same.
        // return *(&*value as *const _ as *const Simd<u8, N>);
        // Create an output array of u8 initialized to zero.
        let mut result_array = [0u8; N];

        // Calculate how many bytes can safely be copied (the smaller of mask or result_array).
        let copy_size = core::cmp::min(mem::size_of_val(&mask), N);

        unsafe {
            let mask_ptr = &mask as *const _ as *const u8;
            let result_ptr = result_array.as_mut_ptr();
            ptr::copy_nonoverlapping(mask_ptr, result_ptr, copy_size);
        }

        Simd::<u8, N>::from_array(result_array) */
    }
}
