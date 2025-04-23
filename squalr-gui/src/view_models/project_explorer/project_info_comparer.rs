use crate::ProjectViewData;
use slint_mvvm::view_data_comparer::ViewDataComparer;

pub struct ProjectInfoComparer {}

impl ProjectInfoComparer {
    pub fn new() -> Self {
        Self {}
    }
}

impl ViewDataComparer<ProjectViewData> for ProjectInfoComparer {
    fn compare(
        &self,
        a: &ProjectViewData,
        b: &ProjectViewData,
    ) -> bool {
        a.name == b.name && a.path == b.path
    }
}
