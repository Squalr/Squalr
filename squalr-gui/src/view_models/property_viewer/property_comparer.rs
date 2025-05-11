use crate::PropertyEntryViewData;
use slint_mvvm::view_data_comparer::ViewDataComparer;

pub struct PropertyComparer {}

impl PropertyComparer {
    pub fn new() -> Self {
        Self {}
    }
}

impl ViewDataComparer<PropertyEntryViewData> for PropertyComparer {
    fn compare(
        &self,
        a: &PropertyEntryViewData,
        b: &PropertyEntryViewData,
    ) -> bool {
        a.name == b.name && a.display_value == b.display_value && a.is_read_only == b.is_read_only
    }
}
