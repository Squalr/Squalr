use crate::MainWindowView;
use crate::ProcessSelectorViewModelBindings;
use crate::ProcessViewData;
use crate::view_models::process_selector::opened_process_info_converter::OpenedProcessInfoConverter;
use crate::view_models::process_selector::process_info_comparer::ProcessInfoComparer;
use crate::view_models::process_selector::process_info_converter::ProcessInfoConverter;
use slint::ComponentHandle;
use slint_mvvm::view_binding::ViewBinding;
use slint_mvvm::view_collection_binding::ViewCollectionBinding;
use slint_mvvm::view_data_converter::ViewDataConverter;
use slint_mvvm_macros::create_view_bindings;
use slint_mvvm_macros::create_view_model_collection;
use squalr_engine::commands::engine_command::EngineCommand;
use squalr_engine::commands::process::process_command::ProcessCommand;
use squalr_engine::events::engine_event::EngineEvent;
use squalr_engine::events::engine_event::EngineEvent::ProcessOpened;
use squalr_engine::events::process::process_event::ProcessEvent;
use squalr_engine::responses::process::process_response::ProcessListResponse;
use squalr_engine::responses::process::process_response::ProcessOpenResponse;
use squalr_engine::responses::process::process_response::ProcessResponse;
use squalr_engine::squalr_engine::SqualrEngine;
use squalr_engine_processes::process_info::ProcessInfo;
use std::sync::mpmc;
use std::thread;

pub struct ProcessSelectorViewModel {
    _view_binding: ViewBinding<MainWindowView>,
    _full_process_list_collection: ViewCollectionBinding<ProcessViewData, ProcessInfo, MainWindowView>,
    _windowed_process_list_collection: ViewCollectionBinding<ProcessViewData, ProcessInfo, MainWindowView>,
}

impl ProcessSelectorViewModel {
    pub fn new(view_binding: ViewBinding<MainWindowView>) -> Self {
        // Create a binding that allows us to easily update the view's process list.
        let full_process_list_collection = create_view_model_collection!(
            view_binding -> MainWindowView,
            ProcessSelectorViewModelBindings -> { set_processes, get_processes },
            ProcessInfoConverter -> [],
            ProcessInfoComparer -> [],
        );

        // Create a binding that allows us to easily update the view's windowed process list.
        let windowed_process_list_collection = create_view_model_collection!(
            view_binding -> MainWindowView,
            ProcessSelectorViewModelBindings -> { set_windowed_processes, get_windowed_processes },
            ProcessInfoConverter -> [],
            ProcessInfoComparer -> [],
        );

        let view = ProcessSelectorViewModel {
            _view_binding: view_binding.clone(),
            _full_process_list_collection: full_process_list_collection.clone(),
            _windowed_process_list_collection: windowed_process_list_collection.clone(),
        };

        // Route all view bindings to Rust.
        create_view_bindings!(view_binding, {
            ProcessSelectorViewModelBindings => {
                on_refresh_full_process_list() -> [full_process_list_collection] -> Self::on_refresh_full_process_list
                on_refresh_windowed_process_list() -> [windowed_process_list_collection] -> Self::on_refresh_windowed_process_list
                on_select_process(process_entry: ProcessViewData) -> [] -> Self::on_select_process
            }
        });

        view.listen_for_process_change(SqualrEngine::get_engine_event_receiver(), view_binding.clone());

        view
    }

    fn listen_for_process_change(
        &self,
        event_receiver: mpmc::Receiver<EngineEvent>,
        view_binding: ViewBinding<MainWindowView>,
    ) {
        thread::spawn(move || {
            loop {
                if let Ok(event) = event_receiver.recv() {
                    match event {
                        ProcessOpened(process_event) => match process_event {
                            ProcessEvent::Open { process_info } => {
                                view_binding.execute_on_ui_thread(move |main_window_view, _view_binding| {
                                    main_window_view
                                        .global::<ProcessSelectorViewModelBindings>()
                                        .set_selected_process(OpenedProcessInfoConverter::new().convert_to_view_data(&process_info));
                                });
                            }
                        },
                    }
                }
            }
        });
    }

    // TODO: This can't live in the GUI project. This needs to be a command, or auto handled.
    /*
    fn start_refresh_process_lists_task(&self) {
        let full_process_list_collection = self.full_process_list_collection.clone();

        thread::spawn(move || {
            // The process list is incredibly laggy in debug mode, so just hard cap this to 20 entries for now until we solve this.
            #[cfg(debug_assertions)]
            let limit = Some(20u64);
            #[cfg(not(debug_assertions))]
            let limit = None;

            // Phase 1: Gradually load the first set of processes.
            loop {
                let initial_processes = ProcessQuery::get_processes(ProcessSelectorViewModel::get_process_query_options(None, false, limit));
                if initial_processes.is_empty() {
                    thread::sleep(Duration::from_millis(25));
                    continue;
                }
                for index in 1..=initial_processes.len() {
                    full_process_list_collection.update_from_source(initial_processes[..index].to_vec());
                    thread::sleep(Duration::from_millis(5));
                }
                break;
            }

            // Phase 2: full loop. We should be hitting cache mostly in the UI by now, so it should be fine.
            loop {
                let processes = ProcessQuery::get_processes(ProcessSelectorViewModel::get_process_query_options(None, false, limit));
                full_process_list_collection.update_from_source(processes);
                thread::sleep(Duration::from_millis(250));
            }
        });
    }

    fn get_process_query_options(
        required_pid: Option<Pid>,
        require_windowed: bool,
        limit: Option<u64>,
    ) -> ProcessQueryOptions {
        /*
        let list_processes_command = EngineCommand::Process {
            0: ProcessCommand::List {
                require_windowed: false,
                search_name: None,
                match_case: false,
                limit: Some(1),
            },
        };
         */
        ProcessQueryOptions {
            required_pid: required_pid,
            search_name: None,
            require_windowed: require_windowed,
            match_case: false,
            fetch_icons: true,
            limit: limit,
        }
    } */

    fn on_refresh_full_process_list(full_process_list_collection: ViewCollectionBinding<ProcessViewData, ProcessInfo, MainWindowView>) {
        let list_all_processes_command = EngineCommand::Process {
            0: ProcessCommand::List {
                require_windowed: false,
                search_name: None,
                match_case: false,
                limit: None,
                fetch_icons: true,
            },
        };

        SqualrEngine::dispatch_command_with_response::<ProcessListResponse, _>(list_all_processes_command, move |processes| {
            full_process_list_collection.update_from_source(processes);
        });
    }

    fn on_refresh_windowed_process_list(windowed_process_list_collection: ViewCollectionBinding<ProcessViewData, ProcessInfo, MainWindowView>) {
        let list_windowed_processes_command = EngineCommand::Process {
            0: ProcessCommand::List {
                require_windowed: true,
                search_name: None,
                match_case: false,
                limit: None,
                fetch_icons: true,
            },
        };

        SqualrEngine::dispatch_command_with_response::<ProcessListResponse, _>(list_windowed_processes_command, move |processes| {
            windowed_process_list_collection.update_from_source(processes);
        });
    }

    fn on_select_process(process_entry: ProcessViewData) {
        let open_process_command = EngineCommand::Process {
            0: ProcessCommand::Open {
                pid: Some(process_entry.process_id as u32),
                search_name: None,
                match_case: false,
            },
        };

        SqualrEngine::dispatch_command_with_response::<ProcessOpenResponse, _>(open_process_command, |_| {});
    }
}
