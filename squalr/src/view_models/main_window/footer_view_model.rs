use crate::view_models::view_model::ViewModel;
use crate::FooterAdapter;
use crate::MainWindowView;
use slint::ComponentHandle;
use std::sync::Arc;

pub struct FooterViewModel {
    view_handle: Arc<MainWindowView>,
}

impl FooterViewModel {
    pub fn new(view_handle: Arc<MainWindowView>) -> Self {
        let view = FooterViewModel {
            view_handle: view_handle.clone(),
        };

        view.create_bindings();

        return view;
    }
}

impl ViewModel for FooterViewModel {
    fn create_bindings(&self) {
        let _ = self.view_handle.global::<FooterAdapter>();
    }
}
