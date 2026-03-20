use crate::pointer_scans::pointer_scan_target_ranges::PointerScanTargetRangeSet;
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use std::mem::size_of;
use std::simd::Simd;
use std::simd::cmp::SimdPartialOrd;

const SIMD_LINEAR_RANGE_THRESHOLD: usize = 8;
const SCALAR_LINEAR_RANGE_THRESHOLD: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointerScanRegionMatch {
    pointer_address: u64,
    pointer_value: u64,
}

impl PointerScanRegionMatch {
    pub fn new(
        pointer_address: u64,
        pointer_value: u64,
    ) -> Self {
        Self {
            pointer_address,
            pointer_value,
        }
    }

    pub fn get_pointer_address(&self) -> u64 {
        self.pointer_address
    }

    pub fn get_pointer_value(&self) -> u64 {
        self.pointer_value
    }
}

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
        let pointer_size_in_bytes = self.pointer_size.get_size_in_bytes() as usize;

        if current_values.len() < pointer_size_in_bytes {
            return Vec::new();
        }

        let pointer_alignment = pointer_size_in_bytes as u64;
        let alignment_remainder = base_address % pointer_alignment;
        let start_offset = if alignment_remainder == 0 {
            0_usize
        } else {
            pointer_alignment.saturating_sub(alignment_remainder) as usize
        };

        if start_offset.saturating_add(pointer_size_in_bytes) > current_values.len() {
            return Vec::new();
        }

        match self.kernel_kind {
            PointerScanRangeSearchKernelKind::ScalarLinear => self.scan_region_scalar(base_address, current_values, start_offset, |pointer_value| {
                self.target_range_set.contains_value_linear(pointer_value)
            }),
            PointerScanRangeSearchKernelKind::ScalarBinary => self.scan_region_scalar(base_address, current_values, start_offset, |pointer_value| {
                self.target_range_set.contains_value_binary(pointer_value)
            }),
            PointerScanRangeSearchKernelKind::SimdLinear => self.scan_region_simd_linear(base_address, current_values, start_offset),
        }
    }

    fn scan_region_scalar<MatchesPointerValue>(
        &self,
        base_address: u64,
        current_values: &[u8],
        start_offset: usize,
        mut matches_pointer_value: MatchesPointerValue,
    ) -> Vec<PointerScanRegionMatch>
    where
        MatchesPointerValue: FnMut(u64) -> bool,
    {
        let pointer_size_in_bytes = self.pointer_size.get_size_in_bytes() as usize;
        let mut pointer_matches = Vec::new();
        let mut pointer_value_offset = start_offset;

        while pointer_value_offset.saturating_add(pointer_size_in_bytes) <= current_values.len() {
            let value_slice = &current_values[pointer_value_offset..pointer_value_offset + pointer_size_in_bytes];
            let Some(pointer_value) = Self::read_pointer_value(value_slice, self.pointer_size) else {
                pointer_value_offset = pointer_value_offset.saturating_add(pointer_size_in_bytes);
                continue;
            };

            if matches_pointer_value(pointer_value) {
                pointer_matches.push(PointerScanRegionMatch::new(
                    base_address.saturating_add(pointer_value_offset as u64),
                    pointer_value,
                ));
            }

            pointer_value_offset = pointer_value_offset.saturating_add(pointer_size_in_bytes);
        }

        pointer_matches
    }

    fn scan_region_simd_linear(
        &self,
        base_address: u64,
        current_values: &[u8],
        start_offset: usize,
    ) -> Vec<PointerScanRegionMatch> {
        match self.pointer_size {
            PointerScanPointerSize::Pointer32 => self.scan_region_simd_linear_u32(base_address, current_values, start_offset),
            PointerScanPointerSize::Pointer64 => self.scan_region_simd_linear_u64(base_address, current_values, start_offset),
        }
    }

    fn scan_region_simd_linear_u32(
        &self,
        base_address: u64,
        current_values: &[u8],
        start_offset: usize,
    ) -> Vec<PointerScanRegionMatch> {
        const SIMD_LANE_COUNT: usize = 16;
        let pointer_size_in_bytes = size_of::<u32>();
        let mut pointer_matches = Vec::new();
        let mut pointer_value_offset = start_offset;

        while pointer_value_offset.saturating_add(pointer_size_in_bytes * SIMD_LANE_COUNT) <= current_values.len() {
            let mut lane_values = [0_u32; SIMD_LANE_COUNT];

            for lane_index in 0..SIMD_LANE_COUNT {
                let value_offset = pointer_value_offset + lane_index * pointer_size_in_bytes;
                let value_slice: [u8; 4] = current_values[value_offset..value_offset + pointer_size_in_bytes]
                    .try_into()
                    .expect("Expected pointer-sized bytes for SIMD u32 pointer scan.");
                lane_values[lane_index] = u32::from_le_bytes(value_slice);
            }

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

                    pointer_matches.push(PointerScanRegionMatch::new(
                        base_address.saturating_add((pointer_value_offset + lane_index * pointer_size_in_bytes) as u64),
                        lane_values[lane_index] as u64,
                    ));
                }
            }

            pointer_value_offset = pointer_value_offset.saturating_add(pointer_size_in_bytes * SIMD_LANE_COUNT);
        }

        pointer_matches.extend(self.scan_region_scalar(base_address, current_values, pointer_value_offset, |pointer_value| {
            self.target_range_set.contains_value_linear(pointer_value)
        }));

        pointer_matches
    }

    fn scan_region_simd_linear_u64(
        &self,
        base_address: u64,
        current_values: &[u8],
        start_offset: usize,
    ) -> Vec<PointerScanRegionMatch> {
        const SIMD_LANE_COUNT: usize = 8;
        let pointer_size_in_bytes = size_of::<u64>();
        let mut pointer_matches = Vec::new();
        let mut pointer_value_offset = start_offset;

        while pointer_value_offset.saturating_add(pointer_size_in_bytes * SIMD_LANE_COUNT) <= current_values.len() {
            let mut lane_values = [0_u64; SIMD_LANE_COUNT];

            for lane_index in 0..SIMD_LANE_COUNT {
                let value_offset = pointer_value_offset + lane_index * pointer_size_in_bytes;
                let value_slice: [u8; 8] = current_values[value_offset..value_offset + pointer_size_in_bytes]
                    .try_into()
                    .expect("Expected pointer-sized bytes for SIMD u64 pointer scan.");
                lane_values[lane_index] = u64::from_le_bytes(value_slice);
            }

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

                    pointer_matches.push(PointerScanRegionMatch::new(
                        base_address.saturating_add((pointer_value_offset + lane_index * pointer_size_in_bytes) as u64),
                        lane_values[lane_index],
                    ));
                }
            }

            pointer_value_offset = pointer_value_offset.saturating_add(pointer_size_in_bytes * SIMD_LANE_COUNT);
        }

        pointer_matches.extend(self.scan_region_scalar(base_address, current_values, pointer_value_offset, |pointer_value| {
            self.target_range_set.contains_value_linear(pointer_value)
        }));

        pointer_matches
    }

    fn read_pointer_value(
        pointer_bytes: &[u8],
        pointer_size: PointerScanPointerSize,
    ) -> Option<u64> {
        match pointer_size {
            PointerScanPointerSize::Pointer32 => {
                let pointer_bytes: [u8; 4] = pointer_bytes.try_into().ok()?;

                Some(u32::from_le_bytes(pointer_bytes) as u64)
            }
            PointerScanPointerSize::Pointer64 => {
                let pointer_bytes: [u8; 8] = pointer_bytes.try_into().ok()?;

                Some(u64::from_le_bytes(pointer_bytes))
            }
        }
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
