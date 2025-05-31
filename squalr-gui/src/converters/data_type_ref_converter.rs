use crate::DataTypeRefViewData;
use slint_mvvm::convert_to_view_data::ConvertToViewData;
use squalr_engine_api::structures::data_types::{built_in_types::data_type_ref::data_type_data_type_ref::DataTypeRefDataType, data_type_ref::DataTypeRef};

pub struct DataTypeRefConverter {}

impl DataTypeRefConverter {
    pub fn new() -> Self {
        Self {}
    }
}

impl ConvertToViewData<DataTypeRef, DataTypeRefViewData> for DataTypeRefConverter {
    fn convert_collection(
        &self,
        data_type_ref_list: &Vec<DataTypeRef>,
    ) -> Vec<DataTypeRefViewData> {
        data_type_ref_list
            .into_iter()
            .map(|item| self.convert_to_view_data(item))
            .collect()
    }

    fn convert_to_view_data(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> DataTypeRefViewData {
        let data_type_id = data_type_ref.get_data_type_id();
        let mut icon_id = data_type_ref.get_icon_id();

        // If the data type is a data type reference, resolve the data type so that we can display the icon of the referenced type.
        if data_type_id == DataTypeRefDataType::get_data_type_id() {
            icon_id = DataTypeRefDataType::resolve_data_type_reference(data_type_ref.get_meta_data()).get_icon_id();
        }

        DataTypeRefViewData {
            data_type_id: data_type_id.into(),
            icon_id: icon_id.into(),
        }
    }
}
