use crate::PropertyEntryViewData;
use slint_mvvm::view_data_converter::ViewDataConverter;
use squalr_engine_api::structures::properties::property::Property;

pub struct PropertyConverter {}

impl PropertyConverter {
    pub fn new() -> Self {
        Self {}
    }
}

impl ViewDataConverter<Property, PropertyEntryViewData> for PropertyConverter {
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
        PropertyEntryViewData {
            name: property.get_name().to_string().into(),
            display_value: property.get_value().get_value_string().into(),
            is_read_only: property.get_is_read_only(),
        }
    }

    fn convert_from_view_data(
        &self,
        _: &PropertyEntryViewData,
    ) -> Property {
        panic!("Not implemented!");
    }
}
