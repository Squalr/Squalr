use crate::PropertyEntryViewData;
use crate::converters::data_value_converter::DataValueConverter;
use slint_mvvm::convert_to_view_data::ConvertToViewData;
use squalr_engine_api::structures::properties::property::Property;

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
        property_list
            .into_iter()
            .map(|item| self.convert_to_view_data(item))
            .collect()
    }

    fn convert_to_view_data(
        &self,
        property: &Property,
    ) -> PropertyEntryViewData {
        let data_value = property.get_value();

        PropertyEntryViewData {
            name: property.get_name().to_string().into(),
            data_value: DataValueConverter {}.convert_to_view_data(data_value),
            is_read_only: property.get_is_read_only(),
        }
    }
}
