use crate::MainWindowView;
use crate::mvvm::view_binding::ViewBinding;
use crate::mvvm::view_binding::ViewModel;

pub struct ScanSettingsViewModel {
    view_binding: ViewBinding<MainWindowView>,
}

impl ScanSettingsViewModel {
    pub fn new(view_binding: ViewBinding<MainWindowView>) -> Self {
        let view = ScanSettingsViewModel { view_binding: view_binding };

        view.create_view_bindings();

        return view;
    }
}

impl ViewModel for ScanSettingsViewModel {
    fn create_view_bindings(&self) {
        self.view_binding
            .execute_on_ui_thread(move |_main_window_view, _view_binding| {
                // TODO
            });
    }
}
