use crate::app_context::AppContext;
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
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let pointer_scanner_view_data = app_context
            .dependency_container
            .get_dependency::<PointerScannerViewData>();

        Self {
            app_context,
            pointer_scanner_view_data,
        }
    }

    fn draw_header(
        &self,
        user_interface: &mut Ui,
    ) {
        let theme = &self.app_context.theme;
        let header_height = 28.0;
        let (header_rectangle, _) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), header_height), Sense::hover());
        let total_width = header_rectangle.width();
        let module_base_x = header_rectangle.min.x + 12.0;
        let offset_chain_x = header_rectangle.min.x + total_width * 0.38;
        let resolved_address_x = header_rectangle.min.x + total_width * 0.67;
        let depth_x = header_rectangle.min.x + total_width * 0.83;
        let state_x = header_rectangle.min.x + total_width * 0.91;

        user_interface
            .painter()
            .rect_filled(header_rectangle, CornerRadius::ZERO, theme.background_primary);
        user_interface.painter().rect_stroke(
            header_rectangle,
            CornerRadius::ZERO,
            Stroke::new(1.0, theme.background_control),
            StrokeKind::Inside,
        );

        let header_font = theme.font_library.font_noto_sans.font_header.clone();
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
        let row_height = 24.0;
        let (row_rectangle, row_response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), row_height), Sense::click());
        let total_width = row_rectangle.width();
        let module_base_x = row_rectangle.min.x + 12.0;
        let offset_chain_x = row_rectangle.min.x + total_width * 0.38;
        let resolved_address_x = row_rectangle.min.x + total_width * 0.67;
        let depth_x = row_rectangle.min.x + total_width * 0.83;
        let state_x = row_rectangle.min.x + total_width * 0.91;
        let indent_width = 18.0 * pointer_scanner_tree_row.tree_depth as f32;
        let toggle_rectangle = Rect::from_min_size(pos2(module_base_x + indent_width, row_rectangle.center().y - 8.0), vec2(16.0, 16.0));
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

        if pointer_scanner_tree_row.has_children {
            let toggle_text = if pointer_scanner_tree_row.is_expanded { "v" } else { ">" };

            user_interface.painter().text(
                toggle_rectangle.center(),
                Align2::CENTER_CENTER,
                toggle_text,
                theme.font_library.font_noto_sans.font_normal.clone(),
                theme.foreground,
            );
        }

        let text_x = module_base_x + indent_width + if pointer_scanner_tree_row.has_children { 20.0 } else { 0.0 };
        let row_center_y = row_rectangle.center().y;
        let text_font = theme.font_library.font_noto_sans.font_normal.clone();
        let state_color = if pointer_scanner_tree_row.state_text == "Static" {
            theme.selected_border
        } else {
            theme.foreground
        };

        user_interface.painter().text(
            pos2(text_x, row_center_y),
            Align2::LEFT_CENTER,
            &pointer_scanner_tree_row.module_base_text,
            text_font.clone(),
            theme.foreground,
        );
        user_interface.painter().text(
            pos2(offset_chain_x, row_center_y),
            Align2::LEFT_CENTER,
            &pointer_scanner_tree_row.offset_chain_text,
            text_font.clone(),
            theme.foreground,
        );
        user_interface.painter().text(
            pos2(resolved_address_x, row_center_y),
            Align2::LEFT_CENTER,
            &pointer_scanner_tree_row.resolved_address_text,
            text_font.clone(),
            theme.foreground,
        );
        user_interface.painter().text(
            pos2(depth_x, row_center_y),
            Align2::LEFT_CENTER,
            &pointer_scanner_tree_row.depth_text,
            text_font.clone(),
            theme.foreground,
        );
        user_interface.painter().text(
            pos2(state_x, row_center_y),
            Align2::LEFT_CENTER,
            &pointer_scanner_tree_row.state_text,
            text_font,
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
        let pointer_scanner_tree_rows = PointerScannerViewData::build_visible_rows(self.pointer_scanner_view_data.clone());
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
                        .show(user_interface, |user_interface| {
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
