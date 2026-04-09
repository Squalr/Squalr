use crate::{app_context::AppContext, ui::widgets::controls::data_value_box::data_value_box_view::DataValueBoxView};
use eframe::egui::{Button, Id, Key, RichText, Ui, vec2};
use epaint::Stroke;
use squalr_engine_api::structures::{data_types::data_type_ref::DataTypeRef, data_values::anonymous_value_string::AnonymousValueString};
use std::sync::Arc;

pub struct ProjectItemValueEditTakeOverView<'lifetime> {
    app_context: Arc<AppContext>,
    project_item_name: &'lifetime str,
    value_edit: &'lifetime mut AnonymousValueString,
    validation_data_type_ref: &'lifetime DataTypeRef,
    display_values: &'lifetime [AnonymousValueString],
    value_editor_id: &'lifetime str,
}

pub struct ProjectItemValueEditTakeOverViewResponse {
    pub should_commit: bool,
    pub should_cancel: bool,
}

impl<'lifetime> ProjectItemValueEditTakeOverView<'lifetime> {
    pub fn new(
        app_context: Arc<AppContext>,
        project_item_name: &'lifetime str,
        value_edit: &'lifetime mut AnonymousValueString,
        validation_data_type_ref: &'lifetime DataTypeRef,
        display_values: &'lifetime [AnonymousValueString],
        value_editor_id: &'lifetime str,
    ) -> Self {
        Self {
            app_context,
            project_item_name,
            value_edit,
            validation_data_type_ref,
            display_values,
            value_editor_id,
        }
    }

    pub fn show(
        self,
        user_interface: &mut Ui,
    ) -> ProjectItemValueEditTakeOverViewResponse {
        let theme = &self.app_context.theme;
        let mut should_commit = false;
        let mut should_cancel = false;

        user_interface.add_space(12.0);
        user_interface.vertical_centered(|user_interface| {
            user_interface.label(
                RichText::new(format!("Edit value for {}.", self.project_item_name))
                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                    .color(theme.foreground),
            );
        });
        user_interface.add_space(12.0);

        user_interface.horizontal_centered(|user_interface| {
            let value_editor_width = user_interface.available_width().min(520.0).max(260.0);
            user_interface.add_space(((user_interface.available_width() - value_editor_width) * 0.5).max(0.0));
            user_interface.add(
                DataValueBoxView::new(
                    self.app_context.clone(),
                    self.value_edit,
                    self.validation_data_type_ref,
                    false,
                    true,
                    "",
                    self.value_editor_id,
                )
                .display_values(self.display_values)
                .width(value_editor_width)
                .height(32.0),
            );
        });

        if DataValueBoxView::consume_commit_on_enter(user_interface, self.value_editor_id) {
            should_commit = true;
        }

        user_interface.add_space(12.0);
        user_interface.allocate_ui(vec2(user_interface.available_width(), 32.0), |user_interface| {
            let button_size = vec2(120.0, 28.0);
            let button_spacing = 12.0;
            let total_button_row_width = button_size.x * 2.0 + button_spacing;
            let side_spacing = ((user_interface.available_width() - total_button_row_width) * 0.5).max(0.0);

            user_interface.horizontal(|user_interface| {
                user_interface.add_space(side_spacing);
                user_interface.spacing_mut().item_spacing.x = button_spacing;

                let button_cancel = user_interface.add_sized(
                    button_size,
                    Button::new(RichText::new("Cancel").color(theme.foreground))
                        .fill(theme.background_control_secondary)
                        .stroke(Stroke::new(1.0, theme.background_control_secondary_dark)),
                );

                if button_cancel.clicked() {
                    should_cancel = true;
                }

                let commit_button_response = user_interface.add_sized(
                    button_size,
                    Button::new(RichText::new("Commit").color(theme.foreground))
                        .fill(theme.background_control_primary)
                        .stroke(Stroke::new(1.0, theme.background_control_primary_dark)),
                );

                if commit_button_response.clicked() {
                    should_commit = true;
                }
            });
        });

        let popup_id = Id::new(("data_value_box_popup", self.value_editor_id, user_interface.id().value()));
        let is_format_popup_open = user_interface.memory(|memory| memory.data.get_temp::<bool>(popup_id).unwrap_or(false));

        if user_interface.input(|input_state| input_state.key_pressed(Key::Escape)) && !is_format_popup_open {
            should_cancel = true;
        }

        ProjectItemValueEditTakeOverViewResponse { should_commit, should_cancel }
    }
}
