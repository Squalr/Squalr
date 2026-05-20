use crate::{
    app_context::AppContext,
    views::project_explorer::{
        project_hierarchy::{project_hierarchy_view::ProjectHierarchyView, view_data::project_hierarchy_view_data::ProjectHierarchyViewData},
        project_selector::{project_selector_view::ProjectSelectorView, view_data::project_selector_view_data::ProjectSelectorViewData},
    },
};
use eframe::egui::{Align, Layout, Response, Ui, Widget};
use squalr_engine_api::{
    dependency_injection::dependency::Dependency,
    engine::engine_execution_context::EngineExecutionContext,
    events::{
        project::{catalog_changed::project_catalog_changed_event::ProjectCatalogChangedEvent, closed::project_closed_event::ProjectClosedEvent},
        project_items::changed::project_items_changed_event::ProjectItemsChangedEvent,
    },
};
use std::sync::Arc;

#[derive(Clone)]
pub struct ProjectExplorerView {
    app_context: Arc<AppContext>,
    project_selector_view: ProjectSelectorView,
    project_hierarchy_view: ProjectHierarchyView,
    _project_selector_view_data: Dependency<ProjectSelectorViewData>,
    _project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
}

impl ProjectExplorerView {
    pub const WINDOW_ID: &'static str = "window_project_explorer";

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let project_selector_view_data = app_context
            .dependency_container
            .register(ProjectSelectorViewData::new());
        let project_hierarchy_view_data = app_context
            .dependency_container
            .register(ProjectHierarchyViewData::new());
        let project_selector_view = ProjectSelectorView::new(app_context.clone());
        let project_hierarchy_view = ProjectHierarchyView::new(app_context.clone());

        Self::install_project_refresh_listeners(&app_context, &project_selector_view_data, &project_hierarchy_view_data);

        Self {
            app_context,
            project_selector_view,
            project_hierarchy_view,
            _project_selector_view_data: project_selector_view_data,
            _project_hierarchy_view_data: project_hierarchy_view_data,
        }
    }

    fn install_project_refresh_listeners(
        app_context: &Arc<AppContext>,
        project_selector_view_data: &Dependency<ProjectSelectorViewData>,
        project_hierarchy_view_data: &Dependency<ProjectHierarchyViewData>,
    ) {
        let project_hierarchy_view_data_for_items = project_hierarchy_view_data.clone();
        let app_context_for_items = app_context.clone();
        app_context
            .engine_unprivileged_state
            .listen_for_engine_event::<ProjectItemsChangedEvent>(move |_project_items_changed_event| {
                ProjectHierarchyViewData::refresh_project_items(project_hierarchy_view_data_for_items.clone(), app_context_for_items.clone());
            });

        let project_hierarchy_view_data_for_close = project_hierarchy_view_data.clone();
        let app_context_for_close = app_context.clone();
        app_context
            .engine_unprivileged_state
            .listen_for_engine_event::<ProjectClosedEvent>(move |_project_closed_event| {
                ProjectHierarchyViewData::refresh_project_items(project_hierarchy_view_data_for_close.clone(), app_context_for_close.clone());
            });

        let project_selector_view_data_for_catalog = project_selector_view_data.clone();
        let app_context_for_catalog = app_context.clone();
        app_context
            .engine_unprivileged_state
            .listen_for_engine_event::<ProjectCatalogChangedEvent>(move |_project_catalog_changed_event| {
                ProjectSelectorViewData::refresh_project_list(project_selector_view_data_for_catalog.clone(), app_context_for_catalog.clone());
            });
    }
}

impl Widget for ProjectExplorerView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                let project_manager = self.app_context.engine_unprivileged_state.get_project_manager();
                let opened_project = project_manager.get_opened_project();
                let has_opened_project = match opened_project.read() {
                    Ok(opened_project) => opened_project.is_some(),
                    Err(error) => {
                        log::error!("Failed to acquire opened project lock: {}", error);
                        return;
                    }
                };

                if has_opened_project {
                    user_interface.add(self.project_hierarchy_view.clone());
                } else {
                    user_interface.add(self.project_selector_view.clone());
                }
            })
            .response;

        response
    }
}
