use crate::{
    app_context::AppContext,
    ui::widgets::controls::{checkbox::Checkbox, groupbox::GroupBox, slider::Slider},
};
use eframe::egui::{Align, Layout, Response, RichText, Ui, Widget};
use squalr_engine_api::{
    commands::{engine_command_request::EngineCommandRequest, settings::scan::list::scan_settings_list_request::ScanSettingsListRequest},
    structures::settings::scan_settings::ScanSettings,
};
use std::{
    rc::Rc,
    sync::{Arc, RwLock},
};

#[derive(Clone)]
pub struct SettingsTabScanView {
    app_context: Rc<AppContext>,
    cached_scan_settings: Arc<RwLock<ScanSettings>>,
}

impl SettingsTabScanView {
    pub fn new(app_context: Rc<AppContext>) -> Self {
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
                let mut groupbox_memory_read_intervals = GroupBox::new_from_theme(theme, "Memory Read Intervals", |user_interface| {
                    let slider = Slider::new_from_theme(theme);

                    user_interface.add(slider);
                });
                let mut groupbox_scan_params = GroupBox::new_from_theme(theme, "Scan Params", |user_interface| {
                    //
                });
                let mut groupbox_scan_internals = GroupBox::new_from_theme(theme, "Scan Internals", |user_interface| {
                    user_interface.horizontal(|user_interface| {
                        if user_interface
                            .add(Checkbox::new_from_theme(theme).checked(cached_scan_settings.is_single_threaded_scan))
                            .clicked()
                        {
                            if let Ok(mut cached_scan_settings) = self.cached_scan_settings.write() {
                                cached_scan_settings.is_single_threaded_scan = !cached_scan_settings.is_single_threaded_scan;
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
                            .add(Checkbox::new_from_theme(theme).checked(cached_scan_settings.debug_perform_validation_scan))
                            .clicked()
                        {
                            if let Ok(mut cached_scan_settings) = self.cached_scan_settings.write() {
                                cached_scan_settings.debug_perform_validation_scan = !cached_scan_settings.debug_perform_validation_scan;
                            }
                        }

                        user_interface.add_space(8.0);
                        user_interface.label(
                            RichText::new("Perform extra debug validation scan")
                                .font(theme.font_library.font_noto_sans.font_normal.clone())
                                .color(theme.foreground),
                        );
                    });
                });

                user_interface.add_space(4.0);
                user_interface.add(groupbox_memory_read_intervals);
                user_interface.add_space(4.0);
                user_interface.add(groupbox_scan_params);
                user_interface.add_space(4.0);
                user_interface.add(groupbox_scan_internals);
                user_interface.add_space(4.0);
            })
            .response;

        response
    }
}
