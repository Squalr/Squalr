use crate::{
    app_context::AppContext,
    ui::{
        draw::icon_draw::IconDraw,
        widgets::controls::{button::Button, data_value_box::data_value_box_view::DataValueBoxView},
    },
    views::project_explorer::project_selector::view_data::project_selector_view_data::ProjectSelectorViewData,
};
use eframe::egui::{Align, Align2, Id, Key, Layout, Response, RichText, Sense, Ui, UiBuilder, pos2, vec2};
use epaint::CornerRadius;
use squalr_engine_api::structures::{
    data_types::{built_in_types::string::utf8::data_type_string_utf8::DataTypeStringUtf8, data_type_ref::DataTypeRef},
    data_values::{anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType},
    projects::project_info::ProjectInfo,
};
use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

pub struct ProjectEditTakeOverView<'lifetime> {
    app_context: Arc<AppContext>,
    project_info: &'lifetime ProjectInfo,
    project_list: &'lifetime [ProjectInfo],
    rename_project_text: &'lifetime Arc<RwLock<(String, bool)>>,
}

pub struct ProjectEditTakeOverViewResponse {
    pub should_cancel: bool,
    pub should_delete: bool,
    pub rename_submission: Option<(PathBuf, String)>,
}

impl<'lifetime> ProjectEditTakeOverView<'lifetime> {
    const HEADER_HEIGHT: f32 = 32.0;
    const HEADER_TITLE_PADDING_X: f32 = 8.0;
    const HEADER_ICON_BUTTON_WIDTH: f32 = 36.0;
    const CONTENT_PADDING_X: f32 = 12.0;
    const CONTENT_PADDING_TOP: f32 = 12.0;
    const ROW_HEIGHT: f32 = 28.0;
    const VALUE_BOX_WIDTH: f32 = 260.0;
    const ROW_SPACING: f32 = 8.0;
    const NAME_EDITOR_ID: &'static str = "project_selector_project_name_edit";

    pub fn new(
        app_context: Arc<AppContext>,
        project_info: &'lifetime ProjectInfo,
        project_list: &'lifetime [ProjectInfo],
        rename_project_text: &'lifetime Arc<RwLock<(String, bool)>>,
    ) -> Self {
        Self {
            app_context,
            project_info,
            project_list,
            rename_project_text,
        }
    }

    fn string_data_type_ref() -> DataTypeRef {
        DataTypeRef::new(DataTypeStringUtf8::DATA_TYPE_ID)
    }

    fn render_header_icon_button(
        &self,
        user_interface: &mut Ui,
        icon_handle: &eframe::egui::TextureHandle,
        tooltip_text: &str,
        background_color: epaint::Color32,
        border_color: epaint::Color32,
        button_height: f32,
        is_disabled: bool,
    ) -> Response {
        let theme = &self.app_context.theme;
        let button_response = user_interface.add_sized(
            vec2(Self::HEADER_ICON_BUTTON_WIDTH, button_height),
            Button::new_from_theme(theme)
                .with_tooltip_text(tooltip_text)
                .background_color(background_color)
                .border_color(border_color)
                .border_width(1.0)
                .disabled(is_disabled),
        );

        IconDraw::draw_tinted(
            user_interface,
            button_response.rect,
            icon_handle,
            if is_disabled { theme.foreground_preview } else { theme.foreground },
        );

        button_response
    }

    pub fn show(
        self,
        user_interface: &mut Ui,
    ) -> ProjectEditTakeOverViewResponse {
        let theme = &self.app_context.theme;
        let mut should_cancel = false;
        let mut should_delete = false;
        let mut rename_submission = None;
        let (panel_rect, _) = user_interface.allocate_exact_size(user_interface.available_size(), Sense::hover());

        user_interface
            .painter()
            .rect_filled(panel_rect, CornerRadius::ZERO, theme.background_panel);

        let mut panel_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(panel_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        panel_user_interface.set_clip_rect(panel_rect);

        let (header_rect, _) =
            panel_user_interface.allocate_exact_size(vec2(panel_user_interface.available_width().max(1.0), Self::HEADER_HEIGHT), Sense::hover());
        panel_user_interface
            .painter()
            .rect_filled(header_rect, CornerRadius::ZERO, theme.background_primary);

        let mut header_user_interface = panel_user_interface.new_child(
            UiBuilder::new()
                .max_rect(header_rect)
                .layout(Layout::left_to_right(Align::Center)),
        );
        header_user_interface.set_clip_rect(header_rect);

        let header_action_width = Self::HEADER_ICON_BUTTON_WIDTH * 2.0 + Self::ROW_SPACING;
        let title_width = (header_rect.width() - header_action_width - Self::HEADER_TITLE_PADDING_X).max(0.0);
        let (title_rect, _) = header_user_interface.allocate_exact_size(vec2(title_width, Self::HEADER_HEIGHT), Sense::hover());
        header_user_interface.painter().text(
            pos2(title_rect.left() + Self::HEADER_TITLE_PADDING_X, title_rect.center().y),
            Align2::LEFT_CENTER,
            "Edit project",
            theme.font_library.font_noto_sans.font_window_title.clone(),
            theme.foreground,
        );

        header_user_interface.allocate_ui_with_layout(
            vec2(header_action_width, Self::HEADER_HEIGHT),
            Layout::right_to_left(Align::Center),
            |user_interface| {
                let delete_response = self.render_header_icon_button(
                    user_interface,
                    &theme.icon_library.icon_handle_common_delete,
                    "Delete this project.",
                    theme.background_control_danger,
                    theme.background_control_danger_dark,
                    Self::HEADER_HEIGHT,
                    false,
                );
                if delete_response.clicked() {
                    should_delete = true;
                }

                user_interface.add_space(Self::ROW_SPACING);

                let cancel_response = self.render_header_icon_button(
                    user_interface,
                    &theme.icon_library.icon_handle_navigation_cancel,
                    "Cancel project editing.",
                    theme.background_control_secondary,
                    theme.submenu_border,
                    Self::HEADER_HEIGHT,
                    false,
                );
                if cancel_response.clicked() {
                    should_cancel = true;
                }
            },
        );

        panel_user_interface.add_space(Self::CONTENT_PADDING_TOP);
        panel_user_interface.horizontal(|user_interface| {
            user_interface.add_space(Self::CONTENT_PADDING_X);
            user_interface.allocate_ui_with_layout(
                vec2(
                    (user_interface.available_width() - Self::CONTENT_PADDING_X * 2.0).max(0.0),
                    user_interface.available_height(),
                ),
                Layout::top_down(Align::Min),
                |user_interface| {
                    user_interface.label(RichText::new("Name").strong().color(theme.foreground));
                    user_interface.add_space(Self::ROW_SPACING);

                    let mut validation_result = Ok(());
                    let mut submitted_name = None;
                    match self.rename_project_text.write() {
                        Ok(mut rename_project_guard) => {
                            let (project_name_text, _should_highlight_text) = &mut *rename_project_guard;
                            let mut project_name_value =
                                AnonymousValueString::new(project_name_text.clone(), AnonymousValueStringFormat::String, ContainerType::None);
                            let string_data_type_ref = Self::string_data_type_ref();
                            let name_editor_width = Self::VALUE_BOX_WIDTH.min(user_interface.available_width().max(0.0));

                            user_interface.horizontal(|user_interface| {
                                user_interface.add_sized(
                                    vec2(name_editor_width, Self::ROW_HEIGHT),
                                    DataValueBoxView::new(
                                        self.app_context.clone(),
                                        &mut project_name_value,
                                        &string_data_type_ref,
                                        false,
                                        true,
                                        "Project name",
                                        Self::NAME_EDITOR_ID,
                                    )
                                    .width(name_editor_width)
                                    .height(Self::ROW_HEIGHT)
                                    .use_format_text_colors(false),
                                );
                                *project_name_text = project_name_value.get_anonymous_value_string().to_string();
                                validation_result = ProjectSelectorViewData::validate_project_name(
                                    self.project_list,
                                    self.project_info.get_project_file_path(),
                                    self.project_info.get_name(),
                                    project_name_text,
                                );

                                user_interface.add_space(Self::ROW_SPACING);

                                let is_save_disabled = validation_result.is_err();
                                let save_response = self.render_header_icon_button(
                                    user_interface,
                                    &theme.icon_library.icon_handle_common_check_mark,
                                    "Save project name.",
                                    theme.background_control_primary,
                                    theme.background_control_primary_dark,
                                    Self::ROW_HEIGHT,
                                    is_save_disabled,
                                );
                                if !is_save_disabled
                                    && (save_response.clicked() || DataValueBoxView::consume_commit_on_enter(user_interface, Self::NAME_EDITOR_ID))
                                {
                                    submitted_name = Some(project_name_text.trim().to_string());
                                }
                            });
                        }
                        Err(error) => {
                            validation_result = Err(format!("Failed to edit project name: {}.", error));
                        }
                    }

                    if let Err(validation_error) = validation_result {
                        user_interface.add_space(Self::ROW_SPACING);
                        user_interface.label(RichText::new(validation_error).color(theme.error_red));
                    }

                    if let Some(submitted_name) = submitted_name {
                        rename_submission = Some((self.project_info.get_project_file_path().to_path_buf(), submitted_name));
                    }
                },
            );
        });

        let popup_id = Id::new(("data_value_box_popup", Self::NAME_EDITOR_ID, user_interface.id().value()));
        let is_format_popup_open = user_interface.memory(|memory| memory.data.get_temp::<bool>(popup_id).unwrap_or(false));
        if user_interface.input(|input_state| input_state.key_pressed(Key::Escape)) && !is_format_popup_open {
            should_cancel = true;
        }

        ProjectEditTakeOverViewResponse {
            should_cancel,
            should_delete,
            rename_submission,
        }
    }
}
