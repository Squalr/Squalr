use crate::{
    app_context::AppContext,
    ui::{draw::icon_draw::IconDraw, widgets::controls::button::Button},
    views::memory_viewer::view_data::memory_viewer_view_data::MemoryViewerViewData,
};
use eframe::egui::{Align, Response, RichText, Sense, TextEdit, Ui, UiBuilder, Widget};
use epaint::{Color32, CornerRadius, Rect, Stroke, StrokeKind, pos2, vec2};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use std::sync::Arc;

#[derive(Clone)]
pub struct MemoryViewerFooterView {
    app_context: Arc<AppContext>,
    memory_viewer_view_data: Dependency<MemoryViewerViewData>,
}

impl MemoryViewerFooterView {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        Self {
            memory_viewer_view_data: app_context
                .dependency_container
                .get_dependency::<MemoryViewerViewData>(),
            app_context,
        }
    }

    pub fn get_height(&self) -> f32 {
        72.0
    }
}

impl Widget for MemoryViewerFooterView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let height = self.get_height();
        let row_height = height * 0.5;
        let font_id = theme.font_library.font_noto_sans.font_normal.clone();
        let (allocated_size_rectangle, response) = user_interface.allocate_exact_size(vec2(user_interface.available_width().max(1.0), height), Sense::empty());

        user_interface
            .painter()
            .rect_filled(allocated_size_rectangle, CornerRadius::ZERO, theme.background_primary);

        let top_row = Rect::from_min_size(allocated_size_rectangle.min, vec2(allocated_size_rectangle.width(), row_height));
        let bottom_row = Rect::from_min_size(
            pos2(allocated_size_rectangle.min.x, allocated_size_rectangle.min.y + row_height),
            vec2(allocated_size_rectangle.width(), row_height),
        );
        let page_box_width = 160.0;
        let page_box_height = 24.0;
        let button_width = 36.0;
        let button_height = 28.0;
        let spacing = 6.0;
        let center_x = top_row.center().x;
        let center_y = top_row.center().y;
        let previous_page_button_x = center_x - page_box_width * 0.5 - spacing - button_width;
        let first_page_button_x = previous_page_button_x - button_width;
        let next_page_button_x = center_x + page_box_width * 0.5 + spacing;
        let last_page_button_x = next_page_button_x + button_width;
        let page_stats_text = self
            .memory_viewer_view_data
            .read("Memory viewer footer stats")
            .map(|memory_viewer_view_data| memory_viewer_view_data.stats_string.clone())
            .unwrap_or_else(|| String::from("No page selected."));
        let mut top_row_user_interface = user_interface.new_child(UiBuilder::new().max_rect(top_row).sense(Sense::click()));
        let page_number_edit_rectangle = Rect::from_center_size(pos2(center_x, center_y), vec2(page_box_width, page_box_height));
        let mut page_index_text = MemoryViewerViewData::get_current_page_index_string(self.memory_viewer_view_data.clone());
        let page_number_edit_response = top_row_user_interface.put(
            page_number_edit_rectangle,
            TextEdit::singleline(&mut page_index_text)
                .horizontal_align(Align::Center)
                .vertical_align(Align::Center)
                .font(font_id.clone())
                .background_color(theme.background_primary)
                .text_color(theme.foreground)
                .frame(true),
        );

        top_row_user_interface.painter().rect_stroke(
            page_number_edit_rectangle,
            CornerRadius::ZERO,
            Stroke::new(1.0, theme.submenu_border),
            StrokeKind::Inside,
        );

        let first_page_button = top_row_user_interface.put(
            Rect::from_min_size(pos2(first_page_button_x, center_y - button_height * 0.5), vec2(button_width, button_height)),
            Button::new_from_theme(theme)
                .background_color(Color32::TRANSPARENT)
                .with_tooltip_text("First page."),
        );
        IconDraw::draw(
            &top_row_user_interface,
            first_page_button.rect,
            &theme.icon_library.icon_handle_navigation_left_arrows,
        );
        let should_navigate_first_page = first_page_button.clicked();

        let previous_page_button = top_row_user_interface.put(
            Rect::from_min_size(pos2(previous_page_button_x, center_y - button_height * 0.5), vec2(button_width, button_height)),
            Button::new_from_theme(theme)
                .background_color(Color32::TRANSPARENT)
                .with_tooltip_text("Previous page."),
        );
        IconDraw::draw(
            &top_row_user_interface,
            previous_page_button.rect,
            &theme.icon_library.icon_handle_navigation_left_arrow,
        );
        let should_navigate_previous_page = previous_page_button.clicked();

        let next_page_button = top_row_user_interface.put(
            Rect::from_min_size(pos2(next_page_button_x, center_y - button_height * 0.5), vec2(button_width, button_height)),
            Button::new_from_theme(theme)
                .background_color(Color32::TRANSPARENT)
                .with_tooltip_text("Next page."),
        );
        IconDraw::draw(
            &top_row_user_interface,
            next_page_button.rect,
            &theme.icon_library.icon_handle_navigation_right_arrow,
        );
        let should_navigate_next_page = next_page_button.clicked();

        let last_page_button = top_row_user_interface.put(
            Rect::from_min_size(pos2(last_page_button_x, center_y - button_height * 0.5), vec2(button_width, button_height)),
            Button::new_from_theme(theme)
                .background_color(Color32::TRANSPARENT)
                .with_tooltip_text("Last page."),
        );
        IconDraw::draw(
            &top_row_user_interface,
            last_page_button.rect,
            &theme.icon_library.icon_handle_navigation_right_arrows,
        );
        let should_navigate_last_page = last_page_button.clicked();

        let mut bottom_row_user_interface = user_interface.new_child(UiBuilder::new().max_rect(bottom_row));
        bottom_row_user_interface.centered_and_justified(|user_interface| {
            user_interface.label(
                RichText::new(page_stats_text)
                    .font(font_id)
                    .color(theme.foreground),
            );
        });

        if should_navigate_first_page {
            MemoryViewerViewData::navigate_first_page(self.memory_viewer_view_data.clone());
        } else if should_navigate_previous_page {
            MemoryViewerViewData::navigate_previous_page(self.memory_viewer_view_data.clone());
        } else if should_navigate_next_page {
            MemoryViewerViewData::navigate_next_page(self.memory_viewer_view_data.clone());
        } else if should_navigate_last_page {
            MemoryViewerViewData::navigate_last_page(self.memory_viewer_view_data.clone());
        } else if page_number_edit_response.changed() {
            MemoryViewerViewData::set_page_index_string(self.memory_viewer_view_data.clone(), &page_index_text);
        }

        response
    }
}
