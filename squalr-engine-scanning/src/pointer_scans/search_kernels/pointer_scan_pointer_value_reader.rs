use squalr_engine_api::structures::memory::endian::Endian;
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use std::ptr;

unsafe fn read_unsigned_24_bit_unchecked(
    value_ptr: *const u8,
    endian: Endian,
) -> u64 {
    let value_bytes = unsafe { [*value_ptr, *value_ptr.add(1), *value_ptr.add(2)] };

    match endian {
        Endian::Little => u32::from_le_bytes([value_bytes[0], value_bytes[1], value_bytes[2], 0]) as u64,
        Endian::Big => u32::from_be_bytes([0, value_bytes[0], value_bytes[1], value_bytes[2]]) as u64,
    }
}

pub(crate) unsafe fn read_pointer_value_unchecked(
    pointer_bytes_ptr: *const u8,
    pointer_size: PointerScanPointerSize,
) -> u64 {
    match pointer_size {
        PointerScanPointerSize::Pointer24 => unsafe { read_unsigned_24_bit_unchecked(pointer_bytes_ptr, Endian::Little) },
        PointerScanPointerSize::Pointer24be => unsafe { read_unsigned_24_bit_unchecked(pointer_bytes_ptr, Endian::Big) },
        PointerScanPointerSize::Pointer32 => u32::from_le(unsafe { ptr::read_unaligned(pointer_bytes_ptr as *const u32) }) as u64,
        PointerScanPointerSize::Pointer32be => u32::from_be(unsafe { ptr::read_unaligned(pointer_bytes_ptr as *const u32) }) as u64,
        PointerScanPointerSize::Pointer64 => u64::from_le(unsafe { ptr::read_unaligned(pointer_bytes_ptr as *const u64) }),
        PointerScanPointerSize::Pointer64be => u64::from_be(unsafe { ptr::read_unaligned(pointer_bytes_ptr as *const u64) }),
    }
}

pub(crate) unsafe fn read_pointer_lane_values_u32<const SIMD_LANE_COUNT: usize>(
    pointer_bytes_ptr: *const u8,
    pointer_size: PointerScanPointerSize,
) -> [u32; SIMD_LANE_COUNT] {
    let lane_values = unsafe { ptr::read_unaligned(pointer_bytes_ptr as *const [u32; SIMD_LANE_COUNT]) };

    match pointer_size {
        PointerScanPointerSize::Pointer24 | PointerScanPointerSize::Pointer24be => {
            unreachable!("24-bit pointer SIMD lane reads are not supported; scalar search should be used instead.")
        }
        PointerScanPointerSize::Pointer32 => lane_values.map(u32::from_le),
        PointerScanPointerSize::Pointer32be => lane_values.map(u32::from_be),
        PointerScanPointerSize::Pointer64 | PointerScanPointerSize::Pointer64be => lane_values,
    }
}

pub(crate) unsafe fn read_pointer_lane_values_u64<const SIMD_LANE_COUNT: usize>(
    pointer_bytes_ptr: *const u8,
    pointer_size: PointerScanPointerSize,
) -> [u64; SIMD_LANE_COUNT] {
    let lane_values = unsafe { ptr::read_unaligned(pointer_bytes_ptr as *const [u64; SIMD_LANE_COUNT]) };

    match pointer_size {
        PointerScanPointerSize::Pointer24 | PointerScanPointerSize::Pointer24be => {
            unreachable!("24-bit pointer SIMD lane reads are not supported; scalar search should be used instead.")
        }
        PointerScanPointerSize::Pointer64 => lane_values.map(u64::from_le),
        PointerScanPointerSize::Pointer64be => lane_values.map(u64::from_be),
        PointerScanPointerSize::Pointer32 | PointerScanPointerSize::Pointer32be => lane_values,
    }
}

#[cfg(test)]
mod tests {
    use super::{read_pointer_lane_values_u32, read_pointer_lane_values_u64, read_pointer_value_unchecked};
    use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;

    #[test]
    fn read_pointer_value_supports_big_endian_formats() {
        let pointer24_bytes = [0x12, 0x34, 0x56];
        let pointer32_bytes = 0x1234_5678_u32.to_be_bytes();
        let pointer64_bytes = 0x1234_5678_9ABC_DEF0_u64.to_be_bytes();

        assert_eq!(
            unsafe { read_pointer_value_unchecked(pointer24_bytes.as_ptr(), PointerScanPointerSize::Pointer24be) },
            0x1234_56
        );
        assert_eq!(
            unsafe { read_pointer_value_unchecked(pointer32_bytes.as_ptr(), PointerScanPointerSize::Pointer32be) },
            0x1234_5678
        );
        assert_eq!(
            unsafe { read_pointer_value_unchecked(pointer64_bytes.as_ptr(), PointerScanPointerSize::Pointer64be) },
            0x1234_5678_9ABC_DEF0
        );
    }

    #[test]
    fn read_pointer_lane_values_support_big_endian_formats() {
        let pointer32_bytes = [0x1234_5678_u32.to_be_bytes(), 0x90AB_CDEF_u32.to_be_bytes()].concat();
        let pointer64_bytes = [
            0x1234_5678_9ABC_DEF0_u64.to_be_bytes(),
            0x1111_2222_3333_4444_u64.to_be_bytes(),
        ]
        .concat();

        assert_eq!(
            unsafe { read_pointer_lane_values_u32::<2>(pointer32_bytes.as_ptr(), PointerScanPointerSize::Pointer32be) },
            [0x1234_5678, 0x90AB_CDEF]
        );
        assert_eq!(
            unsafe { read_pointer_lane_values_u64::<2>(pointer64_bytes.as_ptr(), PointerScanPointerSize::Pointer64be) },
            [0x1234_5678_9ABC_DEF0, 0x1111_2222_3333_4444]
        );
    }
}
