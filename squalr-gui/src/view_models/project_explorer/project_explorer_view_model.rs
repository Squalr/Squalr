use crate::MainWindowView;
use crate::ProjectExplorerViewModelBindings;
use crate::ProjectViewData;
use crate::view_models::project_explorer::project_info_comparer::ProjectInfoComparer;
use crate::view_models::project_explorer::project_info_converter::ProjectInfoConverter;
use slint::ComponentHandle;
use slint_mvvm::view_binding::ViewBinding;
use slint_mvvm::view_collection_binding::ViewCollectionBinding;
use slint_mvvm::view_data_converter::ViewDataConverter;
use slint_mvvm_macros::create_view_bindings;
use slint_mvvm_macros::create_view_model_collection;
use squalr_engine::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use squalr_engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::commands::project::list::project_list_request::ProjectListRequest;
use squalr_engine_api::structures::projects::project_info::ProjectInfo;
use std::sync::Arc;

pub struct ProjectExplorerViewModel {
    _view_binding: ViewBinding<MainWindowView>,
    _project_list_collection: ViewCollectionBinding<ProjectViewData, ProjectInfo, MainWindowView>,
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
            _view_binding: view_binding.clone(),
            _project_list_collection: project_list_collection.clone(),
            engine_execution_context: engine_execution_context.clone(),
        });

        // Route all view bindings to Rust.
        create_view_bindings!(view_binding, {
            ProjectExplorerViewModelBindings => {
                on_refresh_project_list() -> [project_list_collection, engine_execution_context] -> Self::on_refresh_project_list
                on_select_project(project_entry: ProjectViewData) -> [view_binding, engine_execution_context] -> Self::on_select_project
            }
        });

        view
    }

    fn on_refresh_project_list(
        project_list_collection: ViewCollectionBinding<ProjectViewData, ProjectInfo, MainWindowView>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) {
        let list_all_projectes_request = ProjectListRequest {};

        list_all_projectes_request.send(&engine_execution_context, move |project_list_response| {
            project_list_collection.update_from_source(project_list_response.projects);
        });
    }

    fn on_select_project(
        view_binding: ViewBinding<MainWindowView>,
        engine_execution_context: Arc<EngineExecutionContext>,
        project_entry: ProjectViewData,
    ) {
        /*
        let open_project_command = ProjectOpenRequest {
            project_id: Some(project_entry.project_id as u32),
        };

        open_project_command.send(&engine_execution_context, move |project_open_response| {});*/
    }
}
