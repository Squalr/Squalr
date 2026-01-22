use squalr_engine_api::structures::structs::valued_struct::ValuedStruct;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct StructViewerViewData {
    pub struct_under_view: Arc<RwLock<Option<ValuedStruct>>>,
    pub struct_modified_callback: Arc<RwLock<Option<Box<dyn FnOnce(ValuedStruct) + Send + Sync>>>>,
}

impl StructViewerViewData {
    pub fn new() -> Self {
        Self {
            struct_under_view: Arc::new(RwLock::new(None)),
            struct_modified_callback: Arc::new(RwLock::new(None)),
        }
    }

    pub fn focus_valued_struct(
        &self,
        valued_struct: ValuedStruct,
        valued_struct_edited_callback: Box<dyn FnOnce(ValuedStruct) + Send + Sync>,
    ) {
        self.set_valued_struct_and_callback(Some(valued_struct), Some(valued_struct_edited_callback));
    }

    pub fn clear_focus(&self) {
        self.set_valued_struct_and_callback(None, None);
    }

    fn set_valued_struct_and_callback(
        &self,
        valued_struct: Option<ValuedStruct>,
        valued_struct_edited_callback: Option<Box<dyn FnOnce(ValuedStruct) + Send + Sync>>,
    ) {
        let mut struct_under_view = match self.struct_under_view.write() {
            Ok(struct_under_view) => struct_under_view,
            Err(error) => {
                log::error!("Error acquiring write lock for struct under view: {}", error);
                return;
            }
        };
        let mut struct_modified_callback = match self.struct_modified_callback.write() {
            Ok(struct_modified_callback) => struct_modified_callback,
            Err(error) => {
                log::error!("Error acquiring write lock for struct modified callback: {}", error);
                return;
            }
        };

        *struct_under_view = valued_struct;
        *struct_modified_callback = valued_struct_edited_callback;
    }
}
