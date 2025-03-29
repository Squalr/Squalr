pub struct BoyerMooreTable {
    mismatch_shift_table: Vec<u64>,
    matching_suffix_shift_table: Vec<u64>,
}

impl BoyerMooreTable {
    pub fn new(scan_pattern: &[u8]) -> Self {
        let mut table = Self {
            mismatch_shift_table: vec![0u64; u8::MAX as usize + 1usize],
            matching_suffix_shift_table: vec![0u64; scan_pattern.len()],
        };

        table.build_table(scan_pattern);

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
        self.matching_suffix_shift_table[pattern_index]
    }

    fn build_table(
        &mut self,
        scan_pattern: &[u8],
    ) {
        let pattern_length = scan_pattern.len();

        // Build the Mismatch (Bad Character Rule) shift table.
        // This dictates how far we shift our comparison window if a byte match fails.
        // Populated as mismatch_shift_table[byte_value] => length_of_array - byte_index - 1.
        for index in 0..pattern_length {
            let byte_value = scan_pattern[index];
            let shift_value = pattern_length - index - 1;

            // JIRA: When we support masking, skip adding any elements that have a corresponding mask entry.
            self.mismatch_shift_table[byte_value as usize] = shift_value as u64;
        }

        // Build the Matching (good) Suffix Rule shift table.
        // This is an optimization used to more optimally shift when there are partial matches.
        {
            // First pass: identify positions where a suffix of the pattern is also a prefix.
            let mut longest_prefix_suffix_len = 0;

            for pattern_index in (0..pattern_length).rev() {
                // Check if the pattern from this index onward is a prefix of the full pattern.
                let is_suffix_prefix = Self::is_prefix(&scan_pattern, pattern_index, pattern_length);

                if is_suffix_prefix {
                    longest_prefix_suffix_len = pattern_length - 1 - pattern_index;
                }

                // Calculate the shift based on the suffix-prefix match.
                let shift_for_position = longest_prefix_suffix_len + (pattern_length - 1 - pattern_index);
                self.matching_suffix_shift_table[pattern_index] = shift_for_position as u64;
            }

            // Second pass: calculate shifts based on actual suffix matches.
            for pattern_index in 0..pattern_length - 1 {
                let matching_suffix_len = Self::suffix_length(&scan_pattern, pattern_index, pattern_length);

                // Avoid index overflow by clamping to valid range.
                let shift_table_index = matching_suffix_len.min(pattern_length - 1);

                // This shift helps when there's a partial suffix match.
                let shift = (pattern_length - 1 - pattern_index + matching_suffix_len) as u64;

                self.matching_suffix_shift_table[shift_table_index] = shift;
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
        let mut length = 0;
        let mut suffix_index = match_pos as isize;
        let mut pattern_end_index = pattern_length as isize - 1;

        while suffix_index >= 0 && array[suffix_index as usize] == array[pattern_end_index as usize] {
            length += 1;
            suffix_index -= 1;
            pattern_end_index -= 1;
        }

        length
    }
}
