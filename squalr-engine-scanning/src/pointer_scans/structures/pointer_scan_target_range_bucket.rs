#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PointerScanTargetRangeBucket {
    bucket_key: u64,
    start_range_index: usize,
    end_range_index_exclusive: usize,
}

impl PointerScanTargetRangeBucket {
    pub(crate) fn new(
        bucket_key: u64,
        start_range_index: usize,
        end_range_index_exclusive: usize,
    ) -> Self {
        Self {
            bucket_key,
            start_range_index,
            end_range_index_exclusive,
        }
    }

    pub(crate) fn get_bucket_key(&self) -> u64 {
        self.bucket_key
    }

    pub(crate) fn get_start_range_index(&self) -> usize {
        self.start_range_index
    }

    pub(crate) fn get_end_range_index_exclusive(&self) -> usize {
        self.end_range_index_exclusive
    }

    pub(crate) fn set_end_range_index_exclusive(
        &mut self,
        end_range_index_exclusive: usize,
    ) {
        self.end_range_index_exclusive = end_range_index_exclusive;
    }
}
