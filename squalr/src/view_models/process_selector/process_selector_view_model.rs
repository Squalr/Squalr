use crate::MainWindowView;
use crate::ProcessSelectorViewModelBindings;
use crate::ProcessViewData;
use crate::view_models::process_selector::process_info_comparer::ProcessInfoComparer;
use crate::view_models::process_selector::process_info_converter::ProcessInfoConverter;
use slint::ComponentHandle;
use slint_mvvm::view_binding::ViewBinding;
use slint_mvvm::view_collection_binding::ViewCollectionBinding;
use slint_mvvm::view_data_converter::ViewDataConverter;
use slint_mvvm_macros::create_view_bindings;
use slint_mvvm_macros::create_view_model_collection;
use squalr_engine::session_manager::SessionManager;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_processes::process_info::ProcessInfo;
use squalr_engine_processes::process_query::process_queryer::ProcessQuery;
use squalr_engine_processes::process_query::process_queryer::ProcessQueryOptions;
use std::thread;
use std::time::Duration;
use sysinfo::Pid;

pub struct ProcessSelectorViewModel {
    _view_binding: ViewBinding<MainWindowView>,
    processes_collection: ViewCollectionBinding<ProcessViewData, ProcessInfo, MainWindowView>,
    _windowed_processes_collection: ViewCollectionBinding<ProcessViewData, ProcessInfo, MainWindowView>,
}

impl ProcessSelectorViewModel {
    pub fn new(view_binding: ViewBinding<MainWindowView>) -> Self {
        // Create a binding that allows us to easily update the view's process list.
        let processes_collection = create_view_model_collection!(
            view_binding -> MainWindowView,
            ProcessSelectorViewModelBindings -> { set_processes, get_processes },
            ProcessInfoConverter -> [],
            ProcessInfoComparer -> [],
        );

        // Create a binding that allows us to easily update the view's windowed process list.
        let windowed_processes_collection = create_view_model_collection!(
            view_binding -> MainWindowView,
            ProcessSelectorViewModelBindings -> { set_windowed_processes, get_windowed_processes },
            ProcessInfoConverter -> [],
            ProcessInfoComparer -> [],
        );

        let view = ProcessSelectorViewModel {
            _view_binding: view_binding.clone(),
            processes_collection: processes_collection.clone(),
            _windowed_processes_collection: windowed_processes_collection.clone(),
        };

        // Route all view bindings to Rust.
        create_view_bindings!(view_binding, {
            ProcessSelectorViewModelBindings => {
                on_refresh_full_process_list() -> [processes_collection] -> Self::on_refresh_full_process_list
                on_refresh_windowed_process_list() -> [windowed_processes_collection] -> Self::on_refresh_windowed_process_list
                on_select_process(process_entry: ProcessViewData) -> [view_binding] -> Self::on_select_process
            }
        });

        view.start_refresh_process_lists_task();

        view
    }

    fn start_refresh_process_lists_task(&self) {
        let processes_collection = self.processes_collection.clone();

        thread::spawn(move || {
            // Phase 1: Gradually load the first set of processes.
            loop {
                let initial_processes = ProcessQuery::get_processes(ProcessSelectorViewModel::get_process_query_options(None, false, None));
                if initial_processes.is_empty() {
                    thread::sleep(Duration::from_millis(25));
                    continue;
                }
                for index in 1..=initial_processes.len() {
                    processes_collection.update_from_source(initial_processes[..index].to_vec());
                    thread::sleep(Duration::from_millis(5));
                }
                break;
            }

            // Phase 2: full loop. We should be hitting cache mostly in the UI by now, so it should be fine.
            loop {
                let processes = ProcessQuery::get_processes(ProcessSelectorViewModel::get_process_query_options(None, false, None));
                processes_collection.update_from_source(processes);
                thread::sleep(Duration::from_millis(250));
            }
        });
    }

    fn get_process_query_options(
        required_pid: Option<Pid>,
        require_windowed: bool,
        limit: Option<u64>,
    ) -> ProcessQueryOptions {
        ProcessQueryOptions {
            required_pid: required_pid,
            search_name: None,
            require_windowed: require_windowed,
            match_case: false,
            fetch_icons: true,
            limit: limit,
        }
    }

    fn on_refresh_full_process_list(process_info_converter: ViewCollectionBinding<ProcessViewData, ProcessInfo, MainWindowView>) {
        let process_query_options = Self::get_process_query_options(None, false, None);
        let processes = ProcessQuery::get_processes(process_query_options);
        process_info_converter.update_from_source(processes);
    }

    fn on_refresh_windowed_process_list(windowed_process_info_converter: ViewCollectionBinding<ProcessViewData, ProcessInfo, MainWindowView>) {
        let process_query_options = Self::get_process_query_options(None, true, None);
        let processes = ProcessQuery::get_processes(process_query_options);
        windowed_process_info_converter.update_from_source(processes);
    }

    fn on_select_process(
        view_binding: ViewBinding<MainWindowView>,
        process_entry: ProcessViewData,
    ) {
        let process_query_options = Self::get_process_query_options(Some(Pid::from_u32(process_entry.process_id as u32)), true, Some(1));
        let processes = ProcessQuery::get_processes(process_query_options);

        if let Some(process_to_open) = processes.first() {
            match ProcessQuery::open_process(process_to_open) {
                Ok(opened_process) => {
                    if let Ok(mut session_manager) = SessionManager::get_instance().write() {
                        session_manager.set_opened_process(opened_process);

                        let process_to_open = process_to_open.clone();
                        view_binding.execute_on_ui_thread(move |main_window_view, _view_binding| {
                            main_window_view
                                .global::<ProcessSelectorViewModelBindings>()
                                .set_selected_process(ProcessInfoConverter::new().convert_to_view_data(&process_to_open));
                        });
                    } else {
                        Logger::get_instance().log(LogLevel::Warn, "Failed to open process.", None);
                    }
                }
                Err(err) => {
                    Logger::get_instance().log(LogLevel::Error, &format!("Failed to open process {}: {}", process_to_open.pid, err), None);
                }
            }
        }
    }
}
