use crate::ProcessViewData;
use slint_mvvm::view_data_comparer::ViewDataComparer;

pub struct ProcessInfoComparer;

impl ProcessInfoComparer {
    pub fn new() -> Self {
        Self {}
    }
}

impl ViewDataComparer<ProcessViewData> for ProcessInfoComparer {
    fn compare(
        &self,
        a: &ProcessViewData,
        b: &ProcessViewData,
    ) -> bool {
        a.process_id == b.process_id
    }
}
