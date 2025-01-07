use crate::MainWindowView;
use crate::mvvm::view_binding::ViewBinding;

pub struct ManualScanViewModel {
    view_binding: ViewBinding<MainWindowView>,
}

impl ManualScanViewModel {
    pub fn new(view_binding: ViewBinding<MainWindowView>) -> Self {
        let view = ManualScanViewModel { view_binding: view_binding };

        return view;
    }
}
