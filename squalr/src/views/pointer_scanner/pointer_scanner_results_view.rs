use crate::app_context::AppContext;
use crate::ui::draw::icon_draw::IconDraw;
use crate::views::pointer_scanner::view_data::pointer_scanner_view_data::{PointerScannerTreeRow, PointerScannerViewData};
use eframe::egui::{Align2, CursorIcon, Response, ScrollArea, Sense, Ui, Widget, pos2, vec2};
use epaint::{Color32, CornerRadius, Rect, Stroke, StrokeKind};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use std::sync::Arc;

#[derive(Clone)]
pub struct PointerScannerResultsView {
    app_context: Arc<AppContext>,
    pointer_scanner_view_data: Dependency<PointerScannerViewData>,
}

impl PointerScannerResultsView {
    const COLUMN_SEPARATOR_THICKNESS: f32 = 2.0;
    const ROW_HEIGHT: f32 = 22.0;

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let pointer_scanner_view_data = app_context
            .dependency_container
            .get_dependency::<PointerScannerViewData>();

        Self {
            app_context,
            pointer_scanner_view_data,
        }
    }

    fn column_positions(rectangle: Rect) -> (f32, f32, f32, f32, f32) {
        let total_width = rectangle.width();
        let module_base_x = rectangle.min.x + 12.0;
        let offset_chain_x = rectangle.min.x + total_width * 0.36;
        let resolved_address_x = rectangle.min.x + total_width * 0.66;
        let depth_x = rectangle.min.x + total_width * 0.83;
        let state_x = rectangle.min.x + total_width * 0.91;

        (module_base_x, offset_chain_x, resolved_address_x, depth_x, state_x)
    }

    fn draw_column_separators(
        &self,
        user_interface: &mut Ui,
        rectangle: Rect,
    ) {
        let theme = &self.app_context.theme;
        let (_module_base_x, offset_chain_x, resolved_address_x, depth_x, state_x) = Self::column_positions(rectangle);

        for separator_x in [
            offset_chain_x - 10.0,
            resolved_address_x - 10.0,
            depth_x - 10.0,
            state_x - 10.0,
        ] {
            let separator_rectangle = Rect::from_min_max(
                pos2(separator_x - Self::COLUMN_SEPARATOR_THICKNESS * 0.5, rectangle.min.y),
                pos2(separator_x + Self::COLUMN_SEPARATOR_THICKNESS * 0.5, rectangle.max.y),
            );

            user_interface
                .painter()
                .rect_filled(separator_rectangle, CornerRadius::ZERO, theme.background_control);
        }
    }

    fn draw_header(
        &self,
        user_interface: &mut Ui,
    ) {
        let theme = &self.app_context.theme;
        let header_height = 26.0;
        let (header_rectangle, _) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), header_height), Sense::hover());
        let (module_base_x, offset_chain_x, resolved_address_x, depth_x, state_x) = Self::column_positions(header_rectangle);

        user_interface
            .painter()
            .rect_filled(header_rectangle, CornerRadius::ZERO, theme.background_primary);
        user_interface.painter().rect_stroke(
            header_rectangle,
            CornerRadius::ZERO,
            Stroke::new(1.0, theme.background_control),
            StrokeKind::Inside,
        );
        self.draw_column_separators(user_interface, header_rectangle);

        let header_font = theme.font_library.font_noto_sans.font_normal.clone();
        let header_y = header_rectangle.center().y;

        user_interface.painter().text(
            pos2(module_base_x, header_y),
            Align2::LEFT_CENTER,
            "Module / Base",
            header_font.clone(),
            theme.foreground,
        );
        user_interface.painter().text(
            pos2(offset_chain_x, header_y),
            Align2::LEFT_CENTER,
            "Offset Chain",
            header_font.clone(),
            theme.foreground,
        );
        user_interface.painter().text(
            pos2(resolved_address_x, header_y),
            Align2::LEFT_CENTER,
            "Resolved",
            header_font.clone(),
            theme.foreground,
        );
        user_interface
            .painter()
            .text(pos2(depth_x, header_y), Align2::LEFT_CENTER, "Depth", header_font.clone(), theme.foreground);
        user_interface
            .painter()
            .text(pos2(state_x, header_y), Align2::LEFT_CENTER, "State", header_font, theme.foreground);
    }

    fn draw_row(
        &self,
        user_interface: &mut Ui,
        pointer_scanner_tree_row: &PointerScannerTreeRow,
        clicked_node_id: &mut Option<u64>,
        toggled_node_id: &mut Option<u64>,
    ) {
        let theme = &self.app_context.theme;
        let (row_rectangle, row_response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), Self::ROW_HEIGHT), Sense::click());
        let (module_base_x, offset_chain_x, resolved_address_x, depth_x, state_x) = Self::column_positions(row_rectangle);
        let indent_width = 16.0 * pointer_scanner_tree_row.tree_depth as f32;
        let toggle_rectangle = Rect::from_min_size(pos2(module_base_x + indent_width, row_rectangle.center().y - 6.0), vec2(12.0, 12.0));
        let toggle_response = if pointer_scanner_tree_row.has_children {
            user_interface.interact(
                toggle_rectangle,
                user_interface
                    .id()
                    .with(("pointer_scanner_toggle", pointer_scanner_tree_row.node_id)),
                Sense::click(),
            )
        } else {
            user_interface.interact(
                toggle_rectangle,
                user_interface
                    .id()
                    .with(("pointer_scanner_toggle_disabled", pointer_scanner_tree_row.node_id)),
                Sense::hover(),
            )
        };

        let row_background = if pointer_scanner_tree_row.is_selected {
            theme.selected_background
        } else if row_response.hovered() {
            theme.hover_tint
        } else {
            Color32::TRANSPARENT
        };

        user_interface
            .painter()
            .rect_filled(row_rectangle, CornerRadius::ZERO, row_background);
        user_interface.painter().rect_stroke(
            row_rectangle,
            CornerRadius::ZERO,
            Stroke::new(1.0, theme.background_control),
            StrokeKind::Inside,
        );
        self.draw_column_separators(user_interface, row_rectangle);

        if pointer_scanner_tree_row.has_children {
            let toggle_icon = if pointer_scanner_tree_row.is_expanded {
                &theme.icon_library.icon_handle_navigation_down_arrow_small
            } else {
                &theme.icon_library.icon_handle_navigation_right_arrow_small
            };

            IconDraw::draw_sized(user_interface, toggle_rectangle.center(), vec2(10.0, 10.0), toggle_icon);
        }

        let text_x = module_base_x + indent_width + if pointer_scanner_tree_row.has_children { 16.0 } else { 0.0 };
        let row_center_y = row_rectangle.center().y;
        let module_font = theme.font_library.font_ubuntu_mono_bold.font_normal.clone();
        let value_font = theme.font_library.font_ubuntu_mono_bold.font_normal.clone();
        let state_font = theme.font_library.font_noto_sans.font_small.clone();
        let state_color = if pointer_scanner_tree_row.state_text == "Static" {
            theme.selected_border
        } else {
            theme.foreground
        };

        user_interface.painter().text(
            pos2(text_x, row_center_y),
            Align2::LEFT_CENTER,
            &pointer_scanner_tree_row.module_base_text,
            module_font.clone(),
            theme.foreground,
        );
        user_interface.painter().text(
            pos2(offset_chain_x, row_center_y),
            Align2::LEFT_CENTER,
            &pointer_scanner_tree_row.offset_chain_text,
            value_font.clone(),
            theme.foreground,
        );
        user_interface.painter().text(
            pos2(resolved_address_x, row_center_y),
            Align2::LEFT_CENTER,
            &pointer_scanner_tree_row.resolved_address_text,
            value_font.clone(),
            theme.foreground,
        );
        user_interface.painter().text(
            pos2(depth_x, row_center_y),
            Align2::LEFT_CENTER,
            &pointer_scanner_tree_row.depth_text,
            value_font.clone(),
            theme.foreground,
        );
        user_interface.painter().text(
            pos2(state_x, row_center_y),
            Align2::LEFT_CENTER,
            &pointer_scanner_tree_row.state_text,
            state_font,
            state_color,
        );

        if row_response.hovered() {
            user_interface.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }

        if row_response.clicked() {
            *clicked_node_id = Some(pointer_scanner_tree_row.node_id);
        }

        if row_response.double_clicked() && pointer_scanner_tree_row.has_children {
            *toggled_node_id = Some(pointer_scanner_tree_row.node_id);
        }

        if toggle_response.clicked() {
            *toggled_node_id = Some(pointer_scanner_tree_row.node_id);
        }
    }
}

impl Widget for PointerScannerResultsView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let visible_row_count = PointerScannerViewData::get_visible_row_count(self.pointer_scanner_view_data.clone());
        let mut clicked_node_id = None;
        let mut toggled_node_id = None;
        let response = user_interface
            .allocate_ui_with_layout(
                user_interface.available_size(),
                eframe::egui::Layout::top_down(eframe::egui::Align::Min),
                |user_interface| {
                    self.draw_header(user_interface);

                    ScrollArea::vertical()
                        .id_salt("pointer_scanner_rows")
                        .auto_shrink([false, false])
                        .show_rows(user_interface, Self::ROW_HEIGHT, visible_row_count, |user_interface, row_range| {
                            let pointer_scanner_tree_rows =
                                PointerScannerViewData::build_visible_rows_in_range(self.pointer_scanner_view_data.clone(), row_range);

                            for pointer_scanner_tree_row in &pointer_scanner_tree_rows {
                                self.draw_row(user_interface, pointer_scanner_tree_row, &mut clicked_node_id, &mut toggled_node_id);
                            }
                        });
                },
            )
            .response;

        if let Some(clicked_node_id) = clicked_node_id {
            PointerScannerViewData::select_node(self.pointer_scanner_view_data.clone(), clicked_node_id);
        }

        if let Some(toggled_node_id) = toggled_node_id {
            PointerScannerViewData::toggle_node_expansion(
                self.pointer_scanner_view_data.clone(),
                self.app_context.engine_unprivileged_state.clone(),
                toggled_node_id,
            );
        }

        response
    }
}
