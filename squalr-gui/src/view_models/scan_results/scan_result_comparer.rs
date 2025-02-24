use crate::ScanResultDataView;
use slint_mvvm::view_data_comparer::ViewDataComparer;

pub struct ScanResultComparer;

impl ScanResultComparer {
    pub fn new() -> Self {
        Self {}
    }
}

impl ViewDataComparer<ScanResultDataView> for ScanResultComparer {
    fn compare(
        &self,
        a: &ScanResultDataView,
        b: &ScanResultDataView,
    ) -> bool {
        a.address == b.address
    }
}
