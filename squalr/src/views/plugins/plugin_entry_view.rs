use crate::{
    app_context::AppContext,
    ui::{
        theme::Theme,
        widgets::controls::{checkbox::Checkbox, icon_button::IconButtonView, state_layer::StateLayer},
    },
};
use eframe::egui::{Label, Pos2, Rect, RichText, Sense, Ui, UiBuilder, Widget, pos2, vec2};
use epaint::CornerRadius;
use squalr_engine_api::plugins::{PluginActivationState, PluginState};
use std::sync::Arc;

pub struct PluginEntryView<'lifetime> {
    app_context: Arc<AppContext>,
    plugin_state: &'lifetime PluginState,
    is_selected: bool,
    can_increase_priority: bool,
    can_decrease_priority: bool,
}

pub struct PluginEntryViewResponse {
    pub should_select: bool,
    pub toggle_enabled: Option<bool>,
    pub should_increase_priority: bool,
    pub should_decrease_priority: bool,
    pub show_context_menu_at: Option<Pos2>,
}

impl<'lifetime> PluginEntryView<'lifetime> {
    pub fn new(
        app_context: Arc<AppContext>,
        plugin_state: &'lifetime PluginState,
        is_selected: bool,
        can_increase_priority: bool,
        can_decrease_priority: bool,
    ) -> Self {
        Self {
            app_context,
            plugin_state,
            is_selected,
            can_increase_priority,
            can_decrease_priority,
        }
    }

    pub fn show(
        self,
        user_interface: &mut Ui,
    ) -> PluginEntryViewResponse {
        let theme = &self.app_context.theme;
        let row_height = Self::ROW_HEIGHT;
        let (row_rect, row_response) = user_interface.allocate_exact_size(vec2(user_interface.available_width().max(1.0), row_height), Sense::click());
        let row_clip_rect = row_rect.intersect(user_interface.clip_rect());
        let mut row_user_interface = user_interface.new_child(UiBuilder::new().max_rect(row_rect));
        row_user_interface.set_clip_rect(row_clip_rect);

        if self.is_selected {
            row_user_interface
                .painter()
                .rect_filled(row_rect, CornerRadius::ZERO, theme.selected_background);
        }

        StateLayer {
            bounds_min: row_rect.min,
            bounds_max: row_rect.max,
            enabled: true,
            pressed: row_response.is_pointer_button_down_on(),
            has_hover: row_response.hovered(),
            has_focus: row_response.has_focus(),
            corner_radius: CornerRadius::ZERO,
            border_width: 1.0,
            hover_color: theme.hover_tint,
            pressed_color: theme.pressed_tint,
            border_color: if self.is_selected {
                theme.selected_border
            } else {
                theme.background_control_secondary
            },
            border_color_focused: if self.is_selected {
                theme.selected_border
            } else {
                theme.background_control_secondary
            },
        }
        .ui(&mut row_user_interface);

        let mut did_toggle_enabled = false;
        let mut toggle_enabled = None;
        let mut should_increase_priority = false;
        let mut should_decrease_priority = false;
        let status_text = Self::build_status_text(self.plugin_state);
        let status_color = Self::resolve_status_color(theme, self.plugin_state);
        let priority_button_area_width = Self::PRIORITY_BUTTON_WIDTH * 2.0;
        let priority_button_area_right = row_rect.max.x - Self::PRIORITY_BUTTON_RIGHT_PADDING;
        let priority_button_area_left = (priority_button_area_right - priority_button_area_width).max(row_rect.min.x);
        let content_right = (priority_button_area_left - Self::PRIORITY_BUTTON_LEFT_GAP).max(row_rect.min.x);
        let content_rect = Rect::from_min_max(row_rect.min, pos2(content_right, row_rect.max.y));
        let content_clip_rect = row_clip_rect.intersect(content_rect);

        row_user_interface.scope_builder(UiBuilder::new().max_rect(content_rect), |user_interface| {
            user_interface.set_clip_rect(content_clip_rect);
            user_interface.horizontal(|user_interface| {
                user_interface.add_space(8.0);

                let checkbox_response = user_interface.place(
                    eframe::egui::Rect::from_min_size(
                        pos2(row_rect.min.x + 8.0, row_rect.center().y - Checkbox::HEIGHT * 0.5),
                        vec2(Checkbox::WIDTH, Checkbox::HEIGHT),
                    ),
                    Checkbox::new_from_theme(theme)
                        .with_check_state_bool(self.plugin_state.get_is_enabled())
                        .with_tooltip_text("Enable plugin."),
                );

                if checkbox_response.clicked() {
                    did_toggle_enabled = true;
                    toggle_enabled = Some(!self.plugin_state.get_is_enabled());
                }

                user_interface.add_space(28.0);
                user_interface.vertical(|user_interface| {
                    user_interface.add_space(6.0);
                    user_interface.add(
                        Label::new(
                            RichText::new(self.plugin_state.get_metadata().get_display_name())
                                .font(theme.font_library.font_noto_sans.font_normal.clone())
                                .color(theme.foreground),
                        )
                        .selectable(false),
                    );
                    user_interface.add(
                        Label::new(
                            RichText::new(status_text)
                                .font(theme.font_library.font_noto_sans.font_small.clone())
                                .color(status_color),
                        )
                        .selectable(false),
                    );
                    user_interface.add_space(2.0);
                    user_interface.add(
                        Label::new(
                            RichText::new(self.plugin_state.get_metadata().get_description())
                                .font(theme.font_library.font_noto_sans.font_small.clone())
                                .color(theme.foreground_preview),
                        )
                        .wrap()
                        .selectable(false),
                    );
                });
            });
        });

        let mut priority_button_min_x = priority_button_area_left;
        let mut render_priority_button = |icon_handle: &epaint::TextureHandle, tooltip_text: &str, is_disabled: bool| {
            let button_rect = Rect::from_min_size(
                pos2(priority_button_min_x, row_rect.center().y - Self::PRIORITY_BUTTON_HEIGHT * 0.5),
                vec2(Self::PRIORITY_BUTTON_WIDTH, Self::PRIORITY_BUTTON_HEIGHT),
            );
            priority_button_min_x += Self::PRIORITY_BUTTON_WIDTH;

            row_user_interface.place(button_rect, IconButtonView::new(theme, icon_handle, tooltip_text).disabled(is_disabled))
        };
        let increase_priority_response = render_priority_button(
            &theme.icon_library.icon_handle_navigation_up_arrow_small,
            "Increase priority.",
            !self.can_increase_priority,
        );
        if increase_priority_response.clicked() {
            should_increase_priority = true;
        }
        let decrease_priority_response = render_priority_button(
            &theme.icon_library.icon_handle_navigation_down_arrow_small,
            "Decrease priority.",
            !self.can_decrease_priority,
        );
        if decrease_priority_response.clicked() {
            should_decrease_priority = true;
        }

        PluginEntryViewResponse {
            should_select: row_response.clicked() && !did_toggle_enabled && !should_increase_priority && !should_decrease_priority,
            toggle_enabled,
            should_increase_priority,
            should_decrease_priority,
            show_context_menu_at: if row_response.secondary_clicked() {
                row_response
                    .interact_pointer_pos()
                    .or_else(|| user_interface.ctx().pointer_latest_pos())
            } else {
                None
            },
        }
    }

    fn build_status_text(plugin_state: &PluginState) -> String {
        let mut status_parts = vec![Self::format_plugin_capabilities(plugin_state)];

        if plugin_state.get_metadata().get_is_built_in() {
            status_parts.push(String::from("Built in"));
        }

        if !plugin_state.get_is_enabled() {
            status_parts.push(String::from("Disabled"));
        } else {
            status_parts.push(match plugin_state.get_activation_state() {
                PluginActivationState::Idle => String::from("Idle"),
                PluginActivationState::Available => String::from("Available on current target"),
                PluginActivationState::Activating => String::from("Activating on current target"),
                PluginActivationState::Activated => String::from("Activated on current target"),
            });
        }

        status_parts.join(" • ")
    }

    fn format_plugin_capabilities(plugin_state: &PluginState) -> String {
        plugin_state
            .get_metadata()
            .get_plugin_capabilities()
            .iter()
            .map(|plugin_capability| plugin_capability.get_display_name())
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn resolve_status_color(
        theme: &Theme,
        plugin_state: &PluginState,
    ) -> eframe::egui::Color32 {
        match plugin_state.get_activation_state() {
            PluginActivationState::Activated => theme.hexadecimal_green,
            PluginActivationState::Activating => theme.binary_blue,
            PluginActivationState::Available => theme.binary_blue,
            PluginActivationState::Idle => theme.foreground_preview,
        }
    }
}

impl PluginEntryView<'_> {
    const PRIORITY_BUTTON_HEIGHT: f32 = 28.0;
    const PRIORITY_BUTTON_LEFT_GAP: f32 = 8.0;
    const PRIORITY_BUTTON_RIGHT_PADDING: f32 = 8.0;
    const PRIORITY_BUTTON_WIDTH: f32 = 32.0;
    const ROW_HEIGHT: f32 = 88.0;
}
