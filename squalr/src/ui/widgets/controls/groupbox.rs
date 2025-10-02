use crate::ui::theme::Theme;
use eframe::egui::{Color32, Response, Sense, Ui, UiBuilder, UiKind, UiStackInfo, Widget};
use epaint::{CornerRadius, FontId, Rect, Shape, Stroke, StrokeKind, Vec2};

pub struct GroupBox<'a, F: FnOnce(&mut Ui)> {
    pub header_text: &'a str,
    pub background: Color32,
    pub border_color: Color32,
    pub rounding: u8,
    pub top_extra_padding: f32,
    pub inner_padding: f32,
    pub add_contents: F,
    pub header_font_id: FontId,
}

impl<'a, F: FnOnce(&mut Ui)> GroupBox<'a, F> {
    pub fn new_from_theme(
        theme: &Theme,
        header_text: &'a str,
        add_contents: F,
    ) -> Self {
        Self {
            header_text,
            background: theme.background_panel,
            border_color: theme.submenu_border,
            rounding: 4,
            top_extra_padding: 4.0,
            inner_padding: 12.0,
            add_contents,
            header_font_id: theme.font_library.font_noto_sans.font_header.clone(),
        }
    }
}

impl<'a, F: FnOnce(&mut Ui)> Widget for GroupBox<'a, F> {
    fn ui(
        self,
        ui: &mut Ui,
    ) -> Response {
        // Reserve a background slot so our frame paints *behind* children:
        let where_to_put_background = ui.painter().add(Shape::Noop);

        // Available rect we can occupy:
        let outer_rect_bounds = ui.available_rect_before_wrap();
        let max_content_rect = outer_rect_bounds.shrink2(Vec2::splat(self.inner_padding));

        // Build a child Ui that children will render into:
        let mut content_ui = ui.new_child(
            UiBuilder::new()
                .ui_stack_info(UiStackInfo::new(UiKind::Frame))
                .max_rect(max_content_rect),
        );

        // Add top padding + run caller closure
        content_ui.add_space(self.top_extra_padding);
        (self.add_contents)(&mut content_ui);

        // Figure out used rects
        let content_rect = content_ui.min_rect();
        let final_rect = content_rect.expand(self.inner_padding);

        // Paint groupbox background + border:
        let bg_shape = Shape::Rect(epaint::RectShape::new(
            final_rect,
            CornerRadius::same(self.rounding),
            self.background,
            Stroke::new(1.0, self.border_color),
            StrokeKind::Inside,
        ));
        ui.painter().set(where_to_put_background, bg_shape);

        // Paint header
        let header_galley = ui
            .painter()
            .layout_no_wrap(self.header_text.to_owned(), self.header_font_id.clone(), Color32::WHITE);
        let header_pos = final_rect.min + Vec2::new(8.0, 0.0);
        let bg_rect = Rect::from_min_size(header_pos, header_galley.size() + Vec2::new(8.0, 0.0));

        ui.painter().rect_filled(bg_rect, 0.0, self.background);
        ui.painter().galley(header_pos, header_galley, Color32::WHITE);

        // Allocate space so parent knows how big we were:
        ui.allocate_rect(final_rect, Sense::hover())
    }
}
