use crate::{
    app_context::AppContext,
    ui::{draw::icon_draw::IconDraw, widgets::controls::button::Button},
    views::pointer_scanner::view_data::pointer_scanner_view_data::PointerScannerViewData,
};
use eframe::egui::{Align, Layout, Response, Sense, TextEdit, Ui, UiBuilder, Widget, vec2};
use epaint::{Color32, CornerRadius, Rect, pos2};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use std::sync::Arc;

#[derive(Clone)]
pub struct PointerScannerFooterView {
    app_context: Arc<AppContext>,
    pointer_scanner_view_data: Dependency<PointerScannerViewData>,
}

impl PointerScannerFooterView {
    pub const FOOTER_HEIGHT: f32 = 56.0;

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
        let top_row_height = 28.0_f32.min(height);
        let bottom_row_height = (height - top_row_height).max(0.0);
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

        let page_box_width = 112.0;
        let page_box_height = 24.0;
        let button_width = 36.0;
        let button_height = 28.0;
        let top_row_rectangle = Rect::from_min_max(
            allocated_size_rectangle.min,
            pos2(allocated_size_rectangle.max.x, allocated_size_rectangle.min.y + top_row_height),
        );
        let bottom_row_rectangle = Rect::from_min_max(pos2(allocated_size_rectangle.min.x, top_row_rectangle.max.y), allocated_size_rectangle.max);

        let mut should_navigate_first_page = false;
        let mut should_navigate_previous_page = false;
        let mut should_navigate_next_page = false;
        let mut should_navigate_last_page = false;
        let mut page_label_text = page_label_text;

        let top_row_builder = UiBuilder::new()
            .max_rect(top_row_rectangle)
            .layout(Layout::left_to_right(Align::Center))
            .sense(Sense::click());
        let mut top_row_user_interface = user_interface.new_child(top_row_builder);
        top_row_user_interface.set_clip_rect(top_row_rectangle);
        top_row_user_interface.horizontal_centered(|user_interface| {
            let first_page_button = user_interface.add_sized(
                vec2(button_width, button_height),
                Button::new_from_theme(theme)
                    .background_color(Color32::TRANSPARENT)
                    .with_tooltip_text("First page in the current pointer context."),
            );
            IconDraw::draw(user_interface, first_page_button.rect, &theme.icon_library.icon_handle_navigation_left_arrows);
            if first_page_button.clicked() {
                should_navigate_first_page = true;
            }

            let previous_page_button = user_interface.add_sized(
                vec2(button_width, button_height),
                Button::new_from_theme(theme)
                    .background_color(Color32::TRANSPARENT)
                    .with_tooltip_text("Previous page in the current pointer context."),
            );
            IconDraw::draw(
                user_interface,
                previous_page_button.rect,
                &theme.icon_library.icon_handle_navigation_left_arrow_small,
            );
            if previous_page_button.clicked() {
                should_navigate_previous_page = true;
            }

            let page_number_edit_response = user_interface.add_sized(
                vec2(page_box_width, page_box_height),
                TextEdit::singleline(&mut page_label_text)
                    .horizontal_align(Align::Center)
                    .vertical_align(Align::Center)
                    .font(font_id.clone())
                    .background_color(theme.background_primary)
                    .text_color(theme.foreground)
                    .frame(true),
            );

            let next_page_button = user_interface.add_sized(
                vec2(button_width, button_height),
                Button::new_from_theme(theme)
                    .background_color(Color32::TRANSPARENT)
                    .with_tooltip_text("Next page in the current pointer context."),
            );
            IconDraw::draw(
                user_interface,
                next_page_button.rect,
                &theme.icon_library.icon_handle_navigation_right_arrow_small,
            );
            if next_page_button.clicked() {
                should_navigate_next_page = true;
            }

            let last_page_button = user_interface.add_sized(
                vec2(button_width, button_height),
                Button::new_from_theme(theme)
                    .background_color(Color32::TRANSPARENT)
                    .with_tooltip_text("Last page in the current pointer context."),
            );
            IconDraw::draw(user_interface, last_page_button.rect, &theme.icon_library.icon_handle_navigation_right_arrows);
            if last_page_button.clicked() {
                should_navigate_last_page = true;
            }

            if page_number_edit_response.changed() {
                PointerScannerViewData::set_page_index_string(self.pointer_scanner_view_data.clone(), &page_label_text);
            }
        });

        if bottom_row_height > 0.0 {
            let bottom_row_builder = UiBuilder::new()
                .max_rect(bottom_row_rectangle)
                .sense(Sense::hover());
            let mut bottom_row_user_interface = user_interface.new_child(bottom_row_builder);
            bottom_row_user_interface.set_clip_rect(bottom_row_rectangle);
            bottom_row_user_interface.centered_and_justified(|user_interface| {
                user_interface.label(
                    eframe::egui::RichText::new(format!("{context_text} | {stats_text}"))
                        .font(font_id.clone())
                        .color(theme.foreground),
                );
            });
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
