use crate::app_context::AppContext;
use crate::ui::draw::icon_draw::IconDraw;
use crate::views::pointer_scanner::pointer_scanner_footer_view::PointerScannerFooterView;
use crate::views::pointer_scanner::view_data::pointer_scanner_view_data::{PointerScannerTreeRow, PointerScannerViewData};
use crate::views::project_explorer::project_hierarchy::view_data::project_hierarchy_view_data::ProjectHierarchyViewData;
use eframe::egui::{Align, Align2, CursorIcon, FontId, Layout, Response, ScrollArea, Sense, Ui, UiBuilder, Widget, pos2, vec2};
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
    const ROW_HEIGHT: f32 = 28.0;
    const OUTER_HORIZONTAL_PADDING: f32 = 8.0;
    const COLUMN_PADDING: f32 = 8.0;
    const PRIMARY_COLUMN_WIDTH_RATIO: f32 = 0.39;
    const VALUE_COLUMN_WIDTH_RATIO: f32 = 0.21;
    const RESOLVED_COLUMN_WIDTH_RATIO: f32 = 0.24;
    const DISCLOSURE_ICON_SIZE: f32 = 10.0;
    const DISCLOSURE_TEXT_SPACING: f32 = 6.0;

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

    fn column_rectangles(rectangle: Rect) -> [Rect; 4] {
        let separator_total_width = Self::COLUMN_SEPARATOR_THICKNESS * 3.0;
        let content_width = (rectangle.width() - Self::OUTER_HORIZONTAL_PADDING * 2.0 - separator_total_width).max(0.0);
        let primary_column_width = content_width * Self::PRIMARY_COLUMN_WIDTH_RATIO;
        let value_column_width = content_width * Self::VALUE_COLUMN_WIDTH_RATIO;
        let resolved_column_width = content_width * Self::RESOLVED_COLUMN_WIDTH_RATIO;
        let depth_column_width = (content_width - primary_column_width - value_column_width - resolved_column_width).max(0.0);
        let primary_column_min_x = rectangle.min.x + Self::OUTER_HORIZONTAL_PADDING;
        let value_column_min_x = primary_column_min_x + primary_column_width + Self::COLUMN_SEPARATOR_THICKNESS;
        let resolved_column_min_x = value_column_min_x + value_column_width + Self::COLUMN_SEPARATOR_THICKNESS;
        let depth_column_min_x = resolved_column_min_x + resolved_column_width + Self::COLUMN_SEPARATOR_THICKNESS;

        [
            Rect::from_min_max(
                pos2(primary_column_min_x, rectangle.min.y),
                pos2(primary_column_min_x + primary_column_width, rectangle.max.y),
            ),
            Rect::from_min_max(
                pos2(value_column_min_x, rectangle.min.y),
                pos2(value_column_min_x + value_column_width, rectangle.max.y),
            ),
            Rect::from_min_max(
                pos2(resolved_column_min_x, rectangle.min.y),
                pos2(resolved_column_min_x + resolved_column_width, rectangle.max.y),
            ),
            Rect::from_min_max(
                pos2(depth_column_min_x, rectangle.min.y),
                pos2(depth_column_min_x + depth_column_width, rectangle.max.y),
            ),
        ]
    }

    fn draw_clipped_text(
        &self,
        user_interface: &mut Ui,
        clip_rectangle: Rect,
        text: &str,
        font_id: FontId,
        color: Color32,
    ) {
        let clipped_text_rectangle = clip_rectangle.intersect(user_interface.clip_rect());
        let clipped_painter = user_interface.painter().with_clip_rect(clipped_text_rectangle);
        clipped_painter.text(
            pos2(clipped_text_rectangle.min.x, clipped_text_rectangle.center().y),
            Align2::LEFT_CENTER,
            text,
            font_id,
            color,
        );
    }

    fn inset_text_rectangle(column_rectangle: Rect) -> Rect {
        Rect::from_min_max(
            pos2(column_rectangle.min.x + Self::COLUMN_PADDING, column_rectangle.min.y),
            pos2(
                (column_rectangle.max.x - Self::COLUMN_PADDING).max(column_rectangle.min.x + Self::COLUMN_PADDING),
                column_rectangle.max.y,
            ),
        )
    }

    fn draw_column_separators(
        &self,
        user_interface: &mut Ui,
        rectangle: Rect,
    ) {
        let theme = &self.app_context.theme;
        let [
            primary_column_rectangle,
            value_column_rectangle,
            resolved_column_rectangle,
            _depth_column_rectangle,
        ] = Self::column_rectangles(rectangle);

        for separator_x in [
            primary_column_rectangle.max.x,
            value_column_rectangle.max.x,
            resolved_column_rectangle.max.x,
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
        let [
            primary_column_rectangle,
            value_column_rectangle,
            resolved_column_rectangle,
            depth_column_rectangle,
        ] = Self::column_rectangles(header_rectangle);
        let primary_label = if is_root_context { "Module" } else { "Offset" };

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

        self.draw_clipped_text(
            user_interface,
            Self::inset_text_rectangle(primary_column_rectangle),
            primary_label,
            header_font.clone(),
            theme.foreground,
        );
        self.draw_clipped_text(
            user_interface,
            Self::inset_text_rectangle(value_column_rectangle),
            "Value",
            header_font.clone(),
            theme.foreground,
        );
        self.draw_clipped_text(
            user_interface,
            Self::inset_text_rectangle(resolved_column_rectangle),
            "Resolved",
            header_font.clone(),
            theme.foreground,
        );
        self.draw_clipped_text(
            user_interface,
            Self::inset_text_rectangle(depth_column_rectangle),
            "Depth",
            header_font,
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
        let visible_row_width = user_interface.clip_rect().width().max(0.0);
        let (row_rectangle, row_response) = user_interface.allocate_exact_size(vec2(visible_row_width, Self::ROW_HEIGHT), Sense::click());
        let [
            primary_column_rectangle,
            value_column_rectangle,
            resolved_column_rectangle,
            depth_column_rectangle,
        ] = Self::column_rectangles(row_rectangle);
        let disclosure_icon_rectangle = Rect::from_center_size(
            pos2(
                primary_column_rectangle.min.x + Self::COLUMN_PADDING + Self::DISCLOSURE_ICON_SIZE * 0.5,
                row_rectangle.center().y,
            ),
            vec2(Self::DISCLOSURE_ICON_SIZE, Self::DISCLOSURE_ICON_SIZE),
        );
        let disclosure_response = if pointer_scanner_tree_row.has_children {
            user_interface.interact(
                disclosure_icon_rectangle,
                user_interface
                    .id()
                    .with(("pointer_scanner_enter", pointer_scanner_tree_row.node_id)),
                Sense::click(),
            )
        } else {
            user_interface.allocate_rect(disclosure_icon_rectangle, Sense::hover())
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
                disclosure_icon_rectangle.center(),
                vec2(Self::DISCLOSURE_ICON_SIZE, Self::DISCLOSURE_ICON_SIZE),
                &theme.icon_library.icon_handle_navigation_right_arrow_small,
            );
        }

        let module_font = theme.font_library.font_ubuntu_mono_bold.font_normal.clone();
        let value_font = theme.font_library.font_ubuntu_mono_bold.font_normal.clone();
        let primary_text_min_x = if pointer_scanner_tree_row.has_children {
            disclosure_icon_rectangle.max.x + Self::DISCLOSURE_TEXT_SPACING
        } else {
            primary_column_rectangle.min.x + Self::COLUMN_PADDING
        };
        let primary_text_rectangle = Rect::from_min_max(
            pos2(primary_text_min_x, primary_column_rectangle.min.y),
            pos2(
                (primary_column_rectangle.max.x - Self::COLUMN_PADDING).max(primary_text_min_x),
                primary_column_rectangle.max.y,
            ),
        );

        self.draw_clipped_text(
            user_interface,
            primary_text_rectangle,
            &pointer_scanner_tree_row.primary_text,
            module_font.clone(),
            theme.foreground,
        );
        self.draw_clipped_text(
            user_interface,
            Self::inset_text_rectangle(value_column_rectangle),
            &pointer_scanner_tree_row.value_text,
            value_font.clone(),
            theme.foreground,
        );
        self.draw_clipped_text(
            user_interface,
            Self::inset_text_rectangle(resolved_column_rectangle),
            &pointer_scanner_tree_row.resolved_address_text,
            value_font.clone(),
            theme.foreground,
        );
        self.draw_clipped_text(
            user_interface,
            Self::inset_text_rectangle(depth_column_rectangle),
            &pointer_scanner_tree_row.depth_text,
            value_font,
            theme.foreground,
        );

        if row_response.hovered() {
            user_interface.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }

        if row_response.clicked() {
            *clicked_node_id = Some(pointer_scanner_tree_row.node_id);
        }

        if (row_response.double_clicked() || disclosure_response.double_clicked()) && pointer_scanner_tree_row.has_children {
            *entered_node_id = Some(pointer_scanner_tree_row.node_id);
        }

        if row_response.double_clicked() && !pointer_scanner_tree_row.has_children {
            *added_node_id = Some(pointer_scanner_tree_row.node_id);
        }

        if disclosure_response.clicked() && pointer_scanner_tree_row.has_children {
            *entered_node_id = Some(pointer_scanner_tree_row.node_id);
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
