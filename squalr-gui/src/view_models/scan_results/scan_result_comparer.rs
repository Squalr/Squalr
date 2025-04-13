use crate::ScanResultViewData;
use slint_mvvm::view_data_comparer::ViewDataComparer;

pub struct ScanResultComparer {}

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
            && a.current_value == b.current_value
            && a.previous_value == b.previous_value
            && a.data_type == b.data_type
            && a.is_frozen == b.is_frozen
    }
}
