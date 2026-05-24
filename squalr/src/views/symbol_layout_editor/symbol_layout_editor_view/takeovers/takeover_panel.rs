use super::super::SymbolLayoutEditorView;
use eframe::egui::{Align, Align2, Button as EguiButton, Layout, Response, RichText, ScrollArea, Sense, Stroke, Ui, UiBuilder, pos2, vec2};
use epaint::CornerRadius;

impl SymbolLayoutEditorView {
    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn render_take_over_action_buttons(
        &self,
        user_interface: &mut Ui,
        accept_label: &str,
        can_accept: bool,
    ) -> (Response, Response) {
        let theme = &self.app_context.theme;
        let button_size = vec2(Self::TAKE_OVER_ACTION_BUTTON_WIDTH, Self::FIELD_ROW_HEIGHT);
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

    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn render_delete_take_over_action_buttons(
        &self,
        user_interface: &mut Ui,
    ) -> (Response, Response) {
        self.render_danger_take_over_action_buttons(user_interface, "Delete")
    }

    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn render_danger_take_over_action_buttons(
        &self,
        user_interface: &mut Ui,
        danger_label: &str,
    ) -> (Response, Response) {
        let theme = &self.app_context.theme;
        let button_size = vec2(Self::TAKE_OVER_ACTION_BUTTON_WIDTH, Self::FIELD_ROW_HEIGHT);
        let total_button_width = button_size.x * 2.0 + Self::TAKE_OVER_ACTION_BUTTON_SPACING;
        let side_spacing = ((user_interface.available_width() - total_button_width) * 0.5).max(0.0);

        let responses = user_interface
            .horizontal(|user_interface| {
                user_interface.add_space(side_spacing);
                user_interface.spacing_mut().item_spacing.x = Self::TAKE_OVER_ACTION_BUTTON_SPACING;

                let delete_response = user_interface.add_sized(
                    button_size,
                    EguiButton::new(RichText::new(danger_label).color(theme.foreground))
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

    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn render_take_over_panel(
        &self,
        user_interface: &mut Ui,
        title: &str,
        header_action_width: f32,
        content_padding_x: f32,
        body_top_spacing: f32,
        render_header_actions: impl FnOnce(&mut Ui),
        add_contents: impl FnOnce(&mut Ui),
    ) {
        let theme = &self.app_context.theme;
        let (panel_rect, _) = user_interface.allocate_exact_size(user_interface.available_size(), Sense::hover());
        user_interface
            .painter()
            .rect_filled(panel_rect, CornerRadius::ZERO, theme.background_panel);

        let inner_rect = panel_rect.shrink2(vec2(Self::TAKE_OVER_PADDING_X, Self::TAKE_OVER_PADDING_Y));
        let mut panel_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(inner_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        panel_user_interface.set_clip_rect(inner_rect);

        if !title.is_empty() || header_action_width > 0.0 {
            let (header_rect, _) = panel_user_interface.allocate_exact_size(
                vec2(panel_user_interface.available_width().max(1.0), Self::TAKE_OVER_HEADER_HEIGHT),
                Sense::hover(),
            );
            panel_user_interface
                .painter()
                .rect_filled(header_rect, CornerRadius::ZERO, theme.background_primary);
            let header_inner_rect = header_rect;
            let mut header_user_interface = panel_user_interface.new_child(
                UiBuilder::new()
                    .max_rect(header_inner_rect)
                    .layout(Layout::left_to_right(Align::Center)),
            );
            header_user_interface.set_clip_rect(header_inner_rect);

            if header_action_width > 0.0 {
                header_user_interface.allocate_ui_with_layout(
                    vec2(header_action_width, Self::TAKE_OVER_HEADER_HEIGHT),
                    Layout::left_to_right(Align::Center),
                    |user_interface| {
                        render_header_actions(user_interface);
                    },
                );
            }

            let title_width = (header_user_interface.available_width() - Self::TAKE_OVER_HEADER_TITLE_PADDING_X).max(0.0);
            let (title_rect, _) = header_user_interface.allocate_exact_size(vec2(title_width, Self::TAKE_OVER_HEADER_HEIGHT), Sense::hover());
            header_user_interface.painter().text(
                pos2(title_rect.left() + Self::TAKE_OVER_HEADER_TITLE_PADDING_X, title_rect.center().y),
                Align2::LEFT_CENTER,
                title,
                theme.font_library.font_noto_sans.font_window_title.clone(),
                theme.foreground,
            );
        }

        if body_top_spacing > 0.0 {
            panel_user_interface.add_space(body_top_spacing);
        }
        ScrollArea::vertical()
            .id_salt(format!("symbol_layout_editor_take_over_body_{title}"))
            .auto_shrink([false, false])
            .show(&mut panel_user_interface, |user_interface| {
                let content_width = (user_interface.available_width() - content_padding_x * 2.0).max(0.0);
                user_interface.horizontal(|user_interface| {
                    user_interface.add_space(content_padding_x);
                    user_interface.allocate_ui_with_layout(vec2(content_width, 0.0), Layout::top_down(Align::Min), |user_interface| {
                        add_contents(user_interface);
                    });
                });
            });
    }
}
