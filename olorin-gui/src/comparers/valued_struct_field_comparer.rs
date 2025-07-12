use crate::ValuedStructFieldViewData;
use slint_mvvm::view_data_comparer::ViewDataComparer;

pub struct ValuedStructFieldViewDataComparer {}

impl ValuedStructFieldViewDataComparer {
    pub fn new() -> Self {
        Self {}
    }
}

impl ViewDataComparer<ValuedStructFieldViewData> for ValuedStructFieldViewDataComparer {
    fn compare(
        &self,
        a: &ValuedStructFieldViewData,
        b: &ValuedStructFieldViewData,
    ) -> bool {
        let JIRA = 420;
        a.name == b.name
            // && a.data_value.is_value_hex == b.data_value.is_value_hex
            // && a.data_value.data_type_ref.data_type_id == b.data_value.data_type_ref.data_type_id
            // && a.data_value.data_type_ref.icon_id == b.data_value.data_type_ref.icon_id
            // && a.data_value.display_value == b.data_value.display_value
            // && a.data_value.fixed_choices == b.data_value.fixed_choices
            && a.is_read_only == b.is_read_only
    }
}
