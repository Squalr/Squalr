use crate::{
    app_context::AppContext,
    ui::widgets::controls::{
        context_menu::context_menu::{ContextMenu, ContextMenuSizing},
        toolbar_menu::toolbar_menu_item_view::ToolbarMenuItemView,
    },
    views::{
        context_menu_labels::{OPEN_IN_CODE_VIEWER_LABEL, OPEN_IN_MEMORY_VIEWER_LABEL},
        project_explorer::project_hierarchy::{
            project_hierarchy_create_item_menu_view::ProjectHierarchyCreateItemMenuView,
            view_data::{
                project_hierarchy_frame_action::ProjectHierarchyFrameAction, project_hierarchy_menu_target::ProjectHierarchyMenuTarget,
                project_hierarchy_tree_entry::ProjectHierarchyTreeEntry, project_hierarchy_view_data::ProjectHierarchyViewData,
            },
        },
    },
};
use eframe::egui::{Pos2, Response, Ui};
use squalr_engine::services::projects::project_item_symbol_resolution::{
    can_open_project_item_in_memory_viewer, resolve_address_target_runtime_pointer_with_optional_catalog, resolve_pointer_runtime_target,
    resolve_project_item_runtime_value_byte_count, resolve_project_item_runtime_value_target, should_open_project_item_in_code_viewer,
};
use squalr_engine_api::{
    dependency_injection::dependency::Dependency,
    engine::engine_execution_context::EngineExecutionContext,
    structures::{
        memory::pointer::Pointer,
        projects::{
            project_info::ProjectInfo,
            project_items::{
                built_in_types::{project_item_type_address::ProjectItemTypeAddress, project_item_type_pointer::ProjectItemTypePointer},
                project_item::ProjectItem,
            },
        },
    },
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

pub struct ProjectHierarchyProjectItemContextMenuView<'lifetime> {
    app_context: Arc<AppContext>,
    project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
    opened_project_info: Option<&'lifetime ProjectInfo>,
    tree_entry: &'lifetime ProjectHierarchyTreeEntry,
    row_response: &'lifetime Response,
    selected_project_item_paths_in_tree_order: &'lifetime [PathBuf],
    menu_target: Option<&'lifetime ProjectHierarchyMenuTarget>,
    menu_position: Option<Pos2>,
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

impl<'lifetime> ProjectHierarchyProjectItemContextMenuView<'lifetime> {
    const STRIP_SYMBOL_INFORMATION_LABEL: &'static str = "Strip Symbol Information";
    const PROJECT_ITEM_CTX_OPEN_MEMORY_VIEWER_LABEL: &'static str = OPEN_IN_MEMORY_VIEWER_LABEL;
    const PROJECT_ITEM_CTX_OPEN_CODE_VIEWER_LABEL: &'static str = OPEN_IN_CODE_VIEWER_LABEL;
    const PROJECT_ITEM_CTX_OPEN_RUNTIME_VIEWER_ID: &'static str = "project_hierarchy_ctx_open_runtime_viewer";
    const PROJECT_ITEM_CTX_PROMOTE_TO_SYMBOL_LABEL: &'static str = "Promote to Symbol";
    const PROJECT_ITEM_CTX_PROMOTE_TO_SYMBOL_ID: &'static str = "project_hierarchy_ctx_promote_to_symbol";
    const PROJECT_ITEM_CTX_STRIP_SYMBOL_INFORMATION_ID: &'static str = "project_hierarchy_ctx_strip_symbol_information";
    const PROJECT_ITEM_CTX_CUT_LABEL: &'static str = "Cut";
    const PROJECT_ITEM_CTX_CUT_ID: &'static str = "project_hierarchy_ctx_cut";
    const PROJECT_ITEM_CTX_COPY_LABEL: &'static str = "Copy";
    const PROJECT_ITEM_CTX_COPY_ID: &'static str = "project_hierarchy_ctx_copy";
    const PROJECT_ITEM_CTX_PASTE_LABEL: &'static str = "Paste";
    const PROJECT_ITEM_CTX_PASTE_ID: &'static str = "project_hierarchy_ctx_paste";
    const PROJECT_ITEM_CTX_DELETE_LABEL: &'static str = "Delete";
    const PROJECT_ITEM_CTX_DELETE_ID: &'static str = "project_hierarchy_ctx_delete";

    pub fn new(
        app_context: Arc<AppContext>,
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        opened_project_info: Option<&'lifetime ProjectInfo>,
        tree_entry: &'lifetime ProjectHierarchyTreeEntry,
        row_response: &'lifetime Response,
        selected_project_item_paths_in_tree_order: &'lifetime [PathBuf],
        menu_target: Option<&'lifetime ProjectHierarchyMenuTarget>,
        menu_position: Option<Pos2>,
    ) -> Self {
        Self {
            app_context,
            project_hierarchy_view_data,
            opened_project_info,
            tree_entry,
            row_response,
            selected_project_item_paths_in_tree_order,
            menu_target,
            menu_position,
        }
    }

    pub fn show(
        self,
        user_interface: &mut Ui,
    ) -> Vec<ProjectHierarchyFrameAction> {
        let default_context_menu_position = self.row_response.rect.left_bottom();
        let mut frame_actions = Vec::new();

        if self.row_response.secondary_clicked() {
            ProjectHierarchyViewData::show_project_item_menu(
                self.project_hierarchy_view_data.clone(),
                self.tree_entry.project_item_path.clone(),
                self.row_response
                    .hover_pos()
                    .unwrap_or(default_context_menu_position),
            );
        }

        let is_context_menu_visible = matches!(self.menu_target, Some(ProjectHierarchyMenuTarget::ProjectItem(menu_project_item_path)) if menu_project_item_path == &self.tree_entry.project_item_path);

        if !is_context_menu_visible {
            return frame_actions;
        }

        let mut open = true;
        self.show_context_menu(user_interface, default_context_menu_position, &mut open, &mut frame_actions);

        if !open {
            ProjectHierarchyViewData::hide_menu(self.project_hierarchy_view_data.clone());
        }

        frame_actions
    }

    fn show_context_menu(
        &self,
        user_interface: &mut Ui,
        default_context_menu_position: Pos2,
        open: &mut bool,
        frame_actions: &mut Vec<ProjectHierarchyFrameAction>,
    ) {
        let tree_entry_project_item_path = self.tree_entry.project_item_path.clone();
        let project_item_paths_for_action = if self
            .selected_project_item_paths_in_tree_order
            .contains(&tree_entry_project_item_path)
            && self.selected_project_item_paths_in_tree_order.len() > 1
        {
            self.selected_project_item_paths_in_tree_order.to_vec()
        } else {
            vec![tree_entry_project_item_path.clone()]
        };
        let can_delete_project_item_paths =
            ProjectHierarchyViewData::has_deletable_project_item_paths(self.project_hierarchy_view_data.clone(), &project_item_paths_for_action);
        let can_copy_project_item_paths = can_delete_project_item_paths;
        let can_cut_project_item_paths = can_delete_project_item_paths;
        let can_promote_project_item_paths =
            ProjectHierarchyViewData::has_promotable_project_item_paths(self.project_hierarchy_view_data.clone(), &project_item_paths_for_action);
        let can_strip_symbol_project_item_paths =
            ProjectHierarchyViewData::has_strippable_symbol_project_item_paths(self.project_hierarchy_view_data.clone(), &project_item_paths_for_action);
        let has_symbolic_address_project_item_paths =
            ProjectHierarchyViewData::has_symbolic_address_project_item_paths(self.project_hierarchy_view_data.clone(), &project_item_paths_for_action);
        let can_promote_project_item_paths = can_promote_project_item_paths && !has_symbolic_address_project_item_paths;
        let can_paste_project_items =
            ProjectHierarchyViewData::can_paste_project_item_clipboard(self.project_hierarchy_view_data.clone(), &tree_entry_project_item_path);
        let pointer_scanner_context_actions = Self::build_pointer_scanner_context_actions(self.opened_project_info, &self.tree_entry.project_item);
        let can_open_in_memory_viewer = can_open_project_item_in_memory_viewer(&self.tree_entry.project_item);
        let should_open_in_code_viewer = should_open_project_item_in_code_viewer(&self.tree_entry.project_item);
        let runtime_viewer_label = if should_open_in_code_viewer {
            Self::PROJECT_ITEM_CTX_OPEN_CODE_VIEWER_LABEL
        } else {
            Self::PROJECT_ITEM_CTX_OPEN_MEMORY_VIEWER_LABEL
        };
        let mut project_item_menu_labels = pointer_scanner_context_actions
            .iter()
            .map(PointerScannerContextAction::label)
            .collect::<Vec<_>>();
        let has_runtime_actions =
            !pointer_scanner_context_actions.is_empty() || can_open_in_memory_viewer || can_strip_symbol_project_item_paths || can_promote_project_item_paths;
        let has_create_actions = true;
        let has_clipboard_actions = can_cut_project_item_paths || can_copy_project_item_paths || can_paste_project_items;
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
            project_item_menu_labels.extend(ProjectHierarchyCreateItemMenuView::labels());
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

        let project_item_menu_width = ContextMenuSizing::width_for_labels(self.app_context.as_ref(), user_interface, project_item_menu_labels.iter().copied());
        ContextMenu::new(
            self.app_context.clone(),
            "project_hierarchy_context_menu",
            self.menu_position.unwrap_or(default_context_menu_position),
            |user_interface, should_close| {
                self.show_runtime_context_menu_items(
                    user_interface,
                    &tree_entry_project_item_path,
                    &pointer_scanner_context_actions,
                    runtime_viewer_label,
                    should_open_in_code_viewer,
                    can_open_in_memory_viewer,
                    project_item_menu_width,
                    should_close,
                    frame_actions,
                );

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
                        frame_actions.push(ProjectHierarchyFrameAction::PromoteToSymbol {
                            project_item_paths: project_item_paths_for_action.clone(),
                            overwrite_conflicting_symbols: false,
                        });
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
                        frame_actions.push(ProjectHierarchyFrameAction::StripSymbolInformation {
                            project_item_paths: project_item_paths_for_action.clone(),
                        });
                        *should_close = true;
                    }
                }

                if has_runtime_actions && has_create_actions {
                    user_interface.separator();
                }

                let create_project_item_action = ProjectHierarchyCreateItemMenuView::show_items(
                    self.app_context.clone(),
                    user_interface,
                    &tree_entry_project_item_path,
                    project_item_menu_width,
                    should_close,
                );

                if create_project_item_action != ProjectHierarchyFrameAction::None {
                    frame_actions.push(create_project_item_action);
                }

                if (has_runtime_actions || has_create_actions) && has_clipboard_actions {
                    user_interface.separator();
                }

                self.show_clipboard_context_menu_items(
                    user_interface,
                    &tree_entry_project_item_path,
                    &project_item_paths_for_action,
                    can_cut_project_item_paths,
                    can_copy_project_item_paths,
                    can_paste_project_items,
                    project_item_menu_width,
                    should_close,
                    frame_actions,
                );

                if (has_runtime_actions || has_create_actions || has_clipboard_actions) && has_delete_actions {
                    user_interface.separator();
                }

                if has_delete_actions {
                    if user_interface
                        .add(
                            ToolbarMenuItemView::new(
                                self.app_context.clone(),
                                Self::PROJECT_ITEM_CTX_DELETE_LABEL,
                                Self::PROJECT_ITEM_CTX_DELETE_ID,
                                &None,
                                project_item_menu_width,
                            )
                            .icon(
                                self.app_context
                                    .theme
                                    .icon_library
                                    .icon_handle_common_delete
                                    .clone(),
                            )
                            .icon_background(
                                self.app_context.theme.background_control_danger,
                                self.app_context.theme.background_control_danger_dark,
                            ),
                        )
                        .clicked()
                    {
                        frame_actions.push(ProjectHierarchyFrameAction::RequestDeleteConfirmation(project_item_paths_for_action.clone()));
                        *should_close = true;
                    }
                }
            },
        )
        .width(project_item_menu_width)
        .corner_radius(8)
        .show(user_interface, open);
    }

    fn show_runtime_context_menu_items(
        &self,
        user_interface: &mut Ui,
        tree_entry_project_item_path: &Path,
        pointer_scanner_context_actions: &[PointerScannerContextAction],
        runtime_viewer_label: &str,
        should_open_in_code_viewer: bool,
        can_open_in_memory_viewer: bool,
        project_item_menu_width: f32,
        should_close: &mut bool,
        frame_actions: &mut Vec<ProjectHierarchyFrameAction>,
    ) {
        if !pointer_scanner_context_actions.is_empty() {
            let engine_execution_context: Arc<dyn EngineExecutionContext> = self.app_context.engine_unprivileged_state.clone();

            for pointer_scanner_context_action in pointer_scanner_context_actions {
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
                    if let Some((address, module_name, data_type_id)) =
                        Self::resolve_pointer_scanner_context_action(&engine_execution_context, pointer_scanner_context_action)
                    {
                        frame_actions.push(ProjectHierarchyFrameAction::OpenPointerScannerForAddress {
                            address,
                            module_name,
                            data_type_id,
                        });
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
                    .icon(if should_open_in_code_viewer {
                        self.app_context
                            .theme
                            .icon_library
                            .icon_handle_project_cpu_instruction
                            .clone()
                    } else {
                        self.app_context
                            .theme
                            .icon_library
                            .icon_handle_scan_collect_values
                            .clone()
                    }),
                )
                .clicked()
            {
                let engine_execution_context: Arc<dyn EngineExecutionContext> = self.app_context.engine_unprivileged_state.clone();

                let project_symbol_catalog = self
                    .opened_project_info
                    .map(|opened_project_info| opened_project_info.get_project_symbol_catalog());

                if let Some((address, module_name)) =
                    resolve_project_item_runtime_value_target(&engine_execution_context, project_symbol_catalog, &self.tree_entry.project_item)
                {
                    let frame_action = if should_open_in_code_viewer {
                        ProjectHierarchyFrameAction::OpenCodeViewerForAddress { address, module_name }
                    } else {
                        ProjectHierarchyFrameAction::OpenMemoryViewerForAddress {
                            address,
                            module_name,
                            selection_byte_count: resolve_project_item_runtime_value_byte_count(&engine_execution_context, &self.tree_entry.project_item)
                                .unwrap_or(1),
                        }
                    };

                    frame_actions.push(frame_action);
                    *should_close = true;
                } else {
                    log::error!("Failed to resolve memory viewer target for project item: {:?}.", tree_entry_project_item_path);
                }
            }
        }
    }

    fn show_clipboard_context_menu_items(
        &self,
        user_interface: &mut Ui,
        tree_entry_project_item_path: &Path,
        project_item_paths_for_action: &[PathBuf],
        can_cut_project_item_paths: bool,
        can_copy_project_item_paths: bool,
        can_paste_project_items: bool,
        project_item_menu_width: f32,
        should_close: &mut bool,
        frame_actions: &mut Vec<ProjectHierarchyFrameAction>,
    ) {
        if can_cut_project_item_paths {
            if user_interface
                .add(
                    ToolbarMenuItemView::new(
                        self.app_context.clone(),
                        Self::PROJECT_ITEM_CTX_CUT_LABEL,
                        Self::PROJECT_ITEM_CTX_CUT_ID,
                        &None,
                        project_item_menu_width,
                    )
                    .icon(
                        self.app_context
                            .theme
                            .icon_library
                            .icon_handle_data_type_unknown
                            .clone(),
                    ),
                )
                .clicked()
            {
                frame_actions.push(ProjectHierarchyFrameAction::CutProjectItems(project_item_paths_for_action.to_vec()));
                *should_close = true;
            }
        }

        if can_copy_project_item_paths {
            if user_interface
                .add(
                    ToolbarMenuItemView::new(
                        self.app_context.clone(),
                        Self::PROJECT_ITEM_CTX_COPY_LABEL,
                        Self::PROJECT_ITEM_CTX_COPY_ID,
                        &None,
                        project_item_menu_width,
                    )
                    .icon(
                        self.app_context
                            .theme
                            .icon_library
                            .icon_handle_data_type_unknown
                            .clone(),
                    ),
                )
                .clicked()
            {
                frame_actions.push(ProjectHierarchyFrameAction::CopyProjectItems(project_item_paths_for_action.to_vec()));
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
                    .icon(
                        self.app_context
                            .theme
                            .icon_library
                            .icon_handle_data_type_unknown
                            .clone(),
                    ),
                )
                .clicked()
            {
                frame_actions.push(ProjectHierarchyFrameAction::PasteProjectItems {
                    target_project_item_path: tree_entry_project_item_path.to_path_buf(),
                });
                *should_close = true;
            }
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
            let project_symbol_catalog = opened_project_info.map(|opened_project_info| opened_project_info.get_project_symbol_catalog());
            let Some(runtime_pointer) = resolve_address_target_runtime_pointer_with_optional_catalog(project_symbol_catalog, &address_target) else {
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
                let (address, module_name) = resolve_pointer_runtime_target(engine_execution_context, pointer)?;

                Some((address, module_name, data_type_id.clone()))
            }
        }
    }
}
