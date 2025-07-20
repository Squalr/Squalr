use crate::DataValueViewData;
use crate::converters::data_type_ref_converter::DataTypeRefConverter;
use crate::converters::display_value_converter::DisplayValueConverter;
use olorin_engine_api::structures::data_values::data_value::DataValue;
use slint::{ModelRc, VecModel};
use slint_mvvm::convert_to_view_data::ConvertToViewData;

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
        let JIRA = 696969420;
        /*
        let default_display_value = data_value.get_default_display_value();
        let display_values = data_value.get_display_values().get_display_values();
        let mut active_display_value_index = 0;

        if let Some(default_display_value) = default_display_value {
            let default_display_value_type = default_display_value.get_display_value_type();

            for index in 0..display_values.len() {
                if display_values[index].get_display_value_type() == default_display_value_type {
                    active_display_value_index = index;
                    break;
                }
            }
        }

        DataValueViewData {
            data_type_ref: DataTypeRefConverter {}.convert_to_view_data(data_value.get_data_type_ref()),
            display_values: ModelRc::new(VecModel::from(DisplayValueConverter {}.convert_collection(display_values))),
            active_display_value_index: active_display_value_index as i32,
        }*/
        DataValueViewData {
            data_type_ref: DataTypeRefConverter {}.convert_to_view_data(data_value.get_data_type_ref()),
            display_values: ModelRc::new(VecModel::from(DisplayValueConverter {}.convert_collection(&vec![]))),
            active_display_value_index: -1 as i32,
        }
    }
}
