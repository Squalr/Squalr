use crate::DataTypeRefViewData;
use slint_mvvm::convert_to_view_data::ConvertToViewData;
use squalr_engine_api::structures::data_types::{
    built_in_types::data_type::data_type_data_type_ref::DataTypeRefDataType, data_type_meta_data::DataTypeMetaData, data_type_ref::DataTypeRef,
};

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

        // JIRA: It would be nice not to leak internals -- we're very deliberately reaching into metadata to grab a piece of info.
        // This is not super obvious from an API point of view, but this is how we get the underlying data type that a data type ref points to.
        if data_type_id == DataTypeRefDataType::get_data_type_id() {
            match data_type_ref.get_meta_data() {
                DataTypeMetaData::FixedString(data_type_ref_id) => icon_id = DataTypeRef::new(data_type_ref_id, DataTypeMetaData::None).get_icon_id(),
                _ => {}
            }
        }

        DataTypeRefViewData {
            data_type_id: data_type_id.into(),
            icon_id: icon_id.into(),
        }
    }
}
