use crate::pointer_scans::structures::rebuilt_pointer_candidate::RebuiltPointerCandidate;

#[derive(Clone, Debug, Default)]
pub(crate) struct RebuiltPointerLevel {
    pub(crate) static_candidates: Vec<RebuiltPointerCandidate>,
    pub(crate) heap_candidates: Vec<RebuiltPointerCandidate>,
}
