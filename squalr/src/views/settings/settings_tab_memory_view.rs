use crate::{
    app_context::AppContext,
    ui::widgets::controls::{checkbox::Checkbox, groupbox::GroupBox},
};
use eframe::egui::{Align, Layout, Response, RichText, Ui, Widget};
use squalr_engine_api::{
    commands::{
        privileged_command_request::PrivilegedCommandRequest,
        settings::memory::{list::memory_settings_list_request::MemorySettingsListRequest, set::memory_settings_set_request::MemorySettingsSetRequest},
    },
    structures::settings::memory_settings::MemorySettings,
};
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct SettingsTabMemoryView {
    app_context: Arc<AppContext>,
    cached_memory_settings: Arc<RwLock<MemorySettings>>,
}

impl SettingsTabMemoryView {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let settings_view = Self {
            app_context,
            cached_memory_settings: Arc::new(RwLock::new(MemorySettings::default())),
        };

        settings_view.sync_ui_with_memory_settings();

        settings_view
    }

    fn sync_ui_with_memory_settings(&self) {
        let memory_settings_list_request = MemorySettingsListRequest {};
        let cached_memory_settings = self.cached_memory_settings.clone();

        memory_settings_list_request.send(&self.app_context.engine_unprivileged_state, move |scan_results_query_response| {
            if let Ok(memory_settings) = scan_results_query_response.memory_settings {
                if let Ok(mut cached_memory_settings) = cached_memory_settings.write() {
                    *cached_memory_settings = memory_settings;
                }
            }
        });
    }
}

impl Widget for SettingsTabMemoryView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let cached_memory_settings = match self.cached_memory_settings.read() {
            Ok(cached_memory_settings) => *cached_memory_settings,
            Err(_error) => MemorySettings::default(),
        };

        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                user_interface.add_space(4.0);
                user_interface.horizontal(|user_interface| {
                    user_interface.add(
                        GroupBox::new_from_theme(theme, "Required Protection Flags", |user_interface| {
                            user_interface.vertical(|user_interface| {
                                user_interface.horizontal(|user_interface| {
                                    if user_interface
                                        .add(Checkbox::new_from_theme(theme).with_check_state_bool(cached_memory_settings.required_write))
                                        .clicked()
                                    {
                                        if let Ok(mut cached_memory_settings) = self.cached_memory_settings.write() {
                                            cached_memory_settings.required_write = !cached_memory_settings.required_write;
                                        }

                                        let memory_settings_set_request = MemorySettingsSetRequest {
                                            required_write: Some(cached_memory_settings.required_write),
                                            ..MemorySettingsSetRequest::default()
                                        };

                                        memory_settings_set_request.send(&self.app_context.engine_unprivileged_state, move |memory_settings_set_response| {});
                                    }

                                    user_interface.add_space(8.0);
                                    user_interface.label(
                                        RichText::new("Write")
                                            .font(theme.font_library.font_noto_sans.font_normal.clone())
                                            .color(theme.foreground),
                                    );
                                });
                                user_interface.add_space(4.0);
                                user_interface.horizontal(|user_interface| {
                                    if user_interface
                                        .add(Checkbox::new_from_theme(theme).with_check_state_bool(cached_memory_settings.required_execute))
                                        .clicked()
                                    {
                                        if let Ok(mut cached_memory_settings) = self.cached_memory_settings.write() {
                                            cached_memory_settings.required_execute = !cached_memory_settings.required_execute;
                                        }

                                        let memory_settings_set_request = MemorySettingsSetRequest {
                                            required_execute: Some(cached_memory_settings.required_execute),
                                            ..MemorySettingsSetRequest::default()
                                        };

                                        memory_settings_set_request.send(&self.app_context.engine_unprivileged_state, move |memory_settings_set_response| {});
                                    }

                                    user_interface.add_space(8.0);
                                    user_interface.label(
                                        RichText::new("Execute")
                                            .font(theme.font_library.font_noto_sans.font_normal.clone())
                                            .color(theme.foreground),
                                    );
                                });
                                user_interface.add_space(4.0);
                                user_interface.horizontal(|user_interface| {
                                    if user_interface
                                        .add(Checkbox::new_from_theme(theme).with_check_state_bool(cached_memory_settings.required_copy_on_write))
                                        .clicked()
                                    {
                                        if let Ok(mut cached_memory_settings) = self.cached_memory_settings.write() {
                                            cached_memory_settings.required_copy_on_write = !cached_memory_settings.required_copy_on_write;
                                        }

                                        let memory_settings_set_request = MemorySettingsSetRequest {
                                            required_copy_on_write: Some(cached_memory_settings.required_copy_on_write),
                                            ..MemorySettingsSetRequest::default()
                                        };

                                        memory_settings_set_request.send(&self.app_context.engine_unprivileged_state, move |memory_settings_set_response| {});
                                    }

                                    user_interface.add_space(8.0);
                                    user_interface.label(
                                        RichText::new("Copy on Write")
                                            .font(theme.font_library.font_noto_sans.font_normal.clone())
                                            .color(theme.foreground),
                                    );
                                });
                            });
                        })
                        .desired_width(224.0),
                    );
                    user_interface.add_space(8.0);
                    user_interface.add(
                        GroupBox::new_from_theme(theme, "Excluded Protection Flags", |user_interface| {
                            user_interface.vertical(|user_interface| {
                                user_interface.horizontal(|user_interface| {
                                    if user_interface
                                        .add(Checkbox::new_from_theme(theme).with_check_state_bool(cached_memory_settings.excluded_write))
                                        .clicked()
                                    {
                                        if let Ok(mut cached_memory_settings) = self.cached_memory_settings.write() {
                                            cached_memory_settings.excluded_write = !cached_memory_settings.excluded_write;
                                        }

                                        let memory_settings_set_request = MemorySettingsSetRequest {
                                            excluded_write: Some(cached_memory_settings.excluded_write),
                                            ..MemorySettingsSetRequest::default()
                                        };

                                        memory_settings_set_request.send(&self.app_context.engine_unprivileged_state, move |memory_settings_set_response| {});
                                    }

                                    user_interface.add_space(8.0);
                                    user_interface.label(
                                        RichText::new("Write")
                                            .font(theme.font_library.font_noto_sans.font_normal.clone())
                                            .color(theme.foreground),
                                    );
                                });
                                user_interface.add_space(4.0);
                                user_interface.horizontal(|user_interface| {
                                    if user_interface
                                        .add(Checkbox::new_from_theme(theme).with_check_state_bool(cached_memory_settings.excluded_execute))
                                        .clicked()
                                    {
                                        if let Ok(mut cached_memory_settings) = self.cached_memory_settings.write() {
                                            cached_memory_settings.excluded_execute = !cached_memory_settings.excluded_execute;
                                        }

                                        let memory_settings_set_request = MemorySettingsSetRequest {
                                            excluded_execute: Some(cached_memory_settings.excluded_execute),
                                            ..MemorySettingsSetRequest::default()
                                        };

                                        memory_settings_set_request.send(&self.app_context.engine_unprivileged_state, move |memory_settings_set_response| {});
                                    }

                                    user_interface.add_space(8.0);
                                    user_interface.label(
                                        RichText::new("Execute")
                                            .font(theme.font_library.font_noto_sans.font_normal.clone())
                                            .color(theme.foreground),
                                    );
                                });
                                user_interface.add_space(4.0);
                                user_interface.horizontal(|user_interface| {
                                    if user_interface
                                        .add(Checkbox::new_from_theme(theme).with_check_state_bool(cached_memory_settings.excluded_copy_on_write))
                                        .clicked()
                                    {
                                        if let Ok(mut cached_memory_settings) = self.cached_memory_settings.write() {
                                            cached_memory_settings.excluded_copy_on_write = !cached_memory_settings.excluded_copy_on_write;
                                        }

                                        let memory_settings_set_request = MemorySettingsSetRequest {
                                            excluded_copy_on_write: Some(cached_memory_settings.excluded_copy_on_write),
                                            ..MemorySettingsSetRequest::default()
                                        };

                                        memory_settings_set_request.send(&self.app_context.engine_unprivileged_state, move |memory_settings_set_response| {});
                                    }

                                    user_interface.add_space(8.0);
                                    user_interface.label(
                                        RichText::new("Copy on Write")
                                            .font(theme.font_library.font_noto_sans.font_normal.clone())
                                            .color(theme.foreground),
                                    );
                                });
                            });
                        })
                        .desired_width(256.0),
                    );
                });

                user_interface.horizontal(|user_interface| {
                    user_interface.add(
                        GroupBox::new_from_theme(theme, "Memory Types", |user_interface| {
                            user_interface.add_space(4.0);
                            user_interface.vertical(|user_interface| {
                                user_interface.horizontal(|user_interface| {
                                    if user_interface
                                        .add(Checkbox::new_from_theme(theme).with_check_state_bool(cached_memory_settings.memory_type_none))
                                        .clicked()
                                    {
                                        if let Ok(mut cached_memory_settings) = self.cached_memory_settings.write() {
                                            cached_memory_settings.memory_type_none = !cached_memory_settings.memory_type_none;
                                        }

                                        let memory_settings_set_request = MemorySettingsSetRequest {
                                            memory_type_none: Some(cached_memory_settings.memory_type_none),
                                            ..MemorySettingsSetRequest::default()
                                        };

                                        memory_settings_set_request.send(&self.app_context.engine_unprivileged_state, move |memory_settings_set_response| {});
                                    }

                                    user_interface.add_space(8.0);
                                    user_interface.label(
                                        RichText::new("None")
                                            .font(theme.font_library.font_noto_sans.font_normal.clone())
                                            .color(theme.foreground),
                                    );
                                });
                                user_interface.add_space(4.0);
                                user_interface.horizontal(|user_interface| {
                                    if user_interface
                                        .add(Checkbox::new_from_theme(theme).with_check_state_bool(cached_memory_settings.memory_type_image))
                                        .clicked()
                                    {
                                        if let Ok(mut cached_memory_settings) = self.cached_memory_settings.write() {
                                            cached_memory_settings.memory_type_image = !cached_memory_settings.memory_type_image;
                                        }

                                        let memory_settings_set_request = MemorySettingsSetRequest {
                                            memory_type_image: Some(cached_memory_settings.memory_type_image),
                                            ..MemorySettingsSetRequest::default()
                                        };

                                        memory_settings_set_request.send(&self.app_context.engine_unprivileged_state, move |memory_settings_set_response| {});
                                    }

                                    user_interface.add_space(8.0);
                                    user_interface.label(
                                        RichText::new("Image")
                                            .font(theme.font_library.font_noto_sans.font_normal.clone())
                                            .color(theme.foreground),
                                    );
                                });
                                user_interface.add_space(4.0);
                                user_interface.horizontal(|user_interface| {
                                    if user_interface
                                        .add(Checkbox::new_from_theme(theme).with_check_state_bool(cached_memory_settings.memory_type_private))
                                        .clicked()
                                    {
                                        if let Ok(mut cached_memory_settings) = self.cached_memory_settings.write() {
                                            cached_memory_settings.memory_type_private = !cached_memory_settings.memory_type_private;
                                        }

                                        let memory_settings_set_request = MemorySettingsSetRequest {
                                            memory_type_private: Some(cached_memory_settings.memory_type_private),
                                            ..MemorySettingsSetRequest::default()
                                        };

                                        memory_settings_set_request.send(&self.app_context.engine_unprivileged_state, move |memory_settings_set_response| {});
                                    }

                                    user_interface.add_space(8.0);
                                    user_interface.label(
                                        RichText::new("Private")
                                            .font(theme.font_library.font_noto_sans.font_normal.clone())
                                            .color(theme.foreground),
                                    );
                                });
                                user_interface.add_space(4.0);
                                user_interface.horizontal(|user_interface| {
                                    if user_interface
                                        .add(Checkbox::new_from_theme(theme).with_check_state_bool(cached_memory_settings.memory_type_mapped))
                                        .clicked()
                                    {
                                        if let Ok(mut cached_memory_settings) = self.cached_memory_settings.write() {
                                            cached_memory_settings.memory_type_mapped = !cached_memory_settings.memory_type_mapped;
                                        }

                                        let memory_settings_set_request = MemorySettingsSetRequest {
                                            memory_type_mapped: Some(cached_memory_settings.memory_type_mapped),
                                            ..MemorySettingsSetRequest::default()
                                        };

                                        memory_settings_set_request.send(&self.app_context.engine_unprivileged_state, move |memory_settings_set_response| {});
                                    }

                                    user_interface.add_space(8.0);
                                    user_interface.label(
                                        RichText::new("Mapped")
                                            .font(theme.font_library.font_noto_sans.font_normal.clone())
                                            .color(theme.foreground),
                                    );
                                });
                            });
                        })
                        .desired_width(224.0)
                        // JIRA: Bugged. I believe these rows are not allocating sufficient available height, and then groupbox treats desired as a suggestion.
                        .desired_height(320.0),
                    );
                    user_interface.add_space(8.0);
                    user_interface.add(
                        GroupBox::new_from_theme(theme, "Virtual Memory Querying", |user_interface| {
                            user_interface.vertical(|user_interface| {
                                user_interface.horizontal(|user_interface| {
                                    if user_interface
                                        .add(Checkbox::new_from_theme(theme).with_check_state_bool(cached_memory_settings.only_query_usermode))
                                        .clicked()
                                    {
                                        if let Ok(mut cached_memory_settings) = self.cached_memory_settings.write() {
                                            cached_memory_settings.only_query_usermode = !cached_memory_settings.only_query_usermode;
                                        }

                                        let memory_settings_set_request = MemorySettingsSetRequest {
                                            only_query_usermode: Some(cached_memory_settings.only_query_usermode),
                                            ..MemorySettingsSetRequest::default()
                                        };

                                        memory_settings_set_request.send(&self.app_context.engine_unprivileged_state, move |memory_settings_set_response| {});
                                    }

                                    user_interface.add_space(8.0);
                                    user_interface.label(
                                        RichText::new("Query All memory")
                                            .font(theme.font_library.font_noto_sans.font_normal.clone())
                                            .color(theme.foreground),
                                    );
                                });
                                user_interface.add_space(4.0);
                                user_interface.horizontal(|user_interface| {
                                    if user_interface
                                        .add(Checkbox::new_from_theme(theme).with_check_state_bool(cached_memory_settings.only_query_usermode))
                                        .clicked()
                                    {
                                        if let Ok(mut cached_memory_settings) = self.cached_memory_settings.write() {
                                            cached_memory_settings.only_query_usermode = !cached_memory_settings.only_query_usermode;
                                        }

                                        let memory_settings_set_request = MemorySettingsSetRequest {
                                            only_query_usermode: Some(cached_memory_settings.only_query_usermode),
                                            ..MemorySettingsSetRequest::default()
                                        };

                                        memory_settings_set_request.send(&self.app_context.engine_unprivileged_state, move |memory_settings_set_response| {});
                                    }

                                    user_interface.add_space(8.0);
                                    user_interface.label(
                                        RichText::new("Query All Usermode Memory")
                                            .font(theme.font_library.font_noto_sans.font_normal.clone())
                                            .color(theme.foreground),
                                    );
                                });
                                user_interface.add_space(4.0);
                                user_interface.horizontal(|user_interface| {
                                    if user_interface
                                        .add(Checkbox::new_from_theme(theme).with_check_state_bool(cached_memory_settings.only_query_usermode))
                                        .clicked()
                                    {
                                        // JIRA: Implement me
                                    }

                                    user_interface.add_space(8.0);
                                    user_interface.label(
                                        RichText::new("Query Custom Range")
                                            .font(theme.font_library.font_noto_sans.font_normal.clone())
                                            .color(theme.foreground),
                                    );
                                });
                            });
                        })
                        .desired_width(256.0)
                        // JIRA: Bugged. I believe these rows are not allocating sufficient available height, and then groupbox treats desired as a suggestion.
                        .desired_height(320.0),
                    );
                });
            })
            .response;

        response
    }
}
