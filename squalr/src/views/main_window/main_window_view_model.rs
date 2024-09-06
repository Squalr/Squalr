use crate::views::main_window::footer_view_model::FooterViewModel;
use crate::views::main_window::title_bar_view_model::TitleBarViewModel;
use crate::views::view_model::ViewModel;
use crate::MainWindowView;
use slint::ComponentHandle;
use std::sync::Arc;

pub struct MainWindowViewModel {
    view_handle: Arc<MainWindowView>,
    title_bar_view: Arc<TitleBarViewModel>,
    footer_view: Arc<FooterViewModel>,
}

/// Wraps the slint main window to internally manage and track the view handle for later use, as well as setting up
/// view code bindings to the corresponding slint UI.
impl MainWindowViewModel {
    pub fn new() -> Self {
        let view_handle = Arc::new(MainWindowView::new().unwrap());
        let view = MainWindowViewModel {
            view_handle: view_handle.clone(),
            title_bar_view: Arc::new(TitleBarViewModel::new(view_handle.clone())),
            footer_view: Arc::new(FooterViewModel::new(view_handle.clone())),
        };

        view.create_bindings();

        return view;
    }

    pub fn run_event_loop(&self) {
        return self.view_handle.run().unwrap();
    }

    pub fn get_title_bar_view(&self) -> &Arc<TitleBarViewModel> {
        return &self.title_bar_view;
    }
}

impl ViewModel for MainWindowViewModel {
    fn create_bindings(&self) {
        // Bindings here, if any necessary.
    }
}
