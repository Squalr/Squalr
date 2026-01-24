use squalr_engine_api::{
    dependency_injection::dependency::Dependency,
    structures::structs::{valued_struct::ValuedStruct, valued_struct_field::ValuedStructField},
};
use std::sync::Arc;

#[derive(Clone)]
pub struct StructViewerViewData {
    pub struct_under_view: Arc<Option<ValuedStruct>>,
    pub struct_field_modified_callback: Arc<Option<Box<dyn FnOnce(ValuedStructField) + Send + Sync>>>,
}

impl StructViewerViewData {
    pub fn new() -> Self {
        Self {
            struct_under_view: Arc::new(None),
            struct_field_modified_callback: Arc::new(None),
        }
    }

    pub fn focus_valued_struct(
        struct_viewer_view_data: Dependency<Self>,
        valued_struct: ValuedStruct,
        valued_struct_field_edited_callback: Box<dyn FnOnce(ValuedStructField) + Send + Sync>,
    ) {
        let mut struct_viewer_view_data = match struct_viewer_view_data.write("Focus valued struct") {
            Some(struct_viewer_view_data) => struct_viewer_view_data,
            None => return,
        };
        struct_viewer_view_data.set_valued_struct_and_callback(Some(valued_struct), Some(valued_struct_field_edited_callback));
    }

    pub fn focus_valued_structs(
        struct_viewer_view_data: Dependency<Self>,
        valued_structs: Vec<ValuedStruct>,
        valued_struct_field_edited_callback: Box<dyn FnOnce(ValuedStructField) + Send + Sync>,
    ) {
        let mut struct_viewer_view_data = match struct_viewer_view_data.write("Focus valued struct") {
            Some(struct_viewer_view_data) => struct_viewer_view_data,
            None => return,
        };
        let valued_struct = ValuedStruct::combine_exclusive(&valued_structs);

        struct_viewer_view_data.set_valued_struct_and_callback(Some(valued_struct), Some(valued_struct_field_edited_callback));
    }

    pub fn clear_focus(struct_viewer_view_data: Dependency<Self>) {
        let mut struct_viewer_view_data = match struct_viewer_view_data.write("Focus valued struct") {
            Some(struct_viewer_view_data) => struct_viewer_view_data,
            None => return,
        };
        struct_viewer_view_data.set_valued_struct_and_callback(None, None);
    }

    fn set_valued_struct_and_callback(
        &mut self,
        valued_struct: Option<ValuedStruct>,
        valued_struct_field_edited_callback: Option<Box<dyn FnOnce(ValuedStructField) + Send + Sync>>,
    ) {
        self.struct_under_view = Arc::new(valued_struct);
        self.struct_field_modified_callback = Arc::new(valued_struct_field_edited_callback);
    }
}
