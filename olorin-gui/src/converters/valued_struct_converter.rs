use crate::ValuedStructViewData;
use crate::converters::symbolic_struct_ref_converter::SymbolicStructRefConverter;
use crate::converters::valued_struct_field_converter::ValuedStructFieldConverter;
use olorin_engine_api::structures::structs::valued_struct::ValuedStruct;
use slint::ModelRc;
use slint::VecModel;
use slint_mvvm::{convert_from_view_data::ConvertFromViewData, convert_to_view_data::ConvertToViewData};

pub struct ValuedStructConverter {}

impl ValuedStructConverter {
    pub fn new() -> Self {
        Self {}
    }
}

impl ConvertToViewData<ValuedStruct, ValuedStructViewData> for ValuedStructConverter {
    fn convert_collection(
        &self,
        valued_struct_list: &Vec<ValuedStruct>,
    ) -> Vec<ValuedStructViewData> {
        valued_struct_list
            .into_iter()
            .map(|item| self.convert_to_view_data(item))
            .collect()
    }

    fn convert_to_view_data(
        &self,
        valued_struct: &ValuedStruct,
    ) -> ValuedStructViewData {
        let fields = valued_struct
            .get_fields()
            .iter()
            .map(|field| ValuedStructFieldConverter {}.convert_to_view_data(&field))
            .collect::<Vec<_>>();
        ValuedStructViewData {
            name: valued_struct
                .get_symbolic_struct_ref()
                .get_symbolic_struct_namespace()
                .into(),
            symbolic_struct_ref: SymbolicStructRefConverter {}.convert_to_view_data(valued_struct.get_symbolic_struct_ref()),
            fields: ModelRc::new(VecModel::from(fields)),
        }
    }
}

impl ConvertFromViewData<ValuedStruct, ValuedStructViewData> for ValuedStructConverter {
    fn convert_from_view_data(
        &self,
        valued_struct: &ValuedStructViewData,
    ) -> ValuedStruct {
        let symbolic_struct_ref = SymbolicStructRefConverter {}.convert_from_view_data(&valued_struct.symbolic_struct_ref);

        // JIRA: Not implemented.
        let JIRA = 420;
        ValuedStruct::new(symbolic_struct_ref, vec![])
    }
}
