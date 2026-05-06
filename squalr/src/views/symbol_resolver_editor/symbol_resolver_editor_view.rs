use crate::{
    app_context::AppContext,
    ui::{
        draw::icon_draw::IconDraw,
        widgets::controls::{button::Button as ThemeButton, data_value_box::data_value_box_view::DataValueBoxView, state_layer::StateLayer},
    },
    views::{
        struct_viewer::view_data::{struct_viewer_focus_target::StructViewerFocusTarget, struct_viewer_view_data::StructViewerViewData},
        symbol_resolver_editor::view_data::symbol_resolver_editor_view_data::{SymbolResolverEditDraft, SymbolResolverEditorViewData, SymbolResolverNodeKind},
    },
};
use eframe::egui::{Align, Align2, Direction, Id, Key, Layout, Response, RichText, ScrollArea, Sense, TextureHandle, Ui, UiBuilder, Widget, pos2, vec2};
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
    structs::{
        symbolic_resolver_definition::{SymbolicResolverBinaryOperator, SymbolicResolverNode},
        valued_struct::ValuedStruct,
        valued_struct_field::ValuedStructField,
    },
};
use std::sync::Arc;

#[derive(Clone)]
pub struct SymbolResolverEditorView {
    app_context: Arc<AppContext>,
    symbol_resolver_editor_view_data: Dependency<SymbolResolverEditorViewData>,
    struct_viewer_view_data: Dependency<StructViewerViewData>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ResolverFrameAction {
    None,
    BeginCreateResolver,
    BeginEditResolver(String),
    SelectResolver(String),
    SaveDraft,
    CancelDraft,
    DeleteResolver,
}

impl SymbolResolverEditorView {
    pub const WINDOW_ID: &'static str = "window_symbol_resolver_editor";
    const TOOLBAR_HEIGHT: f32 = 28.0;
    const ROW_HEIGHT: f32 = 28.0;
    const ICON_BUTTON_WIDTH: f32 = 36.0;
    const TREE_LEVEL_INDENT: f32 = 18.0;
    const ROW_LEFT_PADDING: f32 = 8.0;
    const SMALL_ARROW_SIZE: f32 = 10.0;
    const TAKE_OVER_HEADER_HEIGHT: f32 = 32.0;
    const TAKE_OVER_CONTENT_PADDING_X: f32 = 12.0;
    const TAKE_OVER_CONTENT_PADDING_TOP: f32 = 12.0;
    const TAKE_OVER_TITLE_PADDING_X: f32 = 8.0;
    const TAKE_OVER_ROW_SPACING: f32 = 8.0;
    const RESOLVER_ID_EDITOR_WIDTH: f32 = 260.0;
    const RESOLVER_ID_EDITOR_ID: &'static str = "symbol_resolver_editor_resolver_id";
    const DETAILS_FIELD_LITERAL_VALUE: &'static str = "literal_value";
    const DETAILS_FIELD_LOCAL_FIELD: &'static str = "local_field";

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let symbol_resolver_editor_view_data = app_context
            .dependency_container
            .register(SymbolResolverEditorViewData::new());
        let struct_viewer_view_data = app_context
            .dependency_container
            .get_dependency::<StructViewerViewData>();

        Self {
            app_context,
            symbol_resolver_editor_view_data,
            struct_viewer_view_data,
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

    fn render_selection_toolbar(
        &self,
        user_interface: &mut Ui,
    ) -> ResolverFrameAction {
        let theme = &self.app_context.theme;
        let (allocated_size_rectangle, _response) =
            user_interface.allocate_exact_size(vec2(user_interface.available_width(), Self::TOOLBAR_HEIGHT), Sense::empty());
        user_interface
            .painter()
            .rect_filled(allocated_size_rectangle, CornerRadius::ZERO, theme.background_primary);

        let mut action = ResolverFrameAction::None;
        let mut toolbar_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(allocated_size_rectangle)
                .layout(Layout::left_to_right(Align::Center)),
        );

        let add_response = self.render_icon_button(&mut toolbar_user_interface, &theme.icon_library.icon_handle_common_add, "Add resolver.", false);
        if add_response.clicked() {
            action = ResolverFrameAction::BeginCreateResolver;
        }

        action
    }

    fn render_take_over_header_icon_button(
        &self,
        user_interface: &mut Ui,
        icon_handle: &TextureHandle,
        tooltip_text: &str,
        background_color: Color32,
        border_color: Color32,
        is_disabled: bool,
        button_height: f32,
    ) -> Response {
        let theme = &self.app_context.theme;
        let button_response = user_interface.add_sized(
            vec2(Self::ICON_BUTTON_WIDTH, button_height),
            ThemeButton::new_from_theme(theme)
                .with_tooltip_text(tooltip_text)
                .background_color(background_color)
                .border_color(border_color)
                .border_width(1.0)
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

    fn render_resolver_list(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        selected_resolver_id: Option<&str>,
    ) -> ResolverFrameAction {
        let mut action = ResolverFrameAction::None;

        ScrollArea::vertical()
            .id_salt("symbol_resolver_list")
            .show(user_interface, |user_interface| {
                for resolver_descriptor in project_symbol_catalog.get_symbolic_resolver_descriptors() {
                    let resolver_id = resolver_descriptor.get_resolver_id();
                    let (row_response, edit_response) = self.render_resolver_list_entry(user_interface, resolver_id, selected_resolver_id == Some(resolver_id));

                    if row_response.clicked() {
                        action = ResolverFrameAction::SelectResolver(resolver_id.to_string());
                    }
                    if edit_response.clicked() {
                        action = ResolverFrameAction::BeginEditResolver(resolver_id.to_string());
                    }
                }

                if project_symbol_catalog
                    .get_symbolic_resolver_descriptors()
                    .is_empty()
                {
                    user_interface.add_space(6.0);
                    user_interface.label(RichText::new("No resolvers.").color(self.app_context.theme.foreground_preview));
                }
            });

        action
    }

    fn render_resolver_list_entry(
        &self,
        user_interface: &mut Ui,
        resolver_id: &str,
        is_selected: bool,
    ) -> (Response, Response) {
        let theme = &self.app_context.theme;
        let (allocated_size_rectangle, row_response) =
            user_interface.allocate_exact_size(vec2(user_interface.available_width(), Self::ROW_HEIGHT), Sense::click());

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
            pressed: row_response.is_pointer_button_down_on(),
            has_hover: row_response.hovered(),
            has_focus: row_response.has_focus(),
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

        let edit_button_width = Self::ICON_BUTTON_WIDTH;
        let label_width = (row_user_interface.available_width() - edit_button_width).max(0.0);
        row_user_interface.allocate_ui_with_layout(vec2(label_width, Self::ROW_HEIGHT), Layout::left_to_right(Align::Center), |user_interface| {
            user_interface.add_space(Self::ROW_LEFT_PADDING);
            user_interface.label(
                RichText::new(resolver_id)
                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                    .color(theme.foreground),
            );
        });

        let edit_response = row_user_interface.add_sized(
            vec2(Self::ICON_BUTTON_WIDTH, Self::ROW_HEIGHT),
            ThemeButton::new_from_theme(theme)
                .background_color(Color32::TRANSPARENT)
                .with_tooltip_text("Edit resolver."),
        );
        IconDraw::draw(&mut row_user_interface, edit_response.rect, &theme.icon_library.icon_handle_common_edit);

        (row_response, edit_response)
    }

    fn render_resolver_node_editor_tree(
        &self,
        user_interface: &mut Ui,
        draft: &SymbolResolverEditDraft,
        selected_node_path: Option<&[usize]>,
    ) {
        ScrollArea::vertical()
            .id_salt("symbol_resolver_node_tree")
            .show(user_interface, |user_interface| {
                self.render_node_tree(
                    user_interface,
                    &draft.resolver_id,
                    draft.resolver_definition.get_root_node(),
                    Vec::new(),
                    0,
                    selected_node_path,
                );
            });
    }

    fn render_resolver_edit_take_over(
        &self,
        user_interface: &mut Ui,
        draft: &SymbolResolverEditDraft,
        selected_node_path: Option<&[usize]>,
        validation_result: Option<&Result<squalr_engine_api::registries::symbols::symbolic_resolver_descriptor::SymbolicResolverDescriptor, String>>,
        can_save: bool,
    ) -> ResolverFrameAction {
        let theme = &self.app_context.theme;
        let mut action = ResolverFrameAction::None;
        let is_existing_resolver = draft.original_resolver_id.is_some();
        let (panel_rect, _) = user_interface.allocate_exact_size(user_interface.available_size(), Sense::hover());

        user_interface
            .painter()
            .rect_filled(panel_rect, CornerRadius::ZERO, theme.background_panel);

        let mut panel_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(panel_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        panel_user_interface.set_clip_rect(panel_rect);

        let (header_rect, _) = panel_user_interface.allocate_exact_size(
            vec2(panel_user_interface.available_width().max(1.0), Self::TAKE_OVER_HEADER_HEIGHT),
            Sense::hover(),
        );
        panel_user_interface
            .painter()
            .rect_filled(header_rect, CornerRadius::ZERO, theme.background_primary);

        let mut header_user_interface = panel_user_interface.new_child(
            UiBuilder::new()
                .max_rect(header_rect)
                .layout(Layout::left_to_right(Align::Center)),
        );
        header_user_interface.set_clip_rect(header_rect);

        let header_button_count: f32 = if is_existing_resolver { 2.0 } else { 1.0 };
        let header_action_width = Self::ICON_BUTTON_WIDTH * header_button_count + Self::TAKE_OVER_ROW_SPACING * (header_button_count - 1.0).max(0.0);
        let title_width = (header_rect.width() - header_action_width - Self::TAKE_OVER_TITLE_PADDING_X).max(0.0);
        let (title_rect, _) = header_user_interface.allocate_exact_size(vec2(title_width, Self::TAKE_OVER_HEADER_HEIGHT), Sense::hover());
        header_user_interface.painter().text(
            pos2(title_rect.left() + Self::TAKE_OVER_TITLE_PADDING_X, title_rect.center().y),
            Align2::LEFT_CENTER,
            if is_existing_resolver { "Edit resolver" } else { "Create resolver" },
            theme.font_library.font_noto_sans.font_window_title.clone(),
            theme.foreground,
        );

        header_user_interface.allocate_ui_with_layout(
            vec2(header_action_width, Self::TAKE_OVER_HEADER_HEIGHT),
            Layout::right_to_left(Align::Center),
            |user_interface| {
                if is_existing_resolver {
                    let delete_response = self.render_take_over_header_icon_button(
                        user_interface,
                        &theme.icon_library.icon_handle_common_delete,
                        "Delete this resolver.",
                        theme.background_control_danger,
                        theme.background_control_danger_dark,
                        false,
                        Self::TAKE_OVER_HEADER_HEIGHT,
                    );
                    if delete_response.clicked() {
                        action = ResolverFrameAction::DeleteResolver;
                    }

                    user_interface.add_space(Self::TAKE_OVER_ROW_SPACING);
                }

                let cancel_response = self.render_take_over_header_icon_button(
                    user_interface,
                    &theme.icon_library.icon_handle_navigation_cancel,
                    "Cancel resolver editing.",
                    theme.background_control_secondary,
                    theme.submenu_border,
                    false,
                    Self::TAKE_OVER_HEADER_HEIGHT,
                );
                if cancel_response.clicked() {
                    action = ResolverFrameAction::CancelDraft;
                }
            },
        );

        panel_user_interface.add_space(Self::TAKE_OVER_CONTENT_PADDING_TOP);
        panel_user_interface.horizontal(|user_interface| {
            user_interface.add_space(Self::TAKE_OVER_CONTENT_PADDING_X);
            user_interface.allocate_ui_with_layout(
                vec2(
                    (user_interface.available_width() - Self::TAKE_OVER_CONTENT_PADDING_X * 2.0).max(0.0),
                    user_interface.available_height(),
                ),
                Layout::top_down(Align::Min),
                |user_interface| {
                    user_interface.label(RichText::new("Name").strong().color(theme.foreground));
                    user_interface.add_space(Self::TAKE_OVER_ROW_SPACING);

                    let mut resolver_id_value = AnonymousValueString::new(draft.resolver_id.clone(), AnonymousValueStringFormat::String, ContainerType::None);
                    let string_data_type_ref = Self::string_data_type_ref();
                    let resolver_id_editor_width = Self::RESOLVER_ID_EDITOR_WIDTH.min(user_interface.available_width().max(0.0));

                    user_interface.horizontal(|user_interface| {
                        user_interface.add_sized(
                            vec2(resolver_id_editor_width, Self::ROW_HEIGHT),
                            DataValueBoxView::new(
                                self.app_context.clone(),
                                &mut resolver_id_value,
                                &string_data_type_ref,
                                false,
                                true,
                                "Resolver name",
                                Self::RESOLVER_ID_EDITOR_ID,
                            )
                            .width(resolver_id_editor_width)
                            .height(Self::ROW_HEIGHT)
                            .use_format_text_colors(false),
                        );

                        let edited_resolver_id = resolver_id_value.get_anonymous_value_string().to_string();
                        if edited_resolver_id != draft.resolver_id {
                            if let Some(mut view_data) = self
                                .symbol_resolver_editor_view_data
                                .write("SymbolResolverEditor update resolver name")
                            {
                                view_data.update_draft_resolver_id(edited_resolver_id);
                            }
                        }

                        user_interface.add_space(Self::TAKE_OVER_ROW_SPACING);

                        let save_response = self.render_take_over_header_icon_button(
                            user_interface,
                            &theme.icon_library.icon_handle_common_check_mark,
                            "Save resolver.",
                            theme.background_control_primary,
                            theme.background_control_primary_dark,
                            !can_save,
                            Self::ROW_HEIGHT,
                        );
                        if can_save && (save_response.clicked() || DataValueBoxView::consume_commit_on_enter(user_interface, Self::RESOLVER_ID_EDITOR_ID)) {
                            action = ResolverFrameAction::SaveDraft;
                        }
                    });

                    if let Some(Err(validation_error)) = validation_result {
                        user_interface.add_space(Self::TAKE_OVER_ROW_SPACING);
                        user_interface.label(RichText::new(validation_error).color(theme.error_red));
                    }

                    user_interface.add_space(Self::TAKE_OVER_CONTENT_PADDING_TOP);
                    self.render_resolver_node_editor_tree(user_interface, draft, selected_node_path);
                },
            );
        });

        let popup_id = Id::new(("data_value_box_popup", Self::RESOLVER_ID_EDITOR_ID, user_interface.id().value()));
        let is_format_popup_open = user_interface.memory(|memory| memory.data.get_temp::<bool>(popup_id).unwrap_or(false));
        if user_interface.input(|input_state| input_state.key_pressed(Key::Escape)) && !is_format_popup_open {
            action = ResolverFrameAction::CancelDraft;
        }

        action
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
            self.focus_current_selection_in_struct_viewer();
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

        let label_position = pos2(arrow_center.x + Self::SMALL_ARROW_SIZE * 0.5 + 8.0, allocated_size_rectangle.center().y);
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

    fn focus_current_selection_in_struct_viewer(&self) {
        let (selected_node_path, draft) = self
            .symbol_resolver_editor_view_data
            .read("SymbolResolverEditor focus details selection")
            .map(|view_data| (view_data.get_selected_node_path().map(<[usize]>::to_vec), view_data.get_draft().cloned()))
            .unwrap_or((None, None));
        let Some(draft) = draft else {
            return;
        };

        self.focus_draft_selection_in_struct_viewer(selected_node_path, &draft);
    }

    fn clear_struct_viewer_if_symbol_resolver_focused(&self) {
        let is_symbol_resolver_focused = self
            .struct_viewer_view_data
            .read("SymbolResolverEditor check details focus")
            .and_then(|struct_viewer_view_data| struct_viewer_view_data.get_focus_target().cloned())
            .is_some_and(|focus_target| matches!(focus_target, StructViewerFocusTarget::SymbolResolverEditor { .. }));

        if is_symbol_resolver_focused {
            StructViewerViewData::clear_focus(self.struct_viewer_view_data.clone());
        }
    }

    fn focus_draft_selection_in_struct_viewer(
        &self,
        selected_node_path: Option<Vec<usize>>,
        draft: &SymbolResolverEditDraft,
    ) {
        let Some(details_struct) = Self::build_details_struct(draft, selected_node_path.as_deref()) else {
            return;
        };
        let selection_key = Self::build_struct_viewer_focus_target_key(draft, selected_node_path.as_deref());
        let edit_callback = Self::build_struct_viewer_edit_callback(
            self.app_context.clone(),
            self.symbol_resolver_editor_view_data.clone(),
            self.struct_viewer_view_data.clone(),
            selected_node_path,
            self.default_data_type_ref(),
        );

        StructViewerViewData::focus_valued_struct_with_focus_target(
            self.struct_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            details_struct,
            edit_callback,
            Some(StructViewerFocusTarget::SymbolResolverEditor { selection_key }),
        );
    }

    fn build_struct_viewer_edit_callback(
        app_context: Arc<AppContext>,
        symbol_resolver_editor_view_data: Dependency<SymbolResolverEditorViewData>,
        struct_viewer_view_data: Dependency<StructViewerViewData>,
        selected_node_path: Option<Vec<usize>>,
        default_data_type_ref: DataTypeRef,
    ) -> Arc<dyn Fn(ValuedStructField) + Send + Sync> {
        Arc::new(move |edited_field: ValuedStructField| {
            let mut should_refocus_details = false;
            let updated_draft = {
                let Some(mut view_data) = symbol_resolver_editor_view_data.write("SymbolResolverEditor apply details edit") else {
                    return;
                };
                let Some(mut draft) = view_data.get_draft().cloned() else {
                    return;
                };

                if let Some(selected_node_path) = selected_node_path.as_deref() {
                    should_refocus_details = Self::apply_node_details_edit(&mut draft, selected_node_path, &edited_field, default_data_type_ref.clone());
                }

                view_data.update_draft(draft.clone());
                draft
            };

            if should_refocus_details {
                let Some(details_struct) = Self::build_details_struct(&updated_draft, selected_node_path.as_deref()) else {
                    return;
                };
                let selection_key = Self::build_struct_viewer_focus_target_key(&updated_draft, selected_node_path.as_deref());
                let edit_callback = Self::build_struct_viewer_edit_callback(
                    app_context.clone(),
                    symbol_resolver_editor_view_data.clone(),
                    struct_viewer_view_data.clone(),
                    selected_node_path.clone(),
                    default_data_type_ref.clone(),
                );

                StructViewerViewData::focus_valued_struct_with_focus_target(
                    struct_viewer_view_data.clone(),
                    app_context.engine_unprivileged_state.clone(),
                    details_struct,
                    edit_callback,
                    Some(StructViewerFocusTarget::SymbolResolverEditor { selection_key }),
                );
            }
        })
    }

    fn apply_node_details_edit(
        draft: &mut SymbolResolverEditDraft,
        selected_node_path: &[usize],
        edited_field: &ValuedStructField,
        default_data_type_ref: DataTypeRef,
    ) -> bool {
        let edited_field_name = edited_field.get_name();
        let edited_text = StructViewerViewData::read_utf8_field_text(edited_field);
        let Some(selected_node) = Self::get_node_mut(draft.resolver_definition.get_root_node_mut(), selected_node_path) else {
            return false;
        };

        match edited_field_name {
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_NODE_KIND => {
                let Some(next_kind) = Self::resolver_node_kind_from_label(&edited_text) else {
                    return false;
                };

                if next_kind != Self::resolver_node_kind(selected_node) {
                    *selected_node = SymbolResolverEditorViewData::default_node_for_kind(next_kind, default_data_type_ref);
                    return true;
                }
            }
            Self::DETAILS_FIELD_LITERAL_VALUE => {
                if let (SymbolicResolverNode::Literal(value), Ok(parsed_value)) = (selected_node, edited_text.trim().parse::<i128>()) {
                    *value = parsed_value;
                }
            }
            Self::DETAILS_FIELD_LOCAL_FIELD => {
                if let SymbolicResolverNode::LocalField { field_name } = selected_node {
                    *field_name = edited_text;
                }
            }
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_DATA_TYPE => {
                if let SymbolicResolverNode::TypeSize { data_type_ref } = selected_node {
                    *data_type_ref = DataTypeRef::new(edited_text.trim());
                }
            }
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_OPERATOR => {
                if let (SymbolicResolverNode::Binary { operator, .. }, Some(next_operator)) = (selected_node, Self::resolver_operator_from_label(&edited_text))
                {
                    *operator = next_operator;
                }
            }
            _ => {}
        }

        false
    }

    fn build_details_struct(
        draft: &SymbolResolverEditDraft,
        selected_node_path: Option<&[usize]>,
    ) -> Option<ValuedStruct> {
        let selected_node_path = selected_node_path?;
        let selected_node = Self::get_node(draft.resolver_definition.get_root_node(), selected_node_path)?;
        let mut fields = vec![
            DataTypeStringUtf8::get_value_from_primitive_string(Self::resolver_node_kind_label(Self::resolver_node_kind(selected_node)))
                .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_NODE_KIND.to_string(), false),
        ];

        match selected_node {
            SymbolicResolverNode::Literal(value) => {
                fields.push(
                    DataTypeStringUtf8::get_value_from_primitive_string(&value.to_string())
                        .to_named_valued_struct_field(Self::DETAILS_FIELD_LITERAL_VALUE.to_string(), false),
                );
            }
            SymbolicResolverNode::LocalField { field_name } => {
                fields.push(
                    DataTypeStringUtf8::get_value_from_primitive_string(field_name)
                        .to_named_valued_struct_field(Self::DETAILS_FIELD_LOCAL_FIELD.to_string(), false),
                );
            }
            SymbolicResolverNode::TypeSize { data_type_ref } => {
                fields.push(
                    DataTypeStringUtf8::get_value_from_primitive_string(data_type_ref.get_data_type_id())
                        .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_DATA_TYPE.to_string(), false),
                );
            }
            SymbolicResolverNode::Binary { operator, .. } => {
                fields.push(
                    DataTypeStringUtf8::get_value_from_primitive_string(operator.label())
                        .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_OPERATOR.to_string(), false),
                );
            }
        }

        Some(ValuedStruct::new_anonymous(fields))
    }

    fn build_struct_viewer_focus_target_key(
        draft: &SymbolResolverEditDraft,
        selected_node_path: Option<&[usize]>,
    ) -> String {
        let resolver_key = draft
            .original_resolver_id
            .as_deref()
            .unwrap_or(draft.resolver_id.as_str());
        let node_path_key = selected_node_path
            .map(|node_path| {
                node_path
                    .iter()
                    .map(usize::to_string)
                    .collect::<Vec<_>>()
                    .join(".")
            })
            .unwrap_or_default();

        format!("{}|{}", resolver_key, node_path_key)
    }

    fn select_resolver(
        &self,
        resolver_id: &str,
    ) {
        if let Some(mut view_data) = self
            .symbol_resolver_editor_view_data
            .write("SymbolResolverEditor select resolver")
        {
            view_data.select_resolver(Some(resolver_id.to_string()));
        }
        self.clear_struct_viewer_if_symbol_resolver_focused();
    }

    fn apply_frame_action(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        action: ResolverFrameAction,
        selected_resolver_id: Option<&str>,
        draft: Option<&SymbolResolverEditDraft>,
    ) {
        match action {
            ResolverFrameAction::None => {}
            ResolverFrameAction::BeginCreateResolver => {
                if let Some(mut view_data) = self
                    .symbol_resolver_editor_view_data
                    .write("SymbolResolverEditor begin create resolver")
                {
                    view_data.begin_create_resolver(project_symbol_catalog);
                }
                self.focus_current_selection_in_struct_viewer();
            }
            ResolverFrameAction::BeginEditResolver(resolver_id) => {
                if let Some(mut view_data) = self
                    .symbol_resolver_editor_view_data
                    .write("SymbolResolverEditor begin edit resolver")
                {
                    view_data.begin_edit_resolver(project_symbol_catalog, &resolver_id);
                }
                self.focus_current_selection_in_struct_viewer();
            }
            ResolverFrameAction::SelectResolver(resolver_id) => {
                self.select_resolver(&resolver_id);
            }
            ResolverFrameAction::SaveDraft => {
                let current_draft = self
                    .symbol_resolver_editor_view_data
                    .read("SymbolResolverEditor read draft for save")
                    .and_then(|view_data| view_data.get_draft().cloned())
                    .or_else(|| draft.cloned());
                if let Some(current_draft) = current_draft.as_ref() {
                    self.save_draft(project_symbol_catalog, current_draft);
                }
            }
            ResolverFrameAction::CancelDraft => {
                if let Some(mut view_data) = self
                    .symbol_resolver_editor_view_data
                    .write("SymbolResolverEditor cancel resolver edit")
                {
                    view_data.cancel_take_over_state();
                }
                self.clear_struct_viewer_if_symbol_resolver_focused();
            }
            ResolverFrameAction::DeleteResolver => {
                if let Some(selected_resolver_id) = selected_resolver_id {
                    let updated_project_symbol_catalog =
                        SymbolResolverEditorViewData::remove_resolver_from_catalog(project_symbol_catalog, selected_resolver_id);
                    self.persist_project_symbol_catalog(updated_project_symbol_catalog);
                    if let Some(mut view_data) = self
                        .symbol_resolver_editor_view_data
                        .write("SymbolResolverEditor delete resolver")
                    {
                        view_data.cancel_take_over_state();
                    }
                    self.clear_struct_viewer_if_symbol_resolver_focused();
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
                self.persist_project_symbol_catalog(updated_project_symbol_catalog.clone());
                if let Some(mut view_data) = self
                    .symbol_resolver_editor_view_data
                    .write("SymbolResolverEditor save resolver")
                {
                    view_data.cancel_take_over_state();
                    view_data.select_resolver(Some(saved_resolver_id));
                }
                self.clear_struct_viewer_if_symbol_resolver_focused();
            }
            Err(error) => {
                log::error!("Failed to apply symbol resolver draft: {}.", error);
            }
        }
    }

    fn resolver_node_kind(resolver_node: &SymbolicResolverNode) -> SymbolResolverNodeKind {
        match resolver_node {
            SymbolicResolverNode::Literal(_) => SymbolResolverNodeKind::Literal,
            SymbolicResolverNode::LocalField { .. } => SymbolResolverNodeKind::LocalField,
            SymbolicResolverNode::TypeSize { .. } => SymbolResolverNodeKind::TypeSize,
            SymbolicResolverNode::Binary { .. } => SymbolResolverNodeKind::Operation,
        }
    }

    fn resolver_node_kind_label(resolver_node_kind: SymbolResolverNodeKind) -> &'static str {
        match resolver_node_kind {
            SymbolResolverNodeKind::Literal => "Literal",
            SymbolResolverNodeKind::LocalField => "Local Field",
            SymbolResolverNodeKind::TypeSize => "Type Size",
            SymbolResolverNodeKind::Operation => "Operation",
        }
    }

    fn resolver_node_kind_from_label(label: &str) -> Option<SymbolResolverNodeKind> {
        match label.trim() {
            "Literal" => Some(SymbolResolverNodeKind::Literal),
            "Local Field" => Some(SymbolResolverNodeKind::LocalField),
            "Type Size" => Some(SymbolResolverNodeKind::TypeSize),
            "Operation" => Some(SymbolResolverNodeKind::Operation),
            _ => None,
        }
    }

    fn resolver_operator_from_label(label: &str) -> Option<SymbolicResolverBinaryOperator> {
        SymbolicResolverBinaryOperator::ALL
            .iter()
            .copied()
            .find(|operator| operator.label() == label.trim())
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

    fn get_node<'resolver>(
        resolver_node: &'resolver SymbolicResolverNode,
        node_path: &[usize],
    ) -> Option<&'resolver SymbolicResolverNode> {
        if node_path.is_empty() {
            return Some(resolver_node);
        }

        match resolver_node {
            SymbolicResolverNode::Binary { left_node, right_node, .. } => match node_path[0] {
                0 => Self::get_node(left_node, &node_path[1..]),
                1 => Self::get_node(right_node, &node_path[1..]),
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
    Literal,
    LocalField,
    TypeSize,
    Operation,
}

impl TreeEntryKind {
    fn has_children(self) -> bool {
        matches!(self, Self::Operation)
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

        let can_handle_window_shortcuts = self
            .app_context
            .window_focus_manager
            .can_window_handle_shortcuts(user_interface.ctx(), Self::WINDOW_ID);
        let validation_result = draft
            .as_ref()
            .map(|draft| SymbolResolverEditorViewData::build_resolver_descriptor(&project_symbol_catalog, draft));
        let has_draft_changes = draft
            .as_ref()
            .zip(baseline_draft.as_ref())
            .map(|(draft, baseline_draft)| draft != baseline_draft)
            .unwrap_or(false);
        let is_creating_resolver = matches!(
            take_over_state,
            Some(crate::views::symbol_resolver_editor::view_data::symbol_resolver_editor_view_data::SymbolResolverEditorTakeOverState::CreateResolver)
        );
        let can_save = draft.is_some() && validation_result.as_ref().is_some_and(Result::is_ok) && (has_draft_changes || is_creating_resolver);
        let mut frame_action = ResolverFrameAction::None;

        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                if let Some(draft) = draft.as_ref() {
                    frame_action =
                        self.render_resolver_edit_take_over(user_interface, draft, selected_node_path.as_deref(), validation_result.as_ref(), can_save);
                } else {
                    frame_action = self.render_selection_toolbar(user_interface);
                    user_interface.add_space(4.0);
                    user_interface.allocate_ui_with_layout(
                        vec2(user_interface.available_width(), user_interface.available_height().max(Self::ROW_HEIGHT)),
                        Layout::top_down(Align::Min),
                        |user_interface| {
                            let list_action = self.render_resolver_list(user_interface, &project_symbol_catalog, selected_resolver_id.as_deref());
                            if !matches!(list_action, ResolverFrameAction::None) {
                                frame_action = list_action;
                            }
                        },
                    );
                }
            })
            .response;

        if can_handle_window_shortcuts
            && user_interface.input(|input_state| input_state.key_pressed(Key::Escape))
            && draft.is_some()
            && matches!(frame_action, ResolverFrameAction::None)
        {
            frame_action = ResolverFrameAction::CancelDraft;
        }

        self.apply_frame_action(&project_symbol_catalog, frame_action, selected_resolver_id.as_deref(), draft.as_ref());

        response
    }
}

#[cfg(test)]
mod tests {
    use super::SymbolResolverEditorView;
    use crate::views::{
        struct_viewer::view_data::struct_viewer_view_data::StructViewerViewData,
        symbol_resolver_editor::view_data::symbol_resolver_editor_view_data::SymbolResolverEditDraft,
    };
    use squalr_engine_api::structures::{
        data_types::{built_in_types::string::utf8::data_type_string_utf8::DataTypeStringUtf8, data_type_ref::DataTypeRef},
        structs::symbolic_resolver_definition::{SymbolicResolverDefinition, SymbolicResolverNode},
    };

    #[test]
    fn node_kind_edit_requests_details_refresh_when_shape_changes() {
        let mut draft = SymbolResolverEditDraft {
            original_resolver_id: Some(String::from("count")),
            resolver_id: String::from("count"),
            resolver_definition: SymbolicResolverDefinition::new(SymbolicResolverNode::new_literal(7)),
        };
        let edited_kind_field = DataTypeStringUtf8::get_value_from_primitive_string("Operation")
            .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_NODE_KIND.to_string(), false);

        let should_refocus_details = SymbolResolverEditorView::apply_node_details_edit(&mut draft, &[], &edited_kind_field, DataTypeRef::new("u32"));

        assert!(should_refocus_details);
        assert!(matches!(draft.resolver_definition.get_root_node(), SymbolicResolverNode::Binary { .. }));

        let details_struct = SymbolResolverEditorView::build_details_struct(&draft, Some(&[])).expect("Expected node details.");

        assert!(
            details_struct
                .get_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_OPERATOR)
                .is_some()
        );
        assert!(
            details_struct
                .get_field(SymbolResolverEditorView::DETAILS_FIELD_LITERAL_VALUE)
                .is_none()
        );
    }

    #[test]
    fn node_kind_edit_skips_details_refresh_when_shape_is_unchanged() {
        let mut draft = SymbolResolverEditDraft {
            original_resolver_id: Some(String::from("count")),
            resolver_id: String::from("count"),
            resolver_definition: SymbolicResolverDefinition::new(SymbolicResolverNode::new_literal(7)),
        };
        let edited_kind_field = DataTypeStringUtf8::get_value_from_primitive_string("Literal")
            .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_NODE_KIND.to_string(), false);

        let should_refocus_details = SymbolResolverEditorView::apply_node_details_edit(&mut draft, &[], &edited_kind_field, DataTypeRef::new("u32"));

        assert!(!should_refocus_details);
        assert!(matches!(draft.resolver_definition.get_root_node(), SymbolicResolverNode::Literal(7)));
    }
}
