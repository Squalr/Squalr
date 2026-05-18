use crate::app_context::AppContext;
use crate::ui::geometry::safe_clamp_ord;
use crate::views::project_explorer::project_hierarchy::{
    project_hierarchy_details_focus::ProjectHierarchyDetailsFocus,
    project_item_preview_details::ProjectItemPreviewDetails,
    view_data::{project_hierarchy_pending_operation::ProjectHierarchyPendingOperation, project_hierarchy_view_data::ProjectHierarchyViewData},
};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_pointer::ProjectItemTypePointer;
use squalr_engine_session::virtual_snapshots::virtual_snapshot_query::VirtualSnapshotQuery;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct ProjectHierarchyRuntimePreviewController {
    app_context: Arc<AppContext>,
    project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
    details_focus: ProjectHierarchyDetailsFocus,
}

impl ProjectHierarchyRuntimePreviewController {
    const MIN_PROJECT_READ_INTERVAL_MS: u64 = 50;
    const MAX_PROJECT_READ_INTERVAL_MS: u64 = 5_000;
    const PROJECT_ITEM_PREVIEW_VIRTUAL_SNAPSHOT_ID: &'static str = "project_hierarchy_preview";

    pub fn new(
        app_context: Arc<AppContext>,
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        details_focus: ProjectHierarchyDetailsFocus,
    ) -> Self {
        Self {
            app_context,
            project_hierarchy_view_data,
            details_focus,
        }
    }

    pub fn get_project_read_interval(&self) -> Duration {
        let configured_project_read_interval_ms = self
            .project_hierarchy_view_data
            .read("Project hierarchy project read interval")
            .map(|project_hierarchy_view_data| project_hierarchy_view_data.project_read_interval_ms)
            .unwrap_or(200);
        let bounded_project_read_interval_ms = safe_clamp_ord(
            configured_project_read_interval_ms,
            Self::MIN_PROJECT_READ_INTERVAL_MS,
            Self::MAX_PROJECT_READ_INTERVAL_MS,
        );

        Duration::from_millis(bounded_project_read_interval_ms)
    }

    pub fn refresh_if_project_preview_values_stale(
        &self,
        project_read_interval: Duration,
    ) {
        let should_refresh_project_items = self
            .project_hierarchy_view_data
            .write("Project hierarchy periodic project read check")
            .map(|mut project_hierarchy_view_data| {
                let has_open_project = project_hierarchy_view_data.opened_project_info.is_some();
                if !has_open_project || project_hierarchy_view_data.pending_operation != ProjectHierarchyPendingOperation::None {
                    return false;
                }

                let now = Instant::now();
                let has_refresh_interval_elapsed = project_hierarchy_view_data
                    .last_project_read_timestamp
                    .map(|last_project_read_timestamp| now.duration_since(last_project_read_timestamp) >= project_read_interval)
                    .unwrap_or(true);

                if !has_refresh_interval_elapsed {
                    return false;
                }

                project_hierarchy_view_data.last_project_read_timestamp = Some(now);

                true
            })
            .unwrap_or(false);

        if should_refresh_project_items {
            self.sync_project_item_virtual_snapshot(project_read_interval);
        }
    }

    pub fn sync_project_item_virtual_snapshot(
        &self,
        project_read_interval: Duration,
    ) {
        let virtual_snapshot_queries = self
            .project_hierarchy_view_data
            .read("Project hierarchy build virtual snapshot queries")
            .map(|project_hierarchy_view_data| {
                if project_hierarchy_view_data.opened_project_info.is_none() {
                    return Vec::new();
                }

                let requested_preview_project_item_paths = project_hierarchy_view_data.collect_requested_preview_project_item_paths();

                requested_preview_project_item_paths
                    .into_iter()
                    .filter_map(|project_item_path| {
                        project_hierarchy_view_data
                            .project_items
                            .iter()
                            .find(|(project_item_ref, _)| project_item_ref.get_project_item_path() == &project_item_path)
                            .and_then(|(_, project_item)| {
                                ProjectItemPreviewDetails::build_project_item_virtual_snapshot_query(
                                    project_hierarchy_view_data.opened_project_info.as_ref(),
                                    &project_item_path,
                                    project_item,
                                    &self.app_context.engine_unprivileged_state,
                                )
                            })
                    })
                    .collect::<Vec<VirtualSnapshotQuery>>()
            })
            .unwrap_or_default();

        self.app_context
            .engine_unprivileged_state
            .set_virtual_snapshot_queries(Self::PROJECT_ITEM_PREVIEW_VIRTUAL_SNAPSHOT_ID, project_read_interval, virtual_snapshot_queries);
        self.app_context
            .engine_unprivileged_state
            .request_virtual_snapshot_refresh(Self::PROJECT_ITEM_PREVIEW_VIRTUAL_SNAPSHOT_ID);
        self.apply_project_item_virtual_snapshot_results();
    }

    fn apply_project_item_virtual_snapshot_results(&self) {
        let Some(virtual_snapshot) = self
            .app_context
            .engine_unprivileged_state
            .get_virtual_snapshot(Self::PROJECT_ITEM_PREVIEW_VIRTUAL_SNAPSHOT_ID)
        else {
            return;
        };
        let preview_fields_by_project_item_path = self
            .project_hierarchy_view_data
            .read("Project hierarchy apply virtual snapshot results")
            .map(|project_hierarchy_view_data| {
                project_hierarchy_view_data
                    .project_items
                    .iter()
                    .filter_map(|(project_item_ref, project_item)| {
                        let project_item_path = project_item_ref.get_project_item_path();
                        let query_id = project_item_path.to_string_lossy().to_string();
                        let query_result = virtual_snapshot.get_query_results().get(&query_id)?;
                        let preview_value = ProjectItemPreviewDetails::build_project_item_preview_value_from_virtual_snapshot_result(
                            &self.app_context.engine_unprivileged_state,
                            project_hierarchy_view_data.opened_project_info.as_ref(),
                            project_item,
                            query_result,
                        );
                        let preview_path = if project_item.get_item_type().get_project_item_type_id() == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
                            query_result.evaluated_pointer_path.clone()
                        } else {
                            String::new()
                        };

                        Some((project_item_path.clone(), (preview_value, preview_path)))
                    })
                    .collect::<HashMap<PathBuf, (String, String)>>()
            })
            .unwrap_or_default();

        if !preview_fields_by_project_item_path.is_empty() {
            let did_update_preview_fields =
                ProjectHierarchyViewData::set_project_item_preview_fields(self.project_hierarchy_view_data.clone(), &preview_fields_by_project_item_path);

            if did_update_preview_fields {
                self.details_focus
                    .refocus_project_item_details_if_preview_changed(&preview_fields_by_project_item_path);
            }
        }
    }
}
