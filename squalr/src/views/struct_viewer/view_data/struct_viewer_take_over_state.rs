use squalr_engine_api::structures::structs::valued_struct_field::ValuedStructField;

#[derive(Clone)]
pub enum StructViewerTakeOverState {
    EditPointerOffsets { valued_struct_field: ValuedStructField },
}
