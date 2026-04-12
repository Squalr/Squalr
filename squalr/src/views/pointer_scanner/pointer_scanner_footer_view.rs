use crate::{
    app_context::AppContext,
    ui::{draw::icon_draw::IconDraw, widgets::controls::button::Button},
    views::pointer_scanner::view_data::pointer_scanner_view_data::PointerScannerViewData,
};
use eframe::egui::{Align, Response, Sense, TextEdit, Ui, UiBuilder, Widget};
use epaint::{Color32, CornerRadius, Rect, pos2, vec2};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use std::sync::Arc;

#[derive(Clone)]
pub struct PointerScannerFooterView {
    app_context: Arc<AppContext>,
    pointer_scanner_view_data: Dependency<PointerScannerViewData>,
}

impl PointerScannerFooterView {
    pub const FOOTER_HEIGHT: f32 = 32.0;

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
}

impl Widget for PointerScannerFooterView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let height = self
            .get_height()
            .min(user_interface.available_height().max(0.0));
        let (allocated_size_rectangle, response) = user_interface.allocate_exact_size(vec2(user_interface.available_width().max(1.0), height), Sense::empty());

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

        let row_rectangle = allocated_size_rectangle;
        let page_box_width = 112.0;
        let page_box_height = 24.0;
        let button_width = 36.0;
        let button_height = 28.0;
        let spacing = 6.0;
        let center_x = row_rectangle.center().x;
        let center_y = row_rectangle.center().y;

        let previous_page_button_x = center_x - page_box_width * 0.5 - spacing - button_width;
        let first_page_button_x = previous_page_button_x - button_width;
        let next_page_button_x = center_x + page_box_width * 0.5 + spacing;
        let last_page_button_x = next_page_button_x + button_width;

        let mut should_navigate_first_page = false;
        let mut should_navigate_previous_page = false;
        let mut should_navigate_next_page = false;
        let mut should_navigate_last_page = false;
        let mut page_label_text = page_label_text;

        let row_builder = UiBuilder::new().max_rect(row_rectangle).sense(Sense::click());
        let mut row_user_interface = user_interface.new_child(row_builder);
        row_user_interface.set_clip_rect(row_rectangle);

        let page_number_edit_rectangle = Rect::from_center_size(pos2(center_x, center_y), vec2(page_box_width, page_box_height));
        let page_number_edit_response = row_user_interface.put(
            page_number_edit_rectangle,
            TextEdit::singleline(&mut page_label_text)
                .horizontal_align(Align::Center)
                .vertical_align(Align::Center)
                .font(font_id.clone())
                .background_color(theme.background_primary)
                .text_color(theme.foreground)
                .frame(true),
        );

        let first_page_button_rectangle = Rect::from_min_size(pos2(first_page_button_x, center_y - button_height * 0.5), vec2(button_width, button_height));
        let first_page_button = row_user_interface.put(
            first_page_button_rectangle,
            Button::new_from_theme(theme)
                .background_color(Color32::TRANSPARENT)
                .with_tooltip_text("First page in the current pointer context."),
        );
        IconDraw::draw(
            &row_user_interface,
            first_page_button.rect,
            &theme.icon_library.icon_handle_navigation_left_arrows,
        );
        if first_page_button.clicked() {
            should_navigate_first_page = true;
        }

        let previous_page_button_rectangle =
            Rect::from_min_size(pos2(previous_page_button_x, center_y - button_height * 0.5), vec2(button_width, button_height));
        let previous_page_button = row_user_interface.put(
            previous_page_button_rectangle,
            Button::new_from_theme(theme)
                .background_color(Color32::TRANSPARENT)
                .with_tooltip_text("Previous page in the current pointer context."),
        );
        IconDraw::draw(
            &row_user_interface,
            previous_page_button.rect,
            &theme.icon_library.icon_handle_navigation_left_arrow_small,
        );
        if previous_page_button.clicked() {
            should_navigate_previous_page = true;
        }

        let next_page_button_rectangle = Rect::from_min_size(pos2(next_page_button_x, center_y - button_height * 0.5), vec2(button_width, button_height));
        let next_page_button = row_user_interface.put(
            next_page_button_rectangle,
            Button::new_from_theme(theme)
                .background_color(Color32::TRANSPARENT)
                .with_tooltip_text("Next page in the current pointer context."),
        );
        IconDraw::draw(
            &row_user_interface,
            next_page_button.rect,
            &theme.icon_library.icon_handle_navigation_right_arrow_small,
        );
        if next_page_button.clicked() {
            should_navigate_next_page = true;
        }

        let last_page_button_rectangle = Rect::from_min_size(pos2(last_page_button_x, center_y - button_height * 0.5), vec2(button_width, button_height));
        let last_page_button = row_user_interface.put(
            last_page_button_rectangle,
            Button::new_from_theme(theme)
                .background_color(Color32::TRANSPARENT)
                .with_tooltip_text("Last page in the current pointer context."),
        );
        IconDraw::draw(
            &row_user_interface,
            last_page_button.rect,
            &theme.icon_library.icon_handle_navigation_right_arrows,
        );
        if last_page_button.clicked() {
            should_navigate_last_page = true;
        }

        let stats_text_rectangle = Rect::from_min_max(pos2(last_page_button_rectangle.max.x + 12.0, row_rectangle.min.y), row_rectangle.max);
        row_user_interface
            .painter()
            .with_clip_rect(stats_text_rectangle.intersect(row_rectangle))
            .text(
                stats_text_rectangle.left_center(),
                eframe::egui::Align2::LEFT_CENTER,
                format!("{context_text} | {stats_text}"),
                font_id,
                theme.foreground,
            );

        if should_navigate_first_page {
            PointerScannerViewData::navigate_first_page(self.pointer_scanner_view_data.clone());
        } else if should_navigate_previous_page {
            PointerScannerViewData::navigate_previous_page(self.pointer_scanner_view_data.clone());
        } else if should_navigate_next_page {
            PointerScannerViewData::navigate_next_page(self.pointer_scanner_view_data.clone());
        } else if should_navigate_last_page {
            PointerScannerViewData::navigate_last_page(self.pointer_scanner_view_data.clone());
        } else if page_number_edit_response.changed() {
            PointerScannerViewData::set_page_index_string(self.pointer_scanner_view_data.clone(), &page_label_text);
        }

        response
    }
}
