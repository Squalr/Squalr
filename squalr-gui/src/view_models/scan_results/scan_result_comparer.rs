use crate::ScanResultViewData;
use slint_mvvm::view_data_comparer::ViewDataComparer;

pub struct ScanResultComparer;

impl ScanResultComparer {
    pub fn new() -> Self {
        Self {}
    }
}

impl ViewDataComparer<ScanResultViewData> for ScanResultComparer {
    fn compare(
        &self,
        a: &ScanResultViewData,
        b: &ScanResultViewData,
    ) -> bool {
        a.address == b.address
    }
}
