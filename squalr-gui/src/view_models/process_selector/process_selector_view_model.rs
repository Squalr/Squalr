use crate::MainWindowView;
use crate::ProcessSelectorViewModelBindings;
use crate::ProcessViewData;
use crate::view_models::process_selector::process_info_comparer::ProcessInfoComparer;
use crate::view_models::process_selector::process_info_converter::ProcessInfoConverter;
use slint::ComponentHandle;
use slint::Image;
use slint_mvvm::view_binding::ViewBinding;
use slint_mvvm::view_collection_binding::ViewCollectionBinding;
use slint_mvvm::view_data_converter::ViewDataConverter;
use slint_mvvm_macros::create_view_bindings;
use slint_mvvm_macros::create_view_model_collection;
use squalr_engine::commands::engine_request::EngineRequest;
use squalr_engine::commands::process::list::process_list_request::ProcessListRequest;
use squalr_engine::commands::process::open::process_open_request::ProcessOpenRequest;
use squalr_engine::engine_execution_context::EngineExecutionContext;
use squalr_engine::events::engine_event::EngineEvent;
use squalr_engine_processes::process_info::OpenedProcessInfo;
use squalr_engine_processes::process_info::ProcessInfo;
use std::sync::Arc;
use std::thread;

use super::opened_process_info_converter::OpenedProcessInfoConverter;

pub struct ProcessSelectorViewModel {
    _view_binding: ViewBinding<MainWindowView>,
    _full_process_list_collection: ViewCollectionBinding<ProcessViewData, ProcessInfo, MainWindowView>,
    _windowed_process_list_collection: ViewCollectionBinding<ProcessViewData, ProcessInfo, MainWindowView>,
    engine_execution_context: Arc<EngineExecutionContext>,
}

impl ProcessSelectorViewModel {
    pub fn new(
        view_binding: ViewBinding<MainWindowView>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) -> Self {
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
            engine_execution_context: engine_execution_context.clone(),
        };

        // Route all view bindings to Rust.
        create_view_bindings!(view_binding, {
            ProcessSelectorViewModelBindings => {
                on_refresh_full_process_list() -> [full_process_list_collection, engine_execution_context] -> Self::on_refresh_full_process_list
                on_refresh_windowed_process_list() -> [windowed_process_list_collection, engine_execution_context] -> Self::on_refresh_windowed_process_list
                on_select_process(process_entry: ProcessViewData) -> [view_binding, engine_execution_context] -> Self::on_select_process
            }
        });

        view.listen_for_process_change(view_binding.clone());

        view
    }

    fn listen_for_process_change(
        &self,
        view_binding: ViewBinding<MainWindowView>,
    ) {
        let engine_execution_context = self.engine_execution_context.clone();

        thread::spawn(move || match engine_execution_context.subscribe_to_engine_events() {
            Ok(receiver) => {
                while let Ok(engine_event) = receiver.recv() {
                    match engine_event {
                        EngineEvent::Process(process_changed_event) => {
                            Self::refresh_opened_process(&view_binding, process_changed_event.process_info);
                        }
                    }
                }
            }
            Err(err) => {
                log::error!("Failed to subscribe to engine process events: {}", err);
            }
        });
    }

    fn refresh_opened_process(
        view_binding: &ViewBinding<MainWindowView>,
        process_info: Option<OpenedProcessInfo>,
    ) {
        view_binding.execute_on_ui_thread(move |main_window_view, _| {
            let process_selector_bindings = main_window_view.global::<ProcessSelectorViewModelBindings>();

            if let Some(process_info) = process_info {
                process_selector_bindings.set_selected_process(OpenedProcessInfoConverter::new().convert_to_view_data(&process_info));
            } else {
                process_selector_bindings.set_selected_process(ProcessViewData {
                    icon: Image::default(),
                    name: "".into(),
                    process_id: 0,
                    process_id_str: "".into(),
                });
            }
        });
    }

    fn on_refresh_full_process_list(
        full_process_list_collection: ViewCollectionBinding<ProcessViewData, ProcessInfo, MainWindowView>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) {
        let list_all_processes_request = ProcessListRequest {
            require_windowed: false,
            search_name: None,
            match_case: false,
            limit: None,
            fetch_icons: true,
        };

        list_all_processes_request.send(&engine_execution_context, move |process_list_response| {
            full_process_list_collection.update_from_source(process_list_response.processes);
        });
    }

    fn on_refresh_windowed_process_list(
        windowed_process_list_collection: ViewCollectionBinding<ProcessViewData, ProcessInfo, MainWindowView>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) {
        let list_windowed_processes_request = ProcessListRequest {
            require_windowed: true,
            search_name: None,
            match_case: false,
            limit: None,
            fetch_icons: true,
        };

        list_windowed_processes_request.send(&engine_execution_context, move |process_list_response| {
            windowed_process_list_collection.update_from_source(process_list_response.processes);
        });
    }

    fn on_select_process(
        view_binding: ViewBinding<MainWindowView>,
        engine_execution_context: Arc<EngineExecutionContext>,
        process_entry: ProcessViewData,
    ) {
        let open_process_command = ProcessOpenRequest {
            process_id: Some(process_entry.process_id as u32),
            search_name: None,
            match_case: false,
        };

        open_process_command.send(&engine_execution_context, move |process_open_response| {
            Self::refresh_opened_process(&view_binding, process_open_response.opened_process_info)
        });
    }
}
