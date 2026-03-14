#[derive(Copy, Clone, PartialEq)]
pub enum ElementScannerViewState {
    NoResults,
    ScanInProgress,
    HasResults,
}

impl ElementScannerViewState {
    pub fn has_active_scan(self) -> bool {
        !matches!(self, Self::NoResults)
    }
}

#[cfg(test)]
mod tests {
    use super::ElementScannerViewState;

    #[test]
    fn no_results_is_not_an_active_scan() {
        assert!(!ElementScannerViewState::NoResults.has_active_scan());
    }

    #[test]
    fn scan_in_progress_is_an_active_scan() {
        assert!(ElementScannerViewState::ScanInProgress.has_active_scan());
    }

    #[test]
    fn completed_scan_is_an_active_scan() {
        assert!(ElementScannerViewState::HasResults.has_active_scan());
    }
}
