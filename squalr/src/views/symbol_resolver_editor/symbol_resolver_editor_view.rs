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
use epaint::{Color32, CornerRadius, Rect, Stroke, StrokeKind};
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

impl SymbolResolverEditorView {
    pub const WINDOW_ID: &'static str = "window_symbol_resolver_editor";
    const TOOLBAR_HEIGHT: f32 = 28.0;
    const ROW_HEIGHT: f32 = 28.0;
    const ICON_BUTTON_WIDTH: f32 = 36.0;
    const TREE_LEVEL_INDENT: f32 = 18.0;
    const ROW_LEFT_PADDING: f32 = 8.0;
    const ICON_SIZE: f32 = 16.0;
    const SMALL_ARROW_SIZE: f32 = 10.0;
    const NODE_KIND_WIDTH: f32 = 132.0;
    const NODE_VALUE_WIDTH: f32 = 184.0;
    const NODE_OPERATOR_WIDTH: f32 = 72.0;
    const SEARCH_BOX_WIDTH: f32 = 220.0;
    const TAKE_OVER_HEADER_HEIGHT: f32 = 32.0;

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

    fn render_toolbar(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        selected_resolver_id: Option<&str>,
        take_over_state: Option<&SymbolResolverEditorTakeOverState>,
    ) {
        let theme = &self.app_context.theme;
        let is_creating = matches!(take_over_state, Some(SymbolResolverEditorTakeOverState::CreateResolver));

        let (allocated_size_rectangle, response) =
            user_interface.allocate_exact_size(vec2(user_interface.available_width(), Self::TOOLBAR_HEIGHT), Sense::empty());
        user_interface
            .painter()
            .rect_filled(allocated_size_rectangle, CornerRadius::ZERO, theme.background_primary);

        let mut toolbar_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(allocated_size_rectangle)
                .layout(Layout::left_to_right(Align::Center)),
        );

        let add_response = self.render_icon_button(
            &mut toolbar_user_interface,
            &theme.icon_library.icon_handle_common_add,
            "Create resolver.",
            is_creating,
        );
        if add_response.clicked() {
            if let Some(mut view_data) = self
                .symbol_resolver_editor_view_data
                .write("SymbolResolverEditor create resolver")
            {
                view_data.begin_create_resolver(project_symbol_catalog);
            }
        }

        toolbar_user_interface.allocate_ui_with_layout(
            vec2(toolbar_user_interface.available_width(), Self::TOOLBAR_HEIGHT),
            Layout::right_to_left(Align::Center),
            |user_interface| {
                let delete_response = self.render_icon_button(
                    user_interface,
                    &theme.icon_library.icon_handle_common_delete,
                    "Delete selected resolver.",
                    selected_resolver_id.is_none() || is_creating,
                );
                if delete_response.clicked() {
                    if let Some(selected_resolver_id) = selected_resolver_id {
                        if let Some(mut view_data) = self
                            .symbol_resolver_editor_view_data
                            .write("SymbolResolverEditor delete resolver")
                        {
                            view_data.request_delete_confirmation(selected_resolver_id.to_string());
                        }
                    }
                }
            },
        );

        response.on_hover_text("Symbol resolver toolbar.");
    }

    fn render_filter_box(
        &self,
        user_interface: &mut Ui,
        filter_text: &str,
    ) {
        let mut edited_filter_text = filter_text.to_string();
        self.render_string_value_box(
            user_interface,
            &mut edited_filter_text,
            "Search resolvers",
            "symbol_resolver_editor_filter",
            Self::SEARCH_BOX_WIDTH.min(user_interface.available_width()),
        );

        if edited_filter_text != filter_text {
            if let Some(mut view_data) = self
                .symbol_resolver_editor_view_data
                .write("SymbolResolverEditor filter")
            {
                view_data.set_filter_text(edited_filter_text);
            }
        }
    }

    fn render_resolver_list(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        selected_resolver_id: Option<&str>,
        filter_text: &str,
        is_create_take_over_active: bool,
    ) {
        self.render_filter_box(user_interface, filter_text);
        user_interface.add_space(4.0);

        ScrollArea::vertical()
            .id_salt("symbol_resolver_editor_list")
            .show(user_interface, |user_interface| {
                let mut rendered_resolver_count = 0_usize;
                for resolver_descriptor in project_symbol_catalog
                    .get_symbolic_resolver_descriptors()
                    .iter()
                    .filter(|resolver_descriptor| SymbolResolverEditorViewData::layout_matches_filter(resolver_descriptor, filter_text))
                {
                    rendered_resolver_count = rendered_resolver_count.saturating_add(1);
                    let resolver_id = resolver_descriptor.get_resolver_id();
                    let selected = selected_resolver_id == Some(resolver_id);
                    let row_response = self.render_resolver_row(user_interface, resolver_id, selected, is_create_take_over_active);

                    if row_response.clicked() && !is_create_take_over_active {
                        if let Some(mut view_data) = self
                            .symbol_resolver_editor_view_data
                            .write("SymbolResolverEditor select resolver")
                        {
                            view_data.select_resolver(Some(resolver_id.to_string()));
                            view_data.begin_edit_resolver(project_symbol_catalog, resolver_id);
                        }
                    }
                }

                if rendered_resolver_count == 0 {
                    user_interface.add_space(6.0);
                    user_interface.label(RichText::new("No resolvers.").color(self.app_context.theme.foreground_preview));
                }
            });
    }

    fn render_resolver_row(
        &self,
        user_interface: &mut Ui,
        resolver_id: &str,
        is_selected: bool,
        is_disabled: bool,
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
            enabled: !is_disabled,
            pressed: !is_disabled && response.is_pointer_button_down_on(),
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

        let icon_center = pos2(
            allocated_size_rectangle.min.x + Self::ROW_LEFT_PADDING + Self::ICON_SIZE * 0.5,
            allocated_size_rectangle.center().y,
        );
        IconDraw::draw_sized_tinted(
            user_interface,
            icon_center,
            vec2(Self::ICON_SIZE, Self::ICON_SIZE),
            &theme.icon_library.icon_handle_common_properties,
            if is_disabled { theme.foreground_preview } else { Color32::WHITE },
        );

        let text_position = pos2(icon_center.x + Self::ICON_SIZE * 0.5 + 6.0, allocated_size_rectangle.center().y);
        user_interface.painter().text(
            text_position,
            Align2::LEFT_CENTER,
            Self::truncate_text_to_width(
                user_interface,
                resolver_id,
                (allocated_size_rectangle.max.x - text_position.x - 8.0).max(0.0),
                theme.foreground,
            ),
            theme.font_library.font_noto_sans.font_normal.clone(),
            if is_disabled { theme.foreground_preview } else { theme.foreground },
        );

        response
    }

    fn render_draft_editor(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        title: &str,
        baseline_draft: Option<&SymbolResolverEditDraft>,
        draft: Option<&SymbolResolverEditDraft>,
    ) {
        let Some(draft) = draft else {
            return;
        };
        let baseline_draft = baseline_draft.unwrap_or(draft);
        let mut edited_draft = draft.clone();
        let validation_result = SymbolResolverEditorViewData::build_resolver_descriptor(project_symbol_catalog, &edited_draft);
        let has_unsaved_changes = edited_draft != *baseline_draft;
        let can_save = validation_result.is_ok() && has_unsaved_changes;
        let mut should_save = false;
        let mut should_cancel = false;
        let theme = &self.app_context.theme;

        self.render_editor_header(
            user_interface,
            title,
            &theme.icon_library.icon_handle_file_system_save,
            "Save resolver.",
            can_save,
            &mut should_save,
            &mut should_cancel,
        );
        user_interface.add_space(8.0);

        user_interface.add(
            GroupBox::new_from_theme(theme, "Resolver", |user_interface| {
                self.render_string_value_box(
                    user_interface,
                    &mut edited_draft.resolver_id,
                    "Resolver id",
                    "symbol_resolver_editor_resolver_id",
                    Self::NODE_VALUE_WIDTH.min(user_interface.available_width()),
                );
            })
            .desired_width(user_interface.available_width()),
        );

        user_interface.add_space(10.0);
        user_interface.add(
            GroupBox::new_from_theme(theme, "Tree", |user_interface| {
                self.render_resolver_node_tree(
                    user_interface,
                    edited_draft.resolver_definition.get_root_node_mut(),
                    "symbol_resolver_editor_root",
                    0,
                    self.default_data_type_ref(),
                );
            })
            .desired_width(user_interface.available_width()),
        );

        user_interface.add_space(8.0);
        match validation_result {
            Ok(_) => {
                user_interface.label(RichText::new("Valid resolver.").color(theme.foreground_preview));
            }
            Err(error) => {
                user_interface.label(RichText::new(error).color(theme.error_red));
            }
        }

        if should_cancel {
            if let Some(mut view_data) = self
                .symbol_resolver_editor_view_data
                .write("SymbolResolverEditor cancel resolver edit")
            {
                view_data.cancel_take_over_state();
            }
            return;
        }

        if should_save {
            match SymbolResolverEditorViewData::apply_draft_to_catalog(project_symbol_catalog, &edited_draft) {
                Ok(updated_project_symbol_catalog) => {
                    let saved_resolver_id = edited_draft.resolver_id.trim().to_string();
                    self.persist_project_symbol_catalog(updated_project_symbol_catalog);
                    if let Some(mut view_data) = self
                        .symbol_resolver_editor_view_data
                        .write("SymbolResolverEditor save resolver")
                    {
                        view_data.select_resolver(Some(saved_resolver_id.clone()));
                        view_data.cancel_take_over_state();
                        view_data.select_resolver(Some(saved_resolver_id));
                    }
                    return;
                }
                Err(error) => {
                    log::error!("Failed to apply symbol resolver draft: {}.", error);
                }
            }
        }

        if edited_draft != *draft {
            if let Some(mut view_data) = self
                .symbol_resolver_editor_view_data
                .write("SymbolResolverEditor update draft")
            {
                view_data.update_draft(edited_draft);
            }
        }
    }

    fn render_editor_header(
        &self,
        user_interface: &mut Ui,
        title: &str,
        primary_icon_handle: &TextureHandle,
        primary_tooltip_text: &str,
        can_save: bool,
        should_save: &mut bool,
        should_cancel: &mut bool,
    ) {
        let theme = &self.app_context.theme;
        let (allocated_size_rectangle, _response) =
            user_interface.allocate_exact_size(vec2(user_interface.available_width(), Self::TAKE_OVER_HEADER_HEIGHT), Sense::empty());
        user_interface
            .painter()
            .rect_filled(allocated_size_rectangle, CornerRadius::ZERO, theme.background_primary);
        user_interface.painter().text(
            pos2(allocated_size_rectangle.min.x + 8.0, allocated_size_rectangle.center().y),
            Align2::LEFT_CENTER,
            title,
            theme.font_library.font_noto_sans.font_header.clone(),
            theme.foreground,
        );

        let mut header_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(allocated_size_rectangle)
                .layout(Layout::right_to_left(Align::Center)),
        );
        let save_response = self.render_icon_button(&mut header_user_interface, primary_icon_handle, primary_tooltip_text, !can_save);
        if save_response.clicked() {
            *should_save = true;
        }

        let cancel_response = self.render_icon_button(
            &mut header_user_interface,
            &theme.icon_library.icon_handle_navigation_cancel,
            "Cancel resolver edits.",
            false,
        );
        if cancel_response.clicked() {
            *should_cancel = true;
        }
    }

    fn render_delete_confirmation(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        resolver_id: &str,
    ) {
        let mut should_delete = false;
        let mut should_cancel = false;
        let theme = &self.app_context.theme;

        self.render_editor_header(
            user_interface,
            "Delete Resolver",
            &theme.icon_library.icon_handle_common_delete,
            "Delete resolver.",
            true,
            &mut should_delete,
            &mut should_cancel,
        );
        user_interface.add_space(8.0);
        user_interface.add(
            GroupBox::new_from_theme(theme, "Confirm", |user_interface| {
                user_interface.label(RichText::new(format!("Delete `{}`?", resolver_id)).color(theme.foreground));
            })
            .desired_width(user_interface.available_width()),
        );

        if should_cancel {
            if let Some(mut view_data) = self
                .symbol_resolver_editor_view_data
                .write("SymbolResolverEditor cancel delete")
            {
                view_data.cancel_take_over_state();
            }
        }

        if should_delete {
            let updated_project_symbol_catalog = SymbolResolverEditorViewData::remove_resolver_from_catalog(project_symbol_catalog, resolver_id);
            self.persist_project_symbol_catalog(updated_project_symbol_catalog);
            if let Some(mut view_data) = self
                .symbol_resolver_editor_view_data
                .write("SymbolResolverEditor delete resolver")
            {
                view_data.cancel_take_over_state();
            }
        }
    }

    fn render_resolver_node_tree(
        &self,
        user_interface: &mut Ui,
        resolver_node: &mut SymbolicResolverNode,
        id_salt: &str,
        depth: usize,
        default_data_type_ref: DataTypeRef,
    ) {
        self.render_resolver_node_row(user_interface, resolver_node, id_salt, depth, default_data_type_ref.clone());

        if let SymbolicResolverNode::Binary { left_node, right_node, .. } = resolver_node {
            self.render_resolver_node_tree(
                user_interface,
                left_node,
                &format!("{}_left_{}", id_salt, depth),
                depth.saturating_add(1),
                default_data_type_ref.clone(),
            );
            self.render_resolver_node_tree(
                user_interface,
                right_node,
                &format!("{}_right_{}", id_salt, depth),
                depth.saturating_add(1),
                default_data_type_ref,
            );
        }
    }

    fn render_resolver_node_row(
        &self,
        user_interface: &mut Ui,
        resolver_node: &mut SymbolicResolverNode,
        id_salt: &str,
        depth: usize,
        default_data_type_ref: DataTypeRef,
    ) {
        let theme = &self.app_context.theme;
        let (allocated_size_rectangle, response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), Self::ROW_HEIGHT), Sense::hover());

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

        let mut row_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(allocated_size_rectangle)
                .layout(Layout::left_to_right(Align::Center)),
        );
        row_user_interface.add_space(Self::ROW_LEFT_PADDING + depth as f32 * Self::TREE_LEVEL_INDENT);

        let arrow_rect = Self::allocate_tree_icon_slot(&mut row_user_interface, Self::SMALL_ARROW_SIZE);
        if matches!(resolver_node, SymbolicResolverNode::Binary { .. }) {
            IconDraw::draw_sized(
                &row_user_interface,
                arrow_rect.center(),
                vec2(Self::SMALL_ARROW_SIZE, Self::SMALL_ARROW_SIZE),
                &theme.icon_library.icon_handle_navigation_down_arrow_small,
            );
        }

        let node_icon_rect = Self::allocate_tree_icon_slot(&mut row_user_interface, Self::ICON_SIZE);
        let node_icon = if matches!(resolver_node, SymbolicResolverNode::Binary { .. }) {
            &theme.icon_library.icon_handle_file_system_open_folder
        } else {
            &theme.icon_library.icon_handle_common_properties
        };
        IconDraw::draw_sized_tinted(
            &row_user_interface,
            node_icon_rect.center(),
            vec2(Self::ICON_SIZE, Self::ICON_SIZE),
            node_icon,
            Color32::WHITE,
        );

        let current_kind = Self::resolver_node_kind(resolver_node);
        let mut selected_kind = current_kind;
        self.render_node_kind_combo(&mut row_user_interface, id_salt, &mut selected_kind);

        if selected_kind != current_kind {
            *resolver_node = Self::default_node_for_kind(selected_kind, default_data_type_ref);
            return;
        }

        match resolver_node {
            SymbolicResolverNode::Literal(value) => {
                let mut value_text = value.to_string();
                self.render_string_value_box(
                    &mut row_user_interface,
                    &mut value_text,
                    "Literal value",
                    &format!("{}_literal", id_salt),
                    Self::NODE_VALUE_WIDTH,
                );
                if let Ok(parsed_value) = value_text.trim().parse::<i128>() {
                    *value = parsed_value;
                }
            }
            SymbolicResolverNode::LocalField { field_name } => {
                self.render_string_value_box(
                    &mut row_user_interface,
                    field_name,
                    "Local field",
                    &format!("{}_local_field", id_salt),
                    Self::NODE_VALUE_WIDTH,
                );
            }
            SymbolicResolverNode::TypeSize { data_type_ref } => {
                let mut type_id = data_type_ref.get_data_type_id().to_string();
                self.render_string_value_box(
                    &mut row_user_interface,
                    &mut type_id,
                    "Data type",
                    &format!("{}_type_size", id_salt),
                    Self::NODE_VALUE_WIDTH,
                );
                if type_id.trim() != data_type_ref.get_data_type_id() {
                    *data_type_ref = DataTypeRef::new(type_id.trim());
                }
            }
            SymbolicResolverNode::Binary { operator, .. } => {
                self.render_operator_combo(&mut row_user_interface, id_salt, operator);
            }
        }
    }

    fn allocate_tree_icon_slot(
        user_interface: &mut Ui,
        size: f32,
    ) -> Rect {
        let (icon_rect, _response) = user_interface.allocate_exact_size(vec2(size + 4.0, Self::ROW_HEIGHT), Sense::hover());
        Rect::from_center_size(icon_rect.center(), vec2(size, size))
    }

    fn render_node_kind_combo(
        &self,
        user_interface: &mut Ui,
        id_salt: &str,
        selected_kind: &mut ResolverNodeKind,
    ) {
        let menu_id = format!("{}_kind", id_salt);
        let selected_label = Self::resolver_node_kind_label(*selected_kind);
        let combo_box_width = Self::NODE_KIND_WIDTH;

        user_interface.add(
            ComboBoxView::new(
                self.app_context.clone(),
                selected_label,
                menu_id.as_str(),
                None,
                |popup_user_interface, should_close| {
                    for candidate_kind in [
                        ResolverNodeKind::Literal,
                        ResolverNodeKind::LocalField,
                        ResolverNodeKind::TypeSize,
                        ResolverNodeKind::Binary,
                    ] {
                        let response = popup_user_interface.add(ComboBoxItemView::new(
                            self.app_context.clone(),
                            Self::resolver_node_kind_label(candidate_kind),
                            None,
                            combo_box_width,
                        ));
                        if response.clicked() {
                            *selected_kind = candidate_kind;
                            *should_close = true;
                        }
                    }
                },
            )
            .width(combo_box_width)
            .height(Self::ROW_HEIGHT),
        );
    }

    fn render_operator_combo(
        &self,
        user_interface: &mut Ui,
        id_salt: &str,
        operator: &mut SymbolicResolverBinaryOperator,
    ) {
        let menu_id = format!("{}_operator", id_salt);
        let selected_label = operator.label();
        let combo_box_width = Self::NODE_OPERATOR_WIDTH;

        user_interface.add(
            ComboBoxView::new(
                self.app_context.clone(),
                selected_label,
                menu_id.as_str(),
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

    fn render_empty_selection_message(
        &self,
        user_interface: &mut Ui,
    ) {
        user_interface.allocate_ui_with_layout(
            vec2(user_interface.available_width(), user_interface.available_height().max(Self::ROW_HEIGHT)),
            Layout::centered_and_justified(Direction::TopDown),
            |user_interface| {
                user_interface.label(RichText::new("Select or create a resolver.").color(self.app_context.theme.foreground_preview));
            },
        );
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

    fn truncate_text_to_width(
        user_interface: &Ui,
        text: &str,
        max_text_width: f32,
        text_color: Color32,
    ) -> String {
        if text.is_empty() || max_text_width <= 0.0 {
            return String::new();
        }

        let font_id = user_interface
            .style()
            .text_styles
            .get(&eframe::egui::TextStyle::Body)
            .cloned()
            .unwrap_or_else(|| eframe::egui::FontId::proportional(14.0));
        let full_text_width = user_interface.ctx().fonts_mut(|fonts| {
            fonts
                .layout_no_wrap(text.to_string(), font_id.clone(), text_color)
                .size()
                .x
        });
        if full_text_width <= max_text_width {
            return text.to_string();
        }

        let ellipsis = "...";
        let ellipsis_width = user_interface.ctx().fonts_mut(|fonts| {
            fonts
                .layout_no_wrap(ellipsis.to_string(), font_id.clone(), text_color)
                .size()
                .x
        });
        if ellipsis_width > max_text_width {
            return String::new();
        }

        let mut truncated_text = text.to_string();
        while !truncated_text.is_empty() {
            truncated_text.pop();
            let candidate_text = format!("{}{}", truncated_text, ellipsis);
            let candidate_width = user_interface.ctx().fonts_mut(|fonts| {
                fonts
                    .layout_no_wrap(candidate_text.clone(), font_id.clone(), text_color)
                    .size()
                    .x
            });
            if candidate_width <= max_text_width {
                return candidate_text;
            }
        }

        String::new()
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

        let (selected_resolver_id, filter_text, take_over_state, baseline_draft, draft) = self
            .symbol_resolver_editor_view_data
            .read("SymbolResolverEditor view")
            .map(|view_data| {
                (
                    view_data.get_selected_resolver_id().map(str::to_string),
                    view_data.get_filter_text().to_string(),
                    view_data.get_take_over_state().cloned(),
                    view_data.get_baseline_draft().cloned(),
                    view_data.get_draft().cloned(),
                )
            })
            .unwrap_or((None, String::new(), None, None, None));

        self.ensure_selected_resolver_has_draft(&project_symbol_catalog, selected_resolver_id.as_deref(), take_over_state.as_ref());

        let (selected_resolver_id, filter_text, take_over_state, baseline_draft, draft) = self
            .symbol_resolver_editor_view_data
            .read("SymbolResolverEditor view after draft ensure")
            .map(|view_data| {
                (
                    view_data.get_selected_resolver_id().map(str::to_string),
                    view_data.get_filter_text().to_string(),
                    view_data.get_take_over_state().cloned(),
                    view_data.get_baseline_draft().cloned(),
                    view_data.get_draft().cloned(),
                )
            })
            .unwrap_or((selected_resolver_id, filter_text, take_over_state, baseline_draft, draft));

        let can_handle_window_shortcuts = self
            .app_context
            .window_focus_manager
            .can_window_handle_shortcuts(user_interface.ctx(), Self::WINDOW_ID);

        if can_handle_window_shortcuts && user_interface.input(|input_state| input_state.key_pressed(Key::Escape)) && take_over_state.is_some() {
            if let Some(mut view_data) = self
                .symbol_resolver_editor_view_data
                .write("SymbolResolverEditor escape")
            {
                view_data.cancel_take_over_state();
            }
        }

        user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                self.render_toolbar(
                    user_interface,
                    &project_symbol_catalog,
                    selected_resolver_id.as_deref(),
                    take_over_state.as_ref(),
                );
                user_interface.add_space(4.0);

                let list_height = (user_interface.available_height() * 0.38).clamp(120.0, 260.0);
                user_interface.allocate_ui_with_layout(
                    vec2(user_interface.available_width(), list_height),
                    Layout::top_down(Align::Min),
                    |user_interface| {
                        self.render_resolver_list(
                            user_interface,
                            &project_symbol_catalog,
                            selected_resolver_id.as_deref(),
                            &filter_text,
                            matches!(take_over_state, Some(SymbolResolverEditorTakeOverState::CreateResolver)),
                        );
                    },
                );

                user_interface.add_space(8.0);

                match take_over_state.as_ref() {
                    Some(SymbolResolverEditorTakeOverState::CreateResolver) => {
                        self.render_draft_editor(user_interface, &project_symbol_catalog, "New Resolver", baseline_draft.as_ref(), draft.as_ref());
                    }
                    Some(SymbolResolverEditorTakeOverState::EditResolver { .. }) => {
                        self.render_draft_editor(
                            user_interface,
                            &project_symbol_catalog,
                            "Resolver Tree",
                            baseline_draft.as_ref(),
                            draft.as_ref(),
                        );
                    }
                    Some(SymbolResolverEditorTakeOverState::DeleteConfirmation { resolver_id }) => {
                        self.render_delete_confirmation(user_interface, &project_symbol_catalog, resolver_id);
                    }
                    None => {
                        self.render_empty_selection_message(user_interface);
                    }
                }
            })
            .response
    }
}
