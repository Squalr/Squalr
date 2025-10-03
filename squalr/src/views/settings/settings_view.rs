use crate::{
    app_context::AppContext,
    models::tab_menu::tab_menu_data::TabMenuData,
    ui::widgets::controls::{checkbox::Checkbox, groupbox::GroupBox, tab_menu::tab_menu_view::TabMenuView},
};
use eframe::egui::{Align, Layout, Response, RichText, Ui, Widget};
use std::{
    rc::Rc,
    sync::atomic::{AtomicI32, Ordering},
};

#[derive(Clone)]
pub struct SettingsView {
    app_context: Rc<AppContext>,
    tab_menu_data: TabMenuData,
}

impl SettingsView {
    pub fn new(app_context: Rc<AppContext>) -> Self {
        let tab_menu_data = TabMenuData {
            headers: vec!["General".to_string(), "Memory".to_string(), "Scan".to_string()].into(),
            active_tab_index: Rc::new(AtomicI32::new(0)),
        };

        Self { app_context, tab_menu_data }
    }
}

impl Widget for SettingsView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                // Compose the menu bar over the painted available space rectangle.
                let tab_menu = TabMenuView::new(self.app_context.clone(), &self.tab_menu_data);

                user_interface.add(tab_menu);

                match self.tab_menu_data.active_tab_index.load(Ordering::Acquire) {
                    1 => {
                        // Memory settings.
                        user_interface.horizontal(|user_interface| {
                            let mut groupbox_required_protection_flags = GroupBox::new_from_theme(theme, "Required Protection Flags", |user_interface| {
                                user_interface.vertical(|user_interface| {
                                    user_interface.horizontal(|user_interface| {
                                        user_interface.add(Checkbox::new_from_theme(theme).checked(true));
                                        user_interface.add_space(8.0);
                                        user_interface.label(RichText::new("Write").font(theme.font_library.font_noto_sans.font_normal.clone()));
                                    });
                                    user_interface.add_space(4.0);
                                    user_interface.horizontal(|user_interface| {
                                        user_interface.add(Checkbox::new_from_theme(theme).checked(true));
                                        user_interface.add_space(8.0);
                                        user_interface.label(RichText::new("Execute").font(theme.font_library.font_noto_sans.font_normal.clone()));
                                    });
                                    user_interface.add_space(4.0);
                                    user_interface.horizontal(|user_interface| {
                                        user_interface.add(Checkbox::new_from_theme(theme).checked(true));
                                        user_interface.add_space(8.0);
                                        user_interface.label(RichText::new("Copy on Write").font(theme.font_library.font_noto_sans.font_normal.clone()));
                                    });
                                });
                            });

                            let mut groupbox_excluded_protection_flags = GroupBox::new_from_theme(theme, "Memory Types", |user_interface| {
                                user_interface.vertical(|user_interface| {
                                    user_interface.horizontal(|user_interface| {
                                        user_interface.add(Checkbox::new_from_theme(theme).checked(true));
                                        user_interface.add_space(8.0);
                                        user_interface.label(RichText::new("Write").font(theme.font_library.font_noto_sans.font_normal.clone()));
                                    });
                                    user_interface.add_space(4.0);
                                    user_interface.horizontal(|user_interface| {
                                        user_interface.add(Checkbox::new_from_theme(theme).checked(true));
                                        user_interface.add_space(8.0);
                                        user_interface.label(RichText::new("Execute").font(theme.font_library.font_noto_sans.font_normal.clone()));
                                    });
                                    user_interface.add_space(4.0);
                                    user_interface.horizontal(|user_interface| {
                                        user_interface.add(Checkbox::new_from_theme(theme).checked(true));
                                        user_interface.add_space(8.0);
                                        user_interface.label(RichText::new("Copy on Write").font(theme.font_library.font_noto_sans.font_normal.clone()));
                                    });
                                });
                            });

                            groupbox_required_protection_flags.desired_width = Some(192.0);
                            groupbox_required_protection_flags.desired_height = Some(192.0);
                            groupbox_excluded_protection_flags.desired_width = Some(192.0);
                            groupbox_excluded_protection_flags.desired_height = Some(192.0);

                            user_interface.add(groupbox_required_protection_flags);
                            user_interface.add(groupbox_excluded_protection_flags);
                        });

                        user_interface.horizontal(|user_interface| {
                            let mut groupbox_memory_types = GroupBox::new_from_theme(theme, "Memory Types", |user_interface| {
                                user_interface.vertical(|user_interface| {
                                    user_interface.horizontal(|user_interface| {
                                        user_interface.add(Checkbox::new_from_theme(theme).checked(true));
                                        user_interface.add_space(8.0);
                                        user_interface.label(RichText::new("Write").font(theme.font_library.font_noto_sans.font_normal.clone()));
                                    });
                                    user_interface.add_space(4.0);
                                    user_interface.horizontal(|user_interface| {
                                        user_interface.add(Checkbox::new_from_theme(theme).checked(true));
                                        user_interface.add_space(8.0);
                                        user_interface.label(RichText::new("Execute").font(theme.font_library.font_noto_sans.font_normal.clone()));
                                    });
                                    user_interface.add_space(4.0);
                                    user_interface.horizontal(|user_interface| {
                                        user_interface.add(Checkbox::new_from_theme(theme).checked(true));
                                        user_interface.add_space(8.0);
                                        user_interface.label(RichText::new("Copy on Write").font(theme.font_library.font_noto_sans.font_normal.clone()));
                                    });
                                });
                            });
                            let mut groupbox_memory_types2 = GroupBox::new_from_theme(theme, "Memory Types", |user_interface| {
                                user_interface.vertical(|user_interface| {
                                    user_interface.horizontal(|user_interface| {
                                        user_interface.add(Checkbox::new_from_theme(theme).checked(true));
                                        user_interface.add_space(8.0);
                                        user_interface.label(RichText::new("Write").font(theme.font_library.font_noto_sans.font_normal.clone()));
                                    });
                                    user_interface.add_space(4.0);
                                    user_interface.horizontal(|user_interface| {
                                        user_interface.add(Checkbox::new_from_theme(theme).checked(true));
                                        user_interface.add_space(8.0);
                                        user_interface.label(RichText::new("Execute").font(theme.font_library.font_noto_sans.font_normal.clone()));
                                    });
                                    user_interface.add_space(4.0);
                                    user_interface.horizontal(|user_interface| {
                                        user_interface.add(Checkbox::new_from_theme(theme).checked(true));
                                        user_interface.add_space(8.0);
                                        user_interface.label(RichText::new("Copy on Write").font(theme.font_library.font_noto_sans.font_normal.clone()));
                                    });
                                });
                            });

                            groupbox_memory_types.desired_width = Some(192.0);
                            groupbox_memory_types2.desired_width = Some(192.0);

                            user_interface.add(groupbox_memory_types);
                            user_interface.add(groupbox_memory_types2);
                        });
                    }
                    2 => {
                        // Scan settings.
                    }
                    _ => {
                        // General settings.
                    }
                }
            })
            .response;

        response
    }
}
