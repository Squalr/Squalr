use crate::{
    app_context::AppContext,
    ui::{draw::icon_draw::IconDraw, widgets::controls::data_value_box::data_value_box_view::DataValueBoxView},
};
use eframe::egui::{Align, Layout, Response, Sense, Ui, UiBuilder, Widget, pos2, vec2};
use epaint::Rect;
use squalr_engine_api::structures::{
    data_types::{built_in_types::string::utf8::data_type_string_utf8::DataTypeStringUtf8, data_type_ref::DataTypeRef},
    data_values::{anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType},
};
use std::sync::Arc;

pub struct SearchBoxView<'lifetime> {
    app_context: Arc<AppContext>,
    value_text: &'lifetime mut String,
    preview_text: &'lifetime str,
    id: &'lifetime str,
    width: f32,
    height: f32,
    icon_left_padding: f32,
    icon_size: f32,
    icon_gap: f32,
}

impl<'lifetime> SearchBoxView<'lifetime> {
    const DEFAULT_HEIGHT: f32 = 28.0;
    const DEFAULT_WIDTH: f32 = 212.0;
    const DEFAULT_ICON_LEFT_PADDING: f32 = 8.0;
    const DEFAULT_ICON_SIZE: f32 = 16.0;
    const DEFAULT_ICON_GAP: f32 = 8.0;

    pub fn new(
        app_context: Arc<AppContext>,
        value_text: &'lifetime mut String,
        preview_text: &'lifetime str,
        id: &'lifetime str,
    ) -> Self {
        Self {
            app_context,
            value_text,
            preview_text,
            id,
            width: Self::DEFAULT_WIDTH,
            height: Self::DEFAULT_HEIGHT,
            icon_left_padding: Self::DEFAULT_ICON_LEFT_PADDING,
            icon_size: Self::DEFAULT_ICON_SIZE,
            icon_gap: Self::DEFAULT_ICON_GAP,
        }
    }

    pub fn width(
        mut self,
        width: f32,
    ) -> Self {
        self.width = width.max(1.0);
        self
    }

    pub fn height(
        mut self,
        height: f32,
    ) -> Self {
        self.height = height;
        self
    }
}

impl<'lifetime> Widget for SearchBoxView<'lifetime> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let original_value_text = self.value_text.clone();
        let (search_row_rect, mut response) = user_interface.allocate_exact_size(vec2(self.width.max(1.0), self.height), Sense::hover());
        let icon_center_x = search_row_rect.left() + self.icon_left_padding + self.icon_size * 0.5;
        let icon_center = pos2(icon_center_x, search_row_rect.center().y);

        IconDraw::draw_sized_tinted(
            user_interface,
            icon_center,
            vec2(self.icon_size, self.icon_size),
            &theme.icon_library.icon_handle_common_search,
            theme.foreground_preview,
        );

        let value_box_left = search_row_rect.left() + self.icon_left_padding + self.icon_size + self.icon_gap;
        let value_box_rect = Rect::from_min_max(pos2(value_box_left, search_row_rect.top()), search_row_rect.right_bottom());
        let value_box_width = value_box_rect.width().max(1.0);
        let mut value_box_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(value_box_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        value_box_user_interface.set_clip_rect(value_box_rect);

        let mut anonymous_value_string = AnonymousValueString::new(self.value_text.clone(), AnonymousValueStringFormat::String, ContainerType::None);
        let string_data_type_ref = DataTypeRef::new(DataTypeStringUtf8::DATA_TYPE_ID);

        value_box_user_interface.add(
            DataValueBoxView::new(
                self.app_context.clone(),
                &mut anonymous_value_string,
                &string_data_type_ref,
                false,
                true,
                self.preview_text,
                self.id,
            )
            .allowed_anonymous_value_string_formats(vec![AnonymousValueStringFormat::String])
            .show_format_button(false)
            .normalize_value_format(false)
            .use_format_text_colors(false)
            .width(value_box_width)
            .height(self.height),
        );

        *self.value_text = anonymous_value_string.get_anonymous_value_string().to_string();
        if *self.value_text != original_value_text {
            response.mark_changed();
        }

        response
    }
}
