use crate::app_context::AppContext;
use eframe::egui::{Align2, Response, Sense, Ui, Widget, pos2, vec2};
use epaint::CornerRadius;
use std::sync::Arc;

pub struct ListHeaderView<'view> {
    app_context: Arc<AppContext>,
    left_label: &'view str,
    right_label: &'view str,
    height: f32,
    horizontal_padding: f32,
}

impl<'view> ListHeaderView<'view> {
    pub fn new(
        app_context: Arc<AppContext>,
        left_label: &'view str,
        right_label: &'view str,
    ) -> Self {
        Self {
            app_context,
            left_label,
            right_label,
            height: 28.0,
            horizontal_padding: 8.0,
        }
    }

    pub fn height(
        mut self,
        height: f32,
    ) -> Self {
        self.height = height;
        self
    }

    pub fn horizontal_padding(
        mut self,
        horizontal_padding: f32,
    ) -> Self {
        self.horizontal_padding = horizontal_padding;
        self
    }
}

impl Widget for ListHeaderView<'_> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let (header_rect, response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), self.height), Sense::hover());

        user_interface
            .painter()
            .rect_filled(header_rect, CornerRadius::ZERO, theme.background_primary);
        user_interface.painter().text(
            pos2(header_rect.min.x + self.horizontal_padding, header_rect.center().y),
            Align2::LEFT_CENTER,
            self.left_label,
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground_preview,
        );
        user_interface.painter().text(
            pos2(header_rect.max.x - self.horizontal_padding, header_rect.center().y),
            Align2::RIGHT_CENTER,
            self.right_label,
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground_preview,
        );

        response
    }
}
