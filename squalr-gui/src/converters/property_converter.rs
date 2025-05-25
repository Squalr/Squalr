use crate::DataValueView;
use crate::PropertyEntryViewData;
use slint::ModelRc;
use slint_mvvm::convert_to_view_data::ConvertToViewData;
use squalr_engine_api::structures::{
    data_types::{built_in_types::data_type::data_type_data_type_ref::DataTypeRefDataType, data_type_meta_data::DataTypeMetaData, data_type_ref::DataTypeRef},
    properties::property::Property,
};

pub struct PropertyConverter {}

impl PropertyConverter {
    pub fn new() -> Self {
        Self {}
    }
}

impl ConvertToViewData<Property, PropertyEntryViewData> for PropertyConverter {
    fn convert_collection(
        &self,
        property_list: &Vec<Property>,
    ) -> Vec<PropertyEntryViewData> {
        return property_list
            .into_iter()
            .map(|item| self.convert_to_view_data(item))
            .collect();
    }

    fn convert_to_view_data(
        &self,
        property: &Property,
    ) -> PropertyEntryViewData {
        let data_value = property.get_value();
        let data_type = data_value.get_data_type();
        let data_type_id = data_type.get_data_type_id();
        let mut icon_id = data_type.get_icon_id();

        // JIRA: It would be nice not to leak internals -- we're very deliberately reaching into metadata to grab a piece of info.
        // This is not super obvious from an API point of view, but this is how we get the underlying data type that a data type ref points to.
        if data_type_id == DataTypeRefDataType::get_data_type_id() {
            match data_type.get_meta_data() {
                DataTypeMetaData::FixedString(data_type_ref_id) => icon_id = DataTypeRef::new(data_type_ref_id, DataTypeMetaData::None).get_icon_id(),
                _ => {}
            }
        }

        // JIRA: Use a converter for this.
        PropertyEntryViewData {
            name: property.get_name().to_string().into(),
            data_value: DataValueView {
                data_type_ref: crate::DataTypeRefView {
                    data_type_id: data_type_id.into(),
                    icon_id: icon_id.into(),
                },
                display_value: property.get_display_value().into(),
                fixed_choices: ModelRc::default(),
                is_value_hex: false,
            },
            is_read_only: property.get_is_read_only(),
        }
    }
}
