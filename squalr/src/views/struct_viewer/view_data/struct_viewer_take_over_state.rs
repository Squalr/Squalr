use squalr_engine_api::structures::{
    data_types::data_type_ref::DataTypeRef, data_values::anonymous_value_string::AnonymousValueString, structs::valued_struct_field::ValuedStructField,
};

#[derive(Clone)]
pub enum StructViewerTakeOverState {
    EditPointerOffsets {
        valued_struct_field: ValuedStructField,
    },
    EditInstruction {
        valued_struct_field: ValuedStructField,
        validation_data_type_ref: DataTypeRef,
        initial_value_edit: AnonymousValueString,
    },
}
