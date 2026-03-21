use crate::pointer_scans::structures::discovered_pointer_candidate::DiscoveredPointerCandidate;

#[derive(Clone, Debug, Default)]
pub(crate) struct DiscoveredPointerLevel {
    pub(crate) static_candidates: Vec<DiscoveredPointerCandidate>,
    pub(crate) heap_candidates: Vec<DiscoveredPointerCandidate>,
}
