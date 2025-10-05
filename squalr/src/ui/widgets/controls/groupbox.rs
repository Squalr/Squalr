use crate::ui::theme::Theme;
use eframe::egui::{Align, Color32, Layout, Response, Sense, Ui, UiBuilder, Widget};
use epaint::{CornerRadius, FontId, Rect, RectShape, Shape, Stroke, StrokeKind, pos2, vec2};

pub struct GroupBox<'lifetime, F: FnOnce(&mut Ui)> {
    pub header_text: &'lifetime str,
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

impl<'lifetime, F: FnOnce(&mut Ui)> GroupBox<'lifetime, F> {
    pub fn new_from_theme(
        theme: &Theme,
        header_text: &'lifetime str,
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

    pub fn desired_width(
        mut self,
        desired_width: f32,
    ) -> Self {
        self.desired_width = Some(desired_width);
        self
    }

    pub fn desired_height(
        mut self,
        desired_height: f32,
    ) -> Self {
        self.desired_height = Some(desired_height);
        self
    }
}

impl<'lifetime, F: FnOnce(&mut Ui)> Widget for GroupBox<'lifetime, F> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        // Measure header without painting.
        let header_galley = user_interface
            .painter()
            .layout_no_wrap(self.header_text.to_owned(), self.header_font_id.clone(), Color32::WHITE);
        let header_size = header_galley.size();
        let header_height = header_size.y;
        let header_min_width = header_size.x + self.header_padding * 2.0;

        // We overlay the header so the border starts at half-header height.
        let vertical_overlap = header_height * 0.5;

        // Decide target width up-front (never grow after we allocate).
        let avail = user_interface.available_size();
        let target_width = self
            .desired_width
            .unwrap_or(header_min_width)
            .max(header_min_width)
            .min(avail.x);

        // Inner content max width (inside left/right padding):
        let inner_width = (target_width - self.content_padding * 2.0).max(0.0);

        // Temporarily build a child UI to layout the contents and find required height. Start from the current cursor position.
        let origin = user_interface.cursor().min;

        // Rect where the content may place widgets (below the overlapped header, with padding).
        let content_min = origin + vec2(self.content_padding, vertical_overlap + self.content_padding);

        // Let content grow downwards as needed; width is fixed.
        let content_max = pos2(content_min.x + inner_width, user_interface.max_rect().max.y);

        let mut content_ui = user_interface.new_child(
            UiBuilder::new()
                .max_rect(Rect::from_min_max(content_min, content_max))
                .layout(Layout::top_down(Align::Min)),
        );

        // Add user contents (this determines the required height).
        (self.add_contents)(&mut content_ui);

        // Laid out content bounds.
        let laid_out = content_ui.min_rect();
        let content_height = (laid_out.max.y - content_min.y).max(0.0);

        // Compute the border (background) block height: top padding + content + bottom padding.
        let mut border_height = self.content_padding + content_height + self.content_padding;

        // If a desired height was specified, respect it (clamped by avail.y).
        if let Some(desired_h) = self.desired_height {
            // Account for header overlap when computing target height.
            let target_total_height = desired_h.min(avail.y).max(header_height);
            let target_border_height = (target_total_height - vertical_overlap).max(0.0);
            border_height = border_height.max(target_border_height);
        }

        // The border box starts *after* the header overlap.
        let border_min = origin + vec2(0.0, vertical_overlap);
        let border_rectangle = Rect::from_min_size(border_min, vec2(target_width, border_height));

        // The full widget (outer) rect must include the floating header above the border.
        let outer_height = vertical_overlap + border_height;
        let outer_rectangle = Rect::from_min_size(origin, vec2(target_width, outer_height));

        // Allocate the exact rect so parent layouts know our true footprint.
        let response = user_interface.allocate_rect(outer_rectangle, Sense::hover());

        // Paint everything relative to 'outer_rect'.
        if user_interface.is_rect_visible(outer_rectangle) {
            let header_position = outer_rectangle.min + vec2(self.header_padding, 0.0);
            let header_bg_rect = Rect::from_min_size(pos2(header_position.x - 4.0, header_position.y), vec2(header_size.x + 8.0, header_size.y));

            // Border around the content area.
            user_interface.painter().add(Shape::Rect(RectShape::new(
                border_rectangle,
                CornerRadius::same(self.rounding),
                Color32::TRANSPARENT,
                Stroke::new(1.0, self.border_color),
                StrokeKind::Inside,
            )));

            // Header background.
            user_interface
                .painter()
                .rect_filled(header_bg_rect, 0.0, self.background_color);

            // Header text.
            user_interface
                .painter()
                .galley(header_position, header_galley, Color32::WHITE);
        }

        response
    }
}
