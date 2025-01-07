use crate::MainWindowView;
use crate::mvvm::view_binding::ViewModel;
use crate::mvvm::view_binding::ViewBinding;

pub struct ManualScanViewModel {
    view_binding: ViewBinding<MainWindowView>,
}

impl ManualScanViewModel {
    pub fn new(view_binding: ViewBinding<MainWindowView>) -> Self {
        let view = ManualScanViewModel {
            view_binding: view_binding,
        };

        view.create_view_bindings();

        return view;
    }
}

impl ViewModel for ManualScanViewModel {
    fn create_view_bindings(&self) {
        self.view_binding
            .execute_on_ui_thread(move |_main_window_view, _view_binding| {
                // TODO
            });
    }
}
