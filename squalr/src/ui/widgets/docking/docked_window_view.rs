use crate::models::docking::hierarchy::types::dock_splitter_drag_direction::DockSplitterDragDirection;
use crate::ui::widgets::docking::docked_window_footer_view::DockedWindowFooterView;
use crate::ui::widgets::docking::docked_window_title_bar_view::DockedWindowTitleBarView;
use crate::{app_context::AppContext, ui::widgets::docking::dockable_window::DockableWindow};
use eframe::egui::{Align, CursorIcon, Layout, Response, Sense, Ui, UiBuilder, Widget};
use epaint::{Rect, vec2};
use std::rc::Rc;

#[derive(Clone)]
pub struct DockedWindowView<W: Widget> {
    app_context: Rc<AppContext>,
    docked_window_title_bar_view: DockedWindowTitleBarView,
    widget: W,
    docked_window_footer_view: DockedWindowFooterView,
    identifier: Rc<String>,
}

impl<W: Widget + Clone + 'static> DockableWindow for DockedWindowView<W> {
    fn get_identifier(&self) -> &str {
        self.get_identifier()
    }

    fn ui(
        &self,
        ui: &mut Ui,
    ) -> Response {
        self.clone().ui(ui)
    }
}

impl<W: Widget> DockedWindowView<W> {
    pub fn new(
        app_context: Rc<AppContext>,
        widget: W,
        title: Rc<String>,
        identifier: Rc<String>,
    ) -> Self {
        let docked_window_title_bar_view = DockedWindowTitleBarView::new(app_context.clone(), title, identifier.clone());
        let docked_window_footer_view = DockedWindowFooterView::new(app_context.clone(), identifier.clone());

        Self {
            app_context,
            docked_window_title_bar_view,
            widget,
            docked_window_footer_view,
            identifier,
        }
    }

    pub fn get_identifier(&self) -> &str {
        &self.identifier
    }
}

impl<W: Widget> Widget for DockedWindowView<W> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        const BAR_THICKNESS: f32 = 4.0;

        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                // Reserve the full outer rect.
                let outer_rectangle = user_interface.available_rect_before_wrap();
                let theme = &self.app_context.theme;
                let docking_manager = &self.app_context.docking_manager;
                let allocate_resize_bar = |resize_rectangle: Rect, id_suffix: &str| -> Response {
                    let id = user_interface.id().with(&self.identifier).with(id_suffix);
                    let response = user_interface.interact(resize_rectangle, id, Sense::drag());

                    user_interface
                        .painter()
                        .rect_filled(resize_rectangle, 0.0, theme.background_control);

                    response
                };

                // Top bar.
                let top_rectangle = Rect::from_min_max(outer_rectangle.min, outer_rectangle.min + vec2(outer_rectangle.width(), BAR_THICKNESS));
                let top_response = allocate_resize_bar(top_rectangle, "top_bar").on_hover_cursor(CursorIcon::ResizeVertical);

                if top_response.dragged() {
                    if let Ok(mut docking_manager) = docking_manager.try_write() {
                        let drag_delta = top_response.drag_delta();

                        docking_manager.adjust_window_size(&self.identifier, &DockSplitterDragDirection::Top, drag_delta.x as i32, drag_delta.y as i32);
                    }
                }

                // Bottom bar.
                let bottom_rectangle = Rect::from_min_max(outer_rectangle.max - vec2(outer_rectangle.width(), BAR_THICKNESS), outer_rectangle.max);
                let bottom_response = allocate_resize_bar(bottom_rectangle, "bottom_bar").on_hover_cursor(CursorIcon::ResizeVertical);

                if bottom_response.dragged() {
                    if let Ok(mut docking_manager) = docking_manager.try_write() {
                        let drag_delta = bottom_response.drag_delta();

                        docking_manager.adjust_window_size(&self.identifier, &DockSplitterDragDirection::Bottom, drag_delta.x as i32, drag_delta.y as i32);
                    }
                }

                // Left bar.
                let left_rectangle = Rect::from_min_max(
                    outer_rectangle.min + vec2(0.0, BAR_THICKNESS),
                    outer_rectangle.max - vec2(outer_rectangle.width() - BAR_THICKNESS, BAR_THICKNESS),
                );
                let left_resposne = allocate_resize_bar(left_rectangle, "left_bar").on_hover_cursor(CursorIcon::ResizeHorizontal);

                if left_resposne.dragged() {
                    if let Ok(mut docking_manager) = docking_manager.try_write() {
                        let drag_delta = left_resposne.drag_delta();

                        docking_manager.adjust_window_size(&self.identifier, &DockSplitterDragDirection::Left, drag_delta.x as i32, drag_delta.y as i32);
                    }
                }

                // Right bar.
                let right_rectangle = Rect::from_min_max(
                    outer_rectangle.max - vec2(BAR_THICKNESS, outer_rectangle.height() - BAR_THICKNESS),
                    outer_rectangle.max - vec2(0.0, BAR_THICKNESS),
                );
                let right_response = allocate_resize_bar(right_rectangle, "right_bar").on_hover_cursor(CursorIcon::ResizeHorizontal);

                if right_response.dragged() {
                    if let Ok(mut docking_manager) = docking_manager.try_write() {
                        let drag_delta = right_response.drag_delta();

                        docking_manager.adjust_window_size(&self.identifier, &DockSplitterDragDirection::Right, drag_delta.x as i32, drag_delta.y as i32);
                    }
                }

                // Inner rect (content area inset by bars).
                let inner_rectangle = outer_rectangle.shrink(BAR_THICKNESS);
                let builder = UiBuilder::new()
                    .max_rect(inner_rectangle)
                    .layout(Layout::top_down(Align::Min));
                let mut inner_user_interface = user_interface.new_child(builder);

                // Title bar.
                inner_user_interface.add(self.docked_window_title_bar_view);

                let has_footer = match docking_manager.try_read() {
                    Ok(docking_manager) => {
                        docking_manager
                            .get_sibling_tab_ids(&self.identifier, true)
                            .len()
                            > 1
                    }
                    Err(_) => false,
                };

                // Content (minus footer).
                let mut content_rect = inner_user_interface.available_rect_before_wrap();
                let footer_height = self.docked_window_footer_view.get_height();

                if has_footer {
                    content_rect.max.y -= footer_height;
                }

                let content_response = inner_user_interface.allocate_rect(content_rect, Sense::empty());
                let mut content_ui = inner_user_interface.new_child(
                    UiBuilder::new()
                        .max_rect(content_response.rect)
                        .layout(Layout::left_to_right(Align::Min)),
                );

                user_interface.add(self.widget);

                // Footer.
                if has_footer {
                    inner_user_interface.add(self.docked_window_footer_view);
                }
            })
            .response;

        response
    }
}
