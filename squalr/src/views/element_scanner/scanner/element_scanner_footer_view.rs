use crate::{
    app_context::AppContext,
    ui::{draw::icon_draw::IconDraw, widgets::controls::button::Button},
    views::element_scanner::results::view_data::element_scanner_results_view_data::ElementScannerResultsViewData,
};
use eframe::egui::{Align, Response, RichText, Sense, TextEdit, Ui, UiBuilder, Widget};
use epaint::{Color32, CornerRadius, Rect, Stroke, StrokeKind, pos2, vec2};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use std::sync::Arc;

#[derive(Clone)]
pub struct ElementScannerFooterView {
    app_context: Arc<AppContext>,
    element_scanner_results_view_data: Dependency<ElementScannerResultsViewData>,
}

impl ElementScannerFooterView {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let element_scanner_results_view_data = app_context
            .dependency_container
            .get_dependency::<ElementScannerResultsViewData>();
        let instance = Self {
            app_context,
            element_scanner_results_view_data,
        };

        instance
    }

    pub fn get_height(&self) -> f32 {
        64.0
    }
}

impl Widget for ElementScannerFooterView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let height = self.get_height();
        let row_height = height * 0.5;

        let (allocated_size_rectangle, response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), height), Sense::empty());

        let theme = &self.app_context.theme;
        let font_id = theme.font_library.font_noto_sans.font_normal.clone();

        // Paint background.
        user_interface
            .painter()
            .rect_filled(allocated_size_rectangle, CornerRadius::ZERO, theme.background_primary);

        let top_row = Rect::from_min_size(allocated_size_rectangle.min, vec2(allocated_size_rectangle.width(), row_height));

        let bottom_row = Rect::from_min_size(
            pos2(allocated_size_rectangle.min.x, allocated_size_rectangle.min.y + row_height),
            vec2(allocated_size_rectangle.width(), row_height),
        );

        let element_scanner_view_data = match self
            .element_scanner_results_view_data
            .read("Element scanner footer view element scanner view data")
        {
            Some(element_scanner_view_data) => element_scanner_view_data,
            None => return response,
        };

        let border_width = 1.0;
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

        let mut should_navigate_first_page = false;
        let mut should_navigate_previous_page = false;
        let mut should_navigate_next_page = false;
        let mut should_navigate_last_page = false;

        let top_row_builder = UiBuilder::new().max_rect(top_row).sense(Sense::click());
        let mut top_row_user_interface = user_interface.new_child(top_row_builder);

        // Page editor (centered, drawn below buttons).
        let page_number_edit_rectangle = Rect::from_center_size(pos2(center_x, center_y), vec2(page_box_width, page_box_height));

        let mut text_value = element_scanner_view_data.current_page_index.to_string();

        let page_number_edit_response = top_row_user_interface.put(
            page_number_edit_rectangle,
            TextEdit::singleline(&mut text_value)
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
            Stroke::new(border_width, theme.submenu_border),
            StrokeKind::Inside,
        );

        // First page.
        let first_page_button_rectangle = Rect::from_min_size(pos2(first_page_button_x, center_y - button_height * 0.5), vec2(button_width, button_height));

        let first_page_button = top_row_user_interface.put(
            first_page_button_rectangle,
            Button::new_from_theme(&theme)
                .background_color(Color32::TRANSPARENT)
                .with_tooltip_text("First page."),
        );

        IconDraw::draw(
            &top_row_user_interface,
            first_page_button.rect,
            &theme.icon_library.icon_handle_navigation_left_arrows,
        );

        if first_page_button.clicked() {
            should_navigate_first_page = true;
        }

        // Previous page.
        let previous_page_button_rectangle =
            Rect::from_min_size(pos2(previous_page_button_x, center_y - button_height * 0.5), vec2(button_width, button_height));

        let previous_page_button = top_row_user_interface.put(
            previous_page_button_rectangle,
            Button::new_from_theme(&theme)
                .background_color(Color32::TRANSPARENT)
                .with_tooltip_text("Previous page."),
        );

        IconDraw::draw(
            &top_row_user_interface,
            previous_page_button.rect,
            &theme.icon_library.icon_handle_navigation_left_arrow,
        );

        if previous_page_button.clicked() {
            should_navigate_previous_page = true;
        }

        // Next page.
        let next_page_button_rectangle = Rect::from_min_size(pos2(next_page_button_x, center_y - button_height * 0.5), vec2(button_width, button_height));

        let next_page_button = top_row_user_interface.put(
            next_page_button_rectangle,
            Button::new_from_theme(&theme)
                .background_color(Color32::TRANSPARENT)
                .with_tooltip_text("Next page."),
        );

        IconDraw::draw(
            &top_row_user_interface,
            next_page_button.rect,
            &theme.icon_library.icon_handle_navigation_right_arrow,
        );

        if next_page_button.clicked() {
            should_navigate_next_page = true;
        }

        // Last page.
        let last_page_button_rectangle = Rect::from_min_size(pos2(last_page_button_x, center_y - button_height * 0.5), vec2(button_width, button_height));

        let last_page_button = top_row_user_interface.put(
            last_page_button_rectangle,
            Button::new_from_theme(&theme)
                .background_color(Color32::TRANSPARENT)
                .with_tooltip_text("Last page."),
        );

        IconDraw::draw(
            &top_row_user_interface,
            last_page_button.rect,
            &theme.icon_library.icon_handle_navigation_right_arrows,
        );

        if last_page_button.clicked() {
            should_navigate_last_page = true;
        }

        let bottom_row_builder = UiBuilder::new().max_rect(bottom_row);

        let mut bottom_row_user_interface = user_interface.new_child(bottom_row_builder);

        bottom_row_user_interface.centered_and_justified(|user_interface| {
            user_interface.label(
                RichText::new(&element_scanner_view_data.stats_string)
                    .font(font_id.clone())
                    .color(theme.foreground),
            );
        });

        drop(element_scanner_view_data);

        if should_navigate_first_page {
            ElementScannerResultsViewData::navigate_first_page(
                self.element_scanner_results_view_data.clone(),
                self.app_context.engine_unprivileged_state.clone(),
            );
        } else if should_navigate_previous_page {
            ElementScannerResultsViewData::navigate_previous_page(
                self.element_scanner_results_view_data.clone(),
                self.app_context.engine_unprivileged_state.clone(),
            );
        } else if should_navigate_next_page {
            ElementScannerResultsViewData::navigate_next_page(
                self.element_scanner_results_view_data.clone(),
                self.app_context.engine_unprivileged_state.clone(),
            );
        } else if should_navigate_last_page {
            ElementScannerResultsViewData::navigate_last_page(
                self.element_scanner_results_view_data.clone(),
                self.app_context.engine_unprivileged_state.clone(),
            );
        } else if page_number_edit_response.changed() {
            ElementScannerResultsViewData::set_page_index_string(
                self.element_scanner_results_view_data.clone(),
                self.app_context.engine_unprivileged_state.clone(),
                &text_value,
            );
        }

        response
    }
}
