use crate::pointer_scans::pointer_scan_target_ranges::PointerScanTargetRangeSet;
pub use crate::pointer_scans::structures::pointer_scan_region_match::PointerScanRegionMatch;
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use std::mem::size_of;
use std::ptr;
use std::simd::Simd;
use std::simd::cmp::SimdPartialOrd;

const SIMD_LINEAR_RANGE_THRESHOLD: usize = 8;
const SCALAR_LINEAR_RANGE_THRESHOLD: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PointerScanRangeSearchKernelKind {
    ScalarLinear,
    ScalarBinary,
    SimdLinear,
}

pub struct PointerScanRangeSearchKernel<'a> {
    target_range_set: &'a PointerScanTargetRangeSet,
    pointer_size: PointerScanPointerSize,
    kernel_kind: PointerScanRangeSearchKernelKind,
}

impl<'a> PointerScanRangeSearchKernel<'a> {
    pub fn new(
        target_range_set: &'a PointerScanTargetRangeSet,
        pointer_size: PointerScanPointerSize,
    ) -> Self {
        let kernel_kind = if target_range_set.get_range_count() <= SIMD_LINEAR_RANGE_THRESHOLD {
            PointerScanRangeSearchKernelKind::SimdLinear
        } else if target_range_set.get_range_count() <= SCALAR_LINEAR_RANGE_THRESHOLD {
            PointerScanRangeSearchKernelKind::ScalarLinear
        } else {
            PointerScanRangeSearchKernelKind::ScalarBinary
        };

        Self {
            target_range_set,
            pointer_size,
            kernel_kind,
        }
    }

    pub fn get_name(&self) -> &'static str {
        match self.kernel_kind {
            PointerScanRangeSearchKernelKind::ScalarLinear => "Scalar Linear",
            PointerScanRangeSearchKernelKind::ScalarBinary => "Scalar Binary",
            PointerScanRangeSearchKernelKind::SimdLinear => "SIMD Linear",
        }
    }

    pub fn get_pointer_size(&self) -> PointerScanPointerSize {
        self.pointer_size
    }

    pub fn is_empty(&self) -> bool {
        self.target_range_set.is_empty()
    }

    pub fn contains_pointer_value(
        &self,
        pointer_value: u64,
    ) -> bool {
        match self.kernel_kind {
            PointerScanRangeSearchKernelKind::ScalarBinary => self.target_range_set.contains_value_binary(pointer_value),
            PointerScanRangeSearchKernelKind::ScalarLinear | PointerScanRangeSearchKernelKind::SimdLinear => {
                self.target_range_set.contains_value_linear(pointer_value)
            }
        }
    }

    pub fn scan_region(
        &self,
        base_address: u64,
        current_values: &[u8],
    ) -> Vec<PointerScanRegionMatch> {
        let mut pointer_matches = Vec::new();
        self.scan_region_with_visitor(base_address, current_values, |pointer_match| pointer_matches.push(pointer_match));

        pointer_matches
    }

    pub fn scan_region_with_visitor<VisitMatch>(
        &self,
        base_address: u64,
        current_values: &[u8],
        mut visit_match: VisitMatch,
    ) where
        VisitMatch: FnMut(PointerScanRegionMatch),
    {
        let pointer_size_in_bytes = self.pointer_size.get_size_in_bytes() as usize;

        if current_values.len() < pointer_size_in_bytes {
            return;
        }

        let pointer_alignment = pointer_size_in_bytes as u64;
        let alignment_remainder = base_address % pointer_alignment;
        let start_offset = if alignment_remainder == 0 {
            0_usize
        } else {
            pointer_alignment.saturating_sub(alignment_remainder) as usize
        };

        if start_offset.saturating_add(pointer_size_in_bytes) > current_values.len() {
            return;
        }

        match self.kernel_kind {
            PointerScanRangeSearchKernelKind::ScalarLinear => self.scan_region_scalar(
                base_address,
                current_values,
                start_offset,
                |pointer_value| self.target_range_set.contains_value_linear(pointer_value),
                &mut visit_match,
            ),
            PointerScanRangeSearchKernelKind::ScalarBinary => self.scan_region_scalar(
                base_address,
                current_values,
                start_offset,
                |pointer_value| self.target_range_set.contains_value_binary(pointer_value),
                &mut visit_match,
            ),
            PointerScanRangeSearchKernelKind::SimdLinear => self.scan_region_simd_linear(base_address, current_values, start_offset, &mut visit_match),
        }
    }

    fn scan_region_scalar<MatchesPointerValue, VisitMatch>(
        &self,
        base_address: u64,
        current_values: &[u8],
        start_offset: usize,
        mut matches_pointer_value: MatchesPointerValue,
        visit_match: &mut VisitMatch,
    ) where
        MatchesPointerValue: FnMut(u64) -> bool,
        VisitMatch: FnMut(PointerScanRegionMatch),
    {
        let pointer_size_in_bytes = self.pointer_size.get_size_in_bytes() as usize;
        let current_values_ptr = current_values.as_ptr();
        let mut pointer_value_offset = start_offset;

        while pointer_value_offset.saturating_add(pointer_size_in_bytes) <= current_values.len() {
            // The loop guard guarantees a full pointer-sized unaligned load.
            let pointer_value = unsafe { Self::read_pointer_value_unchecked(current_values_ptr.add(pointer_value_offset), self.pointer_size) };

            if matches_pointer_value(pointer_value) {
                visit_match(PointerScanRegionMatch::new(
                    base_address.saturating_add(pointer_value_offset as u64),
                    pointer_value,
                ));
            }

            pointer_value_offset = pointer_value_offset.saturating_add(pointer_size_in_bytes);
        }
    }

    fn scan_region_simd_linear<VisitMatch>(
        &self,
        base_address: u64,
        current_values: &[u8],
        start_offset: usize,
        visit_match: &mut VisitMatch,
    ) where
        VisitMatch: FnMut(PointerScanRegionMatch),
    {
        match self.pointer_size {
            PointerScanPointerSize::Pointer32 => self.scan_region_simd_linear_u32(base_address, current_values, start_offset, visit_match),
            PointerScanPointerSize::Pointer64 => self.scan_region_simd_linear_u64(base_address, current_values, start_offset, visit_match),
        }
    }

    fn scan_region_simd_linear_u32<VisitMatch>(
        &self,
        base_address: u64,
        current_values: &[u8],
        start_offset: usize,
        visit_match: &mut VisitMatch,
    ) where
        VisitMatch: FnMut(PointerScanRegionMatch),
    {
        const SIMD_LANE_COUNT: usize = 16;
        let pointer_size_in_bytes = size_of::<u32>();
        let current_values_ptr = current_values.as_ptr();
        let mut pointer_value_offset = start_offset;

        while pointer_value_offset.saturating_add(pointer_size_in_bytes * SIMD_LANE_COUNT) <= current_values.len() {
            // The loop guard guarantees a full SIMD-width unaligned load.
            let lane_values = unsafe { Self::read_pointer_lane_values_u32::<SIMD_LANE_COUNT>(current_values_ptr.add(pointer_value_offset)) };
            let pointer_values = Simd::<u32, SIMD_LANE_COUNT>::from_array(lane_values);
            let mut matching_lane_mask = 0_u64;

            for target_range in self.target_range_set.get_target_ranges() {
                let Ok(lower_bound) = u32::try_from(target_range.get_lower_bound()) else {
                    continue;
                };
                let upper_bound = u32::try_from(target_range.get_upper_bound()).unwrap_or(u32::MAX);
                let lower_bound_vector = Simd::splat(lower_bound);
                let upper_bound_vector = Simd::splat(upper_bound);

                matching_lane_mask |= (pointer_values.simd_ge(lower_bound_vector) & pointer_values.simd_le(upper_bound_vector)).to_bitmask();
            }

            if matching_lane_mask != 0 {
                for lane_index in 0..SIMD_LANE_COUNT {
                    if matching_lane_mask & (1_u64 << lane_index) == 0 {
                        continue;
                    }

                    visit_match(PointerScanRegionMatch::new(
                        base_address.saturating_add((pointer_value_offset + lane_index * pointer_size_in_bytes) as u64),
                        lane_values[lane_index] as u64,
                    ));
                }
            }

            pointer_value_offset = pointer_value_offset.saturating_add(pointer_size_in_bytes * SIMD_LANE_COUNT);
        }

        self.scan_region_scalar(
            base_address,
            current_values,
            pointer_value_offset,
            |pointer_value| self.target_range_set.contains_value_linear(pointer_value),
            visit_match,
        );
    }

    fn scan_region_simd_linear_u64<VisitMatch>(
        &self,
        base_address: u64,
        current_values: &[u8],
        start_offset: usize,
        visit_match: &mut VisitMatch,
    ) where
        VisitMatch: FnMut(PointerScanRegionMatch),
    {
        const SIMD_LANE_COUNT: usize = 8;
        let pointer_size_in_bytes = size_of::<u64>();
        let current_values_ptr = current_values.as_ptr();
        let mut pointer_value_offset = start_offset;

        while pointer_value_offset.saturating_add(pointer_size_in_bytes * SIMD_LANE_COUNT) <= current_values.len() {
            // The loop guard guarantees a full SIMD-width unaligned load.
            let lane_values = unsafe { Self::read_pointer_lane_values_u64::<SIMD_LANE_COUNT>(current_values_ptr.add(pointer_value_offset)) };
            let pointer_values = Simd::<u64, SIMD_LANE_COUNT>::from_array(lane_values);
            let mut matching_lane_mask = 0_u64;

            for target_range in self.target_range_set.get_target_ranges() {
                let lower_bound_vector = Simd::splat(target_range.get_lower_bound());
                let upper_bound_vector = Simd::splat(target_range.get_upper_bound());

                matching_lane_mask |= (pointer_values.simd_ge(lower_bound_vector) & pointer_values.simd_le(upper_bound_vector)).to_bitmask();
            }

            if matching_lane_mask != 0 {
                for lane_index in 0..SIMD_LANE_COUNT {
                    if matching_lane_mask & (1_u64 << lane_index) == 0 {
                        continue;
                    }

                    visit_match(PointerScanRegionMatch::new(
                        base_address.saturating_add((pointer_value_offset + lane_index * pointer_size_in_bytes) as u64),
                        lane_values[lane_index],
                    ));
                }
            }

            pointer_value_offset = pointer_value_offset.saturating_add(pointer_size_in_bytes * SIMD_LANE_COUNT);
        }

        self.scan_region_scalar(
            base_address,
            current_values,
            pointer_value_offset,
            |pointer_value| self.target_range_set.contains_value_linear(pointer_value),
            visit_match,
        );
    }

    unsafe fn read_pointer_value_unchecked(
        pointer_bytes_ptr: *const u8,
        pointer_size: PointerScanPointerSize,
    ) -> u64 {
        match pointer_size {
            PointerScanPointerSize::Pointer32 => u32::from_le(unsafe { ptr::read_unaligned(pointer_bytes_ptr as *const u32) }) as u64,
            PointerScanPointerSize::Pointer64 => u64::from_le(unsafe { ptr::read_unaligned(pointer_bytes_ptr as *const u64) }),
        }
    }

    unsafe fn read_pointer_lane_values_u32<const SIMD_LANE_COUNT: usize>(pointer_bytes_ptr: *const u8) -> [u32; SIMD_LANE_COUNT] {
        let lane_values = unsafe { ptr::read_unaligned(pointer_bytes_ptr as *const [u32; SIMD_LANE_COUNT]) };

        #[cfg(target_endian = "big")]
        let lane_values = lane_values.map(u32::from_le);

        lane_values
    }

    unsafe fn read_pointer_lane_values_u64<const SIMD_LANE_COUNT: usize>(pointer_bytes_ptr: *const u8) -> [u64; SIMD_LANE_COUNT] {
        let lane_values = unsafe { ptr::read_unaligned(pointer_bytes_ptr as *const [u64; SIMD_LANE_COUNT]) };

        #[cfg(target_endian = "big")]
        let lane_values = lane_values.map(u64::from_le);

        lane_values
    }
}

#[cfg(test)]
mod tests {
    use super::PointerScanRangeSearchKernel;
    use crate::pointer_scans::pointer_scan_target_ranges::PointerScanTargetRangeSet;
    use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;

    #[test]
    fn range_search_kernel_scans_region_against_merged_ranges() {
        let target_range_set = PointerScanTargetRangeSet::from_target_addresses(&[0x3000, 0x3010, 0x4000], 0x20);
        let search_kernel = PointerScanRangeSearchKernel::new(&target_range_set, PointerScanPointerSize::Pointer64);
        let mut current_values = Vec::new();

        current_values.extend_from_slice(&0x2FF0_u64.to_le_bytes());
        current_values.extend_from_slice(&0x3018_u64.to_le_bytes());
        current_values.extend_from_slice(&0x3500_u64.to_le_bytes());
        current_values.extend_from_slice(&0x4010_u64.to_le_bytes());

        let pointer_matches = search_kernel.scan_region(0x1000, &current_values);

        assert_eq!(pointer_matches.len(), 3);
        assert_eq!(pointer_matches[0].get_pointer_address(), 0x1000);
        assert_eq!(pointer_matches[1].get_pointer_address(), 0x1008);
        assert_eq!(pointer_matches[2].get_pointer_address(), 0x1018);
    }
}
