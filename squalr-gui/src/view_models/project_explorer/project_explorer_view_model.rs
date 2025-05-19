use crate::MainWindowView;
use crate::ProjectExplorerViewModelBindings;
use crate::ProjectViewData;
use crate::view_models::project_explorer::project_info_comparer::ProjectInfoComparer;
use crate::view_models::project_explorer::project_info_converter::ProjectInfoConverter;
use slint::ComponentHandle;
use slint::SharedString;
use slint_mvvm::convert_to_view_data::ConvertToViewData;
use slint_mvvm::view_binding::ViewBinding;
use slint_mvvm::view_collection_binding::ViewCollectionBinding;
use slint_mvvm_macros::create_view_bindings;
use slint_mvvm_macros::create_view_model_collection;
use squalr_engine::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use squalr_engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::commands::project::close::project_close_request::ProjectCloseRequest;
use squalr_engine_api::commands::project::create::project_create_request::ProjectCreateRequest;
use squalr_engine_api::commands::project::export::project_export_request::ProjectExportRequest;
use squalr_engine_api::commands::project::list::project_list_request::ProjectListRequest;
use squalr_engine_api::commands::project::open::project_open_request::ProjectOpenRequest;
use squalr_engine_api::commands::project::rename::project_rename_request::ProjectRenameRequest;
use squalr_engine_api::commands::project::save::project_save_request::ProjectSaveRequest;
use squalr_engine_api::dependency_injection::dependency_container::DependencyContainer;
use squalr_engine_api::events::project::closed::project_closed_event::ProjectClosedEvent;
use squalr_engine_api::events::project::created::project_created_event::ProjectCreatedEvent;
use squalr_engine_api::events::project::deleted::project_deleted_event::ProjectDeletedEvent;
use squalr_engine_api::structures::projects::project_info::ProjectInfo;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

pub struct ProjectExplorerViewModel {
    view_binding: Arc<ViewBinding<MainWindowView>>,
    project_list_collection: ViewCollectionBinding<ProjectViewData, ProjectInfo, MainWindowView>,
    engine_execution_context: Arc<EngineExecutionContext>,
}

impl ProjectExplorerViewModel {
    pub fn register(dependency_container: &DependencyContainer) {
        dependency_container.resolve_all(Self::on_dependencies_resolved);
    }

    fn on_dependencies_resolved(
        dependency_container: DependencyContainer,
        (view_binding, engine_execution_context): (Arc<ViewBinding<MainWindowView>>, Arc<EngineExecutionContext>),
    ) {
        // Create a binding that allows us to easily update the view's project list.
        let project_list_collection = create_view_model_collection!(
            view_binding -> MainWindowView,
            ProjectExplorerViewModelBindings -> { set_projects, get_projects },
            ProjectInfoConverter -> [],
            ProjectInfoComparer -> [],
        );

        let view_model = Arc::new(ProjectExplorerViewModel {
            view_binding: view_binding.clone(),
            project_list_collection: project_list_collection.clone(),
            engine_execution_context: engine_execution_context.clone(),
        });

        Self::on_refresh_project_list(view_model.clone());

        {
            let view_model = view_model.clone();

            // Route all view bindings to Rust.
            create_view_bindings!(view_binding, {
                ProjectExplorerViewModelBindings => {
                    on_refresh_project_list() -> [view_model] -> Self::on_refresh_project_list
                    on_browse_for_project() -> [view_model] -> Self::on_browse_for_project
                    on_open_project(project_entry: ProjectViewData) -> [view_model] -> Self::on_open_project
                    on_close_opened_project() -> [view_model] -> Self::on_close_opened_project
                    on_save_opened_project() -> [view_model] -> Self::on_save_opened_project
                    on_export_project(project_entry: ProjectViewData) -> [view_model] -> Self::on_export_project
                    on_rename_project(project_entry: ProjectViewData, new_project_name: SharedString) -> [view_model] -> Self::on_rename_project
                    on_create_new_project() -> [view_model] -> Self::on_create_new_project
                }
            });
        }

        Self::listen_for_project_changes(view_model.clone());

        dependency_container.register::<ProjectExplorerViewModel>(view_model);
    }

    fn listen_for_project_changes(view_model: Arc<ProjectExplorerViewModel>) {
        {
            let engine_execution_context = view_model.engine_execution_context.clone();
            let view_model = view_model.clone();

            engine_execution_context.listen_for_engine_event::<ProjectDeletedEvent>(move |_process_deleted_event| {
                Self::on_refresh_project_list(view_model.clone());
            });
        }
        {
            let engine_execution_context = view_model.engine_execution_context.clone();
            let view_model = view_model.clone();

            engine_execution_context.listen_for_engine_event::<ProjectCreatedEvent>(move |_process_created_event| {
                Self::on_refresh_project_list(view_model.clone());
            });
        }
        {
            let view_binding = view_model.view_binding.clone();

            view_model
                .engine_execution_context
                .listen_for_engine_event::<ProjectClosedEvent>(move |_process_closed_event| {
                    view_binding.execute_on_ui_thread(move |main_window_view, _| {
                        let project_explorer_bindings = main_window_view.global::<ProjectExplorerViewModelBindings>();

                        project_explorer_bindings.set_is_project_open(false);
                        project_explorer_bindings.set_opened_project(ProjectViewData::default());
                    });
                });
        }
    }

    fn on_refresh_project_list(view_model: Arc<ProjectExplorerViewModel>) {
        let engine_execution_context = &view_model.engine_execution_context;
        let project_list_collection = view_model.project_list_collection.clone();
        let list_all_projects_request = ProjectListRequest {};

        list_all_projects_request.send(engine_execution_context, move |project_list_response| {
            project_list_collection.update_from_source(project_list_response.projects_info);
        });
    }

    fn on_browse_for_project(view_model: Arc<ProjectExplorerViewModel>) {
        //
    }

    fn on_open_project(
        view_model: Arc<ProjectExplorerViewModel>,
        project_entry: ProjectViewData,
    ) {
        let view_binding = view_model.view_binding.clone();
        let engine_execution_context = &view_model.engine_execution_context;
        let project_open_request = ProjectOpenRequest {
            project_path: Some(PathBuf::from_str(&project_entry.path.to_string()).unwrap_or_default()),
            project_name: None,
        };

        project_open_request.send(engine_execution_context, move |project_open_response| {
            view_binding.execute_on_ui_thread(move |main_window_view, _| {
                let project_explorer_bindings = main_window_view.global::<ProjectExplorerViewModelBindings>();

                project_explorer_bindings.set_is_project_open(project_open_response.opened_project_info.is_some());

                if let Some(opened_project_info) = project_open_response.opened_project_info {
                    project_explorer_bindings.set_opened_project(ProjectInfoConverter {}.convert_to_view_data(&opened_project_info));
                }
            });
        });
    }

    fn on_close_opened_project(view_model: Arc<ProjectExplorerViewModel>) {
        let engine_execution_context = &view_model.engine_execution_context;
        let project_close_request = ProjectCloseRequest {};

        project_close_request.send(engine_execution_context, move |_project_close_response| {});
    }

    fn on_save_opened_project(view_model: Arc<ProjectExplorerViewModel>) {
        let engine_execution_context = &view_model.engine_execution_context;
        let project_save_request = ProjectSaveRequest {};

        project_save_request.send(engine_execution_context, move |_project_save_response| {});
    }

    fn on_export_project(
        view_model: Arc<ProjectExplorerViewModel>,
        project_entry: ProjectViewData,
    ) {
        let engine_execution_context = &view_model.engine_execution_context;
        let project_export_request = ProjectExportRequest {
            project_path: Some(PathBuf::from_str(&project_entry.path.to_string()).unwrap_or_default()),
            project_name: None,
            open_export_folder: true,
        };

        project_export_request.send(engine_execution_context, move |_project_export_response| {});
    }

    fn on_rename_project(
        view_model: Arc<ProjectExplorerViewModel>,
        project_entry: ProjectViewData,
        new_project_name: SharedString,
    ) {
        let engine_execution_context = &view_model.engine_execution_context;
        let project_rename_request = ProjectRenameRequest {
            project_path: PathBuf::from_str(&project_entry.path.to_string()).unwrap_or_default(),
            new_project_name: new_project_name.into(),
        };

        project_rename_request.send(engine_execution_context, move |_project_rename_response| {});
    }

    fn on_create_new_project(view_model: Arc<ProjectExplorerViewModel>) {
        let engine_execution_context = &view_model.engine_execution_context;
        let project_create_request = ProjectCreateRequest {
            project_path: None,
            project_name: None,
        };

        project_create_request.send(engine_execution_context, move |_project_create_response| {});
    }
}
