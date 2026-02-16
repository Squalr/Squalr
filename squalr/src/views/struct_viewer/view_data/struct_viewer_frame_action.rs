use squalr_engine_api::structures::structs::valued_struct_field::ValuedStructField;

#[derive(Clone, PartialEq)]
pub enum StructViewerFrameAction {
    None,
    SelectField(String),
    EditValue(ValuedStructField),
}
