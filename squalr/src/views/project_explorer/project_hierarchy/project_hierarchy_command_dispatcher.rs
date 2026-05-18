use crate::app_context::AppContext;
use crate::views::project_explorer::project_hierarchy::{
    project_item_rename_request_builder::ProjectItemRenameRequestBuilder, view_data::project_hierarchy_view_data::ProjectHierarchyViewData,
};
use squalr_engine_api::commands::project_items::strip_symbol::project_items_strip_symbol_request::ProjectItemsStripSymbolRequest;
use squalr_engine_api::commands::project_items::write_value::project_items_write_value_request::ProjectItemsWriteValueRequest;
use squalr_engine_api::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_values::anonymous_value_string::AnonymousValueString;
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

    pub fn rename_project_item(
        &self,
        project_item_path: PathBuf,
        project_item_type_id: String,
        edited_name: String,
    ) {
        let Some(project_item_rename_request) = ProjectItemRenameRequestBuilder::build(&project_item_path, &project_item_type_id, edited_name.trim()) else {
            return;
        };
        let app_context = self.app_context.clone();
        let project_hierarchy_view_data = self.project_hierarchy_view_data.clone();
        let previous_project_item_path = project_item_path.clone();

        project_item_rename_request.send(&self.app_context.engine_unprivileged_state, move |project_items_rename_response| {
            if !project_items_rename_response.success {
                log::warn!("Project item rename command failed in hierarchy F2 rename flow.");
                return;
            }

            ProjectHierarchyViewData::finish_project_item_rename(
                project_hierarchy_view_data.clone(),
                &previous_project_item_path,
                &project_items_rename_response.renamed_project_item_path,
            );
            ProjectHierarchyViewData::refresh_project_items(project_hierarchy_view_data, app_context);
        });
    }

    pub fn commit_project_item_value_edit(
        &self,
        project_item_path: PathBuf,
        value_field_name: String,
        validation_data_type_ref: DataTypeRef,
        value_edit: AnonymousValueString,
    ) -> bool {
        if let Err(error) = self
            .app_context
            .engine_unprivileged_state
            .deanonymize_value_string(&validation_data_type_ref, &value_edit)
        {
            log::warn!("Failed to commit project hierarchy runtime value edit: {}", error);
            return false;
        }

        ProjectItemsWriteValueRequest {
            project_item_path,
            field_name: value_field_name,
            anonymous_value_string: value_edit,
        }
        .send(&self.app_context.engine_unprivileged_state, |project_items_write_value_response| {
            if !project_items_write_value_response.success {
                log::warn!("Project item write-value command failed while committing value edit takeover.");
            }
        });

        true
    }
}
