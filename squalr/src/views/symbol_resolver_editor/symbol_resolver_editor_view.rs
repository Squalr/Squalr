use crate::{
    app_context::AppContext,
    ui::{
        draw::icon_draw::IconDraw,
        widgets::controls::{
            button::Button as ThemeButton,
            combo_box::{combo_box_item_view::ComboBoxItemView, combo_box_view::ComboBoxView},
            data_value_box::data_value_box_view::DataValueBoxView,
            groupbox::GroupBox,
            state_layer::StateLayer,
        },
    },
    views::symbol_resolver_editor::view_data::symbol_resolver_editor_view_data::{
        SymbolResolverEditDraft, SymbolResolverEditorTakeOverState, SymbolResolverEditorViewData,
    },
};
use eframe::egui::{Align, Align2, Direction, Key, Layout, Response, RichText, ScrollArea, Sense, TextureHandle, Ui, UiBuilder, Widget, pos2, vec2};
use epaint::{Color32, CornerRadius, Stroke, StrokeKind};
use squalr_engine_api::commands::{
    privileged_command_request::PrivilegedCommandRequest, project::save::project_save_request::ProjectSaveRequest,
    registry::set_project_symbols::registry_set_project_symbols_request::RegistrySetProjectSymbolsRequest,
    unprivileged_command_request::UnprivilegedCommandRequest,
};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::{
    data_types::{built_in_types::string::utf8::data_type_string_utf8::DataTypeStringUtf8, data_type_ref::DataTypeRef},
    data_values::{anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType},
    projects::project_symbol_catalog::ProjectSymbolCatalog,
    structs::symbolic_resolver_definition::{SymbolicResolverBinaryOperator, SymbolicResolverNode},
};
use std::sync::Arc;

#[derive(Clone)]
pub struct SymbolResolverEditorView {
    app_context: Arc<AppContext>,
    symbol_resolver_editor_view_data: Dependency<SymbolResolverEditorViewData>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ResolverNodeKind {
    Literal,
    LocalField,
    TypeSize,
    Binary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ResolverToolbarAction {
    None,
    CreateResolver,
    SaveDraft,
    CancelDraft,
    DeleteResolver,
    ReplaceSelectedNode(ResolverNodeKind),
}

impl SymbolResolverEditorView {
    pub const WINDOW_ID: &'static str = "window_symbol_resolver_editor";
    const TOOLBAR_HEIGHT: f32 = 28.0;
    const ROW_HEIGHT: f32 = 28.0;
    const ICON_BUTTON_WIDTH: f32 = 36.0;
    const TREE_LEVEL_INDENT: f32 = 18.0;
    const ROW_LEFT_PADDING: f32 = 8.0;
    const ICON_SIZE: f32 = 16.0;
    const SMALL_ARROW_SIZE: f32 = 10.0;
    const DETAILS_HEIGHT: f32 = 178.0;
    const DETAILS_VALUE_WIDTH: f32 = 224.0;
    const OPERATOR_COMBO_WIDTH: f32 = 88.0;

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let symbol_resolver_editor_view_data = app_context
            .dependency_container
            .register(SymbolResolverEditorViewData::new());

        Self {
            app_context,
            symbol_resolver_editor_view_data,
        }
    }

    fn get_opened_project_symbol_catalog(&self) -> Option<ProjectSymbolCatalog> {
        let opened_project = self
            .app_context
            .engine_unprivileged_state
            .get_project_manager()
            .get_opened_project();
        let opened_project = opened_project.read().ok()?;

        opened_project.as_ref().map(|opened_project| {
            opened_project
                .get_project_info()
                .get_project_symbol_catalog()
                .clone()
        })
    }

    fn persist_project_symbol_catalog(
        &self,
        updated_project_symbol_catalog: ProjectSymbolCatalog,
    ) {
        let opened_project_lock = self
            .app_context
            .engine_unprivileged_state
            .get_project_manager()
            .get_opened_project();
        let did_update_project = match opened_project_lock.write() {
            Ok(mut opened_project) => {
                if let Some(opened_project) = opened_project.as_mut() {
                    let project_info = opened_project.get_project_info_mut();

                    *project_info.get_project_symbol_catalog_mut() = updated_project_symbol_catalog.clone();
                    project_info.set_has_unsaved_changes(true);
                    true
                } else {
                    false
                }
            }
            Err(error) => {
                log::error!("Failed to acquire opened project while persisting symbol resolver changes: {}.", error);
                false
            }
        };

        if !did_update_project {
            return;
        }

        ProjectSaveRequest {}.send(&self.app_context.engine_unprivileged_state, |project_save_response| {
            if !project_save_response.success {
                log::error!("Failed to save project after applying symbol resolver changes.");
            }
        });

        let registry_set_project_symbols_request = RegistrySetProjectSymbolsRequest {
            project_symbol_catalog: updated_project_symbol_catalog,
        };
        if !registry_set_project_symbols_request.send(&self.app_context.engine_unprivileged_state, |_response| {}) {
            log::error!("Failed to dispatch project symbol registry sync after symbol resolver changes.");
        }
    }

    fn default_data_type_ref(&self) -> DataTypeRef {
        self.app_context
            .engine_unprivileged_state
            .get_registered_data_type_refs()
            .first()
            .cloned()
            .unwrap_or_else(|| DataTypeRef::new("u32"))
    }

    fn string_data_type_ref() -> DataTypeRef {
        DataTypeRef::new(DataTypeStringUtf8::DATA_TYPE_ID)
    }

    fn render_toolbar(
        &self,
        user_interface: &mut Ui,
        can_save: bool,
        has_draft: bool,
        has_selected_resolver: bool,
        has_selected_node_target: bool,
    ) -> ResolverToolbarAction {
        let theme = &self.app_context.theme;
        let (allocated_size_rectangle, _response) =
            user_interface.allocate_exact_size(vec2(user_interface.available_width(), Self::TOOLBAR_HEIGHT), Sense::empty());
        user_interface
            .painter()
            .rect_filled(allocated_size_rectangle, CornerRadius::ZERO, theme.background_primary);

        let mut action = ResolverToolbarAction::None;
        let mut toolbar_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(allocated_size_rectangle)
                .layout(Layout::left_to_right(Align::Center)),
        );

        if self
            .render_icon_button(
                &mut toolbar_user_interface,
                &theme.icon_library.icon_handle_common_add,
                "Create resolver.",
                false,
            )
            .clicked()
        {
            action = ResolverToolbarAction::CreateResolver;
        }

        if self
            .render_icon_button(
                &mut toolbar_user_interface,
                &theme.icon_library.icon_handle_display_type_decimal,
                "Set selected node to literal.",
                !has_selected_node_target,
            )
            .clicked()
        {
            action = ResolverToolbarAction::ReplaceSelectedNode(ResolverNodeKind::Literal);
        }

        if self
            .render_icon_button(
                &mut toolbar_user_interface,
                &theme.icon_library.icon_handle_common_properties,
                "Set selected node to local field.",
                !has_selected_node_target,
            )
            .clicked()
        {
            action = ResolverToolbarAction::ReplaceSelectedNode(ResolverNodeKind::LocalField);
        }

        if self
            .render_icon_button(
                &mut toolbar_user_interface,
                &theme.icon_library.icon_handle_data_type_unknown,
                "Set selected node to type size.",
                !has_selected_node_target,
            )
            .clicked()
        {
            action = ResolverToolbarAction::ReplaceSelectedNode(ResolverNodeKind::TypeSize);
        }

        if self
            .render_icon_button(
                &mut toolbar_user_interface,
                &theme.icon_library.icon_handle_file_system_open_folder,
                "Set selected node to operation.",
                !has_selected_node_target,
            )
            .clicked()
        {
            action = ResolverToolbarAction::ReplaceSelectedNode(ResolverNodeKind::Binary);
        }

        toolbar_user_interface.allocate_ui_with_layout(
            vec2(toolbar_user_interface.available_width(), Self::TOOLBAR_HEIGHT),
            Layout::right_to_left(Align::Center),
            |user_interface| {
                if self
                    .render_icon_button(
                        user_interface,
                        &theme.icon_library.icon_handle_common_delete,
                        "Delete selected resolver.",
                        !has_selected_resolver,
                    )
                    .clicked()
                {
                    action = ResolverToolbarAction::DeleteResolver;
                }

                if self
                    .render_icon_button(
                        user_interface,
                        &theme.icon_library.icon_handle_navigation_cancel,
                        "Discard resolver edits.",
                        !has_draft,
                    )
                    .clicked()
                {
                    action = ResolverToolbarAction::CancelDraft;
                }

                if self
                    .render_icon_button(user_interface, &theme.icon_library.icon_handle_file_system_save, "Save resolver.", !can_save)
                    .clicked()
                {
                    action = ResolverToolbarAction::SaveDraft;
                }
            },
        );

        action
    }

    fn render_icon_button(
        &self,
        user_interface: &mut Ui,
        icon_handle: &TextureHandle,
        tooltip_text: &str,
        is_disabled: bool,
    ) -> Response {
        let theme = &self.app_context.theme;
        let button_response = user_interface.add_sized(
            vec2(Self::ICON_BUTTON_WIDTH, Self::TOOLBAR_HEIGHT),
            ThemeButton::new_from_theme(theme)
                .with_tooltip_text(tooltip_text)
                .background_color(Color32::TRANSPARENT)
                .disabled(is_disabled),
        );

        IconDraw::draw_tinted(
            user_interface,
            button_response.rect,
            icon_handle,
            if is_disabled { theme.foreground_preview } else { theme.foreground },
        );

        button_response
    }

    fn render_string_value_box(
        &self,
        user_interface: &mut Ui,
        value: &mut String,
        preview_text: &str,
        id: &str,
        width: f32,
    ) {
        let validation_data_type_ref = Self::string_data_type_ref();
        let mut value_string = AnonymousValueString::new(value.clone(), AnonymousValueStringFormat::String, ContainerType::None);

        user_interface.add(
            DataValueBoxView::new(
                self.app_context.clone(),
                &mut value_string,
                &validation_data_type_ref,
                false,
                true,
                preview_text,
                id,
            )
            .allowed_anonymous_value_string_formats(vec![AnonymousValueStringFormat::String])
            .show_format_button(false)
            .normalize_value_format(false)
            .use_format_text_colors(false)
            .width(width)
            .height(Self::ROW_HEIGHT),
        );

        *value = value_string.get_anonymous_value_string().to_string();
    }

    fn render_resolver_tree(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        selected_resolver_id: Option<&str>,
        selected_node_path: Option<&[usize]>,
        draft: Option<&SymbolResolverEditDraft>,
    ) {
        ScrollArea::vertical()
            .id_salt("symbol_resolver_tree")
            .show(user_interface, |user_interface| {
                for resolver_descriptor in project_symbol_catalog.get_symbolic_resolver_descriptors() {
                    let resolver_id = resolver_descriptor.get_resolver_id();
                    let draft_for_resolver =
                        draft.filter(|draft| draft.original_resolver_id.as_deref() == Some(resolver_id) || draft.resolver_id == resolver_id);
                    let resolver_display_id = draft_for_resolver
                        .map(|draft| draft.resolver_id.as_str())
                        .unwrap_or(resolver_id);
                    let resolver_definition = draft_for_resolver
                        .map(|draft| &draft.resolver_definition)
                        .unwrap_or_else(|| resolver_descriptor.get_resolver_definition());
                    let is_selected_resolver = selected_resolver_id == Some(resolver_id) && selected_node_path.is_none();
                    let is_expanded = selected_resolver_id == Some(resolver_id);

                    let row_response = self.render_tree_entry(
                        user_interface,
                        0,
                        resolver_display_id,
                        "Resolver",
                        TreeEntryKind::Resolver,
                        is_selected_resolver,
                        is_expanded,
                    );
                    if row_response.clicked() {
                        self.select_resolver(project_symbol_catalog, resolver_id);
                    }

                    if is_expanded {
                        self.render_node_tree(
                            user_interface,
                            resolver_id,
                            resolver_definition.get_root_node(),
                            Vec::new(),
                            1,
                            selected_node_path,
                        );
                    }
                }

                if matches!(
                    draft,
                    Some(SymbolResolverEditDraft {
                        original_resolver_id: None,
                        ..
                    })
                ) {
                    if let Some(draft) = draft {
                        let is_selected = selected_resolver_id.is_none() && selected_node_path.is_none();
                        let row_response = self.render_tree_entry(
                            user_interface,
                            0,
                            &draft.resolver_id,
                            "New Resolver",
                            TreeEntryKind::Resolver,
                            is_selected,
                            true,
                        );
                        if row_response.clicked() {
                            if let Some(mut view_data) = self
                                .symbol_resolver_editor_view_data
                                .write("SymbolResolverEditor select new resolver")
                            {
                                view_data.select_resolver(None);
                            }
                        }
                        self.render_node_tree(
                            user_interface,
                            &draft.resolver_id,
                            draft.resolver_definition.get_root_node(),
                            Vec::new(),
                            1,
                            selected_node_path,
                        );
                    }
                }

                if project_symbol_catalog
                    .get_symbolic_resolver_descriptors()
                    .is_empty()
                    && draft.is_none()
                {
                    user_interface.add_space(6.0);
                    user_interface.label(RichText::new("No resolvers.").color(self.app_context.theme.foreground_preview));
                }
            });
    }

    fn render_node_tree(
        &self,
        user_interface: &mut Ui,
        resolver_id: &str,
        resolver_node: &SymbolicResolverNode,
        node_path: Vec<usize>,
        depth: usize,
        selected_node_path: Option<&[usize]>,
    ) {
        let is_selected = selected_node_path == Some(node_path.as_slice());
        let is_expanded = matches!(resolver_node, SymbolicResolverNode::Binary { .. });
        let (label, preview, kind) = Self::node_tree_text(resolver_node);
        let row_response = self.render_tree_entry(user_interface, depth, &label, &preview, kind, is_selected, is_expanded);

        if row_response.clicked() {
            if let Some(mut view_data) = self
                .symbol_resolver_editor_view_data
                .write("SymbolResolverEditor select node")
            {
                view_data.select_node(resolver_id.to_string(), node_path.clone());
            }
        }

        if let SymbolicResolverNode::Binary { left_node, right_node, .. } = resolver_node {
            let mut left_path = node_path.clone();
            left_path.push(0);
            self.render_node_tree(user_interface, resolver_id, left_node, left_path, depth.saturating_add(1), selected_node_path);

            let mut right_path = node_path;
            right_path.push(1);
            self.render_node_tree(user_interface, resolver_id, right_node, right_path, depth.saturating_add(1), selected_node_path);
        }
    }

    fn render_tree_entry(
        &self,
        user_interface: &mut Ui,
        depth: usize,
        label: &str,
        preview: &str,
        entry_kind: TreeEntryKind,
        is_selected: bool,
        is_expanded: bool,
    ) -> Response {
        let theme = &self.app_context.theme;
        let (allocated_size_rectangle, response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), Self::ROW_HEIGHT), Sense::click());

        if is_selected {
            user_interface
                .painter()
                .rect_filled(allocated_size_rectangle, CornerRadius::ZERO, theme.selected_background);
            user_interface.painter().rect_stroke(
                allocated_size_rectangle,
                CornerRadius::ZERO,
                Stroke::new(1.0, theme.selected_border),
                StrokeKind::Inside,
            );
        }

        StateLayer {
            bounds_min: allocated_size_rectangle.min,
            bounds_max: allocated_size_rectangle.max,
            enabled: true,
            pressed: response.is_pointer_button_down_on(),
            has_hover: response.hovered(),
            has_focus: response.has_focus(),
            corner_radius: CornerRadius::ZERO,
            border_width: 0.0,
            hover_color: theme.hover_tint,
            pressed_color: theme.pressed_tint,
            border_color: theme.background_control_secondary_dark,
            border_color_focused: theme.background_control_secondary_dark,
        }
        .ui(user_interface);

        let indentation = depth as f32 * Self::TREE_LEVEL_INDENT;
        let arrow_center = pos2(
            allocated_size_rectangle.min.x + Self::ROW_LEFT_PADDING + indentation + Self::SMALL_ARROW_SIZE * 0.5,
            allocated_size_rectangle.center().y,
        );
        if entry_kind.has_children() {
            let arrow_icon = if is_expanded {
                &theme.icon_library.icon_handle_navigation_down_arrow_small
            } else {
                &theme.icon_library.icon_handle_navigation_right_arrow_small
            };
            IconDraw::draw_sized(user_interface, arrow_center, vec2(Self::SMALL_ARROW_SIZE, Self::SMALL_ARROW_SIZE), arrow_icon);
        }

        let icon_center = pos2(
            arrow_center.x + Self::SMALL_ARROW_SIZE * 0.5 + 6.0 + Self::ICON_SIZE * 0.5,
            allocated_size_rectangle.center().y,
        );
        IconDraw::draw_sized_tinted(
            user_interface,
            icon_center,
            vec2(Self::ICON_SIZE, Self::ICON_SIZE),
            self.icon_for_tree_entry(entry_kind),
            Color32::WHITE,
        );

        let label_position = pos2(icon_center.x + Self::ICON_SIZE * 0.5 + 6.0, allocated_size_rectangle.center().y);
        let preview_width = if preview.is_empty() {
            0.0
        } else {
            Self::measure_text_width(user_interface, preview, &theme.font_library.font_noto_sans.font_small, theme.foreground_preview)
        };
        let label_max_width = (allocated_size_rectangle.max.x - label_position.x - preview_width - 18.0).max(0.0);
        let label_text = Self::truncate_text_to_width(
            user_interface,
            label,
            label_max_width,
            &theme.font_library.font_noto_sans.font_normal,
            theme.foreground,
        );

        user_interface.painter().text(
            label_position,
            Align2::LEFT_CENTER,
            label_text,
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground,
        );

        if !preview.is_empty() {
            user_interface.painter().text(
                pos2(allocated_size_rectangle.max.x - 8.0, allocated_size_rectangle.center().y),
                Align2::RIGHT_CENTER,
                Self::truncate_text_to_width(
                    user_interface,
                    preview,
                    (allocated_size_rectangle.max.x - label_position.x - 48.0).max(0.0),
                    &theme.font_library.font_noto_sans.font_small,
                    theme.foreground_preview,
                ),
                theme.font_library.font_noto_sans.font_small.clone(),
                theme.foreground_preview,
            );
        }

        response
    }

    fn icon_for_tree_entry(
        &self,
        entry_kind: TreeEntryKind,
    ) -> &TextureHandle {
        let theme = &self.app_context.theme;
        match entry_kind {
            TreeEntryKind::Resolver => &theme.icon_library.icon_handle_common_properties,
            TreeEntryKind::Literal => &theme.icon_library.icon_handle_display_type_decimal,
            TreeEntryKind::LocalField => &theme.icon_library.icon_handle_common_properties,
            TreeEntryKind::TypeSize => &theme.icon_library.icon_handle_data_type_unknown,
            TreeEntryKind::Operation => &theme.icon_library.icon_handle_file_system_open_folder,
        }
    }

    fn render_details(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        selected_node_path: Option<&[usize]>,
        _baseline_draft: Option<&SymbolResolverEditDraft>,
        draft: Option<&SymbolResolverEditDraft>,
    ) {
        let Some(draft) = draft else {
            self.render_empty_details(user_interface);
            return;
        };
        let mut edited_draft = draft.clone();
        let theme = &self.app_context.theme;
        let header = if selected_node_path.is_some() { "Node Details" } else { "Resolver Details" };

        user_interface.add(
            GroupBox::new_from_theme(theme, header, |user_interface| {
                if let Some(selected_node_path) = selected_node_path {
                    self.render_node_details(user_interface, &mut edited_draft, selected_node_path);
                } else {
                    self.render_string_value_box(
                        user_interface,
                        &mut edited_draft.resolver_id,
                        "Resolver id",
                        "symbol_resolver_details_id",
                        Self::DETAILS_VALUE_WIDTH.min(user_interface.available_width()),
                    );
                }

                user_interface.add_space(8.0);
                let validation_result = SymbolResolverEditorViewData::build_resolver_descriptor(project_symbol_catalog, &edited_draft);
                match validation_result {
                    Ok(_) => {
                        user_interface.label(RichText::new("Valid resolver.").color(theme.foreground_preview));
                    }
                    Err(error) => {
                        user_interface.label(RichText::new(error).color(theme.error_red));
                    }
                }
            })
            .desired_width(user_interface.available_width())
            .desired_height(Self::DETAILS_HEIGHT),
        );

        if edited_draft != *draft {
            if let Some(mut view_data) = self
                .symbol_resolver_editor_view_data
                .write("SymbolResolverEditor update draft")
            {
                view_data.update_draft(edited_draft);
            }
        }
    }

    fn render_node_details(
        &self,
        user_interface: &mut Ui,
        draft: &mut SymbolResolverEditDraft,
        selected_node_path: &[usize],
    ) {
        let Some(selected_node) = Self::get_node_mut(draft.resolver_definition.get_root_node_mut(), selected_node_path) else {
            user_interface.label(RichText::new("Missing selected node.").color(self.app_context.theme.error_red));
            return;
        };

        user_interface.label(RichText::new(Self::resolver_node_kind_label(Self::resolver_node_kind(selected_node))).color(self.app_context.theme.foreground));
        user_interface.add_space(6.0);

        match selected_node {
            SymbolicResolverNode::Literal(value) => {
                let mut value_text = value.to_string();
                self.render_string_value_box(
                    user_interface,
                    &mut value_text,
                    "Literal value",
                    "symbol_resolver_node_literal",
                    Self::DETAILS_VALUE_WIDTH.min(user_interface.available_width()),
                );
                if let Ok(parsed_value) = value_text.trim().parse::<i128>() {
                    *value = parsed_value;
                }
            }
            SymbolicResolverNode::LocalField { field_name } => {
                self.render_string_value_box(
                    user_interface,
                    field_name,
                    "Local field",
                    "symbol_resolver_node_local_field",
                    Self::DETAILS_VALUE_WIDTH.min(user_interface.available_width()),
                );
            }
            SymbolicResolverNode::TypeSize { data_type_ref } => {
                let mut type_id = data_type_ref.get_data_type_id().to_string();
                self.render_string_value_box(
                    user_interface,
                    &mut type_id,
                    "Data type",
                    "symbol_resolver_node_type_size",
                    Self::DETAILS_VALUE_WIDTH.min(user_interface.available_width()),
                );
                if type_id.trim() != data_type_ref.get_data_type_id() {
                    *data_type_ref = DataTypeRef::new(type_id.trim());
                }
            }
            SymbolicResolverNode::Binary { operator, .. } => {
                self.render_operator_combo(user_interface, operator);
            }
        }
    }

    fn render_operator_combo(
        &self,
        user_interface: &mut Ui,
        operator: &mut SymbolicResolverBinaryOperator,
    ) {
        let selected_label = operator.label();
        let combo_box_width = Self::OPERATOR_COMBO_WIDTH;

        user_interface.add(
            ComboBoxView::new(
                self.app_context.clone(),
                selected_label,
                "symbol_resolver_node_operator",
                None,
                |popup_user_interface, should_close| {
                    for candidate_operator in SymbolicResolverBinaryOperator::ALL {
                        let response = popup_user_interface.add(ComboBoxItemView::new(
                            self.app_context.clone(),
                            candidate_operator.label(),
                            None,
                            combo_box_width,
                        ));
                        if response.clicked() {
                            *operator = candidate_operator;
                            *should_close = true;
                        }
                    }
                },
            )
            .width(combo_box_width)
            .height(Self::ROW_HEIGHT),
        );
    }

    fn render_empty_details(
        &self,
        user_interface: &mut Ui,
    ) {
        let theme = &self.app_context.theme;
        user_interface.add(
            GroupBox::new_from_theme(theme, "Details", |user_interface| {
                user_interface.label(RichText::new("Select a resolver or node.").color(theme.foreground_preview));
            })
            .desired_width(user_interface.available_width())
            .desired_height(Self::DETAILS_HEIGHT),
        );
    }

    fn render_delete_confirmation(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        resolver_id: &str,
    ) {
        let theme = &self.app_context.theme;
        user_interface.add(
            GroupBox::new_from_theme(theme, "Delete Resolver", |user_interface| {
                user_interface.label(RichText::new(format!("Delete `{}`?", resolver_id)).color(theme.foreground));
                user_interface.add_space(8.0);

                let delete_response = self.render_icon_button(user_interface, &theme.icon_library.icon_handle_common_delete, "Delete resolver.", false);
                let cancel_response = self.render_icon_button(user_interface, &theme.icon_library.icon_handle_navigation_cancel, "Cancel delete.", false);

                if cancel_response.clicked() {
                    if let Some(mut view_data) = self
                        .symbol_resolver_editor_view_data
                        .write("SymbolResolverEditor cancel delete")
                    {
                        view_data.cancel_take_over_state();
                    }
                }

                if delete_response.clicked() {
                    let updated_project_symbol_catalog = SymbolResolverEditorViewData::remove_resolver_from_catalog(project_symbol_catalog, resolver_id);
                    self.persist_project_symbol_catalog(updated_project_symbol_catalog);
                    if let Some(mut view_data) = self
                        .symbol_resolver_editor_view_data
                        .write("SymbolResolverEditor delete resolver")
                    {
                        view_data.cancel_take_over_state();
                    }
                }
            })
            .desired_width(user_interface.available_width())
            .desired_height(Self::DETAILS_HEIGHT),
        );
    }

    fn select_resolver(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        resolver_id: &str,
    ) {
        if let Some(mut view_data) = self
            .symbol_resolver_editor_view_data
            .write("SymbolResolverEditor select resolver")
        {
            view_data.select_resolver(Some(resolver_id.to_string()));
            view_data.begin_edit_resolver(project_symbol_catalog, resolver_id);
        }
    }

    fn ensure_selected_resolver_has_draft(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        selected_resolver_id: Option<&str>,
        take_over_state: Option<&SymbolResolverEditorTakeOverState>,
    ) {
        if take_over_state.is_some() {
            return;
        }

        let Some(selected_resolver_id) = selected_resolver_id else {
            return;
        };

        if let Some(mut view_data) = self
            .symbol_resolver_editor_view_data
            .write("SymbolResolverEditor begin selected resolver edit")
        {
            view_data.begin_edit_resolver(project_symbol_catalog, selected_resolver_id);
        }
    }

    fn apply_toolbar_action(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        action: ResolverToolbarAction,
        selected_resolver_id: Option<&str>,
        selected_node_path: Option<&[usize]>,
        draft: Option<&SymbolResolverEditDraft>,
    ) {
        match action {
            ResolverToolbarAction::None => {}
            ResolverToolbarAction::CreateResolver => {
                if let Some(mut view_data) = self
                    .symbol_resolver_editor_view_data
                    .write("SymbolResolverEditor create resolver")
                {
                    view_data.begin_create_resolver(project_symbol_catalog);
                }
            }
            ResolverToolbarAction::SaveDraft => {
                if let Some(draft) = draft {
                    self.save_draft(project_symbol_catalog, draft);
                }
            }
            ResolverToolbarAction::CancelDraft => {
                if let Some(mut view_data) = self
                    .symbol_resolver_editor_view_data
                    .write("SymbolResolverEditor cancel resolver edit")
                {
                    view_data.cancel_take_over_state();
                }
            }
            ResolverToolbarAction::DeleteResolver => {
                if let Some(selected_resolver_id) = selected_resolver_id {
                    if let Some(mut view_data) = self
                        .symbol_resolver_editor_view_data
                        .write("SymbolResolverEditor delete resolver")
                    {
                        view_data.request_delete_confirmation(selected_resolver_id.to_string());
                    }
                }
            }
            ResolverToolbarAction::ReplaceSelectedNode(resolver_node_kind) => {
                let Some(draft) = draft else {
                    return;
                };
                let mut edited_draft = draft.clone();
                let selected_node_path = selected_node_path.unwrap_or(&[]);
                let default_data_type_ref = self.default_data_type_ref();

                if let Some(selected_node) = Self::get_node_mut(edited_draft.resolver_definition.get_root_node_mut(), selected_node_path) {
                    *selected_node = Self::default_node_for_kind(resolver_node_kind, default_data_type_ref);
                    if let Some(mut view_data) = self
                        .symbol_resolver_editor_view_data
                        .write("SymbolResolverEditor replace node")
                    {
                        view_data.update_draft(edited_draft);
                    }
                }
            }
        }
    }

    fn save_draft(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        draft: &SymbolResolverEditDraft,
    ) {
        match SymbolResolverEditorViewData::apply_draft_to_catalog(project_symbol_catalog, draft) {
            Ok(updated_project_symbol_catalog) => {
                let saved_resolver_id = draft.resolver_id.trim().to_string();
                self.persist_project_symbol_catalog(updated_project_symbol_catalog);
                if let Some(mut view_data) = self
                    .symbol_resolver_editor_view_data
                    .write("SymbolResolverEditor save resolver")
                {
                    view_data.cancel_take_over_state();
                    view_data.select_resolver(Some(saved_resolver_id));
                }
            }
            Err(error) => {
                log::error!("Failed to apply symbol resolver draft: {}.", error);
            }
        }
    }

    fn resolver_node_kind(resolver_node: &SymbolicResolverNode) -> ResolverNodeKind {
        match resolver_node {
            SymbolicResolverNode::Literal(_) => ResolverNodeKind::Literal,
            SymbolicResolverNode::LocalField { .. } => ResolverNodeKind::LocalField,
            SymbolicResolverNode::TypeSize { .. } => ResolverNodeKind::TypeSize,
            SymbolicResolverNode::Binary { .. } => ResolverNodeKind::Binary,
        }
    }

    fn resolver_node_kind_label(resolver_node_kind: ResolverNodeKind) -> &'static str {
        match resolver_node_kind {
            ResolverNodeKind::Literal => "Literal",
            ResolverNodeKind::LocalField => "Local Field",
            ResolverNodeKind::TypeSize => "Type Size",
            ResolverNodeKind::Binary => "Operation",
        }
    }

    fn default_node_for_kind(
        resolver_node_kind: ResolverNodeKind,
        default_data_type_ref: DataTypeRef,
    ) -> SymbolicResolverNode {
        match resolver_node_kind {
            ResolverNodeKind::Literal => SymbolicResolverNode::new_literal(0),
            ResolverNodeKind::LocalField => SymbolicResolverNode::new_local_field(String::from("field")),
            ResolverNodeKind::TypeSize => SymbolicResolverNode::new_type_size(default_data_type_ref),
            ResolverNodeKind::Binary => SymbolicResolverNode::new_binary(
                SymbolicResolverBinaryOperator::Add,
                SymbolicResolverNode::new_literal(0),
                SymbolicResolverNode::new_literal(0),
            ),
        }
    }

    fn node_tree_text(resolver_node: &SymbolicResolverNode) -> (String, String, TreeEntryKind) {
        match resolver_node {
            SymbolicResolverNode::Literal(value) => (String::from("Literal"), value.to_string(), TreeEntryKind::Literal),
            SymbolicResolverNode::LocalField { field_name } => (String::from("Local Field"), field_name.to_string(), TreeEntryKind::LocalField),
            SymbolicResolverNode::TypeSize { data_type_ref } => (String::from("Type Size"), data_type_ref.to_string(), TreeEntryKind::TypeSize),
            SymbolicResolverNode::Binary { operator, .. } => (format!("Operation {}", operator.label()), String::new(), TreeEntryKind::Operation),
        }
    }

    fn get_node_mut<'resolver>(
        resolver_node: &'resolver mut SymbolicResolverNode,
        node_path: &[usize],
    ) -> Option<&'resolver mut SymbolicResolverNode> {
        if node_path.is_empty() {
            return Some(resolver_node);
        }

        match resolver_node {
            SymbolicResolverNode::Binary { left_node, right_node, .. } => match node_path[0] {
                0 => Self::get_node_mut(left_node, &node_path[1..]),
                1 => Self::get_node_mut(right_node, &node_path[1..]),
                _ => None,
            },
            SymbolicResolverNode::Literal(_) | SymbolicResolverNode::LocalField { .. } | SymbolicResolverNode::TypeSize { .. } => None,
        }
    }

    fn measure_text_width(
        user_interface: &Ui,
        text: &str,
        font_id: &eframe::egui::FontId,
        text_color: Color32,
    ) -> f32 {
        if text.is_empty() {
            return 0.0;
        }

        user_interface.ctx().fonts_mut(|fonts| {
            fonts
                .layout_no_wrap(text.to_string(), font_id.clone(), text_color)
                .size()
                .x
        })
    }

    fn truncate_text_to_width(
        user_interface: &Ui,
        text: &str,
        max_text_width: f32,
        font_id: &eframe::egui::FontId,
        text_color: Color32,
    ) -> String {
        if text.is_empty() || max_text_width <= 0.0 {
            return String::new();
        }

        let full_text_width = Self::measure_text_width(user_interface, text, font_id, text_color);
        if full_text_width <= max_text_width {
            return text.to_string();
        }

        let ellipsis = "...";
        let ellipsis_width = Self::measure_text_width(user_interface, ellipsis, font_id, text_color);
        if ellipsis_width > max_text_width {
            return String::new();
        }

        let mut truncated_text = text.to_string();
        while !truncated_text.is_empty() {
            truncated_text.pop();
            let candidate_text = format!("{}{}", truncated_text, ellipsis);
            let candidate_width = Self::measure_text_width(user_interface, &candidate_text, font_id, text_color);
            if candidate_width <= max_text_width {
                return candidate_text;
            }
        }

        String::new()
    }

    fn render_empty_project_message(
        &self,
        user_interface: &mut Ui,
    ) -> Response {
        user_interface
            .allocate_ui_with_layout(
                user_interface.available_size(),
                Layout::centered_and_justified(Direction::TopDown),
                |user_interface| {
                    user_interface.label(RichText::new("Open a project to author reusable symbol resolvers.").color(self.app_context.theme.foreground_preview));
                },
            )
            .response
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TreeEntryKind {
    Resolver,
    Literal,
    LocalField,
    TypeSize,
    Operation,
}

impl TreeEntryKind {
    fn has_children(self) -> bool {
        matches!(self, Self::Resolver | Self::Operation)
    }
}

impl Widget for SymbolResolverEditorView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> eframe::egui::Response {
        let Some(project_symbol_catalog) = self.get_opened_project_symbol_catalog() else {
            return self.render_empty_project_message(user_interface);
        };

        if let Some(mut view_data) = self
            .symbol_resolver_editor_view_data
            .write("SymbolResolverEditor synchronize")
        {
            view_data.synchronize(&project_symbol_catalog);
        }

        let (selected_resolver_id, selected_node_path, take_over_state, baseline_draft, draft) = self
            .symbol_resolver_editor_view_data
            .read("SymbolResolverEditor view")
            .map(|view_data| {
                (
                    view_data.get_selected_resolver_id().map(str::to_string),
                    view_data.get_selected_node_path().map(<[usize]>::to_vec),
                    view_data.get_take_over_state().cloned(),
                    view_data.get_baseline_draft().cloned(),
                    view_data.get_draft().cloned(),
                )
            })
            .unwrap_or((None, None, None, None, None));

        self.ensure_selected_resolver_has_draft(&project_symbol_catalog, selected_resolver_id.as_deref(), take_over_state.as_ref());

        let (selected_resolver_id, selected_node_path, take_over_state, baseline_draft, draft) = self
            .symbol_resolver_editor_view_data
            .read("SymbolResolverEditor view after draft ensure")
            .map(|view_data| {
                (
                    view_data.get_selected_resolver_id().map(str::to_string),
                    view_data.get_selected_node_path().map(<[usize]>::to_vec),
                    view_data.get_take_over_state().cloned(),
                    view_data.get_baseline_draft().cloned(),
                    view_data.get_draft().cloned(),
                )
            })
            .unwrap_or((selected_resolver_id, selected_node_path, take_over_state, baseline_draft, draft));

        let can_handle_window_shortcuts = self
            .app_context
            .window_focus_manager
            .can_window_handle_shortcuts(user_interface.ctx(), Self::WINDOW_ID);
        if can_handle_window_shortcuts && user_interface.input(|input_state| input_state.key_pressed(Key::Escape)) && draft.is_some() {
            if let Some(mut view_data) = self
                .symbol_resolver_editor_view_data
                .write("SymbolResolverEditor escape")
            {
                view_data.cancel_take_over_state();
            }
        }

        let validation_result = draft
            .as_ref()
            .map(|draft| SymbolResolverEditorViewData::build_resolver_descriptor(&project_symbol_catalog, draft));
        let can_save = draft
            .as_ref()
            .zip(baseline_draft.as_ref())
            .map(|(draft, baseline_draft)| draft != baseline_draft)
            .unwrap_or(false)
            && validation_result.as_ref().is_some_and(Result::is_ok);
        let has_selected_node_target = draft.is_some() && !matches!(take_over_state, Some(SymbolResolverEditorTakeOverState::DeleteConfirmation { .. }));

        user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                let toolbar_action = self.render_toolbar(
                    user_interface,
                    can_save,
                    draft.is_some(),
                    selected_resolver_id.is_some(),
                    has_selected_node_target,
                );
                self.apply_toolbar_action(
                    &project_symbol_catalog,
                    toolbar_action,
                    selected_resolver_id.as_deref(),
                    selected_node_path.as_deref(),
                    draft.as_ref(),
                );

                user_interface.add_space(4.0);
                let details_height = Self::DETAILS_HEIGHT.min(user_interface.available_height() * 0.45);
                let tree_height = (user_interface.available_height() - details_height - 8.0).max(Self::ROW_HEIGHT);
                user_interface.allocate_ui_with_layout(
                    vec2(user_interface.available_width(), tree_height),
                    Layout::top_down(Align::Min),
                    |user_interface| {
                        self.render_resolver_tree(
                            user_interface,
                            &project_symbol_catalog,
                            selected_resolver_id.as_deref(),
                            selected_node_path.as_deref(),
                            draft.as_ref(),
                        );
                    },
                );

                user_interface.add_space(8.0);
                if let Some(SymbolResolverEditorTakeOverState::DeleteConfirmation { resolver_id }) = take_over_state.as_ref() {
                    self.render_delete_confirmation(user_interface, &project_symbol_catalog, resolver_id);
                } else {
                    self.render_details(
                        user_interface,
                        &project_symbol_catalog,
                        selected_node_path.as_deref(),
                        baseline_draft.as_ref(),
                        draft.as_ref(),
                    );
                }
            })
            .response
    }
}
