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
use squalr_engine::session_manager::SessionManager;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_processes::process_info::ProcessInfo;
use squalr_engine_processes::process_query::process_queryer::ProcessQuery;
use squalr_engine_processes::process_query::process_queryer::ProcessQueryOptions;
use sysinfo::Pid;

impl ViewModelEntry for ProcessViewData {}

pub struct ProcessSelectorViewModel {
    view_model_base: ViewModelBase<MainWindowView>,
    processes: ViewModelCollection<ProcessViewData, ProcessInfo, MainWindowView>,
    windowed_processes: ViewModelCollection<ProcessViewData, ProcessInfo, MainWindowView>,
}

impl ProcessSelectorViewModel {
    pub fn new(view_model_base: ViewModelBase<MainWindowView>) -> Self {
        let view_handle = view_model_base.get_view_handle().lock().unwrap().clone();

        let processes = ViewModelCollection::new(
            view_handle.clone(),
            |process| ProcessInfoConverter.convert(process),
            |view: &MainWindowView, model| {
                view.global::<ProcessSelectorViewModelBindings>()
                    .set_processes(model)
            },
        );

        let windowed_processes = ViewModelCollection::new(
            view_handle,
            |process| ProcessInfoConverter.convert(process),
            |view: &MainWindowView, model| {
                view.global::<ProcessSelectorViewModelBindings>()
                    .set_windowed_processes(model)
            },
        );

        let view = ProcessSelectorViewModel {
            view_model_base,
            processes,
            windowed_processes,
        };

        view.create_view_bindings();
        view
    }
}

impl ViewModel for ProcessSelectorViewModel {
    fn create_view_bindings(&self) {
        let process_info_converter = self.processes.clone();
        let windowed_process_info_converter = self.windowed_processes.clone();

        self.view_model_base
            .execute_on_ui_thread(move |main_window_view, _| {
                let process_selector_view = main_window_view.global::<ProcessSelectorViewModelBindings>();

                process_selector_view.on_refresh_full_process_list(move || {
                    let process_query_options = ProcessQueryOptions {
                        required_pid: None,
                        search_name: None,
                        require_windowed: false,
                        match_case: false,
                        fetch_icons: true,
                        limit: None,
                    };

                    let processes = ProcessQuery::get_processes(process_query_options);
                    process_info_converter.update_from_source(processes);
                });

                process_selector_view.on_refresh_windowed_process_list(move || {
                    let process_query_options = ProcessQueryOptions {
                        required_pid: None,
                        search_name: None,
                        require_windowed: true,
                        match_case: false,
                        fetch_icons: true,
                        limit: None,
                    };

                    let processes = ProcessQuery::get_processes(process_query_options);
                    windowed_process_info_converter.update_from_source(processes);
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
