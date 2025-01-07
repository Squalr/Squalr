use crate::MainWindowView;
use crate::ProcessSelectorViewModelBindings;
use crate::ProcessViewData;
use crate::mvvm::view_data_converter::ViewDataConverter;
use crate::mvvm::view_model_base::ViewModel;
use crate::mvvm::view_model_base::ViewModelBase;
use crate::mvvm::view_model_collection::ViewModelCollection;
use crate::mvvm::view_model_entry::ViewModelEntry;
use crate::view_models::process_selector::process_info_converter::ProcessInfoConverter;
use slint::ComponentHandle;
use slint::ModelRc;
use slint::VecModel;
use squalr_engine::session_manager::SessionManager;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_processes::process_info::ProcessInfo;
use squalr_engine_processes::process_query::process_queryer::ProcessQuery;
use squalr_engine_processes::process_query::process_queryer::ProcessQueryOptions;
use std::rc::Rc;
use std::sync::Arc;
use sysinfo::Pid;

impl ViewModelEntry for ProcessViewData {}

pub struct ProcessSelectorViewModel {
    view_model_base: ViewModelBase<MainWindowView>,
    processes: ViewModelCollection<ProcessViewData, ProcessInfo>,
    windowed_processes: ViewModelCollection<ProcessViewData, ProcessInfo>,
}

impl ProcessSelectorViewModel {
    pub fn new(view_model_base: ViewModelBase<MainWindowView>) -> Self {
        let view = ProcessSelectorViewModel {
            view_model_base: view_model_base.clone(),
            processes: ViewModelCollection::new(|process| ProcessInfoConverter.convert(process)),
            windowed_processes: ViewModelCollection::new(|process| ProcessInfoConverter.convert(process)),
        };

        view.create_view_bindings();
        view
    }

    fn refresh_full_process_list(
        view_model_base: ViewModelBase<MainWindowView>,
        converter: Arc<dyn Fn(ProcessInfo) -> ProcessViewData + Send + Sync>,
    ) {
        view_model_base.execute_on_ui_thread(move |main_window_view, _| {
            let process_selector_view = main_window_view.global::<ProcessSelectorViewModelBindings>();
            let process_query_options = ProcessQueryOptions {
                required_pid: None,
                search_name: None,
                require_windowed: false,
                match_case: false,
                fetch_icons: true,
                limit: None,
            };

            let processes = ProcessQuery::get_processes(process_query_options);
            let view_data: Vec<ProcessViewData> = processes
                .into_iter()
                .map(|process| converter(process))
                .collect();

            let new_model = Rc::new(VecModel::default());
            new_model.set_vec(view_data);

            process_selector_view.set_processes(ModelRc::from(new_model));
        });
    }

    fn refresh_windowed_process_list(
        view_model_base: ViewModelBase<MainWindowView>,
        converter: Arc<dyn Fn(ProcessInfo) -> ProcessViewData + Send + Sync>,
    ) {
        view_model_base.execute_on_ui_thread(move |main_window_view, _| {
            let process_selector_view = main_window_view.global::<ProcessSelectorViewModelBindings>();
            let process_query_options = ProcessQueryOptions {
                required_pid: None,
                search_name: None,
                require_windowed: true,
                match_case: false,
                fetch_icons: true,
                limit: None,
            };

            let processes = ProcessQuery::get_processes(process_query_options);
            let view_data: Vec<ProcessViewData> = processes
                .into_iter()
                .map(|process| converter(process))
                .collect();

            let new_model = Rc::new(VecModel::default());
            new_model.set_vec(view_data);

            process_selector_view.set_windowed_processes(ModelRc::from(new_model));
        });
    }
}

impl ViewModel for ProcessSelectorViewModel {
    fn create_view_bindings(&self) {
        let process_info_converter = self.processes.converter();

        self.view_model_base
            .execute_on_ui_thread(move |main_window_view, view_model_base| {
                let process_selector_view = main_window_view.global::<ProcessSelectorViewModelBindings>();

                let view_model = view_model_base.clone();
                let converter = process_info_converter.clone();
                process_selector_view.on_refresh_full_process_list(move || {
                    Self::refresh_full_process_list(view_model.clone(), converter.clone());
                });

                let view_model = view_model_base.clone();
                let converter = process_info_converter.clone();
                process_selector_view.on_refresh_windowed_process_list(move || {
                    Self::refresh_windowed_process_list(view_model.clone(), converter.clone());
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
