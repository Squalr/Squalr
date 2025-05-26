use crate::converters::data_type_ref_converter::DataTypeRefConverter;
use crate::converters::display_value_converter::DisplayValueConverter;
use crate::converters::display_value_type_converter::DisplayValueTypeConverter;
use crate::{DataValueViewData, DisplayValueTypeView};
use slint::{ModelRc, VecModel};
use slint_mvvm::convert_to_view_data::ConvertToViewData;
use squalr_engine_api::structures::data_values::data_value::DataValue;

pub struct DataValueConverter {}

impl DataValueConverter {
    pub fn new() -> Self {
        Self {}
    }
}

impl ConvertToViewData<DataValue, DataValueViewData> for DataValueConverter {
    fn convert_collection(
        &self,
        data_value_list: &Vec<DataValue>,
    ) -> Vec<DataValueViewData> {
        data_value_list
            .into_iter()
            .map(|item| self.convert_to_view_data(item))
            .collect()
    }

    fn convert_to_view_data(
        &self,
        data_value: &DataValue,
    ) -> DataValueViewData {
        let active_display_value_type = match data_value.get_default_display_value() {
            Some(display_value) => DisplayValueTypeConverter {}.convert_to_view_data(display_value.get_display_value_type()),
            None => DisplayValueTypeView::String,
        };

        DataValueViewData {
            data_type_ref: DataTypeRefConverter {}.convert_to_view_data(data_value.get_data_type()),
            display_values: ModelRc::new(VecModel::from(
                DisplayValueConverter {}.convert_collection(data_value.get_display_values().get_display_values()),
            )),
            active_display_value_type,
        }
    }
}
