use crate::views::title_bar_view::TitleBarView;
use crate::MainWindow;
use slint::*;
use std::sync::Arc;

pub struct MainWindowView {
    view_handle: Arc<MainWindow>,
    title_bar_view: Arc<TitleBarView>,
}

/// Wraps the slint main window to internally manage and track the view handle for later use, as well as setting up
/// view code bindings to the corresponding slint UI.
impl MainWindowView {
    pub fn new() -> Self {
        let view_handle = Arc::new(MainWindow::new().unwrap());
        let view = MainWindowView {
            view_handle: view_handle.clone(),
            title_bar_view: Arc::new(TitleBarView::new(view_handle.clone())),
        };

        return view;
    }

    pub fn run_event_loop(&self) {
        return self.view_handle.run().unwrap();
    }

    pub fn get_title_bar_view(&self) -> &Arc<TitleBarView> {
        return &self.title_bar_view;
    }

    fn bind_view_to_ui(&self) {
        /*
        fn init() -> MainWindow {
            let view_handle = MainWindow::new().unwrap();

            // TODO
            // view_handle.window().set_minimized(true);

            title_bar_adapter::bind(&view_handle);

            let task_list_controller = mvc::TaskListController::new(mvc::task_repo());
            task_list_adapter::connect(&view_handle, task_list_controller.clone());
            navigation_adapter::connect_task_list_controller(&view_handle, task_list_controller.clone());

            let create_task_controller = mvc::CreateTaskController::new(mvc::date_time_repo());
            create_task_adapter::connect(&view_handle, create_task_controller.clone());
            navigation_adapter::connect_create_task_controller(&view_handle, create_task_controller);
            create_task_adapter::connect_task_list_controller(&view_handle, task_list_controller);

            return view_handle;
        }*/
    }
}
