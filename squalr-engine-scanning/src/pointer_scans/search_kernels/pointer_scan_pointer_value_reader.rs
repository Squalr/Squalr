use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use std::ptr;

pub(crate) unsafe fn read_pointer_value_unchecked(
    pointer_bytes_ptr: *const u8,
    pointer_size: PointerScanPointerSize,
) -> u64 {
    match pointer_size {
        PointerScanPointerSize::Pointer32 => u32::from_le(unsafe { ptr::read_unaligned(pointer_bytes_ptr as *const u32) }) as u64,
        PointerScanPointerSize::Pointer64 => u64::from_le(unsafe { ptr::read_unaligned(pointer_bytes_ptr as *const u64) }),
    }
}

pub(crate) unsafe fn read_pointer_lane_values_u32<const SIMD_LANE_COUNT: usize>(pointer_bytes_ptr: *const u8) -> [u32; SIMD_LANE_COUNT] {
    let lane_values = unsafe { ptr::read_unaligned(pointer_bytes_ptr as *const [u32; SIMD_LANE_COUNT]) };

    #[cfg(target_endian = "big")]
    let lane_values = lane_values.map(u32::from_le);

    lane_values
}

pub(crate) unsafe fn read_pointer_lane_values_u64<const SIMD_LANE_COUNT: usize>(pointer_bytes_ptr: *const u8) -> [u64; SIMD_LANE_COUNT] {
    let lane_values = unsafe { ptr::read_unaligned(pointer_bytes_ptr as *const [u64; SIMD_LANE_COUNT]) };

    #[cfg(target_endian = "big")]
    let lane_values = lane_values.map(u64::from_le);

    lane_values
}
