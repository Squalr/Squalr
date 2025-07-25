use crate::DisplayValueViewData;
use crate::converters::container_type_converter::ContainerTypeConverter;
use crate::converters::display_value_type_converter::DisplayValueTypeConverter;
use olorin_engine_api::structures::data_values::display_value::DisplayValue;
use slint_mvvm::convert_from_view_data::ConvertFromViewData;
use slint_mvvm::convert_to_view_data::ConvertToViewData;

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
            display_value: display_value.get_display_value().into(),
            display_value_type: DisplayValueTypeConverter {}.convert_to_view_data(display_value.get_display_value_type()),
            container_type: ContainerTypeConverter {}.convert_to_view_data(display_value.get_container_type()),
        }
    }
}

impl ConvertFromViewData<DisplayValue, DisplayValueViewData> for DisplayValueConverter {
    fn convert_from_view_data(
        &self,
        display_value_view: &DisplayValueViewData,
    ) -> DisplayValue {
        DisplayValue::new(
            display_value_view.display_value.to_string(),
            DisplayValueTypeConverter {}.convert_from_view_data(&display_value_view.display_value_type),
            ContainerTypeConverter {}.convert_from_view_data(&display_value_view.container_type),
        )
    }
}
