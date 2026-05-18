use crate::app_context::AppContext;
use crate::views::project_explorer::project_hierarchy::view_data::project_hierarchy_view_data::ProjectHierarchyViewData;
use squalr_engine_api::commands::project_items::strip_symbol::project_items_strip_symbol_request::ProjectItemsStripSymbolRequest;
use squalr_engine_api::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub struct ProjectHierarchyCommandDispatcher {
    app_context: Arc<AppContext>,
    project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
}

impl ProjectHierarchyCommandDispatcher {
    pub fn new(
        app_context: Arc<AppContext>,
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
    ) -> Self {
        Self {
            app_context,
            project_hierarchy_view_data,
        }
    }

    pub fn strip_symbol_information(
        &self,
        project_item_paths: Vec<PathBuf>,
        details_refresh_callback: Option<Arc<dyn Fn() + Send + Sync>>,
    ) {
        ProjectItemsStripSymbolRequest { project_item_paths }.send(&self.app_context.engine_unprivileged_state, {
            let app_context = self.app_context.clone();
            let project_hierarchy_view_data = self.project_hierarchy_view_data.clone();

            move |project_items_strip_symbol_response| {
                if !project_items_strip_symbol_response.success {
                    log::warn!(
                        "Project item strip-symbol command failed: {}.",
                        project_items_strip_symbol_response
                            .error
                            .as_deref()
                            .unwrap_or("unknown error")
                    );
                    return;
                }

                if project_items_strip_symbol_response.stripped_project_item_count == 0 {
                    return;
                }

                ProjectHierarchyViewData::refresh_project_items(project_hierarchy_view_data, app_context);

                if let Some(details_refresh_callback) = details_refresh_callback {
                    details_refresh_callback();
                }
            }
        });
    }
}
