use crate::SymbolicStructRefViewData;
use crate::ValuedStructViewData;
use slint::ModelRc;
use slint::VecModel;
use slint_mvvm::{convert_from_view_data::ConvertFromViewData, convert_to_view_data::ConvertToViewData};
use squalr_engine_api::structures::structs::symbolic_struct_ref::SymbolicStructRef;
use squalr_engine_api::structures::structs::valued_struct::ValuedStruct;

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
        ValuedStructViewData {
            symbolic_struct_ref: SymbolicStructRefViewData {
                symbolic_struct_ref: "".into(),
            },
            fields: ModelRc::new(VecModel::from(vec![])),
        }
    }
}

impl ConvertFromViewData<ValuedStruct, ValuedStructViewData> for ValuedStructConverter {
    fn convert_from_view_data(
        &self,
        valued_struct: &ValuedStructViewData,
    ) -> ValuedStruct {
        // let symbolic_struct_ref = SymbolicStructRefConverter {}.convert_from_view_data(&valued_struct.symbolic_struct_ref);

        ValuedStruct::new(SymbolicStructRef::new_anonymous(), vec![])
    }
}
