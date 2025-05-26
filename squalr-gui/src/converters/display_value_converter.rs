use crate::DisplayValueViewData;
use crate::converters::display_value_type_converter::DisplayValueTypeConverter;
use slint_mvvm::convert_to_view_data::ConvertToViewData;
use squalr_engine_api::structures::data_values::display_value::DisplayValue;

pub struct DisplayValueConverter {}

impl DisplayValueConverter {
    pub fn new() -> Self {
        Self {}
    }
}

impl ConvertToViewData<DisplayValue, DisplayValueViewData> for DisplayValueConverter {
    fn convert_collection(
        &self,
        display_value_list: &Vec<DisplayValue>,
    ) -> Vec<DisplayValueViewData> {
        display_value_list
            .into_iter()
            .map(|item| self.convert_to_view_data(item))
            .collect()
    }

    fn convert_to_view_data(
        &self,
        display_value: &DisplayValue,
    ) -> DisplayValueViewData {
        DisplayValueViewData {
            display_value: "TODO".into(),
            display_value_type: DisplayValueTypeConverter {}.convert_to_view_data(display_value.get_display_value_type()),
        }
    }
}
