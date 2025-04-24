use crate::MainWindowView;
use crate::ProjectExplorerViewModelBindings;
use crate::ProjectViewData;
use crate::view_models::project_explorer::project_info_comparer::ProjectInfoComparer;
use crate::view_models::project_explorer::project_info_converter::ProjectInfoConverter;
use slint::ComponentHandle;
use slint::SharedString;
use slint_mvvm::view_binding::ViewBinding;
use slint_mvvm::view_collection_binding::ViewCollectionBinding;
use slint_mvvm_macros::create_view_bindings;
use slint_mvvm_macros::create_view_model_collection;
use squalr_engine::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use squalr_engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::commands::project::create::project_create_request::ProjectCreateRequest;
use squalr_engine_api::commands::project::list::project_list_request::ProjectListRequest;
use squalr_engine_api::commands::project::open::project_open_request::ProjectOpenRequest;
use squalr_engine_api::commands::project::rename::project_rename_request::ProjectRenameRequest;
use squalr_engine_api::events::project::closed::project_closed_event::ProjectClosedEvent;
use squalr_engine_api::events::project::created::project_created_event::ProjectCreatedEvent;
use squalr_engine_api::events::project::deleted::project_deleted_event::ProjectDeletedEvent;
use squalr_engine_api::structures::projects::project_info::ProjectInfo;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

pub struct ProjectExplorerViewModel {
    view_binding: ViewBinding<MainWindowView>,
    project_list_collection: ViewCollectionBinding<ProjectViewData, ProjectInfo, MainWindowView>,
    engine_execution_context: Arc<EngineExecutionContext>,
}

impl ProjectExplorerViewModel {
    pub fn new(
        view_binding: ViewBinding<MainWindowView>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) -> Arc<Self> {
        // Create a binding that allows us to easily update the view's project list.
        let project_list_collection = create_view_model_collection!(
            view_binding -> MainWindowView,
            ProjectExplorerViewModelBindings -> { set_projects, get_projects },
            ProjectInfoConverter -> [],
            ProjectInfoComparer -> [],
        );

        let view = Arc::new(ProjectExplorerViewModel {
            view_binding: view_binding.clone(),
            project_list_collection: project_list_collection.clone(),
            engine_execution_context: engine_execution_context.clone(),
        });

        Self::on_refresh_project_list(project_list_collection.clone(), engine_execution_context.clone());

        // Route all view bindings to Rust.
        create_view_bindings!(view_binding, {
            ProjectExplorerViewModelBindings => {
                on_refresh_project_list() -> [project_list_collection, engine_execution_context] -> Self::on_refresh_project_list
                on_open_project(project_entry: ProjectViewData) -> [view_binding, engine_execution_context] -> Self::on_open_project
                on_rename_project(project_entry: ProjectViewData, new_project_name: SharedString) -> [engine_execution_context] -> Self::on_rename_project
                on_create_new_project() -> [engine_execution_context] -> Self::on_create_new_project
            }
        });

        view.listen_for_project_changes();

        view
    }

    fn listen_for_project_changes(&self) {
        {
            let project_list_collection = self.project_list_collection.clone();
            let engine_execution_context = self.engine_execution_context.clone();

            self.engine_execution_context
                .listen_for_engine_event::<ProjectDeletedEvent>(move |_process_deleted_event| {
                    Self::on_refresh_project_list(project_list_collection.clone(), engine_execution_context.clone());
                });
        }
        {
            let project_list_collection = self.project_list_collection.clone();
            let engine_execution_context = self.engine_execution_context.clone();

            self.engine_execution_context
                .listen_for_engine_event::<ProjectCreatedEvent>(move |_process_created_event| {
                    Self::on_refresh_project_list(project_list_collection.clone(), engine_execution_context.clone());
                });
        }
        {
            let view_binding = self.view_binding.clone();

            self.engine_execution_context
                .listen_for_engine_event::<ProjectClosedEvent>(move |_process_closed_event| {
                    view_binding.execute_on_ui_thread(move |main_window_view, _| {
                        let project_explorer_bindings = main_window_view.global::<ProjectExplorerViewModelBindings>();

                        project_explorer_bindings.set_is_project_open(false);
                    });
                });
        }
    }

    fn on_refresh_project_list(
        project_list_collection: ViewCollectionBinding<ProjectViewData, ProjectInfo, MainWindowView>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) {
        let list_all_projects_request = ProjectListRequest {};

        list_all_projects_request.send(&engine_execution_context, move |project_list_response| {
            project_list_collection.update_from_source(project_list_response.projects_info);
        });
    }

    fn on_open_project(
        view_binding: ViewBinding<MainWindowView>,
        engine_execution_context: Arc<EngineExecutionContext>,
        project_entry: ProjectViewData,
    ) {
        let project_open_request = ProjectOpenRequest {
            project_path: Some(PathBuf::from_str(&project_entry.path.to_string()).unwrap_or_default()),
            project_name: None,
        };
        let view_binding = view_binding.clone();

        project_open_request.send(&engine_execution_context, move |project_open_response| {
            view_binding.execute_on_ui_thread(move |main_window_view, _| {
                let project_explorer_bindings = main_window_view.global::<ProjectExplorerViewModelBindings>();

                project_explorer_bindings.set_is_project_open(project_open_response.opened_project_info.is_some());
            });
        });
    }

    fn on_rename_project(
        engine_execution_context: Arc<EngineExecutionContext>,
        project_entry: ProjectViewData,
        new_project_name: SharedString,
    ) {
        let project_rename_request = ProjectRenameRequest {
            project_path: PathBuf::from_str(&project_entry.path.to_string()).unwrap_or_default(),
            new_project_name: new_project_name.into(),
        };

        project_rename_request.send(&engine_execution_context, move |_project_rename_response| {});
    }

    fn on_create_new_project(engine_execution_context: Arc<EngineExecutionContext>) {
        let project_create_request = ProjectCreateRequest {
            project_path: None,
            project_name: None,
        };

        project_create_request.send(&engine_execution_context, move |_project_create_response| {});
    }
}
