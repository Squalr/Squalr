use crate::ui::theme::Theme;
use eframe::egui::{Color32, Response, Sense, Ui, UiBuilder, UiKind, UiStackInfo, Widget};
use epaint::{CornerRadius, FontId, Rect, RectShape, Shape, Stroke, StrokeKind, Vec2, pos2, vec2};

pub struct GroupBox<'a, F: FnOnce(&mut Ui)> {
    pub header_text: &'a str,
    pub background_color: Color32,
    pub border_color: Color32,
    pub add_contents: F,
    pub header_font_id: FontId,
    pub header_padding: f32,
    pub content_padding: f32,
    pub desired_width: Option<f32>,
    pub desired_height: Option<f32>,
    pub rounding: u8,
}

impl<'a, F: FnOnce(&mut Ui)> GroupBox<'a, F> {
    pub fn new_from_theme(
        theme: &Theme,
        header_text: &'a str,
        add_contents: F,
    ) -> Self {
        Self {
            header_text,
            background_color: theme.background_panel,
            border_color: theme.submenu_border,
            add_contents,
            header_font_id: theme.font_library.font_noto_sans.font_header.clone(),
            header_padding: 16.0,
            content_padding: 12.0,
            desired_width: None,
            desired_height: None,
            rounding: 4,
        }
    }
}

impl<'a, F: FnOnce(&mut Ui)> Widget for GroupBox<'a, F> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let where_to_put_background = user_interface.painter().add(Shape::Noop);
        let available_size_rect = user_interface.available_rect_before_wrap();
        let available_size = available_size_rect.size();
        let header_galley = user_interface
            .painter()
            .layout_no_wrap(self.header_text.to_owned(), self.header_font_id.clone(), Color32::WHITE);
        let header_width_with_padding = header_galley.size().x + self.header_padding * 2.0;
        let header_height = header_galley.size().y;

        // Offset contents by half header height.
        let content_offset = Vec2::new(0.0, header_height / 2.0);
        let max_content_rect = available_size_rect
            .translate(content_offset)
            .shrink2(Vec2::splat(self.content_padding));

        // Build child ui to render contents.
        let mut content_user_interface = user_interface.new_child(
            UiBuilder::new()
                .ui_stack_info(UiStackInfo::new(UiKind::Frame))
                .max_rect(max_content_rect),
        );

        (self.add_contents)(&mut content_user_interface);

        let content_rectangle = content_user_interface.min_rect();
        let mut border_rectangle = content_rectangle.expand(self.content_padding);

        // Apply width / height overrides if set.
        if let Some(desired_width) = self.desired_width {
            let clamped_width = desired_width.min(available_size.x);

            border_rectangle = Rect::from_min_max(border_rectangle.min, pos2(border_rectangle.min.x + clamped_width, border_rectangle.max.y));
        }

        if let Some(desired_height) = self.desired_height {
            let clamped_height = desired_height.min(available_size.y);

            border_rectangle = Rect::from_min_max(border_rectangle.min, pos2(border_rectangle.max.x, border_rectangle.min.y + clamped_height));
        }

        // Expand border if shorter than header width.
        if border_rectangle.width() < header_width_with_padding {
            border_rectangle = Rect::from_min_max(
                border_rectangle.min,
                pos2(border_rectangle.min.x + header_width_with_padding, border_rectangle.max.y),
            );
        }

        let border_shape = Shape::Rect(RectShape::new(
            border_rectangle,
            CornerRadius::same(self.rounding),
            Color32::TRANSPARENT,
            Stroke::new(1.0, self.border_color),
            StrokeKind::Inside,
        ));
        let header_offset = Vec2::new(self.header_padding, -header_height / 2.0);
        let header_position = border_rectangle.min + header_offset;
        let header_bg_rect = Rect::from_min_size(
            pos2(header_position.x - 4.0, header_position.y),
            vec2(header_galley.size().x + 8.0, header_galley.size().y),
        );

        user_interface
            .painter()
            .rect_filled(header_bg_rect, 0.0, self.background_color);
        user_interface
            .painter()
            .set(where_to_put_background, border_shape);
        user_interface
            .painter()
            .galley(header_position, header_galley, Color32::WHITE);

        user_interface.allocate_rect(border_rectangle, Sense::hover())
    }
}
