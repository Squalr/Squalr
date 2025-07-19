use crate::MainWindowView;
use crate::ProcessSelectorViewModelBindings;
use crate::ProcessViewData;
use crate::converters::opened_process_info_converter::OpenedProcessInfoConverter;
use crate::converters::process_info_converter::ProcessInfoConverter;
use olorin_engine::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use olorin_engine::engine_execution_context::EngineExecutionContext;
use olorin_engine_api::commands::process::list::process_list_request::ProcessListRequest;
use olorin_engine_api::commands::process::open::process_open_request::ProcessOpenRequest;
use olorin_engine_api::dependency_injection::dependency_container::DependencyContainer;
use olorin_engine_api::events::process::changed::process_changed_event::ProcessChangedEvent;
use olorin_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use olorin_engine_api::structures::processes::process_info::ProcessInfo;
use slint::ComponentHandle;
use slint::Image;
use slint_mvvm::convert_to_view_data::ConvertToViewData;
use slint_mvvm::view_binding::ViewBinding;
use slint_mvvm::view_collection_binding::ViewCollectionBinding;
use slint_mvvm_macros::create_view_bindings;
use slint_mvvm_macros::create_view_model_collection;
use std::sync::Arc;

pub struct ProcessSelectorViewModel {
    view_binding: Arc<ViewBinding<MainWindowView>>,
    full_process_list_collection: ViewCollectionBinding<ProcessViewData, ProcessInfo, MainWindowView>,
    windowed_process_list_collection: ViewCollectionBinding<ProcessViewData, ProcessInfo, MainWindowView>,
    engine_execution_context: Arc<EngineExecutionContext>,
}

impl ProcessSelectorViewModel {
    pub fn register(dependency_container: &DependencyContainer) {
        dependency_container.resolve_all(Self::on_dependencies_resolved);
    }

    fn on_dependencies_resolved(
        dependency_container: DependencyContainer,
        (view_binding, engine_execution_context): (Arc<ViewBinding<MainWindowView>>, Arc<EngineExecutionContext>),
    ) {
        // Create a binding that allows us to easily update the view's process list.
        let full_process_list_collection = create_view_model_collection!(
            view_binding -> MainWindowView,
            ProcessSelectorViewModelBindings -> { set_processes, get_processes },
            ProcessInfoConverter -> [],
        );

        // Create a binding that allows us to easily update the view's windowed process list.
        let windowed_process_list_collection = create_view_model_collection!(
            view_binding -> MainWindowView,
            ProcessSelectorViewModelBindings -> { set_windowed_processes, get_windowed_processes },
            ProcessInfoConverter -> [],
        );

        let view_model = Arc::new(ProcessSelectorViewModel {
            view_binding: view_binding.clone(),
            full_process_list_collection,
            windowed_process_list_collection,
            engine_execution_context,
        });

        {
            let view_model = view_model.clone();

            // Route all view bindings to Rust.
            create_view_bindings!(view_binding, {
                ProcessSelectorViewModelBindings => {
                    on_refresh_full_process_list() -> [view_model] -> Self::on_refresh_full_process_list
                    on_refresh_windowed_process_list() -> [view_model] -> Self::on_refresh_windowed_process_list
                    on_select_process(process_entry: ProcessViewData) -> [view_model] -> Self::on_select_process
                }
            });
        }

        Self::listen_for_process_change(view_model.clone());

        dependency_container.register::<ProcessSelectorViewModel>(view_model);
    }

    fn listen_for_process_change(view_model: Arc<ProcessSelectorViewModel>) {
        let engine_execution_context = view_model.engine_execution_context.clone();

        engine_execution_context.listen_for_engine_event::<ProcessChangedEvent>(move |process_changed_event| {
            Self::refresh_opened_process(view_model.clone(), process_changed_event.process_info.clone());
        });
    }

    fn refresh_opened_process(
        view_model: Arc<ProcessSelectorViewModel>,
        process_info: Option<OpenedProcessInfo>,
    ) {
        view_model
            .view_binding
            .execute_on_ui_thread(move |main_window_view, _| {
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

    fn on_refresh_full_process_list(view_model: Arc<ProcessSelectorViewModel>) {
        let list_all_processes_request = ProcessListRequest {
            require_windowed: false,
            search_name: None,
            match_case: false,
            limit: None,
            fetch_icons: true,
        };
        let engine_execution_context = view_model.engine_execution_context.clone();

        list_all_processes_request.send(&engine_execution_context, move |process_list_response| {
            view_model
                .full_process_list_collection
                .update_from_source(process_list_response.processes);
        });
    }

    fn on_refresh_windowed_process_list(view_model: Arc<ProcessSelectorViewModel>) {
        let list_windowed_processes_request = ProcessListRequest {
            require_windowed: true,
            search_name: None,
            match_case: false,
            limit: None,
            fetch_icons: true,
        };
        let engine_execution_context = view_model.engine_execution_context.clone();

        list_windowed_processes_request.send(&engine_execution_context, move |process_list_response| {
            view_model
                .windowed_process_list_collection
                .update_from_source(process_list_response.processes);
        });
    }

    fn on_select_process(
        view_model: Arc<ProcessSelectorViewModel>,
        process_entry: ProcessViewData,
    ) {
        let process_open_request = ProcessOpenRequest {
            process_id: Some(process_entry.process_id as u32),
            search_name: None,
            match_case: false,
        };
        let engine_execution_context = view_model.engine_execution_context.clone();

        process_open_request.send(&engine_execution_context, move |process_open_response| {
            Self::refresh_opened_process(view_model.clone(), process_open_response.opened_process_info)
        });
    }
}
