use crate::{
    app_context::AppContext,
    ui::{
        draw::icon_draw::IconDraw,
        text::text_fitting::measure_text_width,
        widgets::controls::{
            button::Button as ThemeButton, context_menu::context_menu::ContextMenu, state_layer::StateLayer,
            toolbar_menu::toolbar_menu_item_view::ToolbarMenuItemView,
        },
    },
    views::symbol_resolver_editor::view_data::symbol_resolver_editor_view_data::{SymbolResolverContextMenuTarget, SymbolResolverEditorViewData},
};
use eframe::egui::{Align2, Rect, Response, RichText, ScrollArea, Sense, Ui, Widget, pos2, vec2};
use epaint::{Color32, CornerRadius, Stroke, StrokeKind};
use squalr_engine_api::{dependency_injection::dependency::Dependency, structures::projects::project_symbol_catalog::ProjectSymbolCatalog};
use std::sync::Arc;

pub struct SymbolResolverListView<'lifetime> {
    app_context: Arc<AppContext>,
    symbol_resolver_editor_view_data: Dependency<SymbolResolverEditorViewData>,
    project_symbol_catalog: &'lifetime ProjectSymbolCatalog,
    selected_resolver_id: Option<&'lifetime str>,
    context_menu_target: Option<&'lifetime SymbolResolverContextMenuTarget>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SymbolResolverListAction {
    None,
    BeginRenameResolver(String),
    BeginOpenResolver(String),
    SelectResolver(String),
    ShowResolverContextMenu(String, epaint::Pos2),
    RequestDeleteConfirmation(String),
}

impl<'lifetime> SymbolResolverListView<'lifetime> {
    const ROW_HEIGHT: f32 = 28.0;
    const ICON_BUTTON_WIDTH: f32 = 36.0;
    const ROW_LEFT_PADDING: f32 = 8.0;
    const ROW_LABEL_SPACING: f32 = 8.0;
    const RESOLVER_CONTEXT_MENU_WIDTH: f32 = 160.0;

    pub fn new(
        app_context: Arc<AppContext>,
        symbol_resolver_editor_view_data: Dependency<SymbolResolverEditorViewData>,
        project_symbol_catalog: &'lifetime ProjectSymbolCatalog,
        selected_resolver_id: Option<&'lifetime str>,
        context_menu_target: Option<&'lifetime SymbolResolverContextMenuTarget>,
    ) -> Self {
        Self {
            app_context,
            symbol_resolver_editor_view_data,
            project_symbol_catalog,
            selected_resolver_id,
            context_menu_target,
        }
    }

    pub fn show(
        self,
        user_interface: &mut Ui,
    ) -> SymbolResolverListAction {
        let mut action = SymbolResolverListAction::None;

        self.render_header(user_interface);
        user_interface.allocate_ui_with_layout(
            vec2(user_interface.available_width(), user_interface.available_height().max(Self::ROW_HEIGHT)),
            eframe::egui::Layout::top_down(eframe::egui::Align::Min),
            |user_interface| {
                let list_action = self.render_list_body(user_interface);
                if !matches!(list_action, SymbolResolverListAction::None) {
                    action = list_action;
                }
            },
        );

        if let Some(context_menu_target) = self.context_menu_target
            && let Some(context_menu_action) = self.render_context_menu(user_interface, context_menu_target)
        {
            action = context_menu_action;
        }

        action
    }

    fn render_list_body(
        &self,
        user_interface: &mut Ui,
    ) -> SymbolResolverListAction {
        let mut action = SymbolResolverListAction::None;

        ScrollArea::vertical()
            .id_salt("symbol_resolver_list")
            .show(user_interface, |user_interface| {
                for resolver_descriptor in self.project_symbol_catalog.get_symbolic_resolver_descriptors() {
                    let resolver_id = resolver_descriptor.get_resolver_id();
                    let usage_count = SymbolResolverEditorViewData::count_resolver_usages(self.project_symbol_catalog, resolver_id);
                    let (row_response, edit_response) =
                        self.render_list_entry(user_interface, resolver_id, usage_count, self.selected_resolver_id == Some(resolver_id));

                    if row_response.secondary_clicked() {
                        let context_menu_position = row_response
                            .interact_pointer_pos()
                            .unwrap_or(row_response.rect.left_bottom());
                        action = SymbolResolverListAction::ShowResolverContextMenu(resolver_id.to_string(), context_menu_position);
                    } else if row_response.double_clicked() {
                        action = SymbolResolverListAction::BeginOpenResolver(resolver_id.to_string());
                    } else if row_response.clicked() {
                        action = SymbolResolverListAction::SelectResolver(resolver_id.to_string());
                    }

                    if edit_response.clicked() {
                        action = SymbolResolverListAction::BeginRenameResolver(resolver_id.to_string());
                    }
                }

                if self
                    .project_symbol_catalog
                    .get_symbolic_resolver_descriptors()
                    .is_empty()
                {
                    user_interface.add_space(6.0);
                    user_interface.label(RichText::new("No resolvers.").color(self.app_context.theme.foreground_preview));
                }
            });

        action
    }

    fn render_context_menu(
        &self,
        user_interface: &mut Ui,
        context_menu_target: &SymbolResolverContextMenuTarget,
    ) -> Option<SymbolResolverListAction> {
        let theme = &self.app_context.theme;
        let resolver_id = context_menu_target.get_resolver_id();
        let mut open = true;
        let mut pending_action = None;

        ContextMenu::new(
            self.app_context.clone(),
            "symbol_resolver_list_context_menu",
            context_menu_target.get_position(),
            |user_interface, should_close| {
                if user_interface
                    .add(
                        ToolbarMenuItemView::new(
                            self.app_context.clone(),
                            "Rename",
                            "symbol_resolver_ctx_rename",
                            &None,
                            Self::RESOLVER_CONTEXT_MENU_WIDTH,
                        )
                        .icon(theme.icon_library.icon_handle_common_edit.clone()),
                    )
                    .clicked()
                {
                    pending_action = Some(SymbolResolverListAction::BeginRenameResolver(resolver_id.to_string()));
                    *should_close = true;
                }

                if user_interface
                    .add(
                        ToolbarMenuItemView::new(
                            self.app_context.clone(),
                            "Delete",
                            "symbol_resolver_ctx_delete",
                            &None,
                            Self::RESOLVER_CONTEXT_MENU_WIDTH,
                        )
                        .icon(theme.icon_library.icon_handle_common_delete.clone())
                        .icon_background(theme.background_control_danger, theme.background_control_danger_dark),
                    )
                    .clicked()
                {
                    pending_action = Some(SymbolResolverListAction::RequestDeleteConfirmation(resolver_id.to_string()));
                    *should_close = true;
                }
            },
        )
        .width(Self::RESOLVER_CONTEXT_MENU_WIDTH)
        .corner_radius(8)
        .show(user_interface, &mut open);

        if !open {
            if let Some(mut view_data) = self
                .symbol_resolver_editor_view_data
                .write("SymbolResolverEditor hide resolver context menu")
            {
                view_data.hide_resolver_context_menu();
            }
        }

        pending_action
    }

    fn render_header(
        &self,
        user_interface: &mut Ui,
    ) {
        let theme = &self.app_context.theme;
        let (header_rect, _) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), Self::ROW_HEIGHT), Sense::hover());

        user_interface
            .painter()
            .rect_filled(header_rect, CornerRadius::ZERO, theme.background_primary);
        user_interface.painter().text(
            pos2(header_rect.min.x + Self::ROW_LEFT_PADDING, header_rect.center().y),
            Align2::LEFT_CENTER,
            "Resolver",
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground_preview,
        );
        user_interface.painter().text(
            pos2(header_rect.max.x - Self::ICON_BUTTON_WIDTH - Self::ROW_LEFT_PADDING, header_rect.center().y),
            Align2::RIGHT_CENTER,
            "Uses",
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground_preview,
        );
    }

    fn render_list_entry(
        &self,
        user_interface: &mut Ui,
        resolver_id: &str,
        usage_count: usize,
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

        let edit_button_rect = Rect::from_min_size(
            pos2(allocated_size_rectangle.max.x - Self::ICON_BUTTON_WIDTH, allocated_size_rectangle.min.y),
            vec2(Self::ICON_BUTTON_WIDTH, Self::ROW_HEIGHT),
        );
        let edit_response = user_interface.put(
            edit_button_rect,
            ThemeButton::new_from_theme(theme)
                .background_color(Color32::TRANSPARENT)
                .with_tooltip_text("Edit resolver."),
        );
        IconDraw::draw(user_interface, edit_response.rect, &theme.icon_library.icon_handle_common_edit);

        let usage_text = Self::resolver_usage_preview_text(usage_count);
        let usage_text_width = measure_text_width(
            user_interface,
            &usage_text,
            &theme.font_library.font_noto_sans.font_normal,
            if is_selected { theme.foreground } else { theme.foreground_preview },
        );
        let usage_text_right = edit_button_rect.min.x - Self::ROW_LEFT_PADDING;
        let usage_text_left = (usage_text_right - usage_text_width).max(allocated_size_rectangle.min.x + Self::ROW_LEFT_PADDING);
        user_interface.painter().text(
            pos2(usage_text_right, allocated_size_rectangle.center().y),
            Align2::RIGHT_CENTER,
            usage_text,
            theme.font_library.font_noto_sans.font_normal.clone(),
            if is_selected { theme.foreground } else { theme.foreground_preview },
        );

        let label_position = pos2(allocated_size_rectangle.min.x + Self::ROW_LEFT_PADDING, allocated_size_rectangle.center().y);
        let label_min_x = allocated_size_rectangle.min.x + Self::ROW_LEFT_PADDING;
        let label_max_x = (usage_text_left - Self::ROW_LABEL_SPACING).max(label_min_x);
        let label_clip_rect = Rect::from_min_max(
            pos2(label_min_x, allocated_size_rectangle.min.y),
            pos2(label_max_x, allocated_size_rectangle.max.y),
        );
        user_interface.painter().with_clip_rect(label_clip_rect).text(
            label_position,
            Align2::LEFT_CENTER,
            resolver_id,
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground,
        );

        (row_response, edit_response)
    }

    fn resolver_usage_preview_text(usage_count: usize) -> String {
        if usage_count == 1 {
            return String::from("1 use");
        }

        format!("{} uses", usage_count)
    }
}
