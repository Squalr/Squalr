use crate::DisplayValuesViewData;
use crate::converters::display_value_converter::DisplayValueConverter;
use olorin_engine_api::structures::data_values::display_value_type::DisplayValueType;
use olorin_engine_api::structures::data_values::display_values::DisplayValues;
use slint::ModelRc;
use slint::VecModel;
use slint_mvvm::convert_from_view_data::ConvertFromViewData;
use slint_mvvm::convert_to_view_data::ConvertToViewData;

pub struct DisplayValuesConverter {}

impl DisplayValuesConverter {
    pub fn new() -> Self {
        Self {}
    }
}

impl ConvertToViewData<DisplayValues, DisplayValuesViewData> for DisplayValuesConverter {
    fn convert_collection(
        &self,
        display_values_list: &Vec<DisplayValues>,
    ) -> Vec<DisplayValuesViewData> {
        display_values_list
            .into_iter()
            .map(|item| self.convert_to_view_data(item))
            .collect()
    }

    fn convert_to_view_data(
        &self,
        display_values: &DisplayValues,
    ) -> DisplayValuesViewData {
        let mut active_display_value_index = 0i32;
        let active_display_values = display_values.get_display_values();

        if let Some(default_display_value) = display_values.get_default_display_value() {
            let default_display_value_type = default_display_value.get_display_value_type();

            for index in 0..active_display_values.len() {
                if active_display_values[index].get_display_value_type() == default_display_value_type {
                    active_display_value_index = index as i32;
                    break;
                }
            }
        }

        let display_values = ModelRc::new(VecModel::from(DisplayValueConverter {}.convert_collection(display_values.get_display_values())));

        DisplayValuesViewData {
            active_display_values: display_values,
            active_display_value_index,
        }
    }
}

impl ConvertFromViewData<DisplayValues, DisplayValuesViewData> for DisplayValuesConverter {
    fn convert_from_view_data(
        &self,
        display_values_view: &DisplayValuesViewData,
    ) -> DisplayValues {
        let display_values = vec![];
        DisplayValues::new(display_values, DisplayValueType::String)
    }
}
