use squalr_engine_api::structures::{data_values::anonymous_value_string_format::AnonymousValueStringFormat, structs::valued_struct_field::ValuedStructField};

#[derive(Clone, PartialEq)]
pub enum StructViewerFrameAction {
    None,
    SelectField(String),
    EditValue(ValuedStructField),
    EditDisplayFormat {
        field_name: String,
        display_format: AnonymousValueStringFormat,
    },
    RequestFieldEditor(ValuedStructField),
    OpenInMemoryViewer(String),
    OpenInCodeViewer(String),
}
