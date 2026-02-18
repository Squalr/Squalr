use crate::{
    app_context::AppContext,
    ui::widgets::controls::{groupbox::GroupBox, slider::Slider},
};
use eframe::egui::{Align, Layout, Response, RichText, Ui, Widget};
use epaint::vec2;
use squalr_engine_api::{
    commands::{
        privileged_command_request::PrivilegedCommandRequest,
        settings::general::{list::general_settings_list_request::GeneralSettingsListRequest, set::general_settings_set_request::GeneralSettingsSetRequest},
    },
    structures::settings::general_settings::GeneralSettings,
};
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct SettingsTabGeneralView {
    app_context: Arc<AppContext>,
    cached_general_settings: Arc<RwLock<GeneralSettings>>,
}

impl SettingsTabGeneralView {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let settings_view = Self {
            app_context,
            cached_general_settings: Arc::new(RwLock::new(GeneralSettings::default())),
        };

        settings_view.sync_ui_with_general_settings();

        settings_view
    }

    fn sync_ui_with_general_settings(&self) {
        let general_settings_list_request = GeneralSettingsListRequest {};
        let cached_general_settings = self.cached_general_settings.clone();

        general_settings_list_request.send(&self.app_context.engine_unprivileged_state, move |scan_results_query_response| {
            if let Ok(general_settings) = scan_results_query_response.general_settings {
                if let Ok(mut cached_general_settings) = cached_general_settings.write() {
                    *cached_general_settings = general_settings;
                }
            }
        });
    }
}

impl Widget for SettingsTabGeneralView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let cached_general_settings = match self.cached_general_settings.read() {
            Ok(cached_general_settings) => *cached_general_settings,
            Err(_error) => GeneralSettings::default(),
        };

        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                user_interface.add_space(4.0);
                user_interface.add(
                    GroupBox::new_from_theme(theme, "Developer Debugging", |user_interface| {
                        user_interface.horizontal(|user_interface| {
                            let mut value: i64 = cached_general_settings.debug_engine_request_delay_ms as i64;
                            let slider = Slider::new_from_theme(theme)
                                .current_value(&mut value)
                                .minimum_value(0)
                                .maximum_value(5000);

                            if user_interface.add(slider).changed() {
                                if let Ok(mut cached_general_settings) = self.cached_general_settings.write() {
                                    cached_general_settings.debug_engine_request_delay_ms = value as u64;
                                }

                                let general_settings_set_request = GeneralSettingsSetRequest {
                                    engine_request_delay: Some(cached_general_settings.debug_engine_request_delay_ms),
                                    ..GeneralSettingsSetRequest::default()
                                };

                                general_settings_set_request.send(&self.app_context.engine_unprivileged_state, move |general_settings_set_response| {});
                            }

                            user_interface.add_space(8.0);
                            user_interface.allocate_ui_with_layout(
                                vec2(32.0, user_interface.available_height()),
                                Layout::right_to_left(Align::Center),
                                |user_interface| {
                                    user_interface.label(
                                        RichText::new(value.to_string())
                                            .font(theme.font_library.font_noto_sans.font_normal.clone())
                                            .color(theme.foreground),
                                    );
                                },
                            );

                            user_interface.add_space(8.0);
                            user_interface.label(
                                RichText::new("Engine Request Delay")
                                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                                    .color(theme.foreground),
                            );
                        });
                    })
                    .desired_width(412.0),
                );
            })
            .response;

        response
    }
}
