use crate::view_data_comparer::ViewDataComparer;
use slint::SharedString;

pub struct SharedStringComparer;

impl SharedStringComparer {
    pub fn new() -> Self {
        Self {}
    }
}

/// Compares two instances of WordBookmarkViewData for equality.
impl ViewDataComparer<SharedString> for SharedStringComparer {
    fn compare(
        &self,
        a: &SharedString,
        b: &SharedString,
    ) -> bool {
        return a == b;
    }
}
