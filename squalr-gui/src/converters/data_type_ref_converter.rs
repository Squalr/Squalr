use crate::DataTypeRefViewData;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use slint_mvvm::{convert_from_view_data::ConvertFromViewData, convert_to_view_data::ConvertToViewData};

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

        DataTypeRefViewData {
            data_type_id: data_type_id.into(),
        }
    }
}

impl ConvertFromViewData<DataTypeRef, DataTypeRefViewData> for DataTypeRefConverter {
    fn convert_from_view_data(
        &self,
        data_type_ref: &DataTypeRefViewData,
    ) -> DataTypeRef {
        DataTypeRef::new(&data_type_ref.data_type_id)
    }
}
