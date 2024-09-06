use crate::views::view_model::ViewModel;
use crate::FooterAdapter;
use crate::MainWindowView;
use slint::ComponentHandle;
use std::sync::Arc;

pub struct FooterViewModel {
    view_handle: Arc<MainWindowView>,
}

/// Custom title bar implementation with minimize/maximize/close and dragging.
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
