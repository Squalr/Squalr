use crate::app_context::AppContext;
use crate::ui::widgets::controls::groupbox::GroupBox;
use crate::views::symbol_tree::symbol_tree_take_over_view_helpers::draw_sized_action_button;
use eframe::egui::{Button, Direction, Layout, RichText, Ui, vec2};
use epaint::Stroke;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolTreeDeleteConfirmationAction {
    None,
    Cancel,
    Confirm,
}

pub struct SymbolTreeDeleteConfirmationView<'a> {
    app_context: Arc<AppContext>,
    title: &'a str,
    display_name: &'a str,
    description_text: &'a str,
    is_description_warning: bool,
    confirm_button_label: &'a str,
}

impl<'a> SymbolTreeDeleteConfirmationView<'a> {
    pub fn new(
        app_context: Arc<AppContext>,
        title: &'a str,
        display_name: &'a str,
        description_text: &'a str,
        is_description_warning: bool,
        confirm_button_label: &'a str,
    ) -> Self {
        Self {
            app_context,
            title,
            display_name,
            description_text,
            is_description_warning,
            confirm_button_label,
        }
    }

    pub fn show(
        self,
        user_interface: &mut Ui,
    ) -> SymbolTreeDeleteConfirmationAction {
        let theme = &self.app_context.theme;
        let mut action = SymbolTreeDeleteConfirmationAction::None;
        let description_color = if self.is_description_warning {
            theme.warning
        } else {
            theme.foreground_preview
        };

        user_interface.allocate_ui_with_layout(
            user_interface.available_size(),
            Layout::centered_and_justified(Direction::TopDown),
            |user_interface| {
                let groupbox_side_padding = 8.0;
                let panel_width = (user_interface.available_width() - groupbox_side_padding * 2.0).max(0.0);

                user_interface.horizontal(|user_interface| {
                    user_interface.add_space(groupbox_side_padding);
                    user_interface.add(
                        GroupBox::new_from_theme(theme, self.title, |user_interface| {
                            user_interface.vertical_centered(|user_interface| {
                                user_interface.label(
                                    RichText::new(self.display_name)
                                        .font(theme.font_library.font_ubuntu_mono_bold.font_header.clone())
                                        .color(theme.foreground),
                                );
                                user_interface.add_space(6.0);
                                user_interface.label(RichText::new(self.description_text).color(description_color));
                            });

                            user_interface.add_space(12.0);
                            user_interface.allocate_ui(vec2(user_interface.available_width(), 32.0), |user_interface| {
                                let button_size = vec2(120.0, 28.0);
                                let button_spacing = 12.0;
                                let total_button_row_width = button_size.x * 2.0 + button_spacing;
                                let side_spacing = ((user_interface.available_width() - total_button_row_width) * 0.5).max(0.0);

                                user_interface.horizontal(|user_interface| {
                                    user_interface.add_space(side_spacing);
                                    user_interface.spacing_mut().item_spacing.x = button_spacing;

                                    let button_confirm_delete = user_interface.add_sized(
                                        button_size,
                                        Button::new(RichText::new(self.confirm_button_label).color(theme.foreground))
                                            .fill(theme.background_control_danger)
                                            .stroke(Stroke::new(1.0, theme.background_control_danger_dark)),
                                    );

                                    if button_confirm_delete.clicked() {
                                        action = SymbolTreeDeleteConfirmationAction::Confirm;
                                    }

                                    let button_cancel = draw_sized_action_button(
                                        &self.app_context,
                                        user_interface,
                                        "Cancel",
                                        button_size,
                                        theme.background_control_secondary,
                                        theme.background_control_secondary_dark,
                                        true,
                                    );

                                    if button_cancel.clicked() {
                                        action = SymbolTreeDeleteConfirmationAction::Cancel;
                                    }
                                });
                            });
                        })
                        .desired_width(panel_width),
                    );
                });
            },
        );

        action
    }
}
