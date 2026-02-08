use std::sync::atomic::{AtomicBool, Ordering};

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_memory::memory_reader::MemoryReader;
use squalr_engine_memory::memory_reader::memory_reader_trait::IMemoryReader;

pub trait SnapshotRegionMemoryReader {
    fn read_all_memory(
        &mut self,
        process_info: &OpenedProcessInfo,
    ) -> Result<(), String>;
    fn read_all_memory_chunked(
        &mut self,
        process_info: &OpenedProcessInfo,
    ) -> Result<(), String>;
}

impl SnapshotRegionMemoryReader for SnapshotRegion {
    /// Reads all memory for this snapshot region, updating the current and previous value arrays.
    fn read_all_memory(
        &mut self,
        process_info: &OpenedProcessInfo,
    ) -> Result<(), String> {
        let region_size = self.get_region_size() as usize;

        debug_assert!(region_size > 0);

        // Move current_values to be the previous_values. This is a very efficient way to move these, as instead of
        // discarding the old previous values, we recycle that array for use in the next scan to create new current_values.
        std::mem::swap(&mut self.current_values, &mut self.previous_values);

        // Create current values vector if none exist.
        if self.current_values.is_empty() && region_size > 0 {
            self.current_values = vec![0u8; region_size];
        }

        if self.page_boundaries.is_empty() {
            // If this snapshot is part of a standalone memory page, just read the regions as normal.
            MemoryReader::get_instance().read_bytes(&process_info, self.get_base_address(), &mut self.current_values);
        } else {
            // Otherwise, this snapshot is a merging of two or more OS regions, and special care is taken to separate the read calls.
            // This prevents the case where one page deallocates, causing the read for both to fail.
            // Additionally, we read these chunks of memory in parallel, as they may be quite large due to our merging.
            let mut read_ranges = Vec::with_capacity(self.page_boundaries.len() + 1);
            let mut next_range_start_address = self.get_base_address();
            let mut current_slice = self.current_values.as_mut_slice();

            // Iterate the page boundaries and pull out non-overlapping mutable slices to satisfy the Rust borrow checker.
            for &next_boundary_address in &self.page_boundaries {
                let range_size = next_boundary_address.saturating_sub(next_range_start_address) as usize;
                let (slice, remaining) = current_slice.split_at_mut(range_size);

                debug_assert!(range_size > 0);
                debug_assert!(slice.len() > 0);

                read_ranges.push((next_range_start_address, slice));
                current_slice = remaining;
                next_range_start_address = next_boundary_address;
            }

            // Last slice after final boundary.
            if !current_slice.is_empty() {
                debug_assert!(current_slice.len() > 0);

                read_ranges.push((next_range_start_address, current_slice));
            }

            // And finally parallel read using the obtained non-overlapping mutable slices.
            let read_failures = read_ranges
                .into_par_iter()
                .map(|(address, buffer)| {
                    let success = MemoryReader::get_instance().read_bytes(process_info, address, buffer);

                    if success { None } else { Some(address) }
                })
                .filter_map(|result| result)
                .collect::<Vec<_>>();

            self.page_boundary_tombstones.extend(read_failures);
        }

        Ok(())
    }

    /// Reads all memory for this snapshot region, updating the current and previous value arrays.
    /// Uses a chunked implementation to parallelize read calls over a region.
    fn read_all_memory_chunked(
        &mut self,
        process_info: &OpenedProcessInfo,
    ) -> Result<(), String> {
        const CHUNK_SIZE: usize = 16 * 1024;
        let region_size = self.get_region_size() as usize;
        let base_address = self.get_base_address();

        debug_assert!(region_size > 0);

        // Move current_values to be the previous_values. This is a very efficient way to move these, as instead of
        // discarding the old previous values, we recycle that array for use in the next scan to create new current_values.
        std::mem::swap(&mut self.current_values, &mut self.previous_values);

        // Create current values vector if none exist.
        if self.current_values.is_empty() && region_size > 0 {
            self.current_values = vec![0u8; region_size];
        }

        if self.page_boundaries.is_empty() {
            let has_any_failed = AtomicBool::new(false);

            // If this snapshot is part of a standalone memory page, just read the regions as normal.
            self.current_values
                .chunks_mut(CHUNK_SIZE)
                .enumerate()
                .collect::<Vec<_>>() // force eager collection for par_iter
                .into_par_iter()
                .for_each(|(chunk_index, chunk)| {
                    if !has_any_failed.load(Ordering::Acquire) {
                        let address = base_address + chunk_index as u64 * CHUNK_SIZE as u64;
                        let success = MemoryReader::get_instance().read_bytes(process_info, address, chunk);

                        if !success {
                            has_any_failed.store(true, Ordering::Release);
                        }
                    }
                });
        } else {
            // Otherwise, this snapshot is a merging of two or more OS regions, and special care is taken to separate the read calls.
            // This prevents the case where one page deallocates, causing the read for both to fail.
            // Additionally, we read these chunks of memory in parallel, as they may be quite large due to our merging.
            let mut read_ranges = vec![];
            let mut current_slice = self.current_values.as_mut_slice();
            let mut next_address = base_address;

            // Iterate the page boundaries and pull out non-overlapping mutable slices to satisfy the Rust borrow checker.
            for &boundary in &self.page_boundaries {
                let range_size = boundary.saturating_sub(next_address) as usize;
                let (slice, remaining) = current_slice.split_at_mut(range_size);

                slice
                    .chunks_mut(CHUNK_SIZE)
                    .enumerate()
                    .for_each(|(index, chunk)| {
                        let offset = index as u64 * CHUNK_SIZE as u64;
                        read_ranges.push((next_address.saturating_add(offset), chunk));
                    });

                current_slice = remaining;
                next_address = boundary;
            }

            // Final segment after last boundary.
            current_slice
                .chunks_mut(CHUNK_SIZE)
                .enumerate()
                .for_each(|(chunk_index, chunk)| {
                    let offset = chunk_index as u64 * CHUNK_SIZE as u64;
                    read_ranges.push((next_address.saturating_add(offset), chunk));
                });

            let has_failed = AtomicBool::new(false);

            // And finally parallel read using the obtained non-overlapping mutable slices.
            let read_failures = read_ranges
                .into_par_iter()
                .filter_map(|(address, chunk)| {
                    if has_failed.load(Ordering::Acquire) {
                        return None;
                    }

                    let success = MemoryReader::get_instance().read_bytes(process_info, address, chunk);

                    if !success {
                        has_failed.store(true, Ordering::Release);
                    }

                    if success { None } else { Some(address) }
                })
                .collect::<Vec<_>>();

            self.page_boundary_tombstones.extend(read_failures);
        }

        Ok(())
    }
}
