use crate::{
    app_context::AppContext,
    ui::{draw::icon_draw::IconDraw, widgets::controls::button::Button},
    views::pointer_scanner::view_data::pointer_scanner_view_data::PointerScannerViewData,
};
use eframe::egui::{Align, Align2, Response, Sense, TextEdit, Ui, UiBuilder, Widget, vec2};
use epaint::{Color32, CornerRadius, Rect, pos2};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq)]
struct PointerScannerFooterNavigationLayout {
    button_width: f32,
    page_box_width: f32,
    spacing: f32,
}

impl PointerScannerFooterNavigationLayout {
    fn total_width(&self) -> f32 {
        self.button_width * 4.0 + self.page_box_width + self.spacing * 4.0
    }
}

#[derive(Clone)]
pub struct PointerScannerFooterView {
    app_context: Arc<AppContext>,
    pointer_scanner_view_data: Dependency<PointerScannerViewData>,
}

impl PointerScannerFooterView {
    pub const FOOTER_HEIGHT: f32 = 56.0;
    const HORIZONTAL_PADDING: f32 = 8.0;
    const NAVIGATION_SPACING: f32 = 4.0;
    const MAX_PAGE_BOX_WIDTH: f32 = 112.0;
    const MIN_PAGE_BOX_WIDTH: f32 = 24.0;
    const MAX_BUTTON_WIDTH: f32 = 36.0;
    const MIN_BUTTON_WIDTH: f32 = 12.0;

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let pointer_scanner_view_data = app_context
            .dependency_container
            .get_dependency::<PointerScannerViewData>();

        Self {
            app_context,
            pointer_scanner_view_data,
        }
    }

    pub fn get_height(&self) -> f32 {
        Self::FOOTER_HEIGHT
    }

    fn resolve_navigation_layout(available_width: f32) -> PointerScannerFooterNavigationLayout {
        let clamped_available_width = available_width.max(1.0);
        let horizontal_padding = Self::HORIZONTAL_PADDING.min((clamped_available_width * 0.05).floor());
        let spacing = Self::NAVIGATION_SPACING.min((clamped_available_width * 0.025).floor());
        let content_budget = (clamped_available_width - horizontal_padding * 2.0 - spacing * 4.0).max(1.0);
        let minimum_content_budget = Self::MIN_PAGE_BOX_WIDTH + Self::MIN_BUTTON_WIDTH * 4.0;

        if content_budget >= minimum_content_budget {
            let button_width = ((content_budget - Self::MIN_PAGE_BOX_WIDTH) / 4.0).clamp(Self::MIN_BUTTON_WIDTH, Self::MAX_BUTTON_WIDTH);
            let page_box_width = (content_budget - button_width * 4.0).clamp(Self::MIN_PAGE_BOX_WIDTH, Self::MAX_PAGE_BOX_WIDTH);

            return PointerScannerFooterNavigationLayout {
                button_width,
                page_box_width,
                spacing,
            };
        }

        let compact_button_width = (clamped_available_width / 8.0).clamp(1.0, Self::MAX_BUTTON_WIDTH);
        let compact_page_box_width = (clamped_available_width - compact_button_width * 4.0).max(1.0);

        PointerScannerFooterNavigationLayout {
            button_width: compact_button_width,
            page_box_width: compact_page_box_width,
            spacing: 0.0,
        }
    }
}

impl Widget for PointerScannerFooterView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let available_footer_rectangle = user_interface
            .available_rect_before_wrap()
            .intersect(user_interface.clip_rect());
        let height = self
            .get_height()
            .min(available_footer_rectangle.height().max(0.0));
        let top_row_height = 28.0_f32.min(height);
        let bottom_row_height = (height - top_row_height).max(0.0);
        let allocated_footer_rectangle = Rect::from_min_size(available_footer_rectangle.min, vec2(available_footer_rectangle.width().max(1.0), height));
        let response = user_interface.allocate_rect(allocated_footer_rectangle, Sense::empty());
        let allocated_size_rectangle = response.rect;

        if height <= 0.0 {
            return response;
        }

        let theme = &self.app_context.theme;
        let font_id = theme.font_library.font_noto_sans.font_normal.clone();
        let page_label_text = PointerScannerViewData::build_page_label(self.pointer_scanner_view_data.clone());
        let stats_text = PointerScannerViewData::build_page_stats_text(self.pointer_scanner_view_data.clone());
        let context_text = PointerScannerViewData::build_current_context_text(self.pointer_scanner_view_data.clone());
        user_interface
            .painter()
            .rect_filled(allocated_size_rectangle, CornerRadius::ZERO, theme.background_panel);
        let divider_rectangle = Rect::from_min_max(
            allocated_size_rectangle.min,
            pos2(allocated_size_rectangle.max.x, allocated_size_rectangle.min.y + 1.0),
        );
        user_interface
            .painter()
            .rect_filled(divider_rectangle, CornerRadius::ZERO, theme.submenu_border);

        let top_row_rectangle = Rect::from_min_max(
            allocated_size_rectangle.min,
            pos2(allocated_size_rectangle.max.x, allocated_size_rectangle.min.y + top_row_height),
        );
        let bottom_row_rectangle = Rect::from_min_max(pos2(allocated_size_rectangle.min.x, top_row_rectangle.max.y), allocated_size_rectangle.max);
        let navigation_layout = Self::resolve_navigation_layout(top_row_rectangle.width());
        let spacing = navigation_layout.spacing;
        let page_box_width = navigation_layout.page_box_width;
        let page_box_height = 24.0;
        let button_width = navigation_layout.button_width;
        let button_height = 28.0;
        let navigation_group_width = navigation_layout.total_width();
        let navigation_origin_x = top_row_rectangle.center().x - navigation_group_width * 0.5;
        let control_center_y = top_row_rectangle.center().y;
        let first_page_button_rectangle = Rect::from_center_size(
            pos2(navigation_origin_x + button_width * 0.5, control_center_y),
            vec2(button_width, button_height),
        );
        let previous_page_button_rectangle = Rect::from_center_size(
            pos2(first_page_button_rectangle.max.x + spacing + button_width * 0.5, control_center_y),
            vec2(button_width, button_height),
        );
        let page_number_edit_rectangle = Rect::from_center_size(
            pos2(previous_page_button_rectangle.max.x + spacing + page_box_width * 0.5, control_center_y),
            vec2(page_box_width, page_box_height),
        );
        let next_page_button_rectangle = Rect::from_center_size(
            pos2(page_number_edit_rectangle.max.x + spacing + button_width * 0.5, control_center_y),
            vec2(button_width, button_height),
        );
        let last_page_button_rectangle = Rect::from_center_size(
            pos2(next_page_button_rectangle.max.x + spacing + button_width * 0.5, control_center_y),
            vec2(button_width, button_height),
        );

        let mut should_navigate_first_page = false;
        let mut should_navigate_previous_page = false;
        let mut should_navigate_next_page = false;
        let mut should_navigate_last_page = false;
        let mut page_label_text = page_label_text;

        let top_row_builder = UiBuilder::new()
            .max_rect(top_row_rectangle)
            .sense(Sense::click());
        let mut top_row_user_interface = user_interface.new_child(top_row_builder);
        top_row_user_interface.set_clip_rect(top_row_rectangle);
        let first_page_button = top_row_user_interface.put(
            first_page_button_rectangle,
            Button::new_from_theme(theme)
                .background_color(Color32::TRANSPARENT)
                .with_tooltip_text("First page in the current pointer context."),
        );
        IconDraw::draw(
            &top_row_user_interface,
            first_page_button.rect,
            &theme.icon_library.icon_handle_navigation_left_arrows,
        );
        if first_page_button.clicked() {
            should_navigate_first_page = true;
        }

        let previous_page_button = top_row_user_interface.put(
            previous_page_button_rectangle,
            Button::new_from_theme(theme)
                .background_color(Color32::TRANSPARENT)
                .with_tooltip_text("Previous page in the current pointer context."),
        );
        IconDraw::draw(
            &top_row_user_interface,
            previous_page_button.rect,
            &theme.icon_library.icon_handle_navigation_left_arrow_small,
        );
        if previous_page_button.clicked() {
            should_navigate_previous_page = true;
        }

        let page_number_edit_response = top_row_user_interface.put(
            page_number_edit_rectangle,
            TextEdit::singleline(&mut page_label_text)
                .horizontal_align(Align::Center)
                .vertical_align(Align::Center)
                .font(font_id.clone())
                .background_color(theme.background_primary)
                .text_color(theme.foreground)
                .frame(true),
        );

        let next_page_button = top_row_user_interface.put(
            next_page_button_rectangle,
            Button::new_from_theme(theme)
                .background_color(Color32::TRANSPARENT)
                .with_tooltip_text("Next page in the current pointer context."),
        );
        IconDraw::draw(
            &top_row_user_interface,
            next_page_button.rect,
            &theme.icon_library.icon_handle_navigation_right_arrow_small,
        );
        if next_page_button.clicked() {
            should_navigate_next_page = true;
        }

        let last_page_button = top_row_user_interface.put(
            last_page_button_rectangle,
            Button::new_from_theme(theme)
                .background_color(Color32::TRANSPARENT)
                .with_tooltip_text("Last page in the current pointer context."),
        );
        IconDraw::draw(
            &top_row_user_interface,
            last_page_button.rect,
            &theme.icon_library.icon_handle_navigation_right_arrows,
        );
        if last_page_button.clicked() {
            should_navigate_last_page = true;
        }

        if page_number_edit_response.changed() {
            PointerScannerViewData::set_page_index_string(self.pointer_scanner_view_data.clone(), &page_label_text);
        }

        if bottom_row_height > 0.0 {
            let bottom_row_builder = UiBuilder::new()
                .max_rect(bottom_row_rectangle)
                .sense(Sense::hover());
            let mut bottom_row_user_interface = user_interface.new_child(bottom_row_builder);
            bottom_row_user_interface.set_clip_rect(bottom_row_rectangle);
            bottom_row_user_interface
                .painter()
                .with_clip_rect(bottom_row_rectangle)
                .text(
                    bottom_row_rectangle.center(),
                    Align2::CENTER_CENTER,
                    format!("{context_text} | {stats_text}"),
                    font_id.clone(),
                    theme.foreground,
                );
        }

        if should_navigate_first_page {
            PointerScannerViewData::navigate_first_page(self.pointer_scanner_view_data.clone());
        } else if should_navigate_previous_page {
            PointerScannerViewData::navigate_previous_page(self.pointer_scanner_view_data.clone());
        } else if should_navigate_next_page {
            PointerScannerViewData::navigate_next_page(self.pointer_scanner_view_data.clone());
        } else if should_navigate_last_page {
            PointerScannerViewData::navigate_last_page(self.pointer_scanner_view_data.clone());
        }

        response
    }
}

#[cfg(test)]
mod tests {
    use super::{PointerScannerFooterNavigationLayout, PointerScannerFooterView};

    fn assert_layout_fits(
        available_width: f32,
        navigation_layout: PointerScannerFooterNavigationLayout,
    ) {
        assert!(navigation_layout.button_width > 0.0);
        assert!(navigation_layout.page_box_width > 0.0);
        assert!(navigation_layout.total_width() <= available_width.max(1.0) + f32::EPSILON);
    }

    #[test]
    fn navigation_layout_fits_standard_panel_widths() {
        for available_width in [480.0, 320.0, 180.0, 96.0] {
            assert_layout_fits(available_width, PointerScannerFooterView::resolve_navigation_layout(available_width));
        }
    }

    #[test]
    fn navigation_layout_fits_extremely_narrow_panel_widths() {
        for available_width in [72.0, 48.0, 24.0] {
            assert_layout_fits(available_width, PointerScannerFooterView::resolve_navigation_layout(available_width));
        }
    }
}
