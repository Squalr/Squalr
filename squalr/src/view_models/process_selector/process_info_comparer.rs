use crate::ProcessViewData;
use slint_mvvm::view_data_comparer::ViewDataComparer;

pub struct ProcessInfoComparer;

impl ViewDataComparer<ProcessViewData> for ProcessInfoComparer {
    fn compare(
        a: &ProcessViewData,
        b: &ProcessViewData,
    ) -> bool {
        a.process_id == b.process_id
    }
}
