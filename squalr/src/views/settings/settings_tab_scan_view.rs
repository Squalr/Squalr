use crate::{
    app_context::AppContext,
    ui::widgets::controls::{checkbox::Checkbox, combo_box::combo_box_view::ComboBoxView, groupbox::GroupBox, slider::Slider},
};
use eframe::egui::{Align, Layout, Response, RichText, Ui, Widget};
use epaint::vec2;
use squalr_engine_api::{
    commands::{
        engine_command_request::EngineCommandRequest,
        settings::scan::{list::scan_settings_list_request::ScanSettingsListRequest, set::scan_settings_set_request::ScanSettingsSetRequest},
    },
    structures::settings::scan_settings::ScanSettings,
};
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct SettingsTabScanView {
    app_context: Arc<AppContext>,
    cached_scan_settings: Arc<RwLock<ScanSettings>>,
}

impl SettingsTabScanView {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let settings_view = Self {
            app_context,
            cached_scan_settings: Arc::new(RwLock::new(ScanSettings::default())),
        };

        settings_view.sync_ui_with_scan_settings();

        settings_view
    }

    fn sync_ui_with_scan_settings(&self) {
        let scan_settings_list_request = ScanSettingsListRequest {};
        let cached_scan_settings = self.cached_scan_settings.clone();

        scan_settings_list_request.send(&self.app_context.engine_execution_context, move |scan_results_query_response| {
            if let Ok(scan_settings) = scan_results_query_response.scan_settings {
                if let Ok(mut cached_scan_settings) = cached_scan_settings.write() {
                    *cached_scan_settings = scan_settings;
                }
            }
        });
    }
}

impl Widget for SettingsTabScanView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let cached_scan_settings = match self.cached_scan_settings.read() {
            Ok(cached_scan_settings) => *cached_scan_settings,
            Err(_error) => ScanSettings::default(),
        };

        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                user_interface.add_space(4.0);
                user_interface.add(
                    GroupBox::new_from_theme(theme, "Scan Results", |user_interface| {
                        user_interface.horizontal(|user_interface| {
                            let mut value: i64 = cached_scan_settings.results_page_size as i64;
                            let slider = Slider::new_from_theme(theme)
                                .current_value(&mut value)
                                .minimum_value(8)
                                .maximum_value(128);

                            if user_interface.add(slider).changed() {
                                if let Ok(mut cached_scan_settings) = self.cached_scan_settings.write() {
                                    cached_scan_settings.results_page_size = value as u32;
                                }

                                let scan_settings_set_request = ScanSettingsSetRequest {
                                    results_page_size: Some(cached_scan_settings.results_page_size),
                                    ..ScanSettingsSetRequest::default()
                                };

                                scan_settings_set_request.send(&self.app_context.engine_execution_context, move |scan_settings_set_response| {});
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
                                RichText::new("Results page size")
                                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                                    .color(theme.foreground),
                            );
                        });
                    })
                    .desired_width(412.0),
                );
                user_interface.add_space(4.0);
                user_interface.add(
                    GroupBox::new_from_theme(theme, "Memory Read Intervals", |user_interface| {
                        user_interface.horizontal(|user_interface| {
                            let mut value: i64 = cached_scan_settings.freeze_interval_ms as i64;
                            let slider = Slider::new_from_theme(theme)
                                .current_value(&mut value)
                                .minimum_value(0)
                                .maximum_value(2000);

                            if user_interface.add(slider).changed() {
                                if let Ok(mut cached_scan_settings) = self.cached_scan_settings.write() {
                                    cached_scan_settings.freeze_interval_ms = value as u64;
                                }

                                let scan_settings_set_request = ScanSettingsSetRequest {
                                    freeze_interval_ms: Some(cached_scan_settings.freeze_interval_ms),
                                    ..ScanSettingsSetRequest::default()
                                };

                                scan_settings_set_request.send(&self.app_context.engine_execution_context, move |scan_settings_set_response| {});
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
                                RichText::new("Freeze interval (ms)")
                                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                                    .color(theme.foreground),
                            );
                        });
                        user_interface.horizontal(|user_interface| {
                            let mut value: i64 = cached_scan_settings.project_read_interval_ms as i64;
                            let slider = Slider::new_from_theme(theme)
                                .current_value(&mut value)
                                .minimum_value(0)
                                .maximum_value(2000);

                            if user_interface.add(slider).changed() {
                                if let Ok(mut cached_scan_settings) = self.cached_scan_settings.write() {
                                    cached_scan_settings.project_read_interval_ms = value as u64;

                                    let scan_settings_set_request = ScanSettingsSetRequest {
                                        project_read_interval_ms: Some(cached_scan_settings.project_read_interval_ms),
                                        ..ScanSettingsSetRequest::default()
                                    };

                                    scan_settings_set_request.send(&self.app_context.engine_execution_context, move |scan_settings_set_response| {});
                                }
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
                                RichText::new("Project read interval (ms)")
                                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                                    .color(theme.foreground),
                            );
                        });
                        user_interface.horizontal(|user_interface| {
                            let mut value: i64 = cached_scan_settings.results_read_interval_ms as i64;
                            let slider = Slider::new_from_theme(theme)
                                .current_value(&mut value)
                                .minimum_value(0)
                                .maximum_value(2000);

                            if user_interface.add(slider).changed() {
                                if let Ok(mut cached_scan_settings) = self.cached_scan_settings.write() {
                                    cached_scan_settings.results_read_interval_ms = value as u64;

                                    let scan_settings_set_request = ScanSettingsSetRequest {
                                        results_read_interval_ms: Some(cached_scan_settings.results_read_interval_ms),
                                        ..ScanSettingsSetRequest::default()
                                    };

                                    scan_settings_set_request.send(&self.app_context.engine_execution_context, move |scan_settings_set_response| {});
                                }
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
                                RichText::new("Result read interval (ms)")
                                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                                    .color(theme.foreground),
                            );
                        });
                    })
                    .desired_width(412.0),
                );
                user_interface.add_space(4.0);
                user_interface.add(
                    GroupBox::new_from_theme(theme, "Scan Params", |user_interface| {
                        user_interface.horizontal(|user_interface| {
                            user_interface.add(ComboBoxView::new(
                                self.app_context.clone(),
                                "x-byte aligned",
                                None,
                                |user_interface: &mut Ui, should_close: &mut bool| {
                                    //
                                },
                            ));
                        });
                    })
                    .desired_width(412.0),
                );
                user_interface.add_space(4.0);
                user_interface.add(
                    GroupBox::new_from_theme(theme, "Scan Internals", |user_interface| {
                        user_interface.horizontal(|user_interface| {
                            if user_interface
                                .add(Checkbox::new_from_theme(theme).with_check_state_bool(cached_scan_settings.is_single_threaded_scan))
                                .clicked()
                            {
                                if let Ok(mut cached_scan_settings) = self.cached_scan_settings.write() {
                                    cached_scan_settings.is_single_threaded_scan = !cached_scan_settings.is_single_threaded_scan;

                                    let scan_settings_set_request = ScanSettingsSetRequest {
                                        is_single_threaded_scan: Some(cached_scan_settings.is_single_threaded_scan),
                                        ..ScanSettingsSetRequest::default()
                                    };

                                    scan_settings_set_request.send(&self.app_context.engine_execution_context, move |scan_settings_set_response| {});
                                }
                            }

                            user_interface.add_space(8.0);
                            user_interface.label(
                                RichText::new("Force single threaded scan")
                                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                                    .color(theme.foreground),
                            );
                        });

                        user_interface.add_space(8.0);
                        user_interface.horizontal(|user_interface| {
                            if user_interface
                                .add(Checkbox::new_from_theme(theme).with_check_state_bool(cached_scan_settings.debug_perform_validation_scan))
                                .clicked()
                            {
                                if let Ok(mut cached_scan_settings) = self.cached_scan_settings.write() {
                                    cached_scan_settings.debug_perform_validation_scan = !cached_scan_settings.debug_perform_validation_scan;

                                    let scan_settings_set_request = ScanSettingsSetRequest {
                                        debug_perform_validation_scan: Some(cached_scan_settings.debug_perform_validation_scan),
                                        ..ScanSettingsSetRequest::default()
                                    };

                                    scan_settings_set_request.send(&self.app_context.engine_execution_context, move |scan_settings_set_response| {});
                                }
                            }

                            user_interface.add_space(8.0);
                            user_interface.label(
                                RichText::new("Perform extra debug validation scan")
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
