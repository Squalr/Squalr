use crate::MainWindowView;
use crate::ProcessSelectorViewModelBindings;
use crate::ProcessViewData;
use crate::mvvm::view_data_converter::ViewDataConverter;
use crate::mvvm::view_model_base::ViewModel;
use crate::mvvm::view_model_base::ViewModelBase;
use crate::mvvm::view_model_entry::ModelUpdate;
use crate::mvvm::view_model_entry::ViewModelEntry;
use crate::mvvm::view_model_entry::create_model_from_existing;
use crate::view_models::process_selector::process_info_converter::ProcessInfoConverter;
use slint::ComponentHandle;
use slint::Model;
use slint::ModelRc;
use slint::VecModel;
use squalr_engine::session_manager::SessionManager;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_processes::process_info::ProcessInfo;
use squalr_engine_processes::process_query::process_queryer::ProcessQuery;
use squalr_engine_processes::process_query::process_queryer::ProcessQueryOptions;
use std::rc::Rc;
use sysinfo::Pid;

impl ViewModelEntry for ProcessViewData {}

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
        view_model_base.execute_on_ui_thread(move |main_window_view, _view_model_base| {
            let process_selector_view = main_window_view.global::<ProcessSelectorViewModelBindings>();

            let process_list = if refresh_windowed_list {
                process_selector_view.get_windowed_processes()
            } else {
                process_selector_view.get_processes()
            };

            let process_list = match process_list
                .as_any()
                .downcast_ref::<VecModel<ProcessViewData>>()
            {
                Some(model) => create_model_from_existing(model),
                None => Rc::new(VecModel::default()),
            };

            let process_query_options = ProcessQueryOptions {
                required_pid: None,
                search_name: None,
                require_windowed: refresh_windowed_list,
                match_case: false,
                fetch_icons: true,
                limit: None,
            };

            let processes = ProcessQuery::get_processes(process_query_options);
            let view_data: Vec<ProcessViewData> = processes
                .into_iter()
                .map(|process| ProcessInfoConverter.convert(process))
                .collect();

            process_list.update_model(view_data);

            // Update the UI with the modified list
            if refresh_windowed_list {
                process_selector_view.set_windowed_processes(ModelRc::from(process_list));
            } else {
                process_selector_view.set_processes(ModelRc::from(process_list));
            }
        });
    }
}

impl ViewModel for ProcessSelectorViewModel {
    fn create_view_bindings(&self) {
        self.view_model_base
            .execute_on_ui_thread(move |main_window_view, view_model_base| {
                let process_selector_view = main_window_view.global::<ProcessSelectorViewModelBindings>();

                // Initialize empty lists
                process_selector_view.set_processes(ModelRc::from(Rc::new(VecModel::<ProcessViewData>::default())));
                process_selector_view.set_windowed_processes(ModelRc::from(Rc::new(VecModel::<ProcessViewData>::default())));

                let view_model = view_model_base.clone();
                process_selector_view.on_refresh_full_process_list(move || {
                    Self::refresh_process_list(view_model.clone(), false);
                });

                let view_model = view_model_base.clone();
                process_selector_view.on_refresh_windowed_process_list(move || {
                    Self::refresh_process_list(view_model.clone(), true);
                });

                process_selector_view.on_select_process(|process_entry| {
                    let process_to_open = ProcessInfo {
                        pid: Pid::from_u32(process_entry.process_id as u32),
                        name: String::new(),
                        is_windowed: false,
                        icon: None,
                    };
                    match ProcessQuery::open_process(&process_to_open) {
                        Ok(opened_process) => {
                            if let Ok(mut session_manager) = SessionManager::get_instance().write() {
                                session_manager.set_opened_process(opened_process);
                            } else {
                                Logger::get_instance().log(LogLevel::Warn, "Failed to open process.", None);
                            }
                        }
                        Err(err) => {
                            Logger::get_instance().log(LogLevel::Error, &format!("Failed to open process {}: {}", process_to_open.pid, err), None);
                        }
                    }
                });
            });
    }
}
