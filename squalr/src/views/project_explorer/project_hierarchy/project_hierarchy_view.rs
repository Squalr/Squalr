use crate::{
    app_context::AppContext,
    ui::{
        converters::data_type_to_icon_converter::DataTypeToIconConverter,
        list_navigation::ListNavigationDirection,
        widgets::controls::{
            context_menu::context_menu::{ContextMenu, ContextMenuSizing},
            toolbar_menu::toolbar_menu_item_view::ToolbarMenuItemView,
        },
    },
    views::code_viewer::{code_viewer_view::CodeViewerView, view_data::code_viewer_view_data::CodeViewerViewData},
    views::context_menu_labels::{OPEN_IN_CODE_VIEWER_LABEL, OPEN_IN_MEMORY_VIEWER_LABEL},
    views::memory_viewer::{memory_viewer_view::MemoryViewerView, view_data::memory_viewer_view_data::MemoryViewerViewData},
    views::pointer_scanner::{pointer_scanner_view::PointerScannerView, view_data::pointer_scanner_view_data::PointerScannerViewData},
    views::project_explorer::{
        project_explorer_view::ProjectExplorerView,
        project_hierarchy::{
            project_hierarchy_details_focus::ProjectHierarchyDetailsFocus,
            project_hierarchy_runtime_preview_controller::ProjectHierarchyRuntimePreviewController,
            project_hierarchy_takeover_host_view::{ProjectHierarchyTakeoverHostAction, ProjectHierarchyTakeoverHostView},
            project_hierarchy_toolbar_view::ProjectHierarchyToolbarView,
            project_item_details::ProjectItemDetails,
            project_item_entry_view::ProjectItemEntryView,
            project_item_inline_rename_view::ProjectItemInlineRenameView,
            project_item_rename_request_builder::ProjectItemRenameRequestBuilder,
            view_data::{
                project_hierarchy_create_item_kind::ProjectHierarchyCreateItemKind, project_hierarchy_drop_target::ProjectHierarchyDropTarget,
                project_hierarchy_frame_action::ProjectHierarchyFrameAction, project_hierarchy_menu_target::ProjectHierarchyMenuTarget,
                project_hierarchy_pending_operation::ProjectHierarchyPendingOperation, project_hierarchy_take_over_state::ProjectHierarchyTakeOverState,
                project_hierarchy_tree_entry::ProjectHierarchyTreeEntry, project_hierarchy_view_data::ProjectHierarchyViewData,
            },
        },
    },
    views::struct_viewer::view_data::{struct_viewer_focus_target::StructViewerFocusTarget, struct_viewer_view_data::StructViewerViewData},
};
use eframe::egui::{Align, CursorIcon, Id, Key, Layout, Pos2, Rect, Response, ScrollArea, TextureHandle, Ui, Widget};
use epaint::{CornerRadius, Stroke, StrokeKind};
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::project::save::project_save_request::ProjectSaveRequest;
use squalr_engine_api::commands::project_items::write_value::project_items_write_value_request::ProjectItemsWriteValueRequest;
use squalr_engine_api::commands::settings::scan::list::scan_settings_list_request::ScanSettingsListRequest;
use squalr_engine_api::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_values::anonymous_value_string::AnonymousValueString;
use squalr_engine_api::structures::memory::address_display::try_resolve_virtual_module_address;
use squalr_engine_api::structures::memory::pointer::Pointer;
use squalr_engine_api::structures::projects::project_info::ProjectInfo;
use squalr_engine_api::structures::projects::project_items::built_in_types::{
    project_item_type_address::ProjectItemTypeAddress, project_item_type_directory::ProjectItemTypeDirectory, project_item_type_pointer::ProjectItemTypePointer,
};
use squalr_engine_api::structures::projects::project_items::{
    details::ProjectItemDetailsProjection, project_item::ProjectItem, project_item_ref::ProjectItemRef,
};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct ProjectHierarchyView {
    app_context: Arc<AppContext>,
    project_hierarchy_toolbar_view: ProjectHierarchyToolbarView,
    project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
    code_viewer_view_data: Dependency<CodeViewerViewData>,
    memory_viewer_view_data: Dependency<MemoryViewerViewData>,
    pointer_scanner_view_data: Dependency<PointerScannerViewData>,
    struct_viewer_view_data: Dependency<StructViewerViewData>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ProjectItemChangeSignature {
    project_item_type_id: String,
    project_item_name: String,
    project_item_description: String,
    project_item_properties: Vec<(String, Vec<u8>)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PointerScannerContextAction {
    Address {
        label: &'static str,
        address: u64,
        module_name: String,
        data_type_id: String,
    },
    ResolvedPointer {
        label: &'static str,
        pointer: Pointer,
        data_type_id: String,
    },
}

impl PointerScannerContextAction {
    fn label(&self) -> &'static str {
        match self {
            Self::Address { label, .. } | Self::ResolvedPointer { label, .. } => label,
        }
    }
}

impl ProjectHierarchyView {
    const STRIP_SYMBOL_INFORMATION_LABEL: &str = "Strip Symbol Information";
    const PROJECT_ITEM_CTX_OPEN_MEMORY_VIEWER_LABEL: &str = OPEN_IN_MEMORY_VIEWER_LABEL;
    const PROJECT_ITEM_CTX_OPEN_CODE_VIEWER_LABEL: &str = OPEN_IN_CODE_VIEWER_LABEL;
    const PROJECT_ITEM_CTX_OPEN_RUNTIME_VIEWER_ID: &str = "project_hierarchy_ctx_open_runtime_viewer";
    const PROJECT_ITEM_CTX_PROMOTE_TO_SYMBOL_LABEL: &str = "Promote to Symbol";
    const PROJECT_ITEM_CTX_PROMOTE_TO_SYMBOL_ID: &str = "project_hierarchy_ctx_promote_to_symbol";
    const PROJECT_ITEM_CTX_STRIP_SYMBOL_INFORMATION_ID: &str = "project_hierarchy_ctx_strip_symbol_information";
    const PROJECT_ITEM_CTX_NEW_ADDRESS_LABEL: &str = "New Address";
    const PROJECT_ITEM_CTX_NEW_ADDRESS_ID: &str = "project_hierarchy_ctx_new_project_item";
    const PROJECT_ITEM_CTX_NEW_FOLDER_LABEL: &str = "New Folder";
    const PROJECT_ITEM_CTX_NEW_FOLDER_ID: &str = "project_hierarchy_ctx_new_folder";
    const PROJECT_ITEM_CTX_CUT_LABEL: &str = "Cut";
    const PROJECT_ITEM_CTX_CUT_ID: &str = "project_hierarchy_ctx_cut";
    const PROJECT_ITEM_CTX_COPY_LABEL: &str = "Copy";
    const PROJECT_ITEM_CTX_COPY_ID: &str = "project_hierarchy_ctx_copy";
    const PROJECT_ITEM_CTX_PASTE_LABEL: &str = "Paste";
    const PROJECT_ITEM_CTX_PASTE_ID: &str = "project_hierarchy_ctx_paste";
    const PROJECT_ITEM_CTX_DELETE_LABEL: &str = "Delete";
    const PROJECT_ITEM_CTX_DELETE_ID: &str = "project_hierarchy_ctx_delete";

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let project_hierarchy_view_data = app_context
            .dependency_container
            .get_dependency::<ProjectHierarchyViewData>();
        let project_hierarchy_toolbar_view = ProjectHierarchyToolbarView::new(app_context.clone());
        let struct_viewer_view_data = app_context
            .dependency_container
            .get_dependency::<StructViewerViewData>();
        let memory_viewer_view_data = app_context
            .dependency_container
            .get_dependency::<MemoryViewerViewData>();
        let code_viewer_view_data = app_context
            .dependency_container
            .get_dependency::<CodeViewerViewData>();
        let pointer_scanner_view_data = app_context
            .dependency_container
            .get_dependency::<PointerScannerViewData>();
        ProjectHierarchyViewData::refresh_project_items(project_hierarchy_view_data.clone(), app_context.clone());

        Self {
            app_context,
            project_hierarchy_toolbar_view,
            project_hierarchy_view_data,
            code_viewer_view_data,
            memory_viewer_view_data,
            pointer_scanner_view_data,
            struct_viewer_view_data,
        }
    }

    fn details_focus(&self) -> ProjectHierarchyDetailsFocus {
        ProjectHierarchyDetailsFocus::new(
            self.app_context.clone(),
            self.project_hierarchy_view_data.clone(),
            self.struct_viewer_view_data.clone(),
        )
    }

    fn runtime_preview_controller(&self) -> ProjectHierarchyRuntimePreviewController {
        ProjectHierarchyRuntimePreviewController::new(self.app_context.clone(), self.project_hierarchy_view_data.clone(), self.details_focus())
    }
}
impl Widget for ProjectHierarchyView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        self.sync_scan_settings_if_needed();
        let runtime_preview_controller = self.runtime_preview_controller();
        let project_read_interval = runtime_preview_controller.get_project_read_interval();
        user_interface
            .ctx()
            .request_repaint_after(project_read_interval);

        self.refresh_if_project_changed();
        runtime_preview_controller.refresh_if_project_preview_values_stale(project_read_interval);

        let project_hierarchy_toolbar_view = self.project_hierarchy_toolbar_view.clone();
        let mut project_hierarchy_frame_action = ProjectHierarchyFrameAction::None;
        let mut drag_started_project_item_path: Option<PathBuf> = None;
        let mut hovered_drop_target_project_item_path: Option<ProjectHierarchyDropTarget> = None;
        let mut should_cancel_take_over = false;
        let mut delete_confirmation_project_item_paths: Option<Vec<std::path::PathBuf>> = None;
        let mut promote_symbol_overwrite_project_item_paths: Option<Vec<std::path::PathBuf>> = None;
        let mut rename_project_item_submission: Option<(PathBuf, String, String)> = None;
        let mut value_edit_project_item_submission: Option<(PathBuf, String, DataTypeRef, AnonymousValueString)> = None;
        let mut keyboard_activation_toggle_target: Option<(Vec<PathBuf>, bool)> = None;
        let mut is_delete_confirmation_active = false;
        let mut is_promote_symbol_conflict_active = false;
        let mut is_rename_take_over_active = false;
        let mut is_value_edit_take_over_active = false;
        let mut visible_preview_project_item_paths = Vec::new();
        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                let shared_struct_viewer_focus_target = self
                    .struct_viewer_view_data
                    .read("Project hierarchy shared struct viewer focus target")
                    .and_then(|struct_viewer_view_data| struct_viewer_view_data.get_focus_target().cloned());
                let active_struct_viewer_project_item_paths: HashSet<PathBuf> = match shared_struct_viewer_focus_target.as_ref() {
                    Some(StructViewerFocusTarget::ProjectHierarchy { project_item_paths }) => {
                        project_item_paths.iter().cloned().collect()
                    }
                    _ => HashSet::new(),
                };

                let project_hierarchy_view_data = match self.project_hierarchy_view_data.read("Project hierarchy view") {
                    Some(project_hierarchy_view_data) => project_hierarchy_view_data,
                    None => return,
                };
                let take_over_state = project_hierarchy_view_data.take_over_state.clone();
                let tree_entries = project_hierarchy_view_data.tree_entries.clone();
                let selected_project_item_paths = project_hierarchy_view_data.selected_project_item_paths.clone();
                let dragged_project_item_paths = project_hierarchy_view_data.dragged_project_item_paths.clone();
                let menu_target = project_hierarchy_view_data.menu_target.clone();
                let menu_position = project_hierarchy_view_data.menu_position;
                let selected_project_item_paths_in_tree_order = project_hierarchy_view_data.collect_selected_project_item_paths_in_tree_order();
                let pending_operation = project_hierarchy_view_data.pending_operation.clone();

                user_interface.add(project_hierarchy_toolbar_view);
                self.show_toolbar_add_menu(
                    &mut project_hierarchy_frame_action,
                    user_interface,
                    menu_target.as_ref(),
                    menu_position,
                );

                match pending_operation {
                    ProjectHierarchyPendingOperation::Deleting => {
                        user_interface.label("Deleting project item(s)...");
                    }
                    ProjectHierarchyPendingOperation::Promoting => {
                        user_interface.label("Promoting project item(s) to symbols...");
                    }
                    ProjectHierarchyPendingOperation::Reordering => {
                        user_interface.label("Reordering project item(s)...");
                    }
                    _ => {}
                }

                let active_inline_rename = match &take_over_state {
                    ProjectHierarchyTakeOverState::RenameProjectItem {
                        project_item_path,
                        project_item_type_id,
                    } => Some((project_item_path.clone(), project_item_type_id.clone())),
                    _ => None,
                };
                is_rename_take_over_active = active_inline_rename.is_some();
                let active_value_edit_project_item_path = match &take_over_state {
                    ProjectHierarchyTakeOverState::EditProjectItemValue { project_item_path } => Some(project_item_path.clone()),
                    _ => None,
                };
                is_value_edit_take_over_active = active_value_edit_project_item_path.is_some();
                is_delete_confirmation_active = matches!(take_over_state, ProjectHierarchyTakeOverState::DeleteConfirmation { .. });
                is_promote_symbol_conflict_active = matches!(take_over_state, ProjectHierarchyTakeOverState::PromoteSymbolConflict { .. });
                match &take_over_state {
                    ProjectHierarchyTakeOverState::None | ProjectHierarchyTakeOverState::RenameProjectItem { .. } => {
                        ScrollArea::vertical()
                            .id_salt("project_hierarchy")
                            .auto_shrink([false, false])
                            .show_rows(user_interface, Self::PROJECT_ITEM_ROW_HEIGHT, tree_entries.len(), |user_interface, visible_row_range| {
                                visible_preview_project_item_paths.extend(
                                    tree_entries[visible_row_range.clone()]
                                        .iter()
                                        .map(|tree_entry| tree_entry.project_item_path.clone()),
                                );

                                for tree_entry in &tree_entries[visible_row_range] {
                                    let is_selected = selected_project_item_paths.contains(&tree_entry.project_item_path)
                                        && active_struct_viewer_project_item_paths.contains(&tree_entry.project_item_path);
                                    let icon = Self::resolve_tree_entry_icon(
                                        self.app_context.clone(),
                                        project_hierarchy_view_data.opened_project_info.as_ref(),
                                        &tree_entry.project_item,
                                    );

                                    let is_inline_rename_row = active_inline_rename
                                        .as_ref()
                                        .map(|(project_item_path, _)| project_item_path == &tree_entry.project_item_path)
                                        .unwrap_or(false);
                                    let (row_response, should_request_rename, should_request_value_edit) = if is_inline_rename_row {
                                        let (_, project_item_type_id) = active_inline_rename
                                            .as_ref()
                                            .unwrap_or_else(|| panic!("Expected inline rename state for rename row."));
                                        let rename_text_storage_id =
                                            Self::project_item_rename_text_storage_id(&tree_entry.project_item_path);
                                        let rename_highlight_storage_id =
                                            Self::project_item_rename_highlight_storage_id(&tree_entry.project_item_path);
                                        let mut rename_text = user_interface
                                            .ctx()
                                            .data_mut(|data| data.get_temp::<String>(rename_text_storage_id))
                                            .unwrap_or_else(|| tree_entry.display_name.clone());
                                        let mut should_highlight_text = user_interface
                                            .ctx()
                                            .data_mut(|data| data.get_temp::<bool>(rename_highlight_storage_id))
                                            .unwrap_or(true);
                                        let inline_rename_response = ProjectItemInlineRenameView::new(
                                            self.app_context.clone(),
                                            &tree_entry.project_item_path,
                                            &mut rename_text,
                                            &mut should_highlight_text,
                                            tree_entry.is_activated,
                                            tree_entry.depth,
                                            icon,
                                            is_selected,
                                            tree_entry.is_directory,
                                            tree_entry.has_children,
                                            tree_entry.is_expanded,
                                        )
                                        .show(user_interface);

                                        if inline_rename_response.should_commit {
                                            rename_project_item_submission = Some((
                                                tree_entry.project_item_path.clone(),
                                                project_item_type_id.clone(),
                                                rename_text.clone(),
                                            ));
                                        }

                                        if inline_rename_response.should_cancel {
                                            should_cancel_take_over = true;
                                        }

                                        user_interface.ctx().data_mut(|data| {
                                            data.insert_temp(rename_text_storage_id, rename_text);
                                            data.insert_temp(rename_highlight_storage_id, should_highlight_text);
                                        });

                                        (inline_rename_response.row_response, false, false)
                                    } else {
                                        let project_item_entry_view_response = ProjectItemEntryView::new(
                                            self.app_context.clone(),
                                            &tree_entry.project_item_path,
                                            &tree_entry.display_name,
                                            &tree_entry.preview_path,
                                            &tree_entry.preview_value,
                                            tree_entry.is_activated,
                                            tree_entry.depth,
                                            icon,
                                            is_selected,
                                            ProjectHierarchyViewData::is_cut_project_item_path(
                                                self.project_hierarchy_view_data.clone(),
                                                &tree_entry.project_item_path,
                                            ),
                                            tree_entry.is_directory,
                                            tree_entry.has_children,
                                            tree_entry.is_expanded,
                                            &mut project_hierarchy_frame_action,
                                        )
                                        .show(user_interface);

                                        (
                                            project_item_entry_view_response.row_response,
                                            project_item_entry_view_response.should_request_rename,
                                            project_item_entry_view_response.should_request_value_edit,
                                        )
                                    };

                                    if is_rename_take_over_active || is_value_edit_take_over_active {
                                        continue;
                                    }

                                    if should_request_rename {
                                        project_hierarchy_frame_action =
                                            ProjectHierarchyFrameAction::RequestRename(tree_entry.project_item_path.clone());
                                    } else if should_request_value_edit {
                                        project_hierarchy_frame_action =
                                            ProjectHierarchyFrameAction::RequestValueEdit(tree_entry.project_item_path.clone());
                                    }

                                    if row_response.drag_started() {
                                        drag_started_project_item_path = Some(tree_entry.project_item_path.clone());
                                    }

                                    let tree_entry_project_item_path = tree_entry.project_item_path.clone();
                                    let pointer_scanner_context_actions =
                                        Self::build_pointer_scanner_context_actions(project_hierarchy_view_data.opened_project_info.as_ref(), &tree_entry.project_item);
                                    let can_open_in_memory_viewer =
                                        ProjectItemDetails::can_open_project_item_in_memory_viewer(&tree_entry.project_item);
                                    let is_context_menu_visible =
                                        matches!(menu_target.as_ref(), Some(ProjectHierarchyMenuTarget::ProjectItem(menu_project_item_path)) if menu_project_item_path == &tree_entry.project_item_path);
                                    let default_context_menu_position = row_response.rect.left_bottom();

                                    if row_response.secondary_clicked() {
                                        ProjectHierarchyViewData::show_project_item_menu(
                                            self.project_hierarchy_view_data.clone(),
                                            tree_entry.project_item_path.clone(),
                                            row_response
                                                .hover_pos()
                                                .unwrap_or(default_context_menu_position),
                                        );
                                    }

                                    if is_context_menu_visible {
                                        let mut open = true;
                                        let project_item_paths_for_delete = if selected_project_item_paths_in_tree_order.contains(&tree_entry_project_item_path)
                                            && selected_project_item_paths_in_tree_order.len() > 1
                                        {
                                            selected_project_item_paths_in_tree_order.clone()
                                        } else {
                                            vec![tree_entry_project_item_path.clone()]
                                        };
                                        let can_delete_project_item_paths = ProjectHierarchyViewData::has_deletable_project_item_paths(
                                            self.project_hierarchy_view_data.clone(),
                                            &project_item_paths_for_delete,
                                        );
                                        let can_copy_project_item_paths = can_delete_project_item_paths;
                                        let can_cut_project_item_paths = can_delete_project_item_paths;
                                        let can_promote_project_item_paths = ProjectHierarchyViewData::has_promotable_project_item_paths(
                                            self.project_hierarchy_view_data.clone(),
                                            &project_item_paths_for_delete,
                                        );
                                        let can_strip_symbol_project_item_paths = ProjectHierarchyViewData::has_strippable_symbol_project_item_paths(
                                            self.project_hierarchy_view_data.clone(),
                                            &project_item_paths_for_delete,
                                        );
                                        let has_symbolic_address_project_item_paths = ProjectHierarchyViewData::has_symbolic_address_project_item_paths(
                                            self.project_hierarchy_view_data.clone(),
                                            &project_item_paths_for_delete,
                                        );
                                        let can_promote_project_item_paths = can_promote_project_item_paths && !has_symbolic_address_project_item_paths;
                                        let can_paste_project_items = ProjectHierarchyViewData::can_paste_project_item_clipboard(
                                            self.project_hierarchy_view_data.clone(),
                                            &tree_entry_project_item_path,
                                        );
                                        let should_open_in_code_viewer = ProjectItemDetails::should_open_project_item_in_code_viewer(&tree_entry.project_item);
                                        let runtime_viewer_label = if should_open_in_code_viewer {
                                            Self::PROJECT_ITEM_CTX_OPEN_CODE_VIEWER_LABEL
                                        } else {
                                            Self::PROJECT_ITEM_CTX_OPEN_MEMORY_VIEWER_LABEL
                                        };
                                        let mut project_item_menu_labels =
                                            pointer_scanner_context_actions.iter().map(PointerScannerContextAction::label).collect::<Vec<_>>();
                                        let has_runtime_actions = !pointer_scanner_context_actions.is_empty()
                                            || can_open_in_memory_viewer
                                            || can_strip_symbol_project_item_paths
                                            || can_promote_project_item_paths;
                                        let has_create_actions = true;
                                        let has_clipboard_actions =
                                            can_cut_project_item_paths || can_copy_project_item_paths || can_paste_project_items;
                                        let has_delete_actions = can_delete_project_item_paths;
                                        if can_open_in_memory_viewer {
                                            project_item_menu_labels.push(runtime_viewer_label);
                                        }
                                        if can_promote_project_item_paths {
                                            project_item_menu_labels.push(Self::PROJECT_ITEM_CTX_PROMOTE_TO_SYMBOL_LABEL);
                                        }
                                        if can_strip_symbol_project_item_paths {
                                            project_item_menu_labels.push(Self::STRIP_SYMBOL_INFORMATION_LABEL);
                                        }
                                        if has_create_actions {
                                            project_item_menu_labels.extend([Self::PROJECT_ITEM_CTX_NEW_ADDRESS_LABEL, Self::PROJECT_ITEM_CTX_NEW_FOLDER_LABEL]);
                                        }
                                        if can_cut_project_item_paths {
                                            project_item_menu_labels.push(Self::PROJECT_ITEM_CTX_CUT_LABEL);
                                        }
                                        if can_copy_project_item_paths {
                                            project_item_menu_labels.push(Self::PROJECT_ITEM_CTX_COPY_LABEL);
                                        }
                                        if can_paste_project_items {
                                            project_item_menu_labels.push(Self::PROJECT_ITEM_CTX_PASTE_LABEL);
                                        }
                                        if has_delete_actions {
                                            project_item_menu_labels.push(Self::PROJECT_ITEM_CTX_DELETE_LABEL);
                                        }
                                        let project_item_menu_width = ContextMenuSizing::width_for_labels(
                                            self.app_context.as_ref(),
                                            user_interface,
                                            project_item_menu_labels.iter().copied(),
                                        );
                                        ContextMenu::new(
                                            self.app_context.clone(),
                                            "project_hierarchy_context_menu",
                                            menu_position.unwrap_or(default_context_menu_position),
                                            |user_interface, should_close| {
                                                if !pointer_scanner_context_actions.is_empty() {
                                                    let engine_execution_context: Arc<dyn EngineExecutionContext> =
                                                        self.app_context.engine_unprivileged_state.clone();

                                                    for pointer_scanner_context_action in pointer_scanner_context_actions.clone() {
                                                        if user_interface
                                                            .add(ToolbarMenuItemView::new(
                                                                self.app_context.clone(),
                                                                pointer_scanner_context_action.label(),
                                                                pointer_scanner_context_action.label(),
                                                                &None,
                                                                project_item_menu_width,
                                                            ))
                                                            .clicked()
                                                        {
                                                            if let Some((address, module_name, data_type_id)) = Self::resolve_pointer_scanner_context_action(
                                                                &engine_execution_context,
                                                                &pointer_scanner_context_action,
                                                            ) {
                                                                project_hierarchy_frame_action = ProjectHierarchyFrameAction::OpenPointerScannerForAddress {
                                                                    address,
                                                                    module_name,
                                                                    data_type_id,
                                                                };
                                                                *should_close = true;
                                                            } else {
                                                                log::error!(
                                                                    "Failed to resolve pointer scan target for project item context action: {}.",
                                                                    pointer_scanner_context_action.label()
                                                                );
                                                            }
                                                        }
                                                    }
                                                }

                                                if can_open_in_memory_viewer {
                                                    if user_interface
                                                        .add(
                                                            ToolbarMenuItemView::new(
                                                                self.app_context.clone(),
                                                                runtime_viewer_label,
                                                                Self::PROJECT_ITEM_CTX_OPEN_RUNTIME_VIEWER_ID,
                                                                &None,
                                                                project_item_menu_width,
                                                            )
                                                            .icon(
                                                                if should_open_in_code_viewer {
                                                                    self.app_context.theme.icon_library.icon_handle_project_cpu_instruction.clone()
                                                                } else {
                                                                    self.app_context.theme.icon_library.icon_handle_scan_collect_values.clone()
                                                                },
                                                            ),
                                                        )
                                                        .clicked()
                                                    {
                                                        let engine_execution_context: Arc<dyn EngineExecutionContext> =
                                                            self.app_context.engine_unprivileged_state.clone();

                                                        if let Some((address, module_name)) =
                                                            ProjectItemDetails::resolve_project_item_runtime_value_target(
                                                                &engine_execution_context,
                                                                project_hierarchy_view_data.opened_project_info.as_ref(),
                                                                &tree_entry.project_item,
                                                            )
                                                        {
                                                            project_hierarchy_frame_action = if should_open_in_code_viewer {
                                                                ProjectHierarchyFrameAction::OpenCodeViewerForAddress { address, module_name }
                                                            } else {
                                                                ProjectHierarchyFrameAction::OpenMemoryViewerForAddress {
                                                                    address,
                                                                    module_name,
                                                                    selection_byte_count: ProjectItemDetails::resolve_project_item_runtime_value_byte_count(
                                                                        &self.app_context.engine_unprivileged_state,
                                                                        &tree_entry.project_item,
                                                                    )
                                                                    .unwrap_or(1),
                                                                }
                                                            };
                                                            *should_close = true;
                                                        } else {
                                                            log::error!(
                                                                "Failed to resolve memory viewer target for project item: {:?}.",
                                                                tree_entry_project_item_path
                                                            );
                                                        }
                                                    }
                                                }

                                                if can_promote_project_item_paths {
                                                    if user_interface
                                                        .add(ToolbarMenuItemView::new(
                                                            self.app_context.clone(),
                                                            Self::PROJECT_ITEM_CTX_PROMOTE_TO_SYMBOL_LABEL,
                                                            Self::PROJECT_ITEM_CTX_PROMOTE_TO_SYMBOL_ID,
                                                            &None,
                                                            project_item_menu_width,
                                                        ))
                                                        .clicked()
                                                    {
                                                        project_hierarchy_frame_action = ProjectHierarchyFrameAction::PromoteToSymbol {
                                                            project_item_paths: project_item_paths_for_delete.clone(),
                                                            overwrite_conflicting_symbols: false,
                                                        };
                                                        *should_close = true;
                                                    }
                                                }

                                                if can_strip_symbol_project_item_paths {
                                                    if user_interface
                                                        .add(ToolbarMenuItemView::new(
                                                            self.app_context.clone(),
                                                            Self::STRIP_SYMBOL_INFORMATION_LABEL,
                                                            Self::PROJECT_ITEM_CTX_STRIP_SYMBOL_INFORMATION_ID,
                                                            &None,
                                                            project_item_menu_width,
                                                        ))
                                                        .clicked()
                                                    {
                                                        project_hierarchy_frame_action = ProjectHierarchyFrameAction::StripSymbolInformation {
                                                            project_item_paths: project_item_paths_for_delete.clone(),
                                                        };
                                                        *should_close = true;
                                                    }
                                                }

                                                if has_runtime_actions && has_create_actions {
                                                    user_interface.separator();
                                                }

                                                Self::show_create_project_item_menu_items(
                                                    self.app_context.clone(),
                                                    user_interface,
                                                    &tree_entry_project_item_path,
                                                    &mut project_hierarchy_frame_action,
                                                    project_item_menu_width,
                                                    should_close,
                                                );

                                                if (has_runtime_actions || has_create_actions) && has_clipboard_actions {
                                                    user_interface.separator();
                                                }

                                                if can_cut_project_item_paths {
                                                    if user_interface
                                                        .add(ToolbarMenuItemView::new(
                                                            self.app_context.clone(),
                                                            Self::PROJECT_ITEM_CTX_CUT_LABEL,
                                                            Self::PROJECT_ITEM_CTX_CUT_ID,
                                                            &None,
                                                            project_item_menu_width,
                                                        )
                                                        .icon(self.app_context.theme.icon_library.icon_handle_data_type_unknown.clone()))
                                                        .clicked()
                                                    {
                                                        project_hierarchy_frame_action =
                                                            ProjectHierarchyFrameAction::CutProjectItems(project_item_paths_for_delete.clone());
                                                        *should_close = true;
                                                    }
                                                }

                                                if can_copy_project_item_paths {
                                                    if user_interface
                                                        .add(ToolbarMenuItemView::new(
                                                            self.app_context.clone(),
                                                            Self::PROJECT_ITEM_CTX_COPY_LABEL,
                                                            Self::PROJECT_ITEM_CTX_COPY_ID,
                                                            &None,
                                                            project_item_menu_width,
                                                        )
                                                        .icon(self.app_context.theme.icon_library.icon_handle_data_type_unknown.clone()))
                                                        .clicked()
                                                    {
                                                        project_hierarchy_frame_action =
                                                            ProjectHierarchyFrameAction::CopyProjectItems(project_item_paths_for_delete.clone());
                                                        *should_close = true;
                                                    }
                                                }

                                                if can_paste_project_items {
                                                    if user_interface
                                                        .add(
                                                            ToolbarMenuItemView::new(
                                                                self.app_context.clone(),
                                                                Self::PROJECT_ITEM_CTX_PASTE_LABEL,
                                                                Self::PROJECT_ITEM_CTX_PASTE_ID,
                                                                &None,
                                                                project_item_menu_width,
                                                            )
                                                            .icon(self.app_context.theme.icon_library.icon_handle_data_type_unknown.clone()),
                                                        )
                                                        .clicked()
                                                    {
                                                        project_hierarchy_frame_action = ProjectHierarchyFrameAction::PasteProjectItems {
                                                            target_project_item_path: tree_entry_project_item_path.clone(),
                                                        };
                                                        *should_close = true;
                                                    }
                                                }

                                                if (has_runtime_actions || has_create_actions || has_clipboard_actions) && has_delete_actions {
                                                    user_interface.separator();
                                                }

                                                if has_delete_actions {
                                                    if user_interface
                                                        .add(ToolbarMenuItemView::new(
                                                            self.app_context.clone(),
                                                            Self::PROJECT_ITEM_CTX_DELETE_LABEL,
                                                            Self::PROJECT_ITEM_CTX_DELETE_ID,
                                                            &None,
                                                            project_item_menu_width,
                                                        )
                                                        .icon(self.app_context.theme.icon_library.icon_handle_common_delete.clone())
                                                        .icon_background(
                                                            self.app_context.theme.background_control_danger,
                                                            self.app_context.theme.background_control_danger_dark,
                                                        ))
                                                        .clicked()
                                                    {
                                                        project_hierarchy_frame_action =
                                                            ProjectHierarchyFrameAction::RequestDeleteConfirmation(project_item_paths_for_delete.clone());
                                                        *should_close = true;
                                                    }
                                                }
                                            },
                                        )
                                        .width(project_item_menu_width)
                                        .corner_radius(8)
                                        .show(user_interface, &mut open);

                                        if !open {
                                            ProjectHierarchyViewData::hide_menu(self.project_hierarchy_view_data.clone());
                                        }
                                    }

                                    let active_dragged_project_item_paths = drag_started_project_item_path
                                        .as_ref()
                                        .map(|drag_started_project_item_path| vec![drag_started_project_item_path.clone()])
                                        .or(dragged_project_item_paths.clone());

                                    if let Some(active_dragged_project_item_paths) = active_dragged_project_item_paths {
                                        if let Some(pointer_position) = user_interface.input(|input_state| input_state.pointer.hover_pos()) {
                                            if row_response.rect.contains(pointer_position) {
                                                if let Some(hovered_drop_target) = Self::resolve_drop_target(
                                                    &active_dragged_project_item_paths,
                                                    tree_entry,
                                                    row_response.rect,
                                                    pointer_position,
                                                ) {
                                                    hovered_drop_target_project_item_path = Some(hovered_drop_target.clone());
                                                    self.paint_drop_target_indicator(user_interface, row_response.rect, &hovered_drop_target);
                                                }
                                            }
                                        }
                                    }
                                }
                            });
                    }
                    ProjectHierarchyTakeOverState::EditProjectItemValue { .. }
                    | ProjectHierarchyTakeOverState::DeleteConfirmation { .. }
                    | ProjectHierarchyTakeOverState::PromoteSymbolConflict { .. } => {
                        match ProjectHierarchyTakeoverHostView::new(
                            self.app_context.clone(),
                            project_hierarchy_view_data.opened_project_info.as_ref(),
                            &tree_entries,
                            &take_over_state,
                        )
                        .show(user_interface)
                        {
                            ProjectHierarchyTakeoverHostAction::None => {}
                            ProjectHierarchyTakeoverHostAction::Cancel => {
                                should_cancel_take_over = true;
                            }
                            ProjectHierarchyTakeoverHostAction::DeleteProjectItems(project_item_paths) => {
                                delete_confirmation_project_item_paths = Some(project_item_paths);
                            }
                            ProjectHierarchyTakeoverHostAction::PromoteSymbolOverwrite(project_item_paths) => {
                                promote_symbol_overwrite_project_item_paths = Some(project_item_paths);
                            }
                            ProjectHierarchyTakeoverHostAction::SubmitValueEdit {
                                project_item_path,
                                value_field_name,
                                validation_data_type_ref,
                                value_edit,
                            } => {
                                value_edit_project_item_submission =
                                    Some((project_item_path, value_field_name, validation_data_type_ref, value_edit));
                            }
                        }
                    }
                }
            })
            .response;

        let is_window_focused = self
            .app_context
            .window_focus_manager
            .is_window_focused(ProjectExplorerView::WINDOW_ID);
        let can_handle_window_shortcuts = self
            .app_context
            .window_focus_manager
            .can_window_handle_shortcuts(user_interface.ctx(), ProjectExplorerView::WINDOW_ID);

        if is_window_focused && (is_delete_confirmation_active || is_promote_symbol_conflict_active) {
            if user_interface.input(|input_state| input_state.key_pressed(Key::Escape))
                || user_interface.input(|input_state| input_state.key_pressed(Key::Backspace))
            {
                should_cancel_take_over = true;
            }
        }

        if is_window_focused && is_delete_confirmation_active {
            if user_interface.input(|input_state| input_state.key_pressed(Key::Enter)) {
                delete_confirmation_project_item_paths = self
                    .project_hierarchy_view_data
                    .read("Project hierarchy confirm delete by keyboard")
                    .and_then(|project_hierarchy_view_data| match project_hierarchy_view_data.take_over_state.clone() {
                        ProjectHierarchyTakeOverState::DeleteConfirmation { project_item_paths } => Some(project_item_paths),
                        _ => None,
                    });
            }
        }

        if is_window_focused && is_promote_symbol_conflict_active && user_interface.input(|input_state| input_state.key_pressed(Key::Enter)) {
            promote_symbol_overwrite_project_item_paths = self
                .project_hierarchy_view_data
                .read("Project hierarchy confirm promote overwrite by keyboard")
                .and_then(|project_hierarchy_view_data| match project_hierarchy_view_data.take_over_state.clone() {
                    ProjectHierarchyTakeOverState::PromoteSymbolConflict { project_item_paths, .. } => Some(project_item_paths),
                    _ => None,
                });
        }

        if !is_delete_confirmation_active
            && !is_promote_symbol_conflict_active
            && !is_rename_take_over_active
            && !is_value_edit_take_over_active
            && can_handle_window_shortcuts
            && user_interface.input(|input_state| input_state.key_pressed(Key::ArrowUp))
        {
            let extend_selection = user_interface.input(|input_state| input_state.modifiers.shift);
            if ProjectHierarchyViewData::navigate_project_item_selection(
                self.project_hierarchy_view_data.clone(),
                ListNavigationDirection::Up,
                extend_selection,
            )
            .is_some()
            {
                self.focus_selected_project_items_in_struct_viewer();
            }
        }

        if !is_delete_confirmation_active
            && !is_promote_symbol_conflict_active
            && !is_rename_take_over_active
            && !is_value_edit_take_over_active
            && can_handle_window_shortcuts
            && user_interface.input(|input_state| input_state.key_pressed(Key::ArrowDown))
        {
            let extend_selection = user_interface.input(|input_state| input_state.modifiers.shift);
            if ProjectHierarchyViewData::navigate_project_item_selection(
                self.project_hierarchy_view_data.clone(),
                ListNavigationDirection::Down,
                extend_selection,
            )
            .is_some()
            {
                self.focus_selected_project_items_in_struct_viewer();
            }
        }

        if !is_delete_confirmation_active
            && !is_promote_symbol_conflict_active
            && !is_rename_take_over_active
            && !is_value_edit_take_over_active
            && can_handle_window_shortcuts
            && user_interface.input(|input_state| input_state.key_pressed(Key::Delete))
        {
            ProjectHierarchyViewData::request_delete_confirmation_for_selected_project_item(self.project_hierarchy_view_data.clone());
        }

        if !is_delete_confirmation_active
            && !is_promote_symbol_conflict_active
            && !is_rename_take_over_active
            && !is_value_edit_take_over_active
            && can_handle_window_shortcuts
            && user_interface.input(|input_state| (input_state.modifiers.command || input_state.modifiers.ctrl) && input_state.key_pressed(Key::X))
        {
            if let Some(project_item_paths) = self
                .project_hierarchy_view_data
                .read("Project hierarchy keyboard cut selection")
                .map(|project_hierarchy_view_data| project_hierarchy_view_data.collect_selected_project_item_paths_in_tree_order())
                .filter(|project_item_paths| !project_item_paths.is_empty())
            {
                project_hierarchy_frame_action = ProjectHierarchyFrameAction::CutProjectItems(project_item_paths);
            }
        }

        if !is_delete_confirmation_active
            && !is_promote_symbol_conflict_active
            && !is_rename_take_over_active
            && !is_value_edit_take_over_active
            && can_handle_window_shortcuts
            && user_interface.input(|input_state| (input_state.modifiers.command || input_state.modifiers.ctrl) && input_state.key_pressed(Key::C))
        {
            if let Some(project_item_paths) = self
                .project_hierarchy_view_data
                .read("Project hierarchy keyboard copy selection")
                .map(|project_hierarchy_view_data| project_hierarchy_view_data.collect_selected_project_item_paths_in_tree_order())
                .filter(|project_item_paths| !project_item_paths.is_empty())
            {
                project_hierarchy_frame_action = ProjectHierarchyFrameAction::CopyProjectItems(project_item_paths);
            }
        }

        if !is_delete_confirmation_active
            && !is_promote_symbol_conflict_active
            && !is_rename_take_over_active
            && !is_value_edit_take_over_active
            && can_handle_window_shortcuts
            && user_interface.input(|input_state| (input_state.modifiers.command || input_state.modifiers.ctrl) && input_state.key_pressed(Key::V))
        {
            if let Some(target_project_item_path) = ProjectHierarchyViewData::get_selected_or_root_directory_path(self.project_hierarchy_view_data.clone()) {
                project_hierarchy_frame_action = ProjectHierarchyFrameAction::PasteProjectItems { target_project_item_path };
            }
        }

        if !is_delete_confirmation_active
            && !is_promote_symbol_conflict_active
            && !is_rename_take_over_active
            && !is_value_edit_take_over_active
            && can_handle_window_shortcuts
            && user_interface.input(|input_state| input_state.key_pressed(Key::F2))
        {
            ProjectHierarchyViewData::request_rename_for_selected_project_item(self.project_hierarchy_view_data.clone());
        }

        if !is_delete_confirmation_active
            && !is_promote_symbol_conflict_active
            && !is_rename_take_over_active
            && !is_value_edit_take_over_active
            && can_handle_window_shortcuts
            && user_interface.input(|input_state| input_state.key_pressed(Key::Space))
        {
            keyboard_activation_toggle_target = self
                .project_hierarchy_view_data
                .read("Project hierarchy keyboard activation toggle")
                .and_then(|project_hierarchy_view_data| {
                    let selected_project_item_paths = project_hierarchy_view_data.collect_selected_project_item_paths_in_tree_order();
                    if selected_project_item_paths.is_empty() {
                        return None;
                    }

                    let selected_project_items = project_hierarchy_view_data
                        .project_items
                        .iter()
                        .filter(|(project_item_ref, _)| selected_project_item_paths.contains(project_item_ref.get_project_item_path()))
                        .map(|(_, project_item)| project_item)
                        .collect::<Vec<&ProjectItem>>();
                    let should_activate = selected_project_items
                        .iter()
                        .any(|project_item| !project_item.get_is_activated());

                    Some((selected_project_item_paths, should_activate))
                });
        }

        if !is_delete_confirmation_active
            && !is_value_edit_take_over_active
            && ProjectHierarchyViewData::set_visible_preview_project_item_paths(self.project_hierarchy_view_data.clone(), visible_preview_project_item_paths)
        {
            self.runtime_preview_controller()
                .sync_project_item_virtual_snapshot(project_read_interval);
        }

        if should_cancel_take_over {
            if let Some(project_item_path) = self
                .project_hierarchy_view_data
                .read("Project hierarchy clear inline rename state on cancel")
                .and_then(|project_hierarchy_view_data| match &project_hierarchy_view_data.take_over_state {
                    ProjectHierarchyTakeOverState::RenameProjectItem { project_item_path, .. } => Some(project_item_path.clone()),
                    _ => None,
                })
            {
                Self::clear_project_item_rename_state(user_interface, &project_item_path);
            }
            if let Some(project_item_path) = self
                .project_hierarchy_view_data
                .read("Project hierarchy clear value edit state on cancel")
                .and_then(|project_hierarchy_view_data| match &project_hierarchy_view_data.take_over_state {
                    ProjectHierarchyTakeOverState::EditProjectItemValue { project_item_path } => Some(project_item_path.clone()),
                    _ => None,
                })
            {
                ProjectHierarchyTakeoverHostView::clear_project_item_value_edit_state(user_interface, &project_item_path);
            }
            ProjectHierarchyViewData::cancel_take_over(self.project_hierarchy_view_data.clone());
        }

        if let Some(project_item_paths) = delete_confirmation_project_item_paths {
            ProjectHierarchyViewData::delete_project_items(self.project_hierarchy_view_data.clone(), self.app_context.clone(), project_item_paths);
        }

        if let Some(project_item_paths) = promote_symbol_overwrite_project_item_paths {
            let details_refresh_callback = self
                .details_focus()
                .build_project_item_details_refresh_callback(project_item_paths.clone());
            ProjectHierarchyViewData::promote_project_items_to_symbols(
                self.project_hierarchy_view_data.clone(),
                self.app_context.clone(),
                project_item_paths,
                true,
                Some(details_refresh_callback),
            );
        }

        if let Some((project_item_path, project_item_type_id, edited_name)) = rename_project_item_submission {
            Self::clear_project_item_rename_state(user_interface, &project_item_path);

            if let Some(project_item_rename_request) = ProjectItemRenameRequestBuilder::build(&project_item_path, &project_item_type_id, edited_name.trim()) {
                let project_hierarchy_view_data = self.project_hierarchy_view_data.clone();
                let app_context = self.app_context.clone();
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

            ProjectHierarchyViewData::cancel_take_over(self.project_hierarchy_view_data.clone());
        }

        if let Some((project_item_path, value_field_name, validation_data_type_ref, value_edit)) = value_edit_project_item_submission {
            match self
                .app_context
                .engine_unprivileged_state
                .deanonymize_value_string(&validation_data_type_ref, &value_edit)
            {
                Ok(_) => {
                    ProjectItemsWriteValueRequest {
                        project_item_path: project_item_path.clone(),
                        field_name: value_field_name,
                        anonymous_value_string: value_edit,
                    }
                    .send(&self.app_context.engine_unprivileged_state, |project_items_write_value_response| {
                        if !project_items_write_value_response.success {
                            log::warn!("Project item write-value command failed while committing value edit takeover.");
                        }
                    });
                    ProjectHierarchyTakeoverHostView::clear_project_item_value_edit_state(user_interface, &project_item_path);
                    ProjectHierarchyViewData::cancel_take_over(self.project_hierarchy_view_data.clone());
                }
                Err(error) => {
                    log::warn!("Failed to commit project hierarchy runtime value edit: {}", error);
                }
            }
        }

        if let Some((project_item_paths, is_activated)) = keyboard_activation_toggle_target {
            ProjectHierarchyViewData::set_project_item_activation(
                self.project_hierarchy_view_data.clone(),
                self.app_context.clone(),
                project_item_paths,
                is_activated,
            );
        }

        if let Some(drag_started_project_item_path) = drag_started_project_item_path.clone() {
            ProjectHierarchyViewData::begin_reorder_drag(self.project_hierarchy_view_data.clone(), drag_started_project_item_path);
        }

        let persisted_dragged_project_item_paths = self
            .project_hierarchy_view_data
            .read("Project hierarchy check active drag")
            .and_then(|project_hierarchy_view_data| project_hierarchy_view_data.dragged_project_item_paths.clone());
        let active_dragged_project_item_paths = drag_started_project_item_path
            .map(|drag_started_project_item_path| vec![drag_started_project_item_path])
            .or(persisted_dragged_project_item_paths);

        if active_dragged_project_item_paths.is_some() {
            user_interface.output_mut(|platform_output| {
                platform_output.cursor_icon = CursorIcon::Move;
            });
        }

        if user_interface.input(|input_state| input_state.pointer.any_released()) {
            if active_dragged_project_item_paths.is_some() {
                if let Some(drop_target_project_item_path) = hovered_drop_target_project_item_path {
                    ProjectHierarchyViewData::commit_reorder_drop(
                        self.project_hierarchy_view_data.clone(),
                        self.app_context.clone(),
                        drop_target_project_item_path,
                    );
                } else {
                    ProjectHierarchyViewData::cancel_reorder_drag(self.project_hierarchy_view_data.clone());
                }
            }
        }

        let has_blocking_take_over = is_promote_symbol_conflict_active || is_rename_take_over_active || is_value_edit_take_over_active;

        match project_hierarchy_frame_action {
            ProjectHierarchyFrameAction::None => {}
            ProjectHierarchyFrameAction::SelectProjectItem {
                project_item_path,
                additive_selection,
                range_selection,
            } => {
                if is_rename_take_over_active {
                    Self::clear_active_project_item_rename_state(user_interface, self.project_hierarchy_view_data.clone());
                    ProjectHierarchyViewData::cancel_take_over(self.project_hierarchy_view_data.clone());
                }
                if is_value_edit_take_over_active {
                    ProjectHierarchyViewData::cancel_take_over(self.project_hierarchy_view_data.clone());
                }

                ProjectHierarchyViewData::select_project_item(self.project_hierarchy_view_data.clone(), project_item_path, additive_selection, range_selection);
                self.focus_selected_project_items_in_struct_viewer();
            }
            ProjectHierarchyFrameAction::ToggleDirectoryExpansion(project_item_path) => {
                if is_rename_take_over_active {
                    Self::clear_active_project_item_rename_state(user_interface, self.project_hierarchy_view_data.clone());
                    ProjectHierarchyViewData::cancel_take_over(self.project_hierarchy_view_data.clone());
                }
                if is_value_edit_take_over_active {
                    ProjectHierarchyViewData::cancel_take_over(self.project_hierarchy_view_data.clone());
                }

                ProjectHierarchyViewData::toggle_directory_expansion(self.project_hierarchy_view_data.clone(), project_item_path);
            }
            ProjectHierarchyFrameAction::SetProjectItemActivation(project_item_path, is_activated) => {
                if has_blocking_take_over {
                    return response;
                }

                let project_item_paths = self
                    .project_hierarchy_view_data
                    .read("Project hierarchy checkbox activation selection")
                    .map(|project_hierarchy_view_data| {
                        if project_hierarchy_view_data
                            .selected_project_item_paths
                            .contains(&project_item_path)
                        {
                            project_hierarchy_view_data.collect_selected_project_item_paths_in_tree_order()
                        } else {
                            vec![project_item_path.clone()]
                        }
                    })
                    .unwrap_or_else(|| vec![project_item_path.clone()]);
                ProjectHierarchyViewData::set_project_item_activation(
                    self.project_hierarchy_view_data.clone(),
                    self.app_context.clone(),
                    project_item_paths,
                    is_activated,
                );
            }
            ProjectHierarchyFrameAction::CreateProjectItem {
                target_project_item_path,
                create_item_kind,
            } => {
                if has_blocking_take_over {
                    return response;
                }

                ProjectHierarchyViewData::create_project_item(
                    self.project_hierarchy_view_data.clone(),
                    self.app_context.clone(),
                    target_project_item_path,
                    create_item_kind,
                );
            }
            ProjectHierarchyFrameAction::CopyProjectItems(project_item_paths) => {
                if has_blocking_take_over {
                    return response;
                }

                ProjectHierarchyViewData::copy_project_items(self.project_hierarchy_view_data.clone(), project_item_paths);
            }
            ProjectHierarchyFrameAction::CutProjectItems(project_item_paths) => {
                if has_blocking_take_over {
                    return response;
                }

                ProjectHierarchyViewData::cut_project_items(self.project_hierarchy_view_data.clone(), project_item_paths);
            }
            ProjectHierarchyFrameAction::PasteProjectItems { target_project_item_path } => {
                if has_blocking_take_over {
                    return response;
                }

                ProjectHierarchyViewData::paste_project_item_clipboard(
                    self.project_hierarchy_view_data.clone(),
                    self.app_context.clone(),
                    target_project_item_path,
                );
            }
            ProjectHierarchyFrameAction::OpenPointerScannerForAddress {
                address,
                module_name,
                data_type_id,
            } => {
                if has_blocking_take_over {
                    return response;
                }

                self.focus_pointer_scanner_for_address(address, &module_name, &data_type_id);
            }
            ProjectHierarchyFrameAction::OpenMemoryViewerForAddress {
                address,
                module_name,
                selection_byte_count,
            } => {
                if has_blocking_take_over {
                    return response;
                }

                self.focus_memory_viewer_for_address(address, &module_name, selection_byte_count);
            }
            ProjectHierarchyFrameAction::OpenCodeViewerForAddress { address, module_name } => {
                if has_blocking_take_over {
                    return response;
                }

                self.focus_code_viewer_for_address(address, &module_name);
            }
            ProjectHierarchyFrameAction::PromoteToSymbol {
                project_item_paths,
                overwrite_conflicting_symbols,
            } => {
                if is_rename_take_over_active || is_value_edit_take_over_active {
                    return response;
                }

                let details_refresh_callback = self
                    .details_focus()
                    .build_project_item_details_refresh_callback(project_item_paths.clone());
                ProjectHierarchyViewData::promote_project_items_to_symbols(
                    self.project_hierarchy_view_data.clone(),
                    self.app_context.clone(),
                    project_item_paths,
                    overwrite_conflicting_symbols,
                    Some(details_refresh_callback),
                );
            }
            ProjectHierarchyFrameAction::StripSymbolInformation { project_item_paths } => {
                if is_promote_symbol_conflict_active || is_rename_take_over_active || is_value_edit_take_over_active {
                    return response;
                }

                let details_refresh_callback = self
                    .details_focus()
                    .build_project_item_details_refresh_callback(project_item_paths.clone());

                Self::strip_symbol_information_from_project_items(
                    self.app_context.clone(),
                    self.project_hierarchy_view_data.clone(),
                    project_item_paths,
                    Some(details_refresh_callback),
                );
            }
            ProjectHierarchyFrameAction::RequestRename(project_item_path) => {
                if is_promote_symbol_conflict_active || is_value_edit_take_over_active {
                    return response;
                }

                if is_rename_take_over_active {
                    Self::clear_active_project_item_rename_state(user_interface, self.project_hierarchy_view_data.clone());
                }

                ProjectHierarchyViewData::request_rename_for_project_item(self.project_hierarchy_view_data.clone(), project_item_path);
            }
            ProjectHierarchyFrameAction::RequestValueEdit(project_item_path) => {
                if is_promote_symbol_conflict_active {
                    return response;
                }

                if is_rename_take_over_active {
                    Self::clear_active_project_item_rename_state(user_interface, self.project_hierarchy_view_data.clone());
                    ProjectHierarchyViewData::cancel_take_over(self.project_hierarchy_view_data.clone());
                }

                ProjectHierarchyViewData::request_value_edit_for_project_item(self.project_hierarchy_view_data.clone(), project_item_path);
            }
            ProjectHierarchyFrameAction::RequestDeleteConfirmation(project_item_paths) => {
                if has_blocking_take_over {
                    return response;
                }

                ProjectHierarchyViewData::request_delete_confirmation(self.project_hierarchy_view_data.clone(), project_item_paths);
            }
        }

        response
    }
}

impl ProjectHierarchyView {
    const SCAN_SETTINGS_SYNC_INTERVAL_MS: u64 = 1_000;
    const DROP_INSERTION_BAND_HEIGHT: f32 = 7.0;
    const PROJECT_ITEM_ROW_HEIGHT: f32 = 28.0;

    fn show_toolbar_add_menu(
        &self,
        project_hierarchy_frame_action: &mut ProjectHierarchyFrameAction,
        user_interface: &mut Ui,
        menu_target: Option<&ProjectHierarchyMenuTarget>,
        menu_position: Option<Pos2>,
    ) {
        let Some(ProjectHierarchyMenuTarget::ToolbarAdd { target_project_item_path }) = menu_target else {
            return;
        };

        let Some(menu_position) = menu_position else {
            return;
        };
        let create_project_item_menu_labels = [
            Self::PROJECT_ITEM_CTX_NEW_ADDRESS_LABEL,
            Self::PROJECT_ITEM_CTX_NEW_FOLDER_LABEL,
        ];
        let project_item_menu_width =
            ContextMenuSizing::width_for_labels(self.app_context.as_ref(), user_interface, create_project_item_menu_labels.iter().copied());
        let mut open = true;
        ContextMenu::new(
            self.app_context.clone(),
            "project_hierarchy_toolbar_add_menu",
            menu_position,
            |user_interface, should_close| {
                Self::show_create_project_item_menu_items(
                    self.app_context.clone(),
                    user_interface,
                    target_project_item_path,
                    project_hierarchy_frame_action,
                    project_item_menu_width,
                    should_close,
                );
            },
        )
        .width(project_item_menu_width)
        .corner_radius(8)
        .show(user_interface, &mut open);

        if !open {
            ProjectHierarchyViewData::hide_menu(self.project_hierarchy_view_data.clone());
        }
    }

    fn show_create_project_item_menu_items(
        app_context: Arc<AppContext>,
        user_interface: &mut Ui,
        target_project_item_path: &Path,
        project_hierarchy_frame_action: &mut ProjectHierarchyFrameAction,
        project_item_menu_width: f32,
        should_close: &mut bool,
    ) {
        for (label, item_id, create_item_kind) in [
            (
                Self::PROJECT_ITEM_CTX_NEW_ADDRESS_LABEL,
                Self::PROJECT_ITEM_CTX_NEW_ADDRESS_ID,
                ProjectHierarchyCreateItemKind::Address,
            ),
            (
                Self::PROJECT_ITEM_CTX_NEW_FOLDER_LABEL,
                Self::PROJECT_ITEM_CTX_NEW_FOLDER_ID,
                ProjectHierarchyCreateItemKind::Directory,
            ),
        ] {
            if user_interface
                .add(
                    ToolbarMenuItemView::new(app_context.clone(), label, item_id, &None, project_item_menu_width).icon(match create_item_kind {
                        ProjectHierarchyCreateItemKind::Directory => app_context
                            .theme
                            .icon_library
                            .icon_handle_file_system_open_folder
                            .clone(),
                        ProjectHierarchyCreateItemKind::Address => app_context
                            .theme
                            .icon_library
                            .icon_handle_data_type_blue_blocks_4
                            .clone(),
                    }),
                )
                .clicked()
            {
                *project_hierarchy_frame_action = ProjectHierarchyFrameAction::CreateProjectItem {
                    target_project_item_path: target_project_item_path.to_path_buf(),
                    create_item_kind,
                };
                *should_close = true;
            }
        }
    }

    fn resolve_drop_target(
        active_dragged_project_item_paths: &[PathBuf],
        tree_entry: &ProjectHierarchyTreeEntry,
        row_rect: Rect,
        pointer_position: Pos2,
    ) -> Option<ProjectHierarchyDropTarget> {
        if active_dragged_project_item_paths.contains(&tree_entry.project_item_path) {
            return None;
        }

        let insertion_band_height = Self::DROP_INSERTION_BAND_HEIGHT.min(row_rect.height() / 2.0);

        if pointer_position.y <= row_rect.top() + insertion_band_height
            && Self::can_render_insertion_drop_target(active_dragged_project_item_paths, &tree_entry.project_item_path)
        {
            return Some(ProjectHierarchyDropTarget::Before(tree_entry.project_item_path.clone()));
        }

        if pointer_position.y >= row_rect.bottom() - insertion_band_height
            && Self::can_render_insertion_drop_target(active_dragged_project_item_paths, &tree_entry.project_item_path)
        {
            return Some(ProjectHierarchyDropTarget::After(tree_entry.project_item_path.clone()));
        }

        if tree_entry.is_directory && Self::can_render_into_directory_drop_target(active_dragged_project_item_paths, &tree_entry.project_item_path) {
            return Some(ProjectHierarchyDropTarget::Into(tree_entry.project_item_path.clone()));
        }

        None
    }

    fn can_render_insertion_drop_target(
        active_dragged_project_item_paths: &[PathBuf],
        target_project_item_path: &Path,
    ) -> bool {
        let Some(target_directory_path) = target_project_item_path.parent() else {
            return false;
        };

        !active_dragged_project_item_paths.contains(&target_project_item_path.to_path_buf())
            && active_dragged_project_item_paths
                .iter()
                .all(|dragged_project_item_path| !target_directory_path.starts_with(dragged_project_item_path))
    }

    fn can_render_into_directory_drop_target(
        active_dragged_project_item_paths: &[PathBuf],
        target_project_item_path: &Path,
    ) -> bool {
        !active_dragged_project_item_paths
            .iter()
            .any(|dragged_project_item_path| target_project_item_path.starts_with(dragged_project_item_path))
    }

    fn paint_drop_target_indicator(
        &self,
        user_interface: &mut Ui,
        row_rect: Rect,
        drop_target: &ProjectHierarchyDropTarget,
    ) {
        let theme = &self.app_context.theme;

        match drop_target {
            ProjectHierarchyDropTarget::Into(_) => {
                user_interface
                    .painter()
                    .rect_filled(row_rect, CornerRadius::ZERO, theme.selected_background);
                user_interface
                    .painter()
                    .rect_stroke(row_rect, CornerRadius::ZERO, Stroke::new(1.0, theme.selected_border), StrokeKind::Inside);
            }
            ProjectHierarchyDropTarget::Before(_) | ProjectHierarchyDropTarget::After(_) => {
                let indicator_y = match drop_target {
                    ProjectHierarchyDropTarget::Before(_) => row_rect.top() + 0.5,
                    ProjectHierarchyDropTarget::After(_) => row_rect.bottom() - 0.5,
                    ProjectHierarchyDropTarget::Into(_) => row_rect.center().y,
                };
                let indicator_left = row_rect.left() + 8.0;
                let indicator_right = row_rect.right() - 8.0;
                let indicator_cap_half_height = 5.0;

                user_interface.painter().line_segment(
                    [
                        Pos2::new(indicator_left, indicator_y),
                        Pos2::new(indicator_right, indicator_y),
                    ],
                    Stroke::new(3.0, theme.selected_border),
                );
                user_interface.painter().line_segment(
                    [
                        Pos2::new(indicator_left, indicator_y - indicator_cap_half_height),
                        Pos2::new(indicator_left, indicator_y + indicator_cap_half_height),
                    ],
                    Stroke::new(3.0, theme.selected_border),
                );
                user_interface.painter().line_segment(
                    [
                        Pos2::new(indicator_right, indicator_y - indicator_cap_half_height),
                        Pos2::new(indicator_right, indicator_y + indicator_cap_half_height),
                    ],
                    Stroke::new(3.0, theme.selected_border),
                );
            }
        }
    }

    fn sync_scan_settings_if_needed(&self) {
        let should_request_scan_settings = self
            .project_hierarchy_view_data
            .write("Project hierarchy scan settings sync check")
            .map(|mut project_hierarchy_view_data| {
                let now = Instant::now();
                let has_sync_interval_elapsed = project_hierarchy_view_data
                    .last_scan_settings_sync_timestamp
                    .map(|last_scan_settings_sync_timestamp| {
                        now.duration_since(last_scan_settings_sync_timestamp) >= Duration::from_millis(Self::SCAN_SETTINGS_SYNC_INTERVAL_MS)
                    })
                    .unwrap_or(true);

                if project_hierarchy_view_data.is_querying_scan_settings || !has_sync_interval_elapsed {
                    return false;
                }

                project_hierarchy_view_data.is_querying_scan_settings = true;
                project_hierarchy_view_data.last_scan_settings_sync_timestamp = Some(now);

                true
            })
            .unwrap_or(false);

        if !should_request_scan_settings {
            return;
        }

        let project_hierarchy_view_data = self.project_hierarchy_view_data.clone();
        let scan_settings_list_request = ScanSettingsListRequest {};
        scan_settings_list_request.send(&self.app_context.engine_unprivileged_state, move |scan_settings_list_response| {
            if let Some(mut project_hierarchy_view_data) = project_hierarchy_view_data.write("Project hierarchy scan settings sync response") {
                if let Ok(scan_settings) = scan_settings_list_response.scan_settings {
                    project_hierarchy_view_data.project_read_interval_ms = scan_settings.project_read_interval_ms;
                }

                project_hierarchy_view_data.is_querying_scan_settings = false;
            }
        });
    }

    fn focus_selected_project_items_in_struct_viewer(&self) {
        self.details_focus().focus_selected_project_items();
    }

    fn strip_symbol_information_from_project_items(
        app_context: Arc<AppContext>,
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        project_item_paths: Vec<PathBuf>,
        details_refresh_callback: Option<Arc<dyn Fn() + Send + Sync>>,
    ) {
        let project_item_paths = ProjectHierarchyViewData::filter_strippable_symbol_project_item_paths(project_hierarchy_view_data, project_item_paths);

        if project_item_paths.is_empty() {
            return;
        }

        let project_manager = app_context.engine_unprivileged_state.get_project_manager();
        let opened_project_lock = project_manager.get_opened_project();
        let mut opened_project_guard = match opened_project_lock.write() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                log::error!(
                    "Failed to acquire opened project lock while stripping project item symbol information: {}",
                    error
                );
                return;
            }
        };
        let Some(opened_project) = opened_project_guard.as_mut() else {
            log::warn!("Cannot strip project item symbol information without an opened project.");
            return;
        };
        let project_symbol_catalog = opened_project
            .get_project_info()
            .get_project_symbol_catalog()
            .clone();
        let mut has_persisted_property_edits = false;

        for project_item_path in &project_item_paths {
            let project_item_ref = ProjectItemRef::new(project_item_path.clone());
            let Some(project_item) = opened_project.get_project_item_mut(&project_item_ref) else {
                log::warn!("Cannot strip symbol information, project item was not found: {:?}", project_item_path);
                continue;
            };

            if project_item.get_item_type().get_project_item_type_id() != ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
                continue;
            }

            let address_target = ProjectItemTypeAddress::get_address_target(project_item);

            if !address_target.has_symbolic_offsets() {
                continue;
            }

            let Some(stripped_address_target) = address_target.strip_symbolic_offsets(&project_symbol_catalog) else {
                log::warn!("Cannot strip unresolved symbol information from project item: {:?}", project_item_path);
                continue;
            };

            if stripped_address_target != address_target {
                ProjectItemTypeAddress::set_address_target(project_item, stripped_address_target);
                project_item.set_has_unsaved_changes(true);
                has_persisted_property_edits = true;
            }
        }

        if !has_persisted_property_edits {
            return;
        }

        opened_project
            .get_project_info_mut()
            .set_has_unsaved_changes(true);
        drop(opened_project_guard);

        let project_save_request = ProjectSaveRequest {};

        project_save_request.send(&app_context.engine_unprivileged_state, |project_save_response| {
            if !project_save_response.success {
                log::error!("Failed to persist stripped project item symbol information through project save command.");
            }
        });
        project_manager.notify_project_items_changed();

        if let Some(details_refresh_callback) = details_refresh_callback {
            details_refresh_callback();
        }
    }

    fn build_pointer_scanner_context_actions(
        opened_project_info: Option<&ProjectInfo>,
        project_item: &ProjectItem,
    ) -> Vec<PointerScannerContextAction> {
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            let mut project_item = project_item.clone();
            let data_type_id = ProjectItemTypeAddress::get_field_symbolic_struct_definition_reference(&mut project_item)
                .map(|symbolic_struct_reference| {
                    symbolic_struct_reference
                        .get_symbolic_struct_namespace()
                        .to_string()
                })
                .unwrap_or_default();

            let address_target = ProjectItemTypeAddress::get_address_target(&mut project_item);
            let Some(runtime_pointer) = ProjectItemDetails::resolve_address_target_runtime_pointer(opened_project_info, &address_target) else {
                return Vec::new();
            };

            if runtime_pointer.get_offset_segments().is_empty() {
                return vec![PointerScannerContextAction::Address {
                    label: "Open in Pointer Scan",
                    address: runtime_pointer.get_address(),
                    module_name: runtime_pointer.get_module_name().to_string(),
                    data_type_id,
                }];
            }

            return vec![
                PointerScannerContextAction::Address {
                    label: "Open Base Address in Pointer Scan",
                    address: runtime_pointer.get_address(),
                    module_name: runtime_pointer.get_module_name().to_string(),
                    data_type_id: data_type_id.clone(),
                },
                PointerScannerContextAction::ResolvedPointer {
                    label: "Open Resolved Address in Pointer Scan",
                    pointer: runtime_pointer,
                    data_type_id,
                },
            ];
        }

        if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            let pointer = ProjectItemTypePointer::get_field_pointer(project_item);
            let data_type_id = ProjectItemTypePointer::get_field_symbolic_struct_definition_reference(project_item)
                .map(|symbolic_struct_reference| {
                    symbolic_struct_reference
                        .get_symbolic_struct_namespace()
                        .to_string()
                })
                .unwrap_or_default();

            return vec![
                PointerScannerContextAction::Address {
                    label: "Open Base Address in Pointer Scan",
                    address: pointer.get_address(),
                    module_name: pointer.get_module_name().to_string(),
                    data_type_id: data_type_id.clone(),
                },
                PointerScannerContextAction::ResolvedPointer {
                    label: "Open Resolved Address in Pointer Scan",
                    pointer,
                    data_type_id,
                },
            ];
        }

        Vec::new()
    }

    fn resolve_pointer_scanner_context_action(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        pointer_scanner_context_action: &PointerScannerContextAction,
    ) -> Option<(u64, String, String)> {
        match pointer_scanner_context_action {
            PointerScannerContextAction::Address {
                address,
                module_name,
                data_type_id,
                ..
            } => Some((*address, module_name.clone(), data_type_id.clone())),
            PointerScannerContextAction::ResolvedPointer { pointer, data_type_id, .. } => {
                let (address, module_name) = ProjectItemDetails::resolve_pointer_write_target(engine_execution_context, pointer)?;

                Some((address, module_name, data_type_id.clone()))
            }
        }
    }

    fn focus_pointer_scanner_for_address(
        &self,
        address: u64,
        module_name: &str,
        data_type_id: &str,
    ) {
        let (resolved_target_address, resolved_target_module_name) = if module_name.trim().is_empty() {
            (address, String::new())
        } else if try_resolve_virtual_module_address(module_name, address).is_some() {
            (address, module_name.to_string())
        } else if let Some(resolved_absolute_address) = ProjectItemDetails::dispatch_memory_query_request(&self.app_context.engine_unprivileged_state)
            .and_then(|memory_query_response| ProjectItemDetails::resolve_module_relative_address(&memory_query_response.modules, address, module_name))
        {
            (resolved_absolute_address, String::new())
        } else {
            log::warn!(
                "Failed to resolve pointer scanner target for module-relative address {}+0x{:X}; falling back to unresolved offset.",
                module_name,
                address
            );
            (address, module_name.to_string())
        };

        PointerScannerViewData::set_scan_target_from_project_address(
            self.pointer_scanner_view_data.clone(),
            resolved_target_address,
            &resolved_target_module_name,
            data_type_id,
        );

        match self.app_context.docking_manager.write() {
            Ok(mut docking_manager) => {
                docking_manager.set_window_visibility(PointerScannerView::WINDOW_ID, true);
                docking_manager.select_tab_by_window_id(PointerScannerView::WINDOW_ID);
            }
            Err(error) => {
                log::error!("Failed to acquire docking manager while opening the pointer scanner: {}", error);
            }
        }
    }

    fn focus_memory_viewer_for_address(
        &self,
        address: u64,
        module_name: &str,
        selection_byte_count: u64,
    ) {
        MemoryViewerViewData::request_focus_address_range(
            self.memory_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            address,
            module_name.to_string(),
            selection_byte_count,
        );

        match self.app_context.docking_manager.write() {
            Ok(mut docking_manager) => {
                docking_manager.set_window_visibility(MemoryViewerView::WINDOW_ID, true);
                docking_manager.select_tab_by_window_id(MemoryViewerView::WINDOW_ID);
            }
            Err(error) => {
                log::error!("Failed to acquire docking manager while opening the memory viewer: {}", error);
            }
        }
    }

    fn focus_code_viewer_for_address(
        &self,
        address: u64,
        module_name: &str,
    ) {
        CodeViewerViewData::request_focus_address(
            self.code_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            address,
            module_name.to_string(),
        );

        match self.app_context.docking_manager.write() {
            Ok(mut docking_manager) => {
                docking_manager.set_window_visibility(CodeViewerView::WINDOW_ID, true);
                docking_manager.select_tab_by_window_id(CodeViewerView::WINDOW_ID);
            }
            Err(error) => {
                log::error!("Failed to acquire docking manager while opening the code viewer: {}", error);
            }
        }
    }

    fn resolve_tree_entry_icon(
        app_context: Arc<AppContext>,
        opened_project_info: Option<&ProjectInfo>,
        project_item: &ProjectItem,
    ) -> Option<TextureHandle> {
        let icon_library = &app_context.theme.icon_library;
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id == ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID {
            Some(icon_library.icon_handle_file_system_open_folder.clone())
        } else {
            let icon_data_type_id = ProjectItemDetailsProjection::resolve_project_item_icon_data_type_id(project_item).unwrap_or_default();
            let is_symbol_layout = opened_project_info.is_some_and(|project_info| {
                project_info
                    .get_project_symbol_catalog()
                    .contains_struct_layout_id(&icon_data_type_id)
            });

            Some(DataTypeToIconConverter::convert_data_type_or_symbol_layout_to_icon(
                &icon_data_type_id,
                is_symbol_layout,
                icon_library,
            ))
        }
    }

    fn refresh_if_project_changed(&self) {
        let (opened_project_directory_path, opened_project_item_signatures, opened_project_sort_order) = match self
            .app_context
            .engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .read()
        {
            Ok(opened_project_guard) => opened_project_guard
                .as_ref()
                .map(|opened_project| {
                    let opened_project_directory_path = opened_project.get_project_info().get_project_directory();
                    let opened_project_item_signatures = Self::collect_project_item_change_signatures(opened_project.get_project_items().iter());
                    let opened_project_sort_order = opened_project
                        .get_project_info()
                        .get_project_manifest()
                        .get_project_item_sort_order()
                        .iter()
                        .cloned()
                        .collect::<Vec<PathBuf>>();

                    (opened_project_directory_path, opened_project_item_signatures, opened_project_sort_order)
                })
                .unwrap_or((None, HashMap::new(), Vec::new())),
            Err(error) => {
                log::error!("Failed to acquire opened project lock for hierarchy refresh check: {}", error);
                (None, HashMap::new(), Vec::new())
            }
        };

        let (loaded_project_directory_path, loaded_project_item_signatures, loaded_project_sort_order) = self
            .project_hierarchy_view_data
            .read("Project hierarchy refresh check")
            .map(|project_hierarchy_view_data| {
                let loaded_project_directory_path = project_hierarchy_view_data
                    .opened_project_info
                    .as_ref()
                    .and_then(|project_info| project_info.get_project_directory());
                let loaded_project_item_signatures = Self::collect_project_item_change_signatures(
                    project_hierarchy_view_data
                        .project_items
                        .iter()
                        .map(|(project_item_ref, project_item)| (project_item_ref, project_item)),
                );
                let loaded_project_sort_order = project_hierarchy_view_data
                    .opened_project_info
                    .as_ref()
                    .map(|project_info| {
                        project_info
                            .get_project_manifest()
                            .get_project_item_sort_order()
                            .iter()
                            .cloned()
                            .collect::<Vec<PathBuf>>()
                    })
                    .unwrap_or_default();

                (loaded_project_directory_path, loaded_project_item_signatures, loaded_project_sort_order)
            })
            .unwrap_or((None, HashMap::new(), Vec::new()));

        let project_directory_changed = opened_project_directory_path != loaded_project_directory_path;
        let project_items_changed = opened_project_item_signatures != loaded_project_item_signatures;
        let sort_order_changed = opened_project_sort_order != loaded_project_sort_order;

        if project_directory_changed || project_items_changed || sort_order_changed {
            ProjectHierarchyViewData::refresh_project_items(self.project_hierarchy_view_data.clone(), self.app_context.clone());
        }
    }

    fn collect_project_item_change_signatures<'a>(
        project_items: impl IntoIterator<Item = (&'a ProjectItemRef, &'a ProjectItem)>
    ) -> HashMap<PathBuf, ProjectItemChangeSignature> {
        project_items
            .into_iter()
            .map(|(project_item_ref, project_item)| {
                (
                    project_item_ref.get_project_item_path().clone(),
                    ProjectItemChangeSignature {
                        project_item_type_id: project_item
                            .get_item_type()
                            .get_project_item_type_id()
                            .to_string(),
                        project_item_name: project_item.get_field_name(),
                        project_item_description: project_item.get_field_description(),
                        project_item_properties: project_item
                            .get_properties()
                            .get_fields()
                            .iter()
                            .map(|field| (field.get_name().to_string(), field.get_bytes()))
                            .collect(),
                    },
                )
            })
            .collect()
    }

    fn project_item_rename_text_storage_id(project_item_path: &Path) -> Id {
        Id::new(("project_hierarchy_rename_text", project_item_path.to_path_buf()))
    }

    fn project_item_rename_highlight_storage_id(project_item_path: &Path) -> Id {
        Id::new(("project_hierarchy_rename_highlight", project_item_path.to_path_buf()))
    }

    fn clear_project_item_rename_state(
        user_interface: &Ui,
        project_item_path: &Path,
    ) {
        let rename_text_storage_id = Self::project_item_rename_text_storage_id(project_item_path);
        let rename_highlight_storage_id = Self::project_item_rename_highlight_storage_id(project_item_path);

        user_interface.ctx().data_mut(|data| {
            data.remove::<String>(rename_text_storage_id);
            data.remove::<bool>(rename_highlight_storage_id);
        });
    }

    fn clear_active_project_item_rename_state(
        user_interface: &Ui,
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
    ) {
        let active_project_item_path = project_hierarchy_view_data
            .read("Project hierarchy resolve active inline rename state")
            .and_then(|project_hierarchy_view_data| match &project_hierarchy_view_data.take_over_state {
                ProjectHierarchyTakeOverState::RenameProjectItem { project_item_path, .. } => Some(project_item_path.clone()),
                _ => None,
            });

        if let Some(active_project_item_path) = active_project_item_path {
            Self::clear_project_item_rename_state(user_interface, &active_project_item_path);
        }
    }
}
