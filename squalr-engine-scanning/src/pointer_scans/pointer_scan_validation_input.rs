use squalr_engine_api::structures::pointer_scans::pointer_scan_results::PointerScanResults;
use squalr_engine_api::structures::pointer_scans::pointer_scan_summary::PointerScanSummary;

/// The prior root identities required to validate against a new snapshot.
pub struct PointerScanValidationInput {
    pub summary: PointerScanSummary,
    pub(crate) module_names: Vec<String>,
    pub(crate) roots_by_level: Vec<Vec<(usize, u64)>>,
}

impl From<&PointerScanResults> for PointerScanValidationInput {
    fn from(results: &PointerScanResults) -> Self {
        Self {
            summary: results.summarize(),
            module_names: results.get_module_names().clone(),
            roots_by_level: results
                .get_pointer_scan_level_candidates()
                .iter()
                .map(|level| {
                    level
                        .get_static_candidates()
                        .iter()
                        .map(|root| (root.get_module_index(), root.get_module_offset()))
                        .collect()
                })
                .collect(),
        }
    }
}
