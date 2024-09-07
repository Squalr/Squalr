use crate::view_models::view_model::ViewModel;
use crate::MainWindowView;
use slint::ComponentHandle;
use std::sync::Arc;

pub struct ScanViewModel {
    view_handle: Arc<MainWindowView>,
}

impl ScanViewModel {
    pub fn new(view_handle: Arc<MainWindowView>) -> Self {
        let view = ScanViewModel {
            view_handle: view_handle.clone(),
        };

        view.create_bindings();

        return view;
    }
}

impl ViewModel for ScanViewModel {
    fn create_bindings(&self) {
        // let _ = self.view_handle.global::<ScanAdapter>();
    }
}
