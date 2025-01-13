use crate::DockedWindowViewData;
use slint_mvvm::view_data_comparer::ViewDataComparer;

pub struct DockPanelComparer;

impl DockPanelComparer {
    pub fn new() -> Self {
        Self {}
    }
}

impl ViewDataComparer<DockedWindowViewData> for DockPanelComparer {
    fn compare(
        &self,
        a: &DockedWindowViewData,
        b: &DockedWindowViewData,
    ) -> bool {
        a.identifier == b.identifier
    }
}
