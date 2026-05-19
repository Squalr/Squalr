use crate::app_context::AppContext;
use crate::ui::widgets::controls::{
    context_menu::context_menu::{ContextMenu, ContextMenuSizing},
    toolbar_menu::toolbar_menu_item_view::ToolbarMenuItemView,
};
use crate::views::{
    context_menu_labels::{OPEN_IN_CODE_VIEWER_LABEL, OPEN_IN_MEMORY_VIEWER_LABEL},
    struct_viewer::view_data::struct_viewer_focus_target::StructViewerFocusTarget,
    symbol_tree::{
        symbol_tree_details_focus::SymbolTreeDetailsFocus,
        symbol_tree_entry_view::SymbolTreeEntryView,
        symbol_tree_inline_rename_view::SymbolTreeInlineRenameView,
        view_data::symbol_tree_view_data::{SymbolTreeContextMenuTarget, SymbolTreeSelection, SymbolTreeViewData},
    },
};
use eframe::egui::{Id, Ui};
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::plugins::symbol_tree::symbol_tree_action::{SymbolTreeActionContext, SymbolTreeActionSelection};
use squalr_engine_api::structures::projects::{
    project_symbol_catalog::ProjectSymbolCatalog,
    project_symbol_locator::ProjectSymbolLocator,
    symbol_tree::operations::{
        add_symbol_to_project::{AddSymbolToProjectTarget, build_add_symbol_to_project_target},
        build_symbol_tree::resolve_symbol_tree_node_size_in_bytes,
        delete_symbol::{ModuleChildRangeTarget, build_module_child_range_target},
        edit_symbol_layout::build_symbol_layout_edit_target,
    },
    symbol_tree::symbol_tree_node::{SymbolTreeNode, SymbolTreeNodeKind},
};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub struct SymbolTreeListView<'lifetime> {
    app_context: Arc<AppContext>,
    project_symbol_catalog: &'lifetime ProjectSymbolCatalog,
    symbol_tree_entries: &'lifetime [SymbolTreeNode],
    preview_values_by_node_key: &'lifetime HashMap<String, String>,
    expanded_tree_node_keys: &'lifetime HashSet<String>,
    selected_entry: Option<&'lifetime SymbolTreeSelection>,
    inline_rename_tree_node_key: Option<&'lifetime str>,
    context_menu_target: Option<&'lifetime SymbolTreeContextMenuTarget>,
    shared_struct_viewer_focus_target: Option<&'lifetime StructViewerFocusTarget>,
    allow_interaction: bool,
}

#[derive(Clone, Debug)]
pub enum SymbolTreeListAction {
    Select(SymbolTreeSelection),
    ToggleExpansion(String),
    FocusStructViewer(SymbolTreeNode),
    OpenMemoryViewer(ProjectSymbolLocator),
    OpenCodeViewer(ProjectSymbolLocator),
    AddSymbolToProject(AddSymbolToProjectTarget),
    EditSymbolLayout(String),
    BeginInlineRename(String),
    CommitRename(SymbolTreeRenameTarget),
    CancelInlineRename(String),
    ShowContextMenu(SymbolTreeContextMenuTarget),
    HideContextMenu,
    RequestDelete(SymbolTreeDeleteTarget),
    ExecutePluginAction(SymbolTreePluginActionMenuItem, SymbolTreeActionContext),
    CreateModuleRoot,
}

#[derive(Clone, Debug)]
pub enum SymbolTreeRenameTarget {
    ModuleRoot { module_name: String, new_module_name: String },
    SymbolClaim { symbol_locator_key: String, display_name: String },
}

#[derive(Clone, Debug)]
pub enum SymbolTreeDeleteTarget {
    ModuleRange(ModuleChildRangeTarget),
    SymbolClaim { symbol_locator_key: String, display_name: String },
    ModuleRoot { module_name: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolTreePluginActionMenuItem {
    pub plugin_id: String,
    pub action_id: String,
    pub label: String,
}

impl<'lifetime> SymbolTreeListView<'lifetime> {
    const INLINE_RENAME_TEXT_STORAGE_ID_PREFIX: &'static str = "symbol_tree_inline_rename_text";
    const INLINE_RENAME_HIGHLIGHT_STORAGE_ID_PREFIX: &'static str = "symbol_tree_inline_rename_highlight";
    const SYMBOL_TREE_CTX_OPEN_MEMORY_VIEWER_LABEL: &'static str = OPEN_IN_MEMORY_VIEWER_LABEL;
    const SYMBOL_TREE_CTX_OPEN_MEMORY_VIEWER_ID: &'static str = "symbol_tree_ctx_open_memory_viewer";
    const SYMBOL_TREE_CTX_OPEN_CODE_VIEWER_LABEL: &'static str = OPEN_IN_CODE_VIEWER_LABEL;
    const SYMBOL_TREE_CTX_OPEN_CODE_VIEWER_ID: &'static str = "symbol_tree_ctx_open_code_viewer";
    const SYMBOL_TREE_CTX_ADD_TO_PROJECT_LABEL: &'static str = "Add to Project";
    const SYMBOL_TREE_CTX_ADD_TO_PROJECT_ID: &'static str = "symbol_tree_ctx_add_to_project";
    const SYMBOL_TREE_CTX_EDIT_SYMBOL_LAYOUT_LABEL: &'static str = "Edit Symbol Layout...";
    const SYMBOL_TREE_CTX_EDIT_SYMBOL_LAYOUT_ID: &'static str = "symbol_tree_ctx_edit_symbol_layout";
    const SYMBOL_TREE_CTX_RENAME_LABEL: &'static str = "Rename";
    const SYMBOL_TREE_CTX_RENAME_ID: &'static str = "symbol_tree_ctx_rename";
    const SYMBOL_TREE_CTX_NEW_MODULE_LABEL: &'static str = "New Module";
    const SYMBOL_TREE_CTX_NEW_MODULE_ID: &'static str = "symbol_tree_ctx_new_module";
    const SYMBOL_TREE_CTX_DELETE_LABEL: &'static str = "Delete";
    const SYMBOL_TREE_CTX_DELETE_ID: &'static str = "symbol_tree_ctx_delete";

    pub fn new(
        app_context: Arc<AppContext>,
        project_symbol_catalog: &'lifetime ProjectSymbolCatalog,
        symbol_tree_entries: &'lifetime [SymbolTreeNode],
        preview_values_by_node_key: &'lifetime HashMap<String, String>,
        expanded_tree_node_keys: &'lifetime HashSet<String>,
        selected_entry: Option<&'lifetime SymbolTreeSelection>,
        inline_rename_tree_node_key: Option<&'lifetime str>,
        context_menu_target: Option<&'lifetime SymbolTreeContextMenuTarget>,
        shared_struct_viewer_focus_target: Option<&'lifetime StructViewerFocusTarget>,
        allow_interaction: bool,
    ) -> Self {
        Self {
            app_context,
            project_symbol_catalog,
            symbol_tree_entries,
            preview_values_by_node_key,
            expanded_tree_node_keys,
            selected_entry,
            inline_rename_tree_node_key,
            context_menu_target,
            shared_struct_viewer_focus_target,
            allow_interaction,
        }
    }

    pub fn show(
        self,
        user_interface: &mut Ui,
    ) -> Vec<SymbolTreeListAction> {
        let mut list_actions = Vec::new();

        for symbol_tree_entry in self.symbol_tree_entries {
            let is_locally_selected = Self::is_locally_selected(symbol_tree_entry, self.selected_entry);
            let is_selected = is_locally_selected
                && (matches!(symbol_tree_entry.get_kind(), SymbolTreeNodeKind::ModuleSpace { .. })
                    || Self::is_symbol_tree_entry_struct_viewer_focused(symbol_tree_entry, self.shared_struct_viewer_focus_target));
            let is_inline_rename_row = self
                .inline_rename_tree_node_key
                .is_some_and(|active_inline_rename_tree_node_key| symbol_tree_entry.get_node_key() == active_inline_rename_tree_node_key);

            if is_inline_rename_row {
                Self::render_inline_rename_row(
                    &self.app_context,
                    user_interface,
                    self.expanded_tree_node_keys,
                    symbol_tree_entry,
                    is_selected,
                    &mut list_actions,
                );
                continue;
            }

            self.render_symbol_tree_entry(user_interface, symbol_tree_entry, is_selected, &mut list_actions);
        }

        list_actions
    }

    pub fn build_selection_for_tree_entry(symbol_tree_entry: &SymbolTreeNode) -> Option<SymbolTreeSelection> {
        match symbol_tree_entry.get_kind() {
            SymbolTreeNodeKind::ModuleSpace { module_name, .. } => Some(SymbolTreeSelection::ModuleRoot(module_name.to_string())),
            SymbolTreeNodeKind::SymbolClaim { symbol_locator_key } => {
                if Self::is_module_field_tree_entry(symbol_tree_entry) {
                    Some(SymbolTreeSelection::DerivedNode(symbol_tree_entry.get_node_key().to_string()))
                } else {
                    Some(SymbolTreeSelection::SymbolClaim(symbol_locator_key.to_string()))
                }
            }
            SymbolTreeNodeKind::StructField | SymbolTreeNodeKind::UnassignedSegment { .. } | SymbolTreeNodeKind::PointerTarget => {
                Some(SymbolTreeSelection::DerivedNode(symbol_tree_entry.get_node_key().to_string()))
            }
        }
    }

    pub fn clear_inline_rename_state(
        symbol_tree_view_data: squalr_engine_api::dependency_injection::dependency::Dependency<SymbolTreeViewData>,
        user_interface: &mut Ui,
        tree_node_key: &str,
    ) {
        let rename_text_storage_id = Self::inline_rename_text_storage_id(tree_node_key);
        let rename_highlight_storage_id = Self::inline_rename_highlight_storage_id(tree_node_key);

        user_interface.ctx().data_mut(|data| {
            data.remove::<String>(rename_text_storage_id);
            data.remove::<bool>(rename_highlight_storage_id);
        });
        SymbolTreeViewData::cancel_inline_rename(symbol_tree_view_data);
    }

    fn render_symbol_tree_entry(
        &self,
        user_interface: &mut Ui,
        symbol_tree_entry: &SymbolTreeNode,
        is_selected: bool,
        list_actions: &mut Vec<SymbolTreeListAction>,
    ) {
        let preview_value = self
            .preview_values_by_node_key
            .get(symbol_tree_entry.get_node_key())
            .map(String::as_str)
            .unwrap_or("");
        let size_in_bytes = resolve_symbol_tree_node_size_in_bytes(self.project_symbol_catalog, symbol_tree_entry, |data_type_ref| {
            self.app_context
                .engine_unprivileged_state
                .get_default_value(data_type_ref)
                .map(|default_value| default_value.get_size_in_bytes())
        });
        let size_preview_text = Self::format_symbol_tree_size_preview(size_in_bytes);
        let size_tooltip_text = Self::format_symbol_tree_size_tooltip(size_in_bytes);
        let uses_symbol_layout_icon = Self::symbol_tree_entry_uses_symbol_layout_icon(self.project_symbol_catalog, symbol_tree_entry);
        let is_expanded = Self::symbol_tree_entry_is_expanded(self.expanded_tree_node_keys, symbol_tree_entry);
        let symbol_tree_entry_view_response = SymbolTreeEntryView::new(
            self.app_context.clone(),
            symbol_tree_entry,
            &size_preview_text,
            &size_tooltip_text,
            preview_value,
            uses_symbol_layout_icon,
            is_expanded,
            is_selected,
        )
        .show(user_interface);

        if self.allow_interaction && symbol_tree_entry_view_response.did_click_expand_arrow {
            if let Some(selection) = Self::build_selection_for_tree_entry(symbol_tree_entry) {
                list_actions.push(SymbolTreeListAction::Select(selection));
            }

            list_actions.push(SymbolTreeListAction::ToggleExpansion(symbol_tree_entry.get_node_key().to_string()));
        }

        if self.allow_interaction && symbol_tree_entry_view_response.row_response.double_clicked() && !symbol_tree_entry_view_response.did_click_expand_arrow {
            if let Some(selection) = Self::build_selection_for_tree_entry(symbol_tree_entry) {
                list_actions.push(SymbolTreeListAction::Select(selection));
            }

            if !matches!(symbol_tree_entry.get_kind(), SymbolTreeNodeKind::ModuleSpace { .. }) {
                list_actions.push(SymbolTreeListAction::FocusStructViewer(symbol_tree_entry.clone()));
            }

            return;
        }

        if self.allow_interaction && symbol_tree_entry_view_response.did_click_row {
            if let Some(selection) = Self::build_selection_for_tree_entry(symbol_tree_entry) {
                list_actions.push(SymbolTreeListAction::Select(selection));
                if !matches!(symbol_tree_entry.get_kind(), SymbolTreeNodeKind::ModuleSpace { .. }) {
                    list_actions.push(SymbolTreeListAction::FocusStructViewer(symbol_tree_entry.clone()));
                }
            }
        }

        if self.allow_interaction && symbol_tree_entry_view_response.row_response.secondary_clicked() {
            if let Some(selection) = Self::build_selection_for_tree_entry(symbol_tree_entry) {
                let context_menu_position = symbol_tree_entry_view_response
                    .row_response
                    .interact_pointer_pos()
                    .unwrap_or(symbol_tree_entry_view_response.row_response.rect.left_top());

                list_actions.push(SymbolTreeListAction::Select(selection));
                if !matches!(symbol_tree_entry.get_kind(), SymbolTreeNodeKind::ModuleSpace { .. }) {
                    list_actions.push(SymbolTreeListAction::FocusStructViewer(symbol_tree_entry.clone()));
                }
                list_actions.push(SymbolTreeListAction::ShowContextMenu(SymbolTreeContextMenuTarget::new(
                    symbol_tree_entry.get_node_key().to_string(),
                    context_menu_position,
                )));
            }
        }

        if self.allow_interaction
            && self
                .context_menu_target
                .as_ref()
                .is_some_and(|context_menu_target| context_menu_target.get_tree_node_key() == symbol_tree_entry.get_node_key())
        {
            self.render_context_menu(user_interface, symbol_tree_entry, &symbol_tree_entry_view_response.row_response, list_actions);
        }
    }

    fn render_inline_rename_row(
        app_context: &Arc<AppContext>,
        user_interface: &mut Ui,
        expanded_tree_node_keys: &HashSet<String>,
        symbol_tree_entry: &SymbolTreeNode,
        is_selected: bool,
        list_actions: &mut Vec<SymbolTreeListAction>,
    ) {
        let rename_target_key = symbol_tree_entry.get_node_key();
        let rename_text_storage_id = Self::inline_rename_text_storage_id(rename_target_key);
        let rename_highlight_storage_id = Self::inline_rename_highlight_storage_id(rename_target_key);
        let mut rename_text = user_interface
            .ctx()
            .data_mut(|data| data.get_temp::<String>(rename_text_storage_id))
            .unwrap_or_else(|| symbol_tree_entry.get_display_name().to_string());
        let mut should_highlight_text = user_interface
            .ctx()
            .data_mut(|data| data.get_temp::<bool>(rename_highlight_storage_id))
            .unwrap_or(true);
        let inline_rename_response = SymbolTreeInlineRenameView::new(
            app_context.clone(),
            rename_target_key,
            symbol_tree_entry,
            &mut rename_text,
            &mut should_highlight_text,
            Self::symbol_tree_entry_is_expanded(expanded_tree_node_keys, symbol_tree_entry),
            is_selected,
        )
        .show(user_interface);

        if inline_rename_response.should_commit {
            let trimmed_rename_text = rename_text.trim().to_string();

            if !trimmed_rename_text.is_empty() && trimmed_rename_text != symbol_tree_entry.get_display_name() {
                match symbol_tree_entry.get_kind() {
                    SymbolTreeNodeKind::ModuleSpace { module_name, .. } => {
                        list_actions.push(SymbolTreeListAction::CommitRename(SymbolTreeRenameTarget::ModuleRoot {
                            module_name: module_name.to_string(),
                            new_module_name: trimmed_rename_text,
                        }))
                    }
                    SymbolTreeNodeKind::SymbolClaim { symbol_locator_key } => {
                        list_actions.push(SymbolTreeListAction::CommitRename(SymbolTreeRenameTarget::SymbolClaim {
                            symbol_locator_key: symbol_locator_key.to_string(),
                            display_name: trimmed_rename_text,
                        }))
                    }
                    _ => {}
                }
            }

            list_actions.push(SymbolTreeListAction::CancelInlineRename(rename_target_key.to_string()));
        }

        if inline_rename_response.should_cancel {
            list_actions.push(SymbolTreeListAction::CancelInlineRename(rename_target_key.to_string()));
        }

        user_interface.ctx().data_mut(|data| {
            data.insert_temp(rename_text_storage_id, rename_text);
            data.insert_temp(rename_highlight_storage_id, should_highlight_text);
        });
    }

    fn render_context_menu(
        &self,
        user_interface: &mut Ui,
        symbol_tree_entry: &SymbolTreeNode,
        row_response: &eframe::egui::Response,
        list_actions: &mut Vec<SymbolTreeListAction>,
    ) {
        let can_open_symbol_tree_entry = !matches!(symbol_tree_entry.get_kind(), SymbolTreeNodeKind::ModuleSpace { .. });
        let can_rename_symbol_tree_entry = matches!(
            symbol_tree_entry.get_kind(),
            SymbolTreeNodeKind::ModuleSpace { .. } | SymbolTreeNodeKind::SymbolClaim { .. }
        );
        let context_menu_symbol_claim = match symbol_tree_entry.get_kind() {
            SymbolTreeNodeKind::SymbolClaim { symbol_locator_key } => self
                .project_symbol_catalog
                .get_symbol_claims()
                .iter()
                .find(|symbol_claim| symbol_claim.get_symbol_locator_key() == *symbol_locator_key),
            _ => None,
        };
        let context_menu_module_name = match symbol_tree_entry.get_kind() {
            SymbolTreeNodeKind::ModuleSpace { module_name, .. } => Some(module_name.as_str()),
            _ => None,
        };
        let context_menu_module_child_range_target = build_module_child_range_target(self.project_symbol_catalog, symbol_tree_entry, |data_type_ref| {
            self.app_context
                .engine_unprivileged_state
                .get_default_value(data_type_ref)
                .map(|default_value| default_value.get_size_in_bytes())
        });
        let context_menu_add_symbol_to_project_target = build_add_symbol_to_project_target(symbol_tree_entry);
        let context_menu_symbol_layout_edit_target = build_symbol_layout_edit_target(self.project_symbol_catalog, self.symbol_tree_entries, symbol_tree_entry);
        let context_menu_symbol_tree_action_context = Self::build_symbol_tree_action_context(symbol_tree_entry);
        let context_menu_plugin_action_menu_items = self.build_symbol_tree_plugin_action_menu_items(&context_menu_symbol_tree_action_context);
        let can_delete_symbol_tree_entry =
            context_menu_module_child_range_target.is_some() || context_menu_symbol_claim.is_some() || context_menu_module_name.is_some();
        let mut context_menu_labels = Vec::new();

        if context_menu_symbol_layout_edit_target.is_some() {
            context_menu_labels.push(Self::SYMBOL_TREE_CTX_EDIT_SYMBOL_LAYOUT_LABEL.to_string());
        }
        if can_open_symbol_tree_entry {
            context_menu_labels.push(Self::SYMBOL_TREE_CTX_OPEN_MEMORY_VIEWER_LABEL.to_string());
            context_menu_labels.push(Self::SYMBOL_TREE_CTX_OPEN_CODE_VIEWER_LABEL.to_string());
        }
        if context_menu_add_symbol_to_project_target.is_some() {
            context_menu_labels.push(Self::SYMBOL_TREE_CTX_ADD_TO_PROJECT_LABEL.to_string());
        }
        if can_rename_symbol_tree_entry {
            context_menu_labels.push(Self::SYMBOL_TREE_CTX_RENAME_LABEL.to_string());
        }
        context_menu_labels.extend(
            context_menu_plugin_action_menu_items
                .iter()
                .map(|menu_item| menu_item.label.clone()),
        );
        context_menu_labels.push(Self::SYMBOL_TREE_CTX_NEW_MODULE_LABEL.to_string());
        if can_delete_symbol_tree_entry {
            context_menu_labels.push(Self::SYMBOL_TREE_CTX_DELETE_LABEL.to_string());
        }
        let context_menu_width = ContextMenuSizing::width_for_labels(self.app_context.as_ref(), user_interface, context_menu_labels.iter().map(String::as_str));
        let mut is_context_menu_open = true;

        ContextMenu::new(
            self.app_context.clone(),
            "symbol_tree_context_menu",
            self.context_menu_target
                .as_ref()
                .map(|context_menu_target| context_menu_target.get_position())
                .unwrap_or(row_response.rect.left_top()),
            |user_interface, should_close| {
                if let Some(struct_layout_id) = context_menu_symbol_layout_edit_target.as_deref() {
                    if user_interface
                        .add(
                            ToolbarMenuItemView::new(
                                self.app_context.clone(),
                                Self::SYMBOL_TREE_CTX_EDIT_SYMBOL_LAYOUT_LABEL,
                                Self::SYMBOL_TREE_CTX_EDIT_SYMBOL_LAYOUT_ID,
                                &None,
                                context_menu_width,
                            )
                            .icon(
                                self.app_context
                                    .theme
                                    .icon_library
                                    .icon_handle_common_edit
                                    .clone(),
                            ),
                        )
                        .clicked()
                    {
                        list_actions.push(SymbolTreeListAction::EditSymbolLayout(struct_layout_id.to_string()));
                        *should_close = true;
                    }

                    user_interface.separator();
                }

                if can_open_symbol_tree_entry {
                    if user_interface
                        .add(
                            ToolbarMenuItemView::new(
                                self.app_context.clone(),
                                Self::SYMBOL_TREE_CTX_OPEN_MEMORY_VIEWER_LABEL,
                                Self::SYMBOL_TREE_CTX_OPEN_MEMORY_VIEWER_ID,
                                &None,
                                context_menu_width,
                            )
                            .icon(
                                self.app_context
                                    .theme
                                    .icon_library
                                    .icon_handle_scan_collect_values
                                    .clone(),
                            ),
                        )
                        .clicked()
                    {
                        list_actions.push(SymbolTreeListAction::OpenMemoryViewer(symbol_tree_entry.get_locator().clone()));
                        *should_close = true;
                    }

                    if user_interface
                        .add(
                            ToolbarMenuItemView::new(
                                self.app_context.clone(),
                                Self::SYMBOL_TREE_CTX_OPEN_CODE_VIEWER_LABEL,
                                Self::SYMBOL_TREE_CTX_OPEN_CODE_VIEWER_ID,
                                &None,
                                context_menu_width,
                            )
                            .icon(
                                self.app_context
                                    .theme
                                    .icon_library
                                    .icon_handle_project_cpu_instruction
                                    .clone(),
                            ),
                        )
                        .clicked()
                    {
                        list_actions.push(SymbolTreeListAction::OpenCodeViewer(symbol_tree_entry.get_locator().clone()));
                        *should_close = true;
                    }
                }

                if let Some(add_symbol_to_project_target) = context_menu_add_symbol_to_project_target.as_ref() {
                    if user_interface
                        .add(
                            ToolbarMenuItemView::new(
                                self.app_context.clone(),
                                Self::SYMBOL_TREE_CTX_ADD_TO_PROJECT_LABEL,
                                Self::SYMBOL_TREE_CTX_ADD_TO_PROJECT_ID,
                                &None,
                                context_menu_width,
                            )
                            .icon(
                                self.app_context
                                    .theme
                                    .icon_library
                                    .icon_handle_common_add
                                    .clone(),
                            ),
                        )
                        .clicked()
                    {
                        list_actions.push(SymbolTreeListAction::AddSymbolToProject(add_symbol_to_project_target.clone()));
                        *should_close = true;
                    }
                }

                if (can_open_symbol_tree_entry || context_menu_add_symbol_to_project_target.is_some()) && can_rename_symbol_tree_entry {
                    user_interface.separator();
                }

                if can_rename_symbol_tree_entry
                    && user_interface
                        .add(
                            ToolbarMenuItemView::new(
                                self.app_context.clone(),
                                Self::SYMBOL_TREE_CTX_RENAME_LABEL,
                                Self::SYMBOL_TREE_CTX_RENAME_ID,
                                &None,
                                context_menu_width,
                            )
                            .icon(
                                self.app_context
                                    .theme
                                    .icon_library
                                    .icon_handle_common_edit
                                    .clone(),
                            ),
                        )
                        .clicked()
                {
                    list_actions.push(SymbolTreeListAction::BeginInlineRename(symbol_tree_entry.get_node_key().to_string()));
                    *should_close = true;
                }

                let has_post_layout_edit_menu_items =
                    can_open_symbol_tree_entry || context_menu_add_symbol_to_project_target.is_some() || can_rename_symbol_tree_entry;

                if has_post_layout_edit_menu_items && !context_menu_plugin_action_menu_items.is_empty() {
                    user_interface.separator();
                }

                for plugin_action_menu_item in &context_menu_plugin_action_menu_items {
                    let plugin_action_widget_id = format!(
                        "symbol_tree_ctx_plugin_action_{}_{}",
                        plugin_action_menu_item.plugin_id, plugin_action_menu_item.action_id
                    );

                    if user_interface
                        .add(
                            ToolbarMenuItemView::new(
                                self.app_context.clone(),
                                &plugin_action_menu_item.label,
                                &plugin_action_widget_id,
                                &None,
                                context_menu_width,
                            )
                            .icon(
                                self.app_context
                                    .theme
                                    .icon_library
                                    .icon_handle_data_type_blue_blocks_4
                                    .clone(),
                            ),
                        )
                        .clicked()
                    {
                        list_actions.push(SymbolTreeListAction::ExecutePluginAction(
                            plugin_action_menu_item.clone(),
                            context_menu_symbol_tree_action_context.clone(),
                        ));
                        *should_close = true;
                    }
                }

                if has_post_layout_edit_menu_items || !context_menu_plugin_action_menu_items.is_empty() {
                    user_interface.separator();
                }

                if user_interface
                    .add(
                        ToolbarMenuItemView::new(
                            self.app_context.clone(),
                            Self::SYMBOL_TREE_CTX_NEW_MODULE_LABEL,
                            Self::SYMBOL_TREE_CTX_NEW_MODULE_ID,
                            &None,
                            context_menu_width,
                        )
                        .icon(
                            self.app_context
                                .theme
                                .icon_library
                                .icon_handle_common_add
                                .clone(),
                        ),
                    )
                    .clicked()
                {
                    list_actions.push(SymbolTreeListAction::CreateModuleRoot);
                    *should_close = true;
                }

                if can_delete_symbol_tree_entry {
                    user_interface.separator();

                    if user_interface
                        .add(
                            ToolbarMenuItemView::new(
                                self.app_context.clone(),
                                Self::SYMBOL_TREE_CTX_DELETE_LABEL,
                                Self::SYMBOL_TREE_CTX_DELETE_ID,
                                &None,
                                context_menu_width,
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
                        if let Some(module_child_range_target) = context_menu_module_child_range_target.as_ref() {
                            list_actions.push(SymbolTreeListAction::RequestDelete(SymbolTreeDeleteTarget::ModuleRange(
                                module_child_range_target.clone(),
                            )));
                        } else if let Some(symbol_claim) = context_menu_symbol_claim {
                            list_actions.push(SymbolTreeListAction::RequestDelete(SymbolTreeDeleteTarget::SymbolClaim {
                                symbol_locator_key: symbol_claim.get_symbol_locator_key().to_string(),
                                display_name: symbol_claim.get_display_name().to_string(),
                            }));
                        } else if let Some(module_name) = context_menu_module_name {
                            list_actions.push(SymbolTreeListAction::RequestDelete(SymbolTreeDeleteTarget::ModuleRoot {
                                module_name: module_name.to_string(),
                            }));
                        }
                        *should_close = true;
                    }
                }
            },
        )
        .width(context_menu_width)
        .corner_radius(8)
        .show(user_interface, &mut is_context_menu_open);

        if !is_context_menu_open {
            list_actions.push(SymbolTreeListAction::HideContextMenu);
        }
    }

    fn build_symbol_tree_action_context(symbol_tree_entry: &SymbolTreeNode) -> SymbolTreeActionContext {
        match symbol_tree_entry.get_kind() {
            SymbolTreeNodeKind::ModuleSpace { module_name, .. } => SymbolTreeActionContext::new(SymbolTreeActionSelection::ModuleRoot {
                module_name: module_name.to_string(),
            }),
            SymbolTreeNodeKind::SymbolClaim { symbol_locator_key } => SymbolTreeActionContext::new(SymbolTreeActionSelection::SymbolLocator {
                symbol_locator_key: symbol_locator_key.to_string(),
            }),
            SymbolTreeNodeKind::UnassignedSegment { module_name, offset, length } => SymbolTreeActionContext::new(SymbolTreeActionSelection::ModuleRange {
                module_name: module_name.to_string(),
                offset: *offset,
                length: *length,
            }),
            SymbolTreeNodeKind::StructField | SymbolTreeNodeKind::PointerTarget => SymbolTreeActionContext::new(SymbolTreeActionSelection::DerivedNode {
                tree_node_key: symbol_tree_entry.get_node_key().to_string(),
            }),
        }
    }

    fn build_symbol_tree_plugin_action_menu_items(
        &self,
        context: &SymbolTreeActionContext,
    ) -> Vec<SymbolTreePluginActionMenuItem> {
        let plugin_registry = self.app_context.engine_unprivileged_state.get_plugin_registry();

        plugin_registry
            .get_enabled_symbol_tree_actions()
            .into_iter()
            .filter(|(plugin_id, symbol_tree_action)| {
                symbol_tree_action.is_visible(context) && plugin_registry.plugin_action_has_required_permissions(plugin_id, symbol_tree_action.as_ref())
            })
            .map(|(plugin_id, symbol_tree_action)| SymbolTreePluginActionMenuItem {
                plugin_id,
                action_id: symbol_tree_action.action_id().to_string(),
                label: symbol_tree_action.label(context),
            })
            .collect()
    }

    fn is_locally_selected(
        symbol_tree_entry: &SymbolTreeNode,
        selected_entry: Option<&SymbolTreeSelection>,
    ) -> bool {
        matches!(
            selected_entry,
            Some(SymbolTreeSelection::ModuleRoot(selected_module_name))
                if matches!(symbol_tree_entry.get_kind(), SymbolTreeNodeKind::ModuleSpace { module_name, .. } if selected_module_name == module_name)
        ) || matches!(
            selected_entry,
            Some(SymbolTreeSelection::SymbolClaim(selected_symbol_locator_key))
                if !Self::is_module_field_tree_entry(symbol_tree_entry)
                    && matches!(symbol_tree_entry.get_kind(), SymbolTreeNodeKind::SymbolClaim { symbol_locator_key } if selected_symbol_locator_key == symbol_locator_key)
        ) || matches!(
            selected_entry,
            Some(SymbolTreeSelection::DerivedNode(selected_node_key)) if selected_node_key == symbol_tree_entry.get_node_key()
        )
    }

    fn is_symbol_tree_entry_struct_viewer_focused(
        symbol_tree_entry: &SymbolTreeNode,
        shared_struct_viewer_focus_target: Option<&StructViewerFocusTarget>,
    ) -> bool {
        let Some(StructViewerFocusTarget::SymbolTree { selection_key }) = shared_struct_viewer_focus_target else {
            return false;
        };

        SymbolTreeDetailsFocus::build_struct_viewer_focus_target_key(Some(symbol_tree_entry))
            .as_ref()
            .is_some_and(|row_selection_key| row_selection_key == selection_key)
    }

    fn inline_rename_text_storage_id(symbol_locator_key: &str) -> Id {
        Id::new((Self::INLINE_RENAME_TEXT_STORAGE_ID_PREFIX, symbol_locator_key))
    }

    fn inline_rename_highlight_storage_id(symbol_locator_key: &str) -> Id {
        Id::new((Self::INLINE_RENAME_HIGHLIGHT_STORAGE_ID_PREFIX, symbol_locator_key))
    }

    fn is_module_field_tree_entry(symbol_tree_entry: &SymbolTreeNode) -> bool {
        symbol_tree_entry.get_node_key().starts_with("module_field:")
    }

    fn symbol_tree_entry_uses_symbol_layout_icon(
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entry: &SymbolTreeNode,
    ) -> bool {
        let symbol_layout_id = match symbol_tree_entry.get_kind() {
            SymbolTreeNodeKind::ModuleSpace { module_name, .. } => module_name.as_str(),
            SymbolTreeNodeKind::UnassignedSegment { .. } => return false,
            SymbolTreeNodeKind::SymbolClaim { .. } | SymbolTreeNodeKind::StructField | SymbolTreeNodeKind::PointerTarget => {
                symbol_tree_entry.get_symbol_type_id()
            }
        };

        project_symbol_catalog.contains_struct_layout_id(symbol_layout_id)
    }

    fn symbol_tree_entry_is_expanded(
        expanded_tree_node_keys: &HashSet<String>,
        symbol_tree_entry: &SymbolTreeNode,
    ) -> bool {
        symbol_tree_entry.can_expand() && expanded_tree_node_keys.contains(symbol_tree_entry.get_node_key())
    }

    fn format_symbol_tree_size_preview(size_in_bytes: u64) -> String {
        const KIB: f64 = 1024.0;
        const SIZE_UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];

        if size_in_bytes < 1024 {
            return format!("{} B", size_in_bytes);
        }

        let mut unit_index = 0_usize;
        let mut size_value = size_in_bytes as f64;

        while size_value >= KIB && unit_index + 1 < SIZE_UNITS.len() {
            size_value /= KIB;
            unit_index += 1;
        }

        let formatted_value = if size_value >= 100.0 {
            format!("{:.0}", size_value)
        } else if size_value >= 10.0 {
            format!("{:.1}", size_value)
        } else {
            format!("{:.2}", size_value)
        };
        let formatted_value = formatted_value
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string();

        format!("{} {}", formatted_value, SIZE_UNITS[unit_index])
    }

    fn format_symbol_tree_size_tooltip(size_in_bytes: u64) -> String {
        let scaled_size_text = Self::format_symbol_tree_size_preview(size_in_bytes);
        let raw_size_text = format!("{} B", size_in_bytes);

        if scaled_size_text == raw_size_text {
            raw_size_text
        } else {
            format!("{} ({})", raw_size_text, scaled_size_text)
        }
    }
}
