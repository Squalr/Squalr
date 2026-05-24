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
