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
    const COLUMN_SEPARATOR_THICKNESS: f32 = 3.0;
    const HEADER_HEIGHT: f32 = 32.0;
    const ROW_HEIGHT: f32 = 32.0;
    const COLUMN_PADDING: f32 = 8.0;
    const MINIMUM_COLUMN_PIXEL_WIDTH: f32 = 96.0;
    const MINIMUM_SPLITTER_PIXEL_GAP: f32 = 56.0;
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

    fn draw_clipped_text(
        &self,
        user_interface: &mut Ui,
        clip_rectangle: Rect,
        text: &str,
        font_id: FontId,
        color: Color32,
    ) {
        user_interface
            .painter()
            .with_clip_rect(clip_rectangle.intersect(user_interface.clip_rect()))
            .text(clip_rectangle.left_center(), Align2::LEFT_CENTER, text, font_id, color);
    }

    fn column_text_rectangle(
        left_edge: f32,
        right_edge: f32,
        row_rectangle: Rect,
    ) -> Rect {
        let minimum_text_x = left_edge + Self::COLUMN_PADDING;
        let maximum_text_x = (right_edge - Self::COLUMN_PADDING).max(minimum_text_x);

        Rect::from_min_max(pos2(minimum_text_x, row_rectangle.min.y), pos2(maximum_text_x, row_rectangle.max.y))
    }

    fn draw_header(
        &self,
        user_interface: &mut Ui,
        header_rectangle: Rect,
        is_root_context: bool,
        primary_splitter_position_x: f32,
        value_splitter_position_x: f32,
        resolved_splitter_position_x: f32,
    ) {
        let theme = &self.app_context.theme;
        let header_font = theme.font_library.font_noto_sans.font_header.clone();
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

        self.draw_clipped_text(
            user_interface,
            Self::column_text_rectangle(header_rectangle.min.x, primary_splitter_position_x, header_rectangle),
            primary_label,
            header_font.clone(),
            theme.foreground,
        );
        self.draw_clipped_text(
            user_interface,
            Self::column_text_rectangle(primary_splitter_position_x, value_splitter_position_x, header_rectangle),
            "Value",
            header_font.clone(),
            theme.foreground,
        );
        self.draw_clipped_text(
            user_interface,
            Self::column_text_rectangle(value_splitter_position_x, resolved_splitter_position_x, header_rectangle),
            "Resolved",
            header_font.clone(),
            theme.foreground,
        );
        self.draw_clipped_text(
            user_interface,
            Self::column_text_rectangle(resolved_splitter_position_x, header_rectangle.max.x, header_rectangle),
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
        should_navigate_back: &mut bool,
        primary_splitter_position_x: f32,
        value_splitter_position_x: f32,
        resolved_splitter_position_x: f32,
    ) {
        let theme = &self.app_context.theme;
        let (row_rectangle, row_response) =
            user_interface.allocate_exact_size(vec2(user_interface.available_width().max(1.0), Self::ROW_HEIGHT), Sense::click());
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

        let disclosure_icon_rectangle = Rect::from_center_size(
            pos2(
                row_rectangle.min.x + Self::COLUMN_PADDING + Self::DISCLOSURE_ICON_SIZE * 0.5,
                row_rectangle.center().y,
            ),
            vec2(Self::DISCLOSURE_ICON_SIZE, Self::DISCLOSURE_ICON_SIZE),
        );
        let disclosure_response = user_interface.interact(
            disclosure_icon_rectangle,
            user_interface
                .id()
                .with(("pointer_scanner_enter", pointer_scanner_tree_row.node_id)),
            if pointer_scanner_tree_row.has_children || pointer_scanner_tree_row.is_navigate_up_row {
                Sense::click()
            } else {
                Sense::hover()
            },
        );

        if pointer_scanner_tree_row.is_navigate_up_row {
            IconDraw::draw_sized(
                user_interface,
                disclosure_icon_rectangle.center(),
                vec2(Self::DISCLOSURE_ICON_SIZE, Self::DISCLOSURE_ICON_SIZE),
                &theme.icon_library.icon_handle_navigation_left_arrow_small,
            );
        } else if pointer_scanner_tree_row.has_children {
            IconDraw::draw_sized(
                user_interface,
                disclosure_icon_rectangle.center(),
                vec2(Self::DISCLOSURE_ICON_SIZE, Self::DISCLOSURE_ICON_SIZE),
                &theme.icon_library.icon_handle_navigation_right_arrow_small,
            );
        }

        let primary_text_left_edge = if pointer_scanner_tree_row.has_children || pointer_scanner_tree_row.is_navigate_up_row {
            disclosure_icon_rectangle.max.x + Self::DISCLOSURE_TEXT_SPACING
        } else {
            row_rectangle.min.x
        };
        let primary_text_rectangle = Self::column_text_rectangle(primary_text_left_edge, primary_splitter_position_x, row_rectangle);
        let value_text_rectangle = Self::column_text_rectangle(primary_splitter_position_x, value_splitter_position_x, row_rectangle);
        let resolved_text_rectangle = Self::column_text_rectangle(value_splitter_position_x, resolved_splitter_position_x, row_rectangle);
        let depth_text_rectangle = Self::column_text_rectangle(resolved_splitter_position_x, row_rectangle.max.x, row_rectangle);
        let text_font = theme.font_library.font_ubuntu_mono_bold.font_normal.clone();

        self.draw_clipped_text(
            user_interface,
            primary_text_rectangle,
            &pointer_scanner_tree_row.primary_text,
            text_font.clone(),
            theme.foreground,
        );
        self.draw_clipped_text(
            user_interface,
            value_text_rectangle,
            &pointer_scanner_tree_row.value_text,
            text_font.clone(),
            theme.foreground,
        );
        self.draw_clipped_text(
            user_interface,
            resolved_text_rectangle,
            &pointer_scanner_tree_row.resolved_address_text,
            text_font.clone(),
            theme.foreground,
        );
        self.draw_clipped_text(
            user_interface,
            depth_text_rectangle,
            &pointer_scanner_tree_row.depth_text,
            text_font,
            theme.foreground,
        );

        if row_response.hovered() {
            user_interface.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }

        if pointer_scanner_tree_row.is_navigate_up_row {
            if row_response.clicked() || row_response.double_clicked() || disclosure_response.clicked() || disclosure_response.double_clicked() {
                *should_navigate_back = true;
            }

            return;
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
        let footer_height = PointerScannerFooterView::FOOTER_HEIGHT;
        let theme = &self.app_context.theme;
        let mut clicked_node_id = None;
        let mut entered_node_id = None;
        let mut added_node_id = None;
        let mut should_navigate_back = false;
        let mut new_primary_splitter_ratio = None;
        let mut new_value_splitter_ratio = None;
        let mut new_resolved_splitter_ratio = None;

        let (results_rectangle, response) = user_interface.allocate_exact_size(user_interface.available_size(), Sense::click());
        let mut results_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(results_rectangle)
                .layout(Layout::top_down(Align::Min)),
        );
        results_user_interface.set_clip_rect(results_rectangle);

        results_user_interface
            .painter()
            .rect_filled(results_rectangle, CornerRadius::ZERO, theme.background_panel);

        results_user_interface.allocate_ui_with_layout(results_user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
            let allocate_resize_bar = |user_interface: &mut Ui, resize_rectangle: Rect, id_suffix: &str| -> Response {
                let id = user_interface.id().with(id_suffix);
                let response = user_interface.interact(resize_rectangle, id, Sense::drag());

                user_interface
                    .painter()
                    .rect_filled(resize_rectangle, CornerRadius::ZERO, theme.background_control);

                response
            };

            let (header_rectangle, _) =
                user_interface.allocate_exact_size(vec2(user_interface.available_width().max(1.0), Self::HEADER_HEIGHT), Sense::empty());
            let content_clip_rectangle = user_interface
                .available_rect_before_wrap()
                .with_max_y(user_interface.available_rect_before_wrap().max.y - footer_height);
            let content_width = content_clip_rectangle.width();
            let content_height = content_clip_rectangle.height().max(0.0);
            let content_min_x = content_clip_rectangle.min.x;

            let (mut primary_splitter_ratio, mut value_splitter_ratio, mut resolved_splitter_ratio) = match self
                .pointer_scanner_view_data
                .read("Pointer scanner results column ratios")
            {
                Some(pointer_scanner_view_data) => (
                    pointer_scanner_view_data.primary_splitter_ratio,
                    pointer_scanner_view_data.value_splitter_ratio,
                    pointer_scanner_view_data.resolved_splitter_ratio,
                ),
                None => (
                    PointerScannerViewData::DEFAULT_PRIMARY_SPLITTER_RATIO,
                    PointerScannerViewData::DEFAULT_VALUE_SPLITTER_RATIO,
                    PointerScannerViewData::DEFAULT_RESOLVED_SPLITTER_RATIO,
                ),
            };

            if content_width <= 0.0 {
                return;
            }

            if primary_splitter_ratio <= 0.0 || value_splitter_ratio <= primary_splitter_ratio || resolved_splitter_ratio <= value_splitter_ratio {
                primary_splitter_ratio = PointerScannerViewData::DEFAULT_PRIMARY_SPLITTER_RATIO;
                value_splitter_ratio = PointerScannerViewData::DEFAULT_VALUE_SPLITTER_RATIO;
                resolved_splitter_ratio = PointerScannerViewData::DEFAULT_RESOLVED_SPLITTER_RATIO;
                new_primary_splitter_ratio = Some(primary_splitter_ratio);
                new_value_splitter_ratio = Some(value_splitter_ratio);
                new_resolved_splitter_ratio = Some(resolved_splitter_ratio);
            }

            let primary_splitter_position_x = content_min_x + content_width * primary_splitter_ratio;
            let value_splitter_position_x = content_min_x + content_width * value_splitter_ratio;
            let resolved_splitter_position_x = content_min_x + content_width * resolved_splitter_ratio;

            self.draw_header(
                user_interface,
                header_rectangle,
                is_root_context,
                primary_splitter_position_x,
                value_splitter_position_x,
                resolved_splitter_position_x,
            );

            ScrollArea::vertical()
                .id_salt("pointer_scanner_rows")
                .max_height(content_height)
                .auto_shrink([false, false])
                .show_rows(user_interface, Self::ROW_HEIGHT, visible_row_count, |user_interface, row_range| {
                    user_interface.spacing_mut().item_spacing = vec2(0.0, 0.0);
                    let pointer_scanner_tree_rows = PointerScannerViewData::build_visible_rows_in_range(self.pointer_scanner_view_data.clone(), row_range);

                    for pointer_scanner_tree_row in &pointer_scanner_tree_rows {
                        self.draw_row(
                            user_interface,
                            pointer_scanner_tree_row,
                            &mut clicked_node_id,
                            &mut entered_node_id,
                            &mut added_node_id,
                            &mut should_navigate_back,
                            primary_splitter_position_x,
                            value_splitter_position_x,
                            resolved_splitter_position_x,
                        );
                    }
                });

            user_interface.add(self.pointer_scanner_footer_view.clone());

            let splitter_min_y = header_rectangle.min.y;
            let splitter_max_y = content_clip_rectangle.max.y;

            for splitter_position_x in [
                primary_splitter_position_x,
                value_splitter_position_x,
                resolved_splitter_position_x,
            ] {
                let splitter_rectangle = Rect::from_min_max(
                    pos2(splitter_position_x - Self::COLUMN_SEPARATOR_THICKNESS * 0.5, splitter_min_y),
                    pos2(splitter_position_x + Self::COLUMN_SEPARATOR_THICKNESS * 0.5, splitter_max_y),
                );

                user_interface
                    .painter()
                    .rect_filled(splitter_rectangle, CornerRadius::ZERO, theme.background_control);
            }

            let primary_splitter_rectangle = Rect::from_min_max(
                pos2(primary_splitter_position_x - Self::COLUMN_SEPARATOR_THICKNESS * 0.5, splitter_min_y),
                pos2(primary_splitter_position_x + Self::COLUMN_SEPARATOR_THICKNESS * 0.5, splitter_max_y),
            );
            let value_splitter_rectangle = Rect::from_min_max(
                pos2(value_splitter_position_x - Self::COLUMN_SEPARATOR_THICKNESS * 0.5, splitter_min_y),
                pos2(value_splitter_position_x + Self::COLUMN_SEPARATOR_THICKNESS * 0.5, splitter_max_y),
            );
            let resolved_splitter_rectangle = Rect::from_min_max(
                pos2(resolved_splitter_position_x - Self::COLUMN_SEPARATOR_THICKNESS * 0.5, splitter_min_y),
                pos2(resolved_splitter_position_x + Self::COLUMN_SEPARATOR_THICKNESS * 0.5, splitter_max_y),
            );

            let primary_splitter_response = allocate_resize_bar(user_interface, primary_splitter_rectangle, "pointer_scanner_primary_splitter")
                .on_hover_cursor(CursorIcon::ResizeHorizontal);

            if primary_splitter_response.dragged() {
                let drag_delta = primary_splitter_response.drag_delta();
                let mut new_primary_splitter_position_x = primary_splitter_position_x + drag_delta.x;

                let minimum_primary_splitter_position_x = content_min_x + Self::MINIMUM_COLUMN_PIXEL_WIDTH;
                let maximum_primary_splitter_position_x = value_splitter_position_x - Self::MINIMUM_SPLITTER_PIXEL_GAP;

                new_primary_splitter_position_x =
                    new_primary_splitter_position_x.clamp(minimum_primary_splitter_position_x, maximum_primary_splitter_position_x);

                new_primary_splitter_ratio = Some((new_primary_splitter_position_x - content_min_x) / content_width);
            }

            let value_splitter_response =
                allocate_resize_bar(user_interface, value_splitter_rectangle, "pointer_scanner_value_splitter").on_hover_cursor(CursorIcon::ResizeHorizontal);

            if value_splitter_response.dragged() {
                let drag_delta = value_splitter_response.drag_delta();
                let mut new_value_splitter_position_x = value_splitter_position_x + drag_delta.x;

                let minimum_value_splitter_position_x = primary_splitter_position_x + Self::MINIMUM_SPLITTER_PIXEL_GAP;
                let maximum_value_splitter_position_x = resolved_splitter_position_x - Self::MINIMUM_SPLITTER_PIXEL_GAP;

                new_value_splitter_position_x = new_value_splitter_position_x.clamp(minimum_value_splitter_position_x, maximum_value_splitter_position_x);

                new_value_splitter_ratio = Some((new_value_splitter_position_x - content_min_x) / content_width);
            }

            let resolved_splitter_response = allocate_resize_bar(user_interface, resolved_splitter_rectangle, "pointer_scanner_resolved_splitter")
                .on_hover_cursor(CursorIcon::ResizeHorizontal);

            if resolved_splitter_response.dragged() {
                let drag_delta = resolved_splitter_response.drag_delta();
                let mut new_resolved_splitter_position_x = resolved_splitter_position_x + drag_delta.x;

                let minimum_resolved_splitter_position_x = value_splitter_position_x + Self::MINIMUM_SPLITTER_PIXEL_GAP;
                let maximum_resolved_splitter_position_x = content_min_x + content_width - Self::MINIMUM_COLUMN_PIXEL_WIDTH;

                new_resolved_splitter_position_x =
                    new_resolved_splitter_position_x.clamp(minimum_resolved_splitter_position_x, maximum_resolved_splitter_position_x);

                new_resolved_splitter_ratio = Some((new_resolved_splitter_position_x - content_min_x) / content_width);
            }
        });

        if new_primary_splitter_ratio.is_some() || new_value_splitter_ratio.is_some() || new_resolved_splitter_ratio.is_some() {
            if let Some(mut pointer_scanner_view_data) = self
                .pointer_scanner_view_data
                .write("Pointer scanner results column ratios")
            {
                if let Some(new_primary_splitter_ratio) = new_primary_splitter_ratio {
                    pointer_scanner_view_data.primary_splitter_ratio = new_primary_splitter_ratio;
                }

                if let Some(new_value_splitter_ratio) = new_value_splitter_ratio {
                    pointer_scanner_view_data.value_splitter_ratio = new_value_splitter_ratio;
                }

                if let Some(new_resolved_splitter_ratio) = new_resolved_splitter_ratio {
                    pointer_scanner_view_data.resolved_splitter_ratio = new_resolved_splitter_ratio;
                }
            }
        }

        if let Some(clicked_node_id) = clicked_node_id {
            PointerScannerViewData::select_node(self.pointer_scanner_view_data.clone(), clicked_node_id);
        }

        if should_navigate_back {
            PointerScannerViewData::navigate_back(self.pointer_scanner_view_data.clone());
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
