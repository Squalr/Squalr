use crate::pointer_scans::structures::pointer_scan_collected_candidate::PointerScanCollectedCandidate;

#[derive(Clone, Debug, Default)]
pub(crate) struct PointerScanCollectedLevel {
    pub(crate) static_candidates: Vec<PointerScanCollectedCandidate>,
    pub(crate) heap_candidates: Vec<PointerScanCollectedCandidate>,
}
