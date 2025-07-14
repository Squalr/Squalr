use crate::{
    ValuedStructFieldViewData,
    converters::{data_value_converter::DataValueConverter, display_value_converter::DisplayValueConverter},
};
use olorin_engine_api::structures::structs::valued_struct_field::ValuedStructField;
use slint_mvvm::convert_from_view_data::ConvertFromViewData;
use slint_mvvm::convert_to_view_data::ConvertToViewData;

pub struct ValuedStructFieldConverter {}

impl ValuedStructFieldConverter {
    pub fn new() -> Self {
        Self {}
    }
}

impl ConvertToViewData<ValuedStructField, ValuedStructFieldViewData> for ValuedStructFieldConverter {
    fn convert_collection(
        &self,
        valued_struct_field_list: &Vec<ValuedStructField>,
    ) -> Vec<ValuedStructFieldViewData> {
        valued_struct_field_list
            .into_iter()
            .map(|item| self.convert_to_view_data(item))
            .collect()
    }

    fn convert_to_view_data(
        &self,
        valued_struct_field: &ValuedStructField,
    ) -> ValuedStructFieldViewData {
        let data_value = DataValueConverter {}.convert_to_view_data(
            &valued_struct_field
                .get_data_value()
                .cloned()
                .unwrap_or_default(),
        );
        let display_value = DisplayValueConverter {}.convert_to_view_data(
            &valued_struct_field
                .get_display_value()
                .cloned()
                .unwrap_or_default(),
        );

        ValuedStructFieldViewData {
            name: valued_struct_field.get_name().to_string().into(),
            data_value: data_value,
            display_value: display_value,
            namespaced_name: valued_struct_field.get_name().to_string().into(),
            is_read_only: valued_struct_field.get_is_read_only(),
        }
    }
}
