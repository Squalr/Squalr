use std::simd::{
    LaneCount,
    Simd,
    SimdElement,
    SupportedLaneCount,
};

pub trait SimdType: SimdElement {
    type SimdVector<const N: usize>: Copy
    where
        LaneCount<N>: SupportedLaneCount;

    fn splat<const N: usize>(value: Self) -> Self::SimdVector<N>
    where
        LaneCount<N>: SupportedLaneCount;
}

impl SimdType for u8 {
    type SimdVector<const N: usize> = Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount;

    fn splat<const N: usize>(value: u8) -> Self::SimdVector<N>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        Simd::splat(value)
    }
}

impl SimdType for i8 {
    type SimdVector<const N: usize> = Simd<i8, N>
    where
        LaneCount<N>: SupportedLaneCount;

    fn splat<const N: usize>(value: i8) -> Self::SimdVector<N>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        Simd::splat(value)
    }
}

impl SimdType for u16 {
    type SimdVector<const N: usize> = Simd<u16, N>
    where
        LaneCount<N>: SupportedLaneCount;

    fn splat<const N: usize>(value: u16) -> Self::SimdVector<N>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        Simd::splat(value)
    }
}

impl SimdType for i16 {
    type SimdVector<const N: usize> = Simd<i16, N>
    where
        LaneCount<N>: SupportedLaneCount;

    fn splat<const N: usize>(value: i16) -> Self::SimdVector<N>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        Simd::splat(value)
    }
}

impl SimdType for u32 {
    type SimdVector<const N: usize> = Simd<u32, N>
    where
        LaneCount<N>: SupportedLaneCount;

    fn splat<const N: usize>(value: u32) -> Self::SimdVector<N>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        Simd::splat(value)
    }
}

impl SimdType for i32 {
    type SimdVector<const N: usize> = Simd<i32, N>
    where
        LaneCount<N>: SupportedLaneCount;

    fn splat<const N: usize>(value: i32) -> Self::SimdVector<N>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        Simd::splat(value)
    }
}

impl SimdType for u64 {
    type SimdVector<const N: usize> = Simd<u64, N>
    where
        LaneCount<N>: SupportedLaneCount;

    fn splat<const N: usize>(value: u64) -> Self::SimdVector<N>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        Simd::splat(value)
    }
}

impl SimdType for i64 {
    type SimdVector<const N: usize> = Simd<i64, N>
    where
        LaneCount<N>: SupportedLaneCount;

    fn splat<const N: usize>(value: i64) -> Self::SimdVector<N>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        Simd::splat(value)
    }
}

impl SimdType for f32 {
    type SimdVector<const N: usize> = Simd<f32, N>
    where
        LaneCount<N>: SupportedLaneCount;

    fn splat<const N: usize>(value: f32) -> Self::SimdVector<N>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        Simd::splat(value)
    }
}

impl SimdType for f64 {
    type SimdVector<const N: usize> = Simd<f64, N>
    where
        LaneCount<N>: SupportedLaneCount;

    fn splat<const N: usize>(value: f64) -> Self::SimdVector<N>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        Simd::splat(value)
    }
}
