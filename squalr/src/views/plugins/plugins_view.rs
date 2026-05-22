use crate::{
    app_context::AppContext,
    ui::{
        draw::icon_draw::IconDraw,
        widgets::controls::{
            button::Button,
            context_menu::context_menu::{ContextMenu, ContextMenuSizing},
            toolbar_menu::toolbar_menu_item_view::ToolbarMenuItemView,
        },
    },
    views::plugins::{
        plugin_entry_view::PluginEntryView,
        view_data::plugin_list_view_data::{PluginListViewData, PluginPriorityShiftDirection},
    },
};
use eframe::egui::{Align, Label, Layout, Pos2, Response, RichText, ScrollArea, Sense, Ui, UiBuilder, Widget, vec2};
use epaint::{Color32, CornerRadius, Rect, Stroke, pos2};
use squalr_engine_api::{
    dependency_injection::dependency::Dependency,
    events::{plugins::changed::plugins_changed_event::PluginsChangedEvent, process::changed::process_changed_event::ProcessChangedEvent},
    plugins::PluginActivationState,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct PluginsView {
    app_context: Arc<AppContext>,
    plugin_list_view_data: Dependency<PluginListViewData>,
}

impl PluginsView {
    pub const WINDOW_ID: &'static str = "window_plugins";
    const DETAILS_HEIGHT: f32 = 132.0;

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let plugin_list_view_data = app_context
            .dependency_container
            .get_dependency::<PluginListViewData>();
        let instance = Self {
            app_context,
            plugin_list_view_data,
        };

        PluginListViewData::observe_command_responses(instance.plugin_list_view_data.clone(), instance.app_context.clone());
        PluginListViewData::refresh(instance.plugin_list_view_data.clone(), instance.app_context.clone());
        instance.listen_for_process_change();
        instance.listen_for_plugins_changed();

        instance
    }

    fn listen_for_process_change(&self) {
        let app_context = self.app_context.clone();
        let plugin_list_view_data = self.plugin_list_view_data.clone();

        self.app_context
            .engine_unprivileged_state
            .listen_for_engine_event::<ProcessChangedEvent>(move |_process_changed_event| {
                PluginListViewData::refresh(plugin_list_view_data.clone(), app_context.clone());
            });
    }

    fn listen_for_plugins_changed(&self) {
        let app_context = self.app_context.clone();
        let plugin_list_view_data = self.plugin_list_view_data.clone();

        self.app_context
            .engine_unprivileged_state
            .listen_for_engine_event::<PluginsChangedEvent>(move |_plugins_changed_event| {
                PluginListViewData::refresh(plugin_list_view_data.clone(), app_context.clone());
            });
    }

    fn format_bool(value: bool) -> &'static str {
        if value { "Yes" } else { "No" }
    }

    fn format_plugin_capabilities(plugin_capabilities: &[squalr_engine_api::plugins::PluginCapability]) -> String {
        plugin_capabilities
            .iter()
            .map(|plugin_capability| plugin_capability.get_display_name())
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn format_activation_state(activation_state: PluginActivationState) -> &'static str {
        match activation_state {
            PluginActivationState::Idle => "Idle",
            PluginActivationState::Available => "Available",
            PluginActivationState::Activating => "Activating",
            PluginActivationState::Activated => "Activated",
        }
    }
}

impl Widget for PluginsView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = self.app_context.theme.clone();
        let plugin_list_view_data = match self.plugin_list_view_data.read("Plugins view") {
            Some(plugin_list_view_data) => plugin_list_view_data,
            None => return user_interface.response(),
        };

        let plugin_states = plugin_list_view_data.get_plugin_states().to_vec();
        let selected_plugin_id = plugin_list_view_data
            .get_selected_plugin_id()
            .map(str::to_string);
        let context_menu_state = plugin_list_view_data
            .get_context_menu_state()
            .map(|(plugin_id, position)| (plugin_id.to_string(), position));
        let is_loading = plugin_list_view_data.get_is_loading();
        drop(plugin_list_view_data);
        let has_opened_project = PluginListViewData::has_opened_project(self.app_context.clone());
        let selected_plugin_state = selected_plugin_id.as_deref().and_then(|selected_plugin_id| {
            plugin_states
                .iter()
                .find(|plugin_state| plugin_state.get_metadata().get_plugin_id() == selected_plugin_id)
        });

        let mut selected_plugin_id_new = None;
        let mut toggle_request = None;
        let mut context_menu_request = None;
        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                let toolbar_height = 28.0;
                let (toolbar_rect, _toolbar_response) =
                    user_interface.allocate_exact_size(vec2(user_interface.available_width().max(1.0), toolbar_height), Sense::empty());
                user_interface
                    .painter()
                    .rect_filled(toolbar_rect, CornerRadius::ZERO, theme.background_primary);

                let mut toolbar_user_interface = user_interface.new_child(
                    UiBuilder::new()
                        .max_rect(toolbar_rect)
                        .layout(Layout::left_to_right(Align::Center)),
                );
                toolbar_user_interface.with_layout(Layout::left_to_right(Align::Center), |user_interface| {
                    let button_refresh = user_interface.add_sized(
                        vec2(36.0, toolbar_height),
                        Button::new_from_theme(&theme)
                            .with_tooltip_text("Refresh plugin list.")
                            .background_color(Color32::TRANSPARENT),
                    );
                    IconDraw::draw(user_interface, button_refresh.rect, &theme.icon_library.icon_handle_navigation_refresh);

                    if button_refresh.clicked() {
                        PluginListViewData::refresh(self.plugin_list_view_data.clone(), self.app_context.clone());
                    }

                    if is_loading {
                        user_interface.add_space(4.0);
                        user_interface.spinner();
                    }

                    if !has_opened_project {
                        user_interface.add_space(8.0);
                        user_interface.add(
                            Label::new(
                                RichText::new("No project open — plugin changes will not be saved.")
                                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                                    .color(theme.background_control_warning),
                            )
                            .selectable(false),
                        );
                    }
                });

                let content_height = (user_interface.available_height() - Self::DETAILS_HEIGHT).max(0.0);
                let (content_rectangle, _content_response) =
                    user_interface.allocate_exact_size(vec2(user_interface.available_width().max(1.0), content_height), Sense::empty());
                let mut content_user_interface = user_interface.new_child(
                    UiBuilder::new()
                        .max_rect(content_rectangle)
                        .layout(Layout::top_down(Align::Min)),
                );
                content_user_interface.set_clip_rect(content_rectangle.intersect(user_interface.clip_rect()));
                content_user_interface.visuals_mut().clip_rect_margin = 0.0;

                ScrollArea::vertical()
                    .id_salt("plugins_list")
                    .auto_shrink([false, false])
                    .max_height(content_height)
                    .show(&mut content_user_interface, |user_interface| {
                        for plugin_state in &plugin_states {
                            let plugin_id = plugin_state.get_metadata().get_plugin_id().to_string();
                            let entry_response = PluginEntryView::new(
                                self.app_context.clone(),
                                plugin_state,
                                selected_plugin_id.as_deref() == Some(plugin_id.as_str()),
                            )
                            .show(user_interface);

                            if entry_response.should_select {
                                selected_plugin_id_new = Some(plugin_id.clone());
                            }

                            if let Some(is_enabled) = entry_response.toggle_enabled {
                                toggle_request = Some((plugin_id.clone(), is_enabled));
                            }

                            if let Some(context_menu_position) = entry_response.show_context_menu_at {
                                context_menu_request = Some((plugin_id.clone(), context_menu_position));
                            }
                        }
                    });

                let (details_rectangle, _details_response) =
                    user_interface.allocate_exact_size(vec2(user_interface.available_width().max(1.0), Self::DETAILS_HEIGHT), Sense::empty());

                user_interface
                    .painter()
                    .rect_filled(details_rectangle, CornerRadius::ZERO, theme.background_primary);
                user_interface.painter().line_segment(
                    [
                        details_rectangle.left_top(),
                        pos2(details_rectangle.right(), details_rectangle.top()),
                    ],
                    Stroke::new(1.0, theme.submenu_border),
                );

                let details_content_rectangle = Rect::from_min_max(details_rectangle.min + vec2(12.0, 10.0), details_rectangle.max - vec2(12.0, 10.0));
                let mut details_user_interface = user_interface.new_child(
                    UiBuilder::new()
                        .max_rect(details_content_rectangle)
                        .layout(Layout::top_down(Align::Min)),
                );
                details_user_interface.set_clip_rect(details_content_rectangle.intersect(user_interface.clip_rect()));

                if let Some(selected_plugin_state) = selected_plugin_state {
                    let plugin_metadata = selected_plugin_state.get_metadata();

                    details_user_interface.add(
                        Label::new(
                            RichText::new(plugin_metadata.get_display_name())
                                .font(theme.font_library.font_noto_sans.font_header.clone())
                                .color(theme.foreground),
                        )
                        .selectable(false),
                    );
                    details_user_interface.add(
                        Label::new(
                            RichText::new(plugin_metadata.get_description())
                                .font(theme.font_library.font_noto_sans.font_normal.clone())
                                .color(theme.foreground_preview),
                        )
                        .wrap()
                        .selectable(false),
                    );
                    details_user_interface.add_space(8.0);
                    details_user_interface.horizontal_wrapped(|user_interface| {
                        user_interface.add(
                            Label::new(
                                RichText::new(format!(
                                    "Capabilities: {}",
                                    Self::format_plugin_capabilities(plugin_metadata.get_plugin_capabilities())
                                ))
                                .font(theme.font_library.font_noto_sans.font_normal.clone())
                                .color(theme.foreground),
                            )
                            .selectable(false),
                        );
                        user_interface.add_space(12.0);
                        user_interface.add(
                            Label::new(
                                RichText::new(format!("Built in: {}", Self::format_bool(plugin_metadata.get_is_built_in())))
                                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                                    .color(theme.foreground),
                            )
                            .selectable(false),
                        );
                        user_interface.add_space(12.0);
                        user_interface.add(
                            Label::new(
                                RichText::new(format!("Enabled: {}", Self::format_bool(selected_plugin_state.get_is_enabled())))
                                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                                    .color(theme.foreground),
                            )
                            .selectable(false),
                        );
                        user_interface.add_space(12.0);
                        user_interface.add(
                            Label::new(
                                RichText::new(format!(
                                    "Available on current target: {}",
                                    Self::format_bool(selected_plugin_state.get_can_activate_for_current_process())
                                ))
                                .font(theme.font_library.font_noto_sans.font_normal.clone())
                                .color(theme.foreground),
                            )
                            .selectable(false),
                        );
                        user_interface.add_space(12.0);
                        user_interface.add(
                            Label::new(
                                RichText::new(format!(
                                    "Activation on current target: {}",
                                    Self::format_activation_state(selected_plugin_state.get_activation_state())
                                ))
                                .font(theme.font_library.font_noto_sans.font_normal.clone())
                                .color(theme.foreground),
                            )
                            .selectable(false),
                        );
                    });
                    details_user_interface.add_space(4.0);
                    details_user_interface.add(
                        Label::new(
                            RichText::new(format!("Plugin ID: {}", plugin_metadata.get_plugin_id()))
                                .font(theme.font_library.font_noto_sans.font_small.clone())
                                .color(theme.foreground_preview),
                        )
                        .selectable(false),
                    );
                } else {
                    details_user_interface.add(
                        Label::new(
                            RichText::new("Select a plugin to inspect its details.")
                                .font(theme.font_library.font_noto_sans.font_normal.clone())
                                .color(theme.foreground_preview),
                        )
                        .selectable(false),
                    );
                }
            })
            .response;

        let active_context_menu_state = if let Some((plugin_id, context_menu_position)) = context_menu_request {
            PluginListViewData::select_plugin(self.plugin_list_view_data.clone(), Some(plugin_id.clone()));
            PluginListViewData::show_context_menu(self.plugin_list_view_data.clone(), plugin_id.clone(), context_menu_position);
            Some((plugin_id, context_menu_position))
        } else {
            context_menu_state
        };

        if let Some((context_menu_plugin_id, context_menu_position)) = active_context_menu_state {
            self.show_plugin_context_menu(user_interface, &plugin_states, &context_menu_plugin_id, context_menu_position);
        }

        if let Some(selected_plugin_id_new) = selected_plugin_id_new {
            PluginListViewData::select_plugin(self.plugin_list_view_data.clone(), Some(selected_plugin_id_new));
        }

        if let Some((plugin_id, is_enabled)) = toggle_request {
            PluginListViewData::set_plugin_enabled(self.plugin_list_view_data.clone(), self.app_context.clone(), plugin_id, is_enabled);
        }

        response
    }
}

impl PluginsView {
    fn show_plugin_context_menu(
        &self,
        user_interface: &mut Ui,
        plugin_states: &[squalr_engine_api::plugins::PluginState],
        plugin_id: &str,
        context_menu_position: Pos2,
    ) {
        let Some(plugin_position) = plugin_states
            .iter()
            .position(|plugin_state| plugin_state.get_metadata().get_plugin_id() == plugin_id)
        else {
            PluginListViewData::hide_context_menu(self.plugin_list_view_data.clone());
            return;
        };
        let can_increase_priority = plugin_position > 0;
        let can_decrease_priority = plugin_position + 1 < plugin_states.len();
        let menu_width = ContextMenuSizing::width_for_labels(&self.app_context, user_interface, ["Increase priority", "Decrease priority"]);
        let mut is_menu_open = true;

        ContextMenu::new(
            self.app_context.clone(),
            "plugin_priority_context_menu",
            context_menu_position,
            |user_interface, should_close| {
                let theme = &self.app_context.theme;
                let no_check_state = None;
                let increase_priority_response = user_interface.add(
                    ToolbarMenuItemView::new(self.app_context.clone(), "Increase priority", "increase_priority", &no_check_state, menu_width)
                        .icon(theme.icon_library.icon_handle_navigation_up_arrow_small.clone())
                        .disabled(!can_increase_priority),
                );

                if increase_priority_response.clicked() {
                    PluginListViewData::shift_plugin_priority(
                        self.plugin_list_view_data.clone(),
                        self.app_context.clone(),
                        plugin_id.to_string(),
                        PluginPriorityShiftDirection::Increase,
                    );
                    *should_close = true;
                }

                let decrease_priority_response = user_interface.add(
                    ToolbarMenuItemView::new(self.app_context.clone(), "Decrease priority", "decrease_priority", &no_check_state, menu_width)
                        .icon(
                            theme
                                .icon_library
                                .icon_handle_navigation_down_arrow_small
                                .clone(),
                        )
                        .disabled(!can_decrease_priority),
                );

                if decrease_priority_response.clicked() {
                    PluginListViewData::shift_plugin_priority(
                        self.plugin_list_view_data.clone(),
                        self.app_context.clone(),
                        plugin_id.to_string(),
                        PluginPriorityShiftDirection::Decrease,
                    );
                    *should_close = true;
                }
            },
        )
        .width(menu_width)
        .show(user_interface, &mut is_menu_open);

        if !is_menu_open {
            PluginListViewData::hide_context_menu(self.plugin_list_view_data.clone());
        }
    }
}
