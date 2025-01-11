use crate::MainWindowView;
use crate::mvvm::view_binding::ViewBinding;

pub struct ManualScanViewModel {
    _view_binding: ViewBinding<MainWindowView>,
}

impl ManualScanViewModel {
    pub fn new(view_binding: ViewBinding<MainWindowView>) -> Self {
        let view = ManualScanViewModel { _view_binding: view_binding };

        view
    }
}
