use squalr_engine_api::structures::{data_values::data_value::DataValue, structs::valued_struct_field::ValuedStructField};

#[derive(Clone, PartialEq)]
pub enum StructViewerFrameAction {
    None,
    SelectField(String),
    EditValue(ValuedStructField, DataValue),
}
