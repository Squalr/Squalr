#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ValidatedPointerCandidateKey {
    pub(crate) level_index: usize,
    pub(crate) candidate_id: u64,
    pub(crate) current_pointer_address: u64,
}

impl ValidatedPointerCandidateKey {
    pub(crate) fn new(
        level_index: usize,
        candidate_id: u64,
        current_pointer_address: u64,
    ) -> Self {
        Self {
            level_index,
            candidate_id,
            current_pointer_address,
        }
    }
}
