use crate::view_data_comparer::ViewDataComparer;
use slint::SharedString;

pub struct SharedStringComparer;

/// Compares two instances of WordBookmarkViewData for equality.
impl ViewDataComparer<SharedString> for SharedStringComparer {
    fn compare(
        a: &SharedString,
        b: &SharedString,
    ) -> bool {
        return a == b;
    }
}
