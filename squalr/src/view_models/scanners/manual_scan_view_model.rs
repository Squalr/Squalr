use crate::MainWindowView;
use crate::mvvm::view_model_base::ViewModel;
use crate::mvvm::view_model_base::ViewModelBase;

pub struct ManualScanViewModel {
    view_model_base: ViewModelBase<MainWindowView>,
}

impl ManualScanViewModel {
    pub fn new(view_model_base: ViewModelBase<MainWindowView>) -> Self {
        let view = ManualScanViewModel {
            view_model_base: view_model_base,
        };

        view.create_view_bindings();

        return view;
    }
}

impl ViewModel for ManualScanViewModel {
    fn create_view_bindings(&self) {
        self.view_model_base
            .execute_on_ui_thread(move |_main_window_view, _view_model_base| {
                // TODO
            });
    }
}
