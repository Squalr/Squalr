pub struct BoyerMooreTable {
    mismatch_shift_table: Vec<u64>,
    matching_suffix_shift_table: Vec<u64>,
    pattern_length: u64,
    aligned_pattern_length: u64,
}

impl BoyerMooreTable {
    pub fn new(
        scan_pattern: &[u8],
        memory_alignment: u64,
    ) -> Self {
        let pattern_length = scan_pattern.len();
        let aligned_pattern_length = Self::round_up_to_alignment(pattern_length as u64, memory_alignment);

        let mut table = Self {
            mismatch_shift_table: vec![aligned_pattern_length; u8::MAX as usize + 1usize],
            matching_suffix_shift_table: vec![0u64; pattern_length],
            pattern_length: pattern_length as u64,
            aligned_pattern_length,
        };

        table.build_table(scan_pattern, memory_alignment);

        table
    }

    pub fn get_mismatch_shift(
        &self,
        value: u8,
    ) -> u64 {
        self.mismatch_shift_table[value as usize]
    }

    pub fn get_good_suffix_shift(
        &self,
        pattern_index: usize,
    ) -> u64 {
        if pattern_index + 1 < self.pattern_length as usize {
            self.matching_suffix_shift_table[pattern_index]
        } else {
            0
        }
    }

    pub fn get_aligned_pattern_length(&self) -> u64 {
        self.aligned_pattern_length
    }

    fn build_table(
        &mut self,
        scan_pattern: &[u8],
        memory_alignment: u64,
    ) {
        let pattern_length = scan_pattern.len();
        let pattern_length_minus_one = pattern_length.saturating_sub(1);

        // Build the Mismatch (Bad Character Rule) shift table.
        // This dictates how far we shift our comparison window if a byte match fails.
        {
            // Build the table from right to left.
            for index in (0..pattern_length).rev() {
                let byte_value = scan_pattern[index];
                let shift_value = pattern_length_minus_one.saturating_sub(index).max(1);
                let aligned_shift = Self::round_up_to_alignment(shift_value as u64, memory_alignment);

                // Only set if not already set (this ensures rightmost occurrence is used).
                if self.mismatch_shift_table[byte_value as usize] == self.aligned_pattern_length {
                    self.mismatch_shift_table[byte_value as usize] = aligned_shift;
                }
            }
        }

        // Build the Matching (good) Suffix Rule shift table. This is an optimization used to more optimally shift when there are partial matches.
        {
            let default_good_suffix_shift = Self::round_up_to_alignment(pattern_length as u64, memory_alignment);

            for pattern_index in 0..pattern_length {
                self.matching_suffix_shift_table[pattern_index] = default_good_suffix_shift;
            }

            // First pass: If the suffix from 'start_index' is also a prefix, shift = pattern_length - start_index.
            for start_index in (0..pattern_length).rev() {
                if Self::is_prefix(scan_pattern, start_index, pattern_length) {
                    let raw_shift = (pattern_length.saturating_sub(start_index)) as u64;
                    let aligned_shift = Self::round_up_to_alignment(raw_shift, memory_alignment);

                    self.matching_suffix_shift_table[start_index] = self.matching_suffix_shift_table[start_index].min(aligned_shift);
                }
            }

            // Second pass: calculate shifts based on actual suffix matches.
            for pattern_index in 0..pattern_length_minus_one {
                let matching_suffix_len = Self::suffix_length(scan_pattern, pattern_index, pattern_length);
                let shift_table_index = pattern_length_minus_one.saturating_sub(matching_suffix_len);

                // Option A: shift = entire pattern length minus the matched suffix.
                let option_a = (pattern_length as u64).saturating_sub(matching_suffix_len as u64);

                // Option B: shift = (pattern_length_minus_one - pattern_index) + matching_suffix_len.
                let option_b = (pattern_length_minus_one.saturating_sub(pattern_index) + matching_suffix_len) as u64;

                // Take whichever shift is smaller.
                let shift = option_a.min(option_b);
                let aligned_shift = Self::round_up_to_alignment(shift, memory_alignment);
                self.matching_suffix_shift_table[shift_table_index] = self.matching_suffix_shift_table[shift_table_index].min(aligned_shift);
            }
        }
    }

    fn is_prefix(
        array: &[u8],
        suffix_start: usize,
        pattern_length: usize,
    ) -> bool {
        let suffix_len = pattern_length.saturating_sub(suffix_start);
        for index in 0..suffix_len {
            if array[index] != array[suffix_start + index] {
                return false;
            }
        }
        true
    }

    fn suffix_length(
        array: &[u8],
        match_pos: usize,
        pattern_length: usize,
    ) -> usize {
        let mut length = 0usize;
        let mut suffix_index = match_pos;
        let mut pattern_end_index = pattern_length.saturating_sub(1);

        while suffix_index < pattern_length && pattern_end_index < pattern_length && array[suffix_index] == array[pattern_end_index] {
            length = length.saturating_add(1);

            if suffix_index == 0 || pattern_end_index == 0 {
                break;
            }

            suffix_index = suffix_index.saturating_sub(1);
            pattern_end_index = pattern_end_index.saturating_sub(1);
        }

        length
    }

    fn round_up_to_alignment(
        value: u64,
        alignment: u64,
    ) -> u64 {
        debug_assert!(alignment > 0);

        let remainder = value % alignment;
        value + (alignment - remainder) % alignment
    }
}
