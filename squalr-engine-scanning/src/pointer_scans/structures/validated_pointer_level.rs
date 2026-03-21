use crate::pointer_scans::structures::validated_pointer_candidate::ValidatedPointerCandidate;

#[derive(Clone, Debug, Default)]
pub(crate) struct ValidatedPointerLevel {
    pub(crate) static_candidates: Vec<ValidatedPointerCandidate>,
    pub(crate) heap_candidates: Vec<ValidatedPointerCandidate>,
}
