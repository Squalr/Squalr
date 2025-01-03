use crate::view_models::view_model_base::ViewModel;
use crate::MainWindowView;
use crate::ScanSettingsViewModelBindings;
use slint::ComponentHandle;
use std::sync::Arc;

pub struct ManualScanViewModel {
    view_handle: Arc<MainWindowView>,
}

impl ManualScanViewModel {
    pub fn new(view_handle: Arc<MainWindowView>) -> Self {
        let view = ManualScanViewModel {
            view_handle: view_handle.clone(),
        };

        view.create_view_bindings();

        return view;
    }
}

impl ViewModel for ManualScanViewModel {
    fn create_view_bindings(&self) {
        let _ = self.view_handle.global::<ScanSettingsViewModelBindings>();
    }
}
