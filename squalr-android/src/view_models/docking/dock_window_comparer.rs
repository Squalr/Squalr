use crate::DockedWindowViewData;
use slint_mvvm::view_data_comparer::ViewDataComparer;

pub struct DockWindowComparer;

impl DockWindowComparer {
    pub fn new() -> Self {
        Self {}
    }
}

impl ViewDataComparer<DockedWindowViewData> for DockWindowComparer {
    fn compare(
        &self,
        a: &DockedWindowViewData,
        b: &DockedWindowViewData,
    ) -> bool {
        a.identifier == b.identifier
    }
}
