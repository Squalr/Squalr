use crate::MainWindowView;
use crate::mvvm::view_binding::ViewBinding;

pub struct ScanSettingsViewModel {
    view_binding: ViewBinding<MainWindowView>,
}

impl ScanSettingsViewModel {
    pub fn new(view_binding: ViewBinding<MainWindowView>) -> Self {
        let view = ScanSettingsViewModel { view_binding: view_binding };

        return view;
    }
}
