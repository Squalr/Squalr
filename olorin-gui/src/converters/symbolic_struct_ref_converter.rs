use crate::SymbolicStructRefViewData;
use olorin_engine_api::structures::structs::symbolic_struct_ref::SymbolicStructRef;
use slint_mvvm::{convert_from_view_data::ConvertFromViewData, convert_to_view_data::ConvertToViewData};

pub struct SymbolicStructRefConverter {}

impl SymbolicStructRefConverter {
    pub fn new() -> Self {
        Self {}
    }
}

impl ConvertToViewData<SymbolicStructRef, SymbolicStructRefViewData> for SymbolicStructRefConverter {
    fn convert_collection(
        &self,
        symbolic_struct_ref_list: &Vec<SymbolicStructRef>,
    ) -> Vec<SymbolicStructRefViewData> {
        symbolic_struct_ref_list
            .into_iter()
            .map(|item| self.convert_to_view_data(item))
            .collect()
    }

    fn convert_to_view_data(
        &self,
        symbolic_struct_ref: &SymbolicStructRef,
    ) -> SymbolicStructRefViewData {
        let symbolic_struct_namespace = symbolic_struct_ref.get_symbolic_struct_namespace();

        SymbolicStructRefViewData {
            symbolic_struct_ref: symbolic_struct_namespace.into(),
        }
    }
}

impl ConvertFromViewData<SymbolicStructRef, SymbolicStructRefViewData> for SymbolicStructRefConverter {
    fn convert_from_view_data(
        &self,
        symbolic_struct_ref: &SymbolicStructRefViewData,
    ) -> SymbolicStructRef {
        SymbolicStructRef::new(symbolic_struct_ref.symbolic_struct_ref.to_string())
    }
}
