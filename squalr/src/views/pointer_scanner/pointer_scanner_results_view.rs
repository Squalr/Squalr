use crate::app_context::AppContext;
use crate::ui::draw::icon_draw::IconDraw;
use crate::views::pointer_scanner::pointer_scanner_footer_view::PointerScannerFooterView;
use crate::views::pointer_scanner::view_data::pointer_scanner_view_data::{PointerScannerTreeRow, PointerScannerViewData};
use crate::views::project_explorer::project_hierarchy::view_data::project_hierarchy_view_data::ProjectHierarchyViewData;
use eframe::egui::{Align, Align2, CursorIcon, Layout, Response, ScrollArea, Sense, Ui, UiBuilder, Widget, pos2, vec2};
use epaint::{Color32, CornerRadius, Rect, Stroke, StrokeKind};
use squalr_engine_api::{commands::unprivileged_command_request::UnprivilegedCommandRequest, dependency_injection::dependency::Dependency};
use std::sync::Arc;

#[derive(Clone)]
pub struct PointerScannerResultsView {
    app_context: Arc<AppContext>,
    pointer_scanner_view_data: Dependency<PointerScannerViewData>,
    project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
    pointer_scanner_footer_view: PointerScannerFooterView,
}

impl PointerScannerResultsView {
    const COLUMN_SEPARATOR_THICKNESS: f32 = 2.0;
    const HEADER_HEIGHT: f32 = 26.0;
    const ROW_HEIGHT: f32 = 22.0;

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let pointer_scanner_view_data = app_context
            .dependency_container
            .get_dependency::<PointerScannerViewData>();
        let project_hierarchy_view_data = app_context
            .dependency_container
            .get_dependency::<ProjectHierarchyViewData>();
        let pointer_scanner_footer_view = PointerScannerFooterView::new(app_context.clone());

        Self {
            app_context,
            pointer_scanner_view_data,
            project_hierarchy_view_data,
            pointer_scanner_footer_view,
        }
    }

    fn column_positions(rectangle: Rect) -> (f32, f32, f32, f32, f32, f32) {
        let total_width = rectangle.width();
        let location_x = rectangle.min.x + 12.0;
        let offset_x = rectangle.min.x + total_width * 0.34;
        let resolved_address_x = rectangle.min.x + total_width * 0.57;
        let depth_x = rectangle.min.x + total_width * 0.76;
        let state_x = rectangle.min.x + total_width * 0.84;
        let action_x = rectangle.min.x + total_width * 0.92;

        (location_x, offset_x, resolved_address_x, depth_x, state_x, action_x)
    }

    fn draw_column_separators(
        &self,
        user_interface: &mut Ui,
        rectangle: Rect,
    ) {
        let theme = &self.app_context.theme;
        let (_location_x, offset_x, resolved_address_x, depth_x, state_x, action_x) = Self::column_positions(rectangle);

        for separator_x in [
            offset_x - 10.0,
            resolved_address_x - 10.0,
            depth_x - 10.0,
            state_x - 10.0,
            action_x - 10.0,
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
        header_rectangle: Rect,
        is_root_context: bool,
    ) {
        let theme = &self.app_context.theme;
        let (location_x, offset_x, resolved_address_x, depth_x, state_x, action_x) = Self::column_positions(header_rectangle);
        let primary_label = if is_root_context { "Root" } else { "Offset" };

        user_interface
            .painter()
            .rect_filled(header_rectangle, CornerRadius::ZERO, theme.background_primary);
        user_interface.painter().rect_stroke(
            header_rectangle,
            CornerRadius::ZERO,
            Stroke::new(1.0, theme.background_control),
            StrokeKind::Inside,
        );

        let header_font = theme.font_library.font_noto_sans.font_normal.clone();
        let header_y = header_rectangle.center().y;

        user_interface.painter().text(
            pos2(location_x, header_y),
            Align2::LEFT_CENTER,
            primary_label,
            header_font.clone(),
            theme.foreground,
        );
        user_interface
            .painter()
            .text(pos2(offset_x, header_y), Align2::LEFT_CENTER, "Pointer", header_font.clone(), theme.foreground);
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
        user_interface.painter().text(
            pos2(action_x, header_y),
            Align2::LEFT_CENTER,
            "Action",
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground,
        );
    }

    fn draw_row(
        &self,
        user_interface: &mut Ui,
        pointer_scanner_tree_row: &PointerScannerTreeRow,
        clicked_node_id: &mut Option<u64>,
        entered_node_id: &mut Option<u64>,
        added_node_id: &mut Option<u64>,
    ) {
        let theme = &self.app_context.theme;
        let (row_rectangle, row_response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), Self::ROW_HEIGHT), Sense::click());
        let (location_x, offset_x, resolved_address_x, depth_x, state_x, action_x) = Self::column_positions(row_rectangle);
        let action_rectangle = Rect::from_center_size(pos2(action_x + 10.0, row_rectangle.center().y), vec2(16.0, 16.0));
        let action_response = if pointer_scanner_tree_row.has_children {
            user_interface.interact(
                action_rectangle,
                user_interface
                    .id()
                    .with(("pointer_scanner_enter", pointer_scanner_tree_row.node_id)),
                Sense::click(),
            )
        } else {
            user_interface.interact(
                action_rectangle,
                user_interface
                    .id()
                    .with(("pointer_scanner_add", pointer_scanner_tree_row.node_id)),
                Sense::click(),
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
            IconDraw::draw_sized(
                user_interface,
                action_rectangle.center(),
                vec2(10.0, 10.0),
                &theme.icon_library.icon_handle_navigation_right_arrow_small,
            );
        } else {
            IconDraw::draw_sized(
                user_interface,
                action_rectangle.center(),
                vec2(12.0, 12.0),
                &theme.icon_library.icon_handle_project_pointer_type,
            );
        }

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
            pos2(location_x, row_center_y),
            Align2::LEFT_CENTER,
            &pointer_scanner_tree_row.location_text,
            module_font.clone(),
            theme.foreground,
        );
        user_interface.painter().text(
            pos2(offset_x, row_center_y),
            Align2::LEFT_CENTER,
            &pointer_scanner_tree_row.offset_text,
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
            *entered_node_id = Some(pointer_scanner_tree_row.node_id);
        }

        if row_response.double_clicked() && !pointer_scanner_tree_row.has_children {
            *added_node_id = Some(pointer_scanner_tree_row.node_id);
        }

        if action_response.clicked() {
            if pointer_scanner_tree_row.has_children {
                *entered_node_id = Some(pointer_scanner_tree_row.node_id);
            } else {
                *added_node_id = Some(pointer_scanner_tree_row.node_id);
            }
        }
    }
}

impl Widget for PointerScannerResultsView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let visible_row_count = PointerScannerViewData::get_visible_row_count(self.pointer_scanner_view_data.clone());
        let is_root_context = PointerScannerViewData::is_root_context(self.pointer_scanner_view_data.clone());
        let mut clicked_node_id = None;
        let mut entered_node_id = None;
        let mut added_node_id = None;
        let footer_height = PointerScannerFooterView::FOOTER_HEIGHT;
        let (allocated_size_rectangle, response) = user_interface.allocate_exact_size(user_interface.available_size(), Sense::hover());

        if allocated_size_rectangle.width() <= 0.0 || allocated_size_rectangle.height() <= 0.0 {
            return response;
        }

        let theme = &self.app_context.theme;
        let available_footer_height = (allocated_size_rectangle.height() - Self::HEADER_HEIGHT).max(0.0);
        let clamped_footer_height = footer_height.min(available_footer_height);
        let header_rectangle = Rect::from_min_size(
            allocated_size_rectangle.min,
            vec2(allocated_size_rectangle.width(), Self::HEADER_HEIGHT.min(allocated_size_rectangle.height())),
        );
        let footer_rectangle = Rect::from_min_size(
            pos2(
                allocated_size_rectangle.min.x,
                (allocated_size_rectangle.max.y - footer_height).max(header_rectangle.max.y),
            ),
            vec2(allocated_size_rectangle.width(), clamped_footer_height),
        );
        let content_rectangle = Rect::from_min_max(
            pos2(allocated_size_rectangle.min.x, header_rectangle.max.y),
            pos2(allocated_size_rectangle.max.x, footer_rectangle.min.y.max(header_rectangle.max.y)),
        );

        user_interface
            .painter()
            .rect_filled(allocated_size_rectangle, CornerRadius::ZERO, theme.background_panel);

        {
            let header_builder = UiBuilder::new()
                .max_rect(header_rectangle)
                .layout(Layout::top_down(Align::Min))
                .sense(Sense::hover());
            let mut header_user_interface = user_interface.new_child(header_builder);
            self.draw_header(&mut header_user_interface, header_rectangle, is_root_context);
        }

        if content_rectangle.height() > 0.0 {
            let content_builder = UiBuilder::new()
                .max_rect(content_rectangle)
                .layout(Layout::top_down(Align::Min))
                .sense(Sense::hover());
            let mut content_user_interface = user_interface.new_child(content_builder);

            ScrollArea::vertical()
                .id_salt("pointer_scanner_rows")
                .auto_shrink([false, false])
                .show_rows(&mut content_user_interface, Self::ROW_HEIGHT, visible_row_count, |user_interface, row_range| {
                    let pointer_scanner_tree_rows = PointerScannerViewData::build_visible_rows_in_range(self.pointer_scanner_view_data.clone(), row_range);

                    for pointer_scanner_tree_row in &pointer_scanner_tree_rows {
                        self.draw_row(
                            user_interface,
                            pointer_scanner_tree_row,
                            &mut clicked_node_id,
                            &mut entered_node_id,
                            &mut added_node_id,
                        );
                    }
                });
        }

        if footer_rectangle.height() > 0.0 {
            let footer_builder = UiBuilder::new()
                .max_rect(footer_rectangle)
                .layout(Layout::top_down(Align::Min))
                .sense(Sense::hover());
            let mut footer_user_interface = user_interface.new_child(footer_builder);
            footer_user_interface.add(self.pointer_scanner_footer_view.clone());
        }

        self.draw_column_separators(user_interface, allocated_size_rectangle);

        if let Some(clicked_node_id) = clicked_node_id {
            PointerScannerViewData::select_node(self.pointer_scanner_view_data.clone(), clicked_node_id);
        }

        if let Some(entered_node_id) = entered_node_id {
            PointerScannerViewData::navigate_into_node_context(self.pointer_scanner_view_data.clone(), entered_node_id);
        }

        if let Some(added_node_id) = added_node_id {
            let target_directory_path = ProjectHierarchyViewData::get_selected_directory_path(self.project_hierarchy_view_data.clone());

            if let Some(project_item_create_request) =
                PointerScannerViewData::build_project_item_create_request_for_node(self.pointer_scanner_view_data.clone(), added_node_id, target_directory_path)
            {
                project_item_create_request.send(&self.app_context.engine_unprivileged_state, |project_items_create_response| {
                    if !project_items_create_response.success {
                        log::error!("Failed to add pointer chain to the project.");
                    }
                });
            }
        }

        response
    }
}
