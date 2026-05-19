use crate::{
    app_context::AppContext,
    ui::widgets::controls::{data_value_box::data_value_box_view::DataValueBoxView, groupbox::GroupBox},
    views::symbol_resolver_editor::{
        symbol_resolver_node_tree_view::SymbolResolverNodeTreeView,
        view_data::symbol_resolver_editor_view_data::{SymbolResolverEditDraft, SymbolResolverEditorTakeOverState, SymbolResolverEditorViewData},
    },
};
use eframe::egui::{Align, Align2, Button as EguiButton, Id, Key, Layout, RichText, Sense, Ui, UiBuilder, pos2, vec2};
use epaint::{CornerRadius, Stroke};
use squalr_engine_api::{
    dependency_injection::dependency::Dependency,
    registries::symbols::symbolic_resolver_descriptor::SymbolicResolverDescriptor,
    structures::{
        data_types::{built_in_types::string::utf8::data_type_string_utf8::DataTypeStringUtf8, data_type_ref::DataTypeRef},
        data_values::{anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType},
        projects::project_symbol_catalog::ProjectSymbolCatalog,
    },
};
use std::sync::Arc;

pub struct SymbolResolverTakeoverHostView<'lifetime> {
    app_context: Arc<AppContext>,
    symbol_resolver_editor_view_data: Dependency<SymbolResolverEditorViewData>,
    project_symbol_catalog: &'lifetime ProjectSymbolCatalog,
    take_over_state: &'lifetime SymbolResolverEditorTakeOverState,
    draft: Option<&'lifetime SymbolResolverEditDraft>,
    selected_node_path: Option<&'lifetime [usize]>,
    validation_result: Option<&'lifetime Result<SymbolicResolverDescriptor, String>>,
    can_save: bool,
    can_handle_window_shortcuts: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolResolverTakeoverAction {
    None,
    SaveDraft,
    CancelDraft,
    ConfirmDeleteResolver(String),
    SelectNode { resolver_id: String, node_path: Vec<usize> },
}

impl<'lifetime> SymbolResolverTakeoverHostView<'lifetime> {
    const ROW_HEIGHT: f32 = 28.0;
    const TAKE_OVER_HEADER_HEIGHT: f32 = 32.0;
    const TAKE_OVER_CONTENT_PADDING_X: f32 = 12.0;
    const TAKE_OVER_CONTENT_PADDING_TOP: f32 = 12.0;
    const TAKE_OVER_TITLE_PADDING_X: f32 = 8.0;
    const TAKE_OVER_ROW_SPACING: f32 = 8.0;
    const TAKE_OVER_GROUPBOX_SIDE_PADDING: f32 = 8.0;
    const TAKE_OVER_BOTTOM_PADDING: f32 = 8.0;
    const TAKE_OVER_ACTION_BUTTON_WIDTH: f32 = 120.0;
    const TAKE_OVER_ACTION_BUTTON_SPACING: f32 = 12.0;
    const RESOLVER_ID_EDITOR_WIDTH: f32 = 260.0;
    const RESOLVER_ID_EDITOR_ID: &'static str = "symbol_resolver_editor_resolver_id";

    pub fn new(
        app_context: Arc<AppContext>,
        symbol_resolver_editor_view_data: Dependency<SymbolResolverEditorViewData>,
        project_symbol_catalog: &'lifetime ProjectSymbolCatalog,
        take_over_state: &'lifetime SymbolResolverEditorTakeOverState,
        draft: Option<&'lifetime SymbolResolverEditDraft>,
        selected_node_path: Option<&'lifetime [usize]>,
        validation_result: Option<&'lifetime Result<SymbolicResolverDescriptor, String>>,
        can_save: bool,
        can_handle_window_shortcuts: bool,
    ) -> Self {
        Self {
            app_context,
            symbol_resolver_editor_view_data,
            project_symbol_catalog,
            take_over_state,
            draft,
            selected_node_path,
            validation_result,
            can_save,
            can_handle_window_shortcuts,
        }
    }

    pub fn show(
        self,
        user_interface: &mut Ui,
    ) -> SymbolResolverTakeoverAction {
        match (self.take_over_state, self.draft) {
            (SymbolResolverEditorTakeOverState::CreateResolver | SymbolResolverEditorTakeOverState::RenameResolver { .. }, Some(draft)) => {
                self.render_resolver_name_take_over(user_interface, draft)
            }
            (SymbolResolverEditorTakeOverState::OpenResolver { .. }, Some(draft)) => self.render_resolver_open_take_over(user_interface, draft),
            (SymbolResolverEditorTakeOverState::DeleteConfirmation { resolver_id }, _) => {
                self.render_delete_confirmation_take_over(user_interface, resolver_id)
            }
            _ => SymbolResolverTakeoverAction::None,
        }
    }

    fn render_take_over_action_buttons(
        &self,
        user_interface: &mut Ui,
        accept_label: &str,
        can_accept: bool,
    ) -> (eframe::egui::Response, eframe::egui::Response) {
        let theme = &self.app_context.theme;
        let button_size = vec2(Self::TAKE_OVER_ACTION_BUTTON_WIDTH, Self::ROW_HEIGHT);
        let total_button_width = button_size.x * 2.0 + Self::TAKE_OVER_ACTION_BUTTON_SPACING;
        let side_spacing = ((user_interface.available_width() - total_button_width) * 0.5).max(0.0);

        let responses = user_interface
            .horizontal(|user_interface| {
                user_interface.add_space(side_spacing);
                user_interface.spacing_mut().item_spacing.x = Self::TAKE_OVER_ACTION_BUTTON_SPACING;

                let cancel_response = user_interface.add_sized(
                    button_size,
                    EguiButton::new(RichText::new("Cancel").color(theme.foreground))
                        .fill(theme.background_control_secondary)
                        .stroke(Stroke::new(1.0, theme.background_control_secondary_dark)),
                );

                let accept_button = EguiButton::new(RichText::new(accept_label).color(if can_accept { theme.foreground } else { theme.foreground_preview }))
                    .fill(if can_accept {
                        theme.background_control_primary
                    } else {
                        theme.background_control_secondary
                    })
                    .stroke(Stroke::new(
                        1.0,
                        if can_accept {
                            theme.background_control_primary_dark
                        } else {
                            theme.background_control_secondary_dark
                        },
                    ));
                let accept_response = user_interface
                    .add_enabled_ui(can_accept, |user_interface| user_interface.add_sized(button_size, accept_button))
                    .inner;

                (cancel_response, accept_response)
            })
            .inner;

        user_interface.add_space(Self::TAKE_OVER_BOTTOM_PADDING);

        responses
    }

    fn render_delete_take_over_action_buttons(
        &self,
        user_interface: &mut Ui,
    ) -> (eframe::egui::Response, eframe::egui::Response) {
        let theme = &self.app_context.theme;
        let button_size = vec2(Self::TAKE_OVER_ACTION_BUTTON_WIDTH, Self::ROW_HEIGHT);
        let total_button_width = button_size.x * 2.0 + Self::TAKE_OVER_ACTION_BUTTON_SPACING;
        let side_spacing = ((user_interface.available_width() - total_button_width) * 0.5).max(0.0);

        let responses = user_interface
            .horizontal(|user_interface| {
                user_interface.add_space(side_spacing);
                user_interface.spacing_mut().item_spacing.x = Self::TAKE_OVER_ACTION_BUTTON_SPACING;

                let delete_response = user_interface.add_sized(
                    button_size,
                    EguiButton::new(RichText::new("Delete").color(theme.foreground))
                        .fill(theme.background_control_danger)
                        .stroke(Stroke::new(1.0, theme.background_control_danger_dark)),
                );

                let cancel_response = user_interface.add_sized(
                    button_size,
                    EguiButton::new(RichText::new("Cancel").color(theme.foreground))
                        .fill(theme.background_control_secondary)
                        .stroke(Stroke::new(1.0, theme.background_control_secondary_dark)),
                );

                (delete_response, cancel_response)
            })
            .inner;

        user_interface.add_space(Self::TAKE_OVER_BOTTOM_PADDING);

        responses
    }

    fn render_resolver_name_take_over(
        &self,
        user_interface: &mut Ui,
        draft: &SymbolResolverEditDraft,
    ) -> SymbolResolverTakeoverAction {
        let theme = &self.app_context.theme;
        let mut action = SymbolResolverTakeoverAction::None;
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

        let title_width = (header_rect.width() - Self::TAKE_OVER_TITLE_PADDING_X).max(0.0);
        let (title_rect, _) = header_user_interface.allocate_exact_size(vec2(title_width, Self::TAKE_OVER_HEADER_HEIGHT), Sense::hover());
        header_user_interface.painter().text(
            pos2(title_rect.left() + Self::TAKE_OVER_TITLE_PADDING_X, title_rect.center().y),
            Align2::LEFT_CENTER,
            if is_existing_resolver { "Rename resolver" } else { "Create resolver" },
            theme.font_library.font_noto_sans.font_window_title.clone(),
            theme.foreground,
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
                    let string_data_type_ref = DataTypeRef::new(DataTypeStringUtf8::DATA_TYPE_ID);
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
                        let did_commit_resolver_id = DataValueBoxView::consume_commit_on_enter(user_interface, Self::RESOLVER_ID_EDITOR_ID);
                        if edited_resolver_id != draft.resolver_id {
                            if let Some(mut view_data) = self
                                .symbol_resolver_editor_view_data
                                .write("SymbolResolverEditor update resolver name")
                            {
                                view_data.update_draft_resolver_id(edited_resolver_id);
                            }
                        }

                        if self.can_save && did_commit_resolver_id {
                            action = SymbolResolverTakeoverAction::SaveDraft;
                        }
                    });

                    if let Some(Err(validation_error)) = self.validation_result {
                        user_interface.add_space(Self::TAKE_OVER_ROW_SPACING);
                        user_interface.label(RichText::new(validation_error).color(theme.error_red));
                    }

                    user_interface.add_space(Self::TAKE_OVER_ROW_SPACING + Self::TAKE_OVER_ROW_SPACING);
                    let (cancel_response, accept_response) = self.render_take_over_action_buttons(user_interface, "Accept", self.can_save);
                    if cancel_response.clicked() {
                        action = SymbolResolverTakeoverAction::CancelDraft;
                    }
                    if accept_response.clicked() {
                        action = SymbolResolverTakeoverAction::SaveDraft;
                    }
                },
            );
        });

        let popup_id = Id::new(("data_value_box_popup", Self::RESOLVER_ID_EDITOR_ID, user_interface.id().value()));
        let is_format_popup_open = user_interface.memory(|memory| memory.data.get_temp::<bool>(popup_id).unwrap_or(false));
        if user_interface.input(|input_state| input_state.key_pressed(Key::Escape)) && !is_format_popup_open {
            action = SymbolResolverTakeoverAction::CancelDraft;
        }

        action
    }

    fn render_resolver_open_take_over(
        &self,
        user_interface: &mut Ui,
        draft: &SymbolResolverEditDraft,
    ) -> SymbolResolverTakeoverAction {
        let theme = &self.app_context.theme;
        let mut action = SymbolResolverTakeoverAction::None;
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

        let title_width = (header_rect.width() - Self::TAKE_OVER_TITLE_PADDING_X).max(0.0);
        let (title_rect, _) = header_user_interface.allocate_exact_size(vec2(title_width, Self::TAKE_OVER_HEADER_HEIGHT), Sense::hover());
        header_user_interface.painter().text(
            pos2(title_rect.left() + Self::TAKE_OVER_TITLE_PADDING_X, title_rect.center().y),
            Align2::LEFT_CENTER,
            draft.resolver_id.as_str(),
            theme.font_library.font_noto_sans.font_window_title.clone(),
            theme.foreground,
        );

        panel_user_interface.add_space(4.0);
        let tree_height = (panel_user_interface.available_height() - Self::ROW_HEIGHT - Self::TAKE_OVER_ROW_SPACING).max(Self::ROW_HEIGHT);
        panel_user_interface.allocate_ui_with_layout(
            vec2(panel_user_interface.available_width(), tree_height),
            Layout::top_down(Align::Min),
            |user_interface| {
                if let Some(node_tree_action) = SymbolResolverNodeTreeView::new(
                    self.app_context.clone(),
                    &draft.resolver_id,
                    draft.resolver_definition.get_root_node(),
                    self.selected_node_path,
                )
                .show(user_interface)
                {
                    action = SymbolResolverTakeoverAction::SelectNode {
                        resolver_id: node_tree_action.resolver_id,
                        node_path: node_tree_action.node_path,
                    };
                }
            },
        );
        panel_user_interface.add_space(Self::TAKE_OVER_ROW_SPACING);
        let (cancel_response, accept_response) = self.render_take_over_action_buttons(&mut panel_user_interface, "Accept", self.can_save);
        if cancel_response.clicked() {
            action = SymbolResolverTakeoverAction::CancelDraft;
        }
        if accept_response.clicked() {
            action = SymbolResolverTakeoverAction::SaveDraft;
        }

        action
    }

    fn render_delete_confirmation_take_over(
        &self,
        user_interface: &mut Ui,
        resolver_id: &str,
    ) -> SymbolResolverTakeoverAction {
        let theme = &self.app_context.theme;
        let mut action = SymbolResolverTakeoverAction::None;
        let usage_count = SymbolResolverEditorViewData::count_resolver_usages(self.project_symbol_catalog, resolver_id);

        if self.can_handle_window_shortcuts && user_interface.input(|input_state| input_state.key_pressed(Key::Enter)) {
            action = SymbolResolverTakeoverAction::ConfirmDeleteResolver(resolver_id.to_string());
        } else if self.can_handle_window_shortcuts && user_interface.input(|input_state| input_state.key_pressed(Key::Escape)) {
            action = SymbolResolverTakeoverAction::CancelDraft;
        }

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

        panel_user_interface.add_space(Self::TAKE_OVER_CONTENT_PADDING_TOP);
        panel_user_interface.horizontal(|user_interface| {
            user_interface.add_space(Self::TAKE_OVER_GROUPBOX_SIDE_PADDING);
            user_interface.allocate_ui_with_layout(
                vec2(
                    (user_interface.available_width() - Self::TAKE_OVER_GROUPBOX_SIDE_PADDING).max(0.0),
                    user_interface.available_height(),
                ),
                Layout::top_down(Align::Min),
                |user_interface| {
                    user_interface.add(
                        GroupBox::new_from_theme(theme, "Delete Resolver", |user_interface| {
                            user_interface.label(RichText::new(format!("Delete `{}`?", resolver_id)).color(theme.foreground));
                            user_interface.add_space(4.0);
                            let (usage_text, usage_text_color) = if usage_count == 0 {
                                (String::from("No existing fields reference this resolver."), theme.foreground_preview)
                            } else if usage_count == 1 {
                                (String::from("1 existing resolver reference will become unresolved."), theme.warning)
                            } else {
                                (format!("{} existing resolver references will become unresolved.", usage_count), theme.warning)
                            };
                            user_interface.label(RichText::new(usage_text).color(usage_text_color));
                        })
                        .desired_width(user_interface.available_width()),
                    );

                    user_interface.add_space(Self::TAKE_OVER_ROW_SPACING + Self::TAKE_OVER_ROW_SPACING);
                    let (delete_response, cancel_response) = self.render_delete_take_over_action_buttons(user_interface);
                    if delete_response.clicked() {
                        action = SymbolResolverTakeoverAction::ConfirmDeleteResolver(resolver_id.to_string());
                    }
                    if cancel_response.clicked() {
                        action = SymbolResolverTakeoverAction::CancelDraft;
                    }
                },
            );
        });

        action
    }
}
