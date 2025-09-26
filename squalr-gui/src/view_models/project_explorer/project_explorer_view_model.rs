use crate::MainWindowView;
use crate::ProjectExplorerViewModelBindings;
use crate::ProjectInfoViewData;
use crate::ProjectItemViewData;
use crate::converters::project_info_converter::ProjectInfoConverter;
use crate::converters::project_item_converter::ProjectItemConverter;
use squalr_engine_api::commands::engine_command_request::EngineCommandRequest;
use squalr_engine_api::commands::project::close::project_close_request::ProjectCloseRequest;
use squalr_engine_api::commands::project::create::project_create_request::ProjectCreateRequest;
use squalr_engine_api::commands::project::export::project_export_request::ProjectExportRequest;
use squalr_engine_api::commands::project::list::project_list_request::ProjectListRequest;
use squalr_engine_api::commands::project::open::project_open_request::ProjectOpenRequest;
use squalr_engine_api::commands::project::rename::project_rename_request::ProjectRenameRequest;
use squalr_engine_api::commands::project::save::project_save_request::ProjectSaveRequest;
use squalr_engine_api::commands::project_items::activate::project_items_activate_request::ProjectItemsActivateRequest;
use squalr_engine_api::commands::project_items::list::project_items_list_request::ProjectItemsListRequest;
use squalr_engine_api::dependency_injection::dependency_container::DependencyContainer;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::events::project::closed::project_closed_event::ProjectClosedEvent;
use squalr_engine_api::events::project::created::project_created_event::ProjectCreatedEvent;
use squalr_engine_api::events::project::deleted::project_deleted_event::ProjectDeletedEvent;
use squalr_engine_api::events::project_items::changed::project_items_changed_event::ProjectItemsChangedEvent;
use squalr_engine_api::structures::projects::project_info::ProjectInfo;
use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
use slint::ComponentHandle;
use slint::SharedString;
use slint_mvvm::convert_to_view_data::ConvertToViewData;
use slint_mvvm::view_binding::ViewBinding;
use slint_mvvm::view_collection_binding::ViewCollectionBinding;
use slint_mvvm_macros::create_view_bindings;
use slint_mvvm_macros::create_view_model_collection;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct ProjectExplorerViewModel {
    view_binding: Arc<ViewBinding<MainWindowView>>,
    project_list_collection: ViewCollectionBinding<ProjectInfoViewData, ProjectInfo, MainWindowView>,
    opened_project_items_list_collection: ViewCollectionBinding<ProjectItemViewData, ProjectItem, MainWindowView>,
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
        // Create view binding to the project list.
        let project_list_collection = create_view_model_collection!(
            view_binding -> MainWindowView,
            ProjectExplorerViewModelBindings -> { set_projects, get_projects },
            ProjectInfoConverter -> [],
        );

        // Create view binding to the opened project item list.
        let opened_project_items_list_collection = create_view_model_collection!(
            view_binding -> MainWindowView,
            ProjectExplorerViewModelBindings -> { set_project_items, get_project_items },
            ProjectItemConverter -> [],
        );

        let view_model = Arc::new(ProjectExplorerViewModel {
            view_binding: view_binding.clone(),
            project_list_collection: project_list_collection.clone(),
            opened_project_items_list_collection: opened_project_items_list_collection.clone(),
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
                    on_open_project(project_entry: ProjectInfoViewData) -> [view_model] -> Self::on_open_project
                    on_close_opened_project() -> [view_model] -> Self::on_close_opened_project
                    on_save_opened_project() -> [view_model] -> Self::on_save_opened_project
                    on_export_project(project_entry: ProjectInfoViewData) -> [view_model] -> Self::on_export_project
                    on_rename_project(project_entry: ProjectInfoViewData, new_project_name: SharedString) -> [view_model] -> Self::on_rename_project
                    on_create_new_project() -> [view_model] -> Self::on_create_new_project
                    on_set_project_entry_activated(project_item_path: SharedString, is_activated: bool) -> [view_model] -> Self::on_set_project_entry_activated
                }
            });
        }

        Self::listen_for_project_changes(view_model.clone());
        Self::poll_project_items(view_model.clone());

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
                        project_explorer_bindings.set_opened_project(ProjectInfoViewData::default());
                    });
                });
        }
        {
            let engine_execution_context = view_model.engine_execution_context.clone();
            let view_model = view_model.clone();
            let opened_project_items_list_collection = view_model.opened_project_items_list_collection.clone();

            engine_execution_context.listen_for_engine_event::<ProjectItemsChangedEvent>(move |project_items_changed_event| {
                if let Some(project_root) = &project_items_changed_event.project_root {
                    // opened_project_items_list_collection.update_from_source(project_root.get_children().to_owned());
                }
            });
        }
    }

    fn poll_project_items(view_model: Arc<ProjectExplorerViewModel>) {
        // Refresh scan values on a loop. JIRA: This is so inefficient, please fix this.
        thread::spawn(move || {
            loop {
                Self::on_refresh_opened_project_items_list(view_model.clone());

                thread::sleep(Duration::from_millis(100));
            }
        });
    }

    fn on_refresh_project_list(view_model: Arc<ProjectExplorerViewModel>) {
        let engine_execution_context = &view_model.engine_execution_context;
        let project_list_collection = view_model.project_list_collection.clone();
        let list_all_projects_request = ProjectListRequest {};

        list_all_projects_request.send(engine_execution_context, move |project_list_response| {
            project_list_collection.update_from_source(project_list_response.projects_info);
        });
    }

    fn on_refresh_opened_project_items_list(view_model: Arc<ProjectExplorerViewModel>) {
        let view_binding = view_model.view_binding.clone();
        let engine_execution_context = &view_model.engine_execution_context;
        let project_open_request = ProjectItemsListRequest {};
        let opened_project_items_list_collection = view_model.opened_project_items_list_collection.clone();

        project_open_request.send(engine_execution_context, move |project_items_list_response| {
            view_binding.execute_on_ui_thread(move |main_window_view, _| {
                if let Some(project_root) = project_items_list_response.opened_project_root {
                    // opened_project_items_list_collection.update_from_source(project_root.get_children().to_owned());
                }
            });
        });
    }

    fn on_browse_for_project(view_model: Arc<ProjectExplorerViewModel>) {
        //
    }

    fn on_open_project(
        view_model: Arc<ProjectExplorerViewModel>,
        project_entry: ProjectInfoViewData,
    ) {
        let view_binding = view_model.view_binding.clone();
        let engine_execution_context = &view_model.engine_execution_context;
        let project_open_request = ProjectOpenRequest {
            project_path: Some(PathBuf::from_str(&project_entry.path.to_string()).unwrap_or_default()),
            project_name: None,
        };
        let opened_project_items_list_collection = view_model.opened_project_items_list_collection.clone();

        project_open_request.send(engine_execution_context, move |project_open_response| {
            view_binding.execute_on_ui_thread(move |main_window_view, _| {
                let project_explorer_bindings = main_window_view.global::<ProjectExplorerViewModelBindings>();

                project_explorer_bindings.set_is_project_open(project_open_response.opened_project_info.is_some());

                if let Some(opened_project_info) = project_open_response.opened_project_info {
                    project_explorer_bindings.set_opened_project(ProjectInfoConverter {}.convert_to_view_data(&opened_project_info));
                }

                if let Some(project_root) = project_open_response.opened_project_root {
                    // opened_project_items_list_collection.update_from_source(project_root.get_children().to_owned());
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
        project_entry: ProjectInfoViewData,
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
        project_entry: ProjectInfoViewData,
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

    fn on_set_project_entry_activated(
        view_model: Arc<ProjectExplorerViewModel>,
        project_item_path: SharedString,
        is_activated: bool,
    ) {
        let engine_execution_context = &view_model.engine_execution_context;
        let project_items_activate_request = ProjectItemsActivateRequest {
            project_item_paths: vec![project_item_path.to_string()],
            is_activated,
        };

        project_items_activate_request.send(engine_execution_context, move |_project_items_activate_response| {});
    }
}
