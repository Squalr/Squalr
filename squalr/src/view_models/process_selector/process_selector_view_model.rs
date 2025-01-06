use crate::view_models::view_model_base::ViewModel;
use crate::view_models::view_model_base::ViewModelBase;
use crate::MainWindowView;
use crate::ProcessSelectorViewModelBindings;
use crate::ProcessViewData;
use slint::ComponentHandle;
use slint::Image;
use slint::SharedPixelBuffer;
use squalr_engine_processes::process_query::process_queryer::ProcessQuery;
use squalr_engine_processes::process_query::process_queryer::ProcessQueryOptions;

pub struct ProcessSelectorViewModel {
    view_model_base: ViewModelBase<MainWindowView>,
}

impl ProcessSelectorViewModel {
    pub fn new(view_model_base: ViewModelBase<MainWindowView>) -> Self {
        let view = ProcessSelectorViewModel {
            view_model_base: view_model_base,
        };

        view.create_view_bindings();

        return view;
    }

    fn refresh_process_list(
        view_model_base: ViewModelBase<MainWindowView>,
        refresh_windowed_list: bool,
    ) {
        view_model_base.execute_on_ui_thread(move |main_window_view, view_model_base| {
            let process_query_options = ProcessQueryOptions {
                require_windowed: refresh_windowed_list,
                search_name: None,
                match_case: false,
                limit: None,
            };
            let process_list = ProcessQuery::get_instance().get_processes(process_query_options);
            let process_selector_view = main_window_view.global::<ProcessSelectorViewModelBindings>();

            let process_data: Vec<ProcessViewData> = process_list
                .iter()
                .map(|process_info| ProcessViewData {
                    process_id: process_info.pid.to_string().into(),
                    name: process_info.name.to_string().into(),
                    icon: Image::from_rgb8(SharedPixelBuffer::new(1, 1)),
                })
                .collect();

            if refresh_windowed_list {
                process_selector_view.set_windowed_processes(process_data.as_slice().into());
            } else {
                process_selector_view.set_processes(process_data.as_slice().into());
            }
        });
    }
}

impl ViewModel for ProcessSelectorViewModel {
    fn create_view_bindings(&self) {
        self.view_model_base
            .execute_on_ui_thread(move |main_window_view, view_model_base| {
                let process_selector_view = main_window_view.global::<ProcessSelectorViewModelBindings>();

                let view_model = view_model_base.clone();
                process_selector_view.on_refresh_full_process_list(move || {
                    Self::refresh_process_list(view_model.clone(), false);
                });

                let view_model = view_model_base.clone();
                process_selector_view.on_refresh_windowed_process_list(move || {
                    Self::refresh_process_list(view_model.clone(), true);
                });

                process_selector_view.on_select_process(|process_entry| {
                    /*
                    SessionManager::set_opened_process(
                        &mut self,
                        ProcessInfo {
                            pid: Pid,
                            handle: u64,
                            bitness: Bitness,
                        },
                    );*/
                });
            });
    }
}
