#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ValidatedPointerCandidate {
    pub(crate) pointer_address: u64,
    pub(crate) pointer_value: u64,
    pub(crate) module_index: usize,
    pub(crate) module_offset: u64,
}
