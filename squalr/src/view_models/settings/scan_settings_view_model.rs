use crate::MainWindowView;
use slint_mvvm::view_binding::ViewBinding;

pub struct ScanSettingsViewModel {
    _view_binding: ViewBinding<MainWindowView>,
}

impl ScanSettingsViewModel {
    pub fn new(view_binding: ViewBinding<MainWindowView>) -> Self {
        let view = ScanSettingsViewModel { _view_binding: view_binding };

        view
    }
}
