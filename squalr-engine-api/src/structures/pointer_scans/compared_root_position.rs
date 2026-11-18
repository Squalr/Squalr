/// Index into the primary graph and exclusive end of this root's display rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ComparedRootPosition {
    pub level_index: usize,
    pub candidate_index: usize,
    pub display_end: u64,
}
