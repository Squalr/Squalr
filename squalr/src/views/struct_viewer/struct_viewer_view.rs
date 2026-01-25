use crate::views::struct_viewer::struct_viewer_entry_view::StructViewerEntryView;
use crate::views::struct_viewer::view_data::struct_viewer_frame_action::StructViewerFrameAction;
use crate::{app_context::AppContext, views::struct_viewer::view_data::struct_viewer_view_data::StructViewerViewData};
use eframe::egui::{Align, CursorIcon, Layout, Response, ScrollArea, Sense, Ui, Widget};
use epaint::{Rect, pos2};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use std::sync::Arc;

#[derive(Clone)]
pub struct StructViewerView {
    app_context: Arc<AppContext>,
    struct_viewer_view_data: Dependency<StructViewerViewData>,
}

impl StructViewerView {
    pub const WINDOW_ID: &'static str = "window_struct_viewer";

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let struct_viewer_view_data = app_context
            .dependency_container
            .register(StructViewerViewData::new());

        Self {
            app_context,
            struct_viewer_view_data,
        }
    }
}

impl Widget for StructViewerView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        const ICON_COLUMN_WIDTH: f32 = 32.0;
        const BAR_THICKNESS: f32 = 4.0;
        const MINIMUM_COLUMN_PIXEL_WIDTH: f32 = 80.0;

        let theme = &self.app_context.theme;
        let mut frame_action = StructViewerFrameAction::None;

        let mut new_value_splitter_ratio: Option<f32> = None;

        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |mut user_interface| {
                let struct_viewer_view_data = match self.struct_viewer_view_data.read("Struct viewer view") {
                    Some(data) => data,
                    None => return,
                };
                let mut value_splitter_ratio = struct_viewer_view_data.value_splitter_ratio;
                let content_rect = user_interface.available_rect_before_wrap();
                let content_width = content_rect.width();
                let content_min_x = content_rect.min.x;

                if content_width <= 0.0 {
                    return;
                }

                if value_splitter_ratio <= 0.0 {
                    value_splitter_ratio = StructViewerViewData::DEFAULT_NAME_SPLITTER_RATIO;

                    new_value_splitter_ratio = Some(value_splitter_ratio);
                }

                let value_splitter_x = content_min_x + content_width * value_splitter_ratio;

                let splitter_min_y = content_rect.min.y;
                let splitter_max_y = content_rect.max.y;

                let value_splitter_rect = Rect::from_min_max(
                    pos2(value_splitter_x - BAR_THICKNESS * 0.5, splitter_min_y),
                    pos2(value_splitter_x + BAR_THICKNESS * 0.5, splitter_max_y),
                );

                // Rows
                ScrollArea::vertical()
                    .id_salt("struct_viewer")
                    .auto_shrink([false, false])
                    .show(&mut user_interface, |inner_ui| {
                        if let Some(struct_under_view) = struct_viewer_view_data.struct_under_view.as_ref() {
                            for field in struct_under_view.get_fields() {
                                let is_selected = struct_viewer_view_data
                                    .selected_field_name
                                    .as_deref()
                                    .unwrap_or_default()
                                    == field.get_name();

                                inner_ui.add(StructViewerEntryView::new(
                                    self.app_context.clone(),
                                    &field,
                                    is_selected,
                                    &mut frame_action,
                                    ICON_COLUMN_WIDTH + BAR_THICKNESS,
                                    value_splitter_x + BAR_THICKNESS,
                                ));
                            }
                        }
                    });

                // Draw non-resizable icon/name divider.
                let icon_divider_x = content_min_x + ICON_COLUMN_WIDTH;
                let icon_divider_rect = Rect::from_min_max(
                    pos2(icon_divider_x - BAR_THICKNESS * 0.5, splitter_min_y),
                    pos2(icon_divider_x + BAR_THICKNESS * 0.5, splitter_max_y),
                );

                user_interface
                    .painter()
                    .rect_filled(icon_divider_rect, 0.0, theme.background_control);

                // Draw the name/value divider.
                let value_splitter_response = user_interface
                    .interact(value_splitter_rect, user_interface.id().with("value_splitter"), Sense::drag())
                    .on_hover_cursor(CursorIcon::ResizeHorizontal);

                user_interface
                    .painter()
                    .rect_filled(value_splitter_rect, 0.0, theme.background_control);

                if value_splitter_response.dragged() {
                    let drag_delta = value_splitter_response.drag_delta();
                    let mut new_x = value_splitter_x + drag_delta.x;
                    let min_x = content_min_x + ICON_COLUMN_WIDTH + MINIMUM_COLUMN_PIXEL_WIDTH;
                    let max_x = content_min_x + content_width - MINIMUM_COLUMN_PIXEL_WIDTH;

                    new_x = new_x.clamp(min_x, max_x);
                    new_value_splitter_ratio = Some((new_x - content_min_x) / content_width);
                }
            })
            .response;

        // Commit splitter changes.
        if new_value_splitter_ratio.is_some() {
            if let Some(mut data) = self.struct_viewer_view_data.write("Struct viewer view") {
                if let Some(ratio) = new_value_splitter_ratio {
                    data.value_splitter_ratio = ratio;
                }
            }
        }

        match frame_action {
            StructViewerFrameAction::None => {}
            StructViewerFrameAction::SelectField(field_name) => {
                StructViewerViewData::set_selected_field(self.struct_viewer_view_data.clone(), field_name);
            }
            StructViewerFrameAction::EditValue(field, value) => {
                //
            }
        }

        response
    }
}
