use crate::app_context::AppContext;
use crate::views::project_explorer::project_hierarchy::{
    project_hierarchy_command_dispatcher::ProjectHierarchyCommandDispatcher, project_item_preview_details::ProjectItemPreviewDetails,
    view_data::project_hierarchy_view_data::ProjectHierarchyViewData,
};
use crate::views::struct_viewer::view_data::{struct_viewer_focus_target::StructViewerFocusTarget, struct_viewer_view_data::StructViewerViewData};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::data_values::{
    anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType,
    data_value::DataValue,
};
use squalr_engine_api::structures::details::{DetailsEdit, DetailsEditOperation, DetailsEditPlan, DetailsValue};
use squalr_engine_api::structures::projects::project_items::{
    details::{ProjectItemDetailsEditPlanner, ProjectItemDetailsProjection},
    project_item::ProjectItem,
    project_item_ref::ProjectItemRef,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub struct ProjectHierarchyDetailsFocus {
    app_context: Arc<AppContext>,
    project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
    struct_viewer_view_data: Dependency<StructViewerViewData>,
}

impl ProjectHierarchyDetailsFocus {
    pub fn new(
        app_context: Arc<AppContext>,
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        struct_viewer_view_data: Dependency<StructViewerViewData>,
    ) -> Self {
        Self {
            app_context,
            project_hierarchy_view_data,
            struct_viewer_view_data,
        }
    }

    pub fn focus_selected_project_items(&self) {
        let selected_project_item_paths = self
            .project_hierarchy_view_data
            .read("Project hierarchy selected project items for struct viewer focus")
            .map(|project_hierarchy_view_data| project_hierarchy_view_data.collect_selected_project_item_paths_in_tree_order())
            .unwrap_or_default();

        self.focus_project_item_paths(selected_project_item_paths);
    }

    pub fn refocus_project_item_details_if_preview_changed(
        &self,
        preview_fields_by_project_item_path: &HashMap<PathBuf, (String, String)>,
    ) {
        let focused_project_item_paths = self
            .struct_viewer_view_data
            .read("Project hierarchy details preview refresh focus target")
            .and_then(|struct_viewer_view_data| match struct_viewer_view_data.get_focus_target() {
                Some(StructViewerFocusTarget::ProjectHierarchy { project_item_paths }) => Some(project_item_paths.clone()),
                _ => None,
            })
            .unwrap_or_default();

        if focused_project_item_paths.is_empty()
            || !focused_project_item_paths
                .iter()
                .any(|project_item_path| preview_fields_by_project_item_path.contains_key(project_item_path))
        {
            return;
        }

        self.focus_project_item_paths(focused_project_item_paths);
    }

    pub fn build_project_item_details_refresh_callback(
        &self,
        project_item_paths: Vec<PathBuf>,
    ) -> Arc<dyn Fn() + Send + Sync> {
        let details_focus = self.clone();

        Arc::new(move || {
            details_focus.focus_project_item_paths(project_item_paths.clone());
        })
    }

    fn command_dispatcher(&self) -> ProjectHierarchyCommandDispatcher {
        ProjectHierarchyCommandDispatcher::new(self.app_context.clone(), self.project_hierarchy_view_data.clone())
    }

    pub fn focus_project_item_paths(
        &self,
        project_item_paths: Vec<PathBuf>,
    ) {
        if project_item_paths.is_empty() {
            StructViewerViewData::clear_focus(self.struct_viewer_view_data.clone());
            return;
        }
        let project_manager = self.app_context.engine_unprivileged_state.get_project_manager();
        let opened_project_lock = project_manager.get_opened_project();
        let opened_project_guard = match opened_project_lock.read() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                log::error!("Failed to acquire opened project lock while focusing project item details: {}", error);
                return;
            }
        };
        let Some(opened_project) = opened_project_guard.as_ref() else {
            StructViewerViewData::clear_focus(self.struct_viewer_view_data.clone());
            return;
        };
        let preview_project_item_map = self
            .project_hierarchy_view_data
            .read("Project hierarchy details preview project items")
            .map(|project_hierarchy_view_data| {
                project_hierarchy_view_data
                    .project_items
                    .iter()
                    .map(|(project_item_ref, project_item)| (project_item_ref.get_project_item_path().clone(), project_item.clone()))
                    .collect::<HashMap<PathBuf, ProjectItem>>()
            })
            .unwrap_or_default();
        let selected_project_items = project_item_paths
            .iter()
            .filter_map(|project_item_path| {
                let mut selected_project_item = opened_project
                    .get_project_items()
                    .get(&ProjectItemRef::new(project_item_path.clone()))
                    .cloned()?;

                if let Some(preview_project_item) = preview_project_item_map.get(project_item_path) {
                    ProjectItemPreviewDetails::copy_project_item_preview_fields(preview_project_item, &mut selected_project_item);
                }

                Some(selected_project_item)
            })
            .collect::<Vec<ProjectItem>>();

        if selected_project_items.is_empty() {
            StructViewerViewData::clear_focus(self.struct_viewer_view_data.clone());
            return;
        }

        if selected_project_items.len() == 1 {
            if let Some(selected_project_item) = selected_project_items.into_iter().next() {
                let details_edit_callback = self.build_project_item_details_projection_edit_callback(project_item_paths.clone());
                let details_projection = ProjectItemDetailsProjection::build(&selected_project_item, project_item_paths[0].to_string_lossy().to_string());

                StructViewerViewData::focus_details_projection_with_focus_target(
                    self.struct_viewer_view_data.clone(),
                    self.app_context.engine_unprivileged_state.clone(),
                    details_projection,
                    details_edit_callback,
                    Some(StructViewerFocusTarget::ProjectHierarchy { project_item_paths }),
                );
            }
        } else {
            let details_edit_callback = self.build_project_item_details_projection_edit_callback(project_item_paths.clone());
            let selected_project_item_details_projections = selected_project_items
                .into_iter()
                .zip(project_item_paths.iter())
                .map(|(selected_project_item, project_item_path)| {
                    ProjectItemDetailsProjection::build(&selected_project_item, project_item_path.to_string_lossy().to_string())
                })
                .collect::<Vec<_>>();
            StructViewerViewData::focus_details_projections_with_focus_target(
                self.struct_viewer_view_data.clone(),
                self.app_context.engine_unprivileged_state.clone(),
                selected_project_item_details_projections,
                details_edit_callback,
                Some(StructViewerFocusTarget::ProjectHierarchy { project_item_paths }),
            );
        }
    }

    fn build_project_item_details_projection_edit_callback(
        &self,
        project_item_paths: Vec<PathBuf>,
    ) -> Arc<dyn Fn(DetailsEdit) + Send + Sync> {
        let details_focus = self.clone();

        Arc::new(move |details_edit: DetailsEdit| {
            let Some(edit_plan) = details_focus.plan_project_item_details_edit(&project_item_paths, &details_edit) else {
                return;
            };
            let should_refocus_details = edit_plan
                .get_operations()
                .iter()
                .any(|operation| matches!(operation, DetailsEditOperation::RefreshProjection { .. }));

            for operation in edit_plan.get_operations() {
                match operation {
                    DetailsEditOperation::Noop { reason } => {
                        if let Some(reason) = reason {
                            log::debug!("Skipping project item details edit: {}", reason);
                        }
                    }
                    DetailsEditOperation::Reject { reason } => {
                        log::warn!("Rejected project item details edit: {}", reason);
                    }
                    DetailsEditOperation::RefreshProjection { .. } => {}
                    DetailsEditOperation::UpdateStoredField { source, value, .. } => {
                        details_focus
                            .command_dispatcher()
                            .update_project_item_details_stored_field(&project_item_paths, source, value);
                    }
                    DetailsEditOperation::WriteRuntimeValue { source, value, .. } => {
                        let Some(anonymous_value_string) = details_focus.details_value_to_anonymous_value_string(value) else {
                            log::warn!("Failed to anonymize project item write-value command for details operation: {:?}", operation);
                            continue;
                        };

                        if !details_focus
                            .command_dispatcher()
                            .write_project_item_details_runtime_value(&project_item_paths, source, anonymous_value_string)
                        {
                            log::warn!("Failed to build project item write-value command for details operation: {:?}", operation);
                        }
                    }
                }
            }

            if should_refocus_details {
                details_focus.focus_project_item_paths(project_item_paths.clone());
            }
        })
    }

    fn plan_project_item_details_edit(
        &self,
        project_item_paths: &[PathBuf],
        details_edit: &DetailsEdit,
    ) -> Option<DetailsEditPlan> {
        let project_item_path = project_item_paths.first()?;
        let project_manager = self.app_context.engine_unprivileged_state.get_project_manager();
        let opened_project_lock = project_manager.get_opened_project();
        let opened_project_guard = match opened_project_lock.read() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                log::error!("Failed to acquire opened project lock while planning project item details edit: {}", error);
                return None;
            }
        };
        let opened_project = match opened_project_guard.as_ref() {
            Some(opened_project) => opened_project,
            None => {
                log::warn!("Cannot plan project item details edit without an opened project.");
                return None;
            }
        };
        let project_item_ref = ProjectItemRef::new(project_item_path.clone());
        let project_item = match opened_project.get_project_items().get(&project_item_ref) {
            Some(project_item) => project_item,
            None => {
                log::warn!("Cannot plan project item details edit, project item was not found: {:?}", project_item_path);
                return None;
            }
        };

        Some(ProjectItemDetailsEditPlanner::plan_edit(project_item, details_edit))
    }

    fn details_value_to_anonymous_value_string(
        &self,
        details_value: &DetailsValue,
    ) -> Option<AnonymousValueString> {
        match details_value {
            DetailsValue::AnonymousValue(anonymous_value_string) => Some(anonymous_value_string.clone()),
            DetailsValue::DataValue(data_value) => self.data_value_to_anonymous_value_string(data_value),
            DetailsValue::Text(text) => Some(AnonymousValueString::new(text.clone(), AnonymousValueStringFormat::String, ContainerType::None)),
            DetailsValue::Boolean(value) => Some(AnonymousValueString::new(
                value.to_string(),
                AnonymousValueStringFormat::Bool,
                ContainerType::None,
            )),
            DetailsValue::UnsignedInteger(value) => Some(AnonymousValueString::new(
                value.to_string(),
                AnonymousValueStringFormat::Decimal,
                ContainerType::None,
            )),
            DetailsValue::SignedInteger(value) => Some(AnonymousValueString::new(
                value.to_string(),
                AnonymousValueStringFormat::Decimal,
                ContainerType::None,
            )),
            DetailsValue::Empty => Some(AnonymousValueString::new(
                String::new(),
                AnonymousValueStringFormat::String,
                ContainerType::None,
            )),
        }
    }

    fn data_value_to_anonymous_value_string(
        &self,
        data_value: &DataValue,
    ) -> Option<AnonymousValueString> {
        let anonymous_value_string_format = self
            .app_context
            .engine_unprivileged_state
            .get_default_anonymous_value_string_format(data_value.get_data_type_ref());

        self.app_context
            .engine_unprivileged_state
            .anonymize_value(data_value, anonymous_value_string_format)
            .map_err(|error| {
                log::warn!("Failed to anonymize project item runtime value edit: {}", error);
                error
            })
            .ok()
    }
}
