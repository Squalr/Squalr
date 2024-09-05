use crate::views::docking::panel_title_bar_view_model::PanelTitleBarViewModel;
use crate::PanelWindowView;
use slint::*;
use std::sync::Arc;

pub struct PanelWindowViewModel {
    view_handle: Arc<PanelWindowView>,
    title_bar_view: Arc<PanelTitleBarViewModel>,
}

/// Wraps the slint main window to internally manage and track the view handle for later use, as well as setting up
/// view code bindings to the corresponding slint UI.
impl PanelWindowViewModel {
    pub fn new() -> Self {
        let view_handle = Arc::new(PanelWindowView::new().unwrap());
        let view = PanelWindowViewModel {
            view_handle: view_handle.clone(),
            title_bar_view: Arc::new(PanelTitleBarViewModel::new(view_handle.clone())),
        };

        return view;
    }

    pub fn show(&self) {
        return self.view_handle.show().unwrap();
    }

    pub fn get_title_bar_view(&self) -> &Arc<PanelTitleBarViewModel> {
        return &self.title_bar_view;
    }
}
