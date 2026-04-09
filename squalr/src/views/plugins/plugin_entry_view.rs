use crate::{
    app_context::AppContext,
    ui::{
        theme::Theme,
        widgets::controls::{checkbox::Checkbox, state_layer::StateLayer},
    },
};
use eframe::egui::{Label, RichText, Sense, Ui, UiBuilder, Widget, pos2, vec2};
use epaint::CornerRadius;
use squalr_engine_api::plugins::{PluginActivationState, PluginState};
use std::sync::Arc;

pub struct PluginEntryView<'lifetime> {
    app_context: Arc<AppContext>,
    plugin_state: &'lifetime PluginState,
    is_selected: bool,
}

pub struct PluginEntryViewResponse {
    pub should_select: bool,
    pub toggle_enabled: Option<bool>,
}

impl<'lifetime> PluginEntryView<'lifetime> {
    pub fn new(
        app_context: Arc<AppContext>,
        plugin_state: &'lifetime PluginState,
        is_selected: bool,
    ) -> Self {
        Self {
            app_context,
            plugin_state,
            is_selected,
        }
    }

    pub fn show(
        self,
        user_interface: &mut Ui,
    ) -> PluginEntryViewResponse {
        let theme = &self.app_context.theme;
        let row_height = 88.0;
        let (row_rect, row_response) = user_interface.allocate_exact_size(vec2(user_interface.available_width().max(1.0), row_height), Sense::click());

        if self.is_selected {
            user_interface
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
        .ui(user_interface);

        let mut did_toggle_enabled = false;
        let mut toggle_enabled = None;
        let status_text = Self::build_status_text(self.plugin_state);
        let status_color = Self::resolve_status_color(theme, self.plugin_state);

        let mut row_user_interface = user_interface.new_child(UiBuilder::new().max_rect(row_rect));
        row_user_interface.set_clip_rect(row_rect);

        row_user_interface.scope_builder(UiBuilder::new().max_rect(row_rect), |user_interface| {
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

        PluginEntryViewResponse {
            should_select: row_response.clicked() && !did_toggle_enabled,
            toggle_enabled,
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
