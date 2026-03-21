#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DiscoveredPointerCandidate {
    pub(crate) pointer_address: u64,
    pub(crate) pointer_value: u64,
    pub(crate) module_index: usize,
    pub(crate) module_offset: u64,
}
