use crate::ui::theme::Theme;
use crate::ui::widgets::main_window::main_window_view::MainWindowView;
use eframe::egui::{CentralPanel, Context, Frame, Id, ResizeDirection, Response, Sense, Ui, ViewportCommand, Visuals};
use epaint::{CornerRadius, Rect, Rgba, pos2, vec2};
use squalr_engine_api::dependency_injection::dependency_container::DependencyContainer;
use std::rc::Rc;

#[derive(Clone)]
pub struct App {
    main_window_view: MainWindowView,
    corner_radius: CornerRadius,
    resize_thickness: f32,
}

impl App {
    pub fn new(
        context: &Context,
        dependency_container: &DependencyContainer,
    ) -> Self {
        let theme = Rc::new(Theme::new(context));
        let corner_radius = CornerRadius::same(8);
        let resize_thickness = 4.0;
        let main_window_view = MainWindowView::new(context.clone(), theme, corner_radius);

        Self {
            main_window_view,
            corner_radius,
            resize_thickness,
        }
    }

    fn add_resize_handles(
        context: &Context,
        user_interface: &mut Ui,
        resize_thickness: f32,
    ) {
        let rect = user_interface.max_rect();

        // Top-left corner.
        Self::handle_resize(
            context,
            user_interface,
            Rect::from_min_max(rect.min, pos2(rect.min.x + resize_thickness, rect.min.y + resize_thickness)),
            "resize_top_left",
            ResizeDirection::NorthWest,
        );

        // Top-right corner.
        Self::handle_resize(
            context,
            user_interface,
            Rect::from_min_max(pos2(rect.max.x - resize_thickness, rect.min.y), pos2(rect.max.x, rect.min.y + resize_thickness)),
            "resize_top_right",
            ResizeDirection::NorthEast,
        );

        // Bottom-left corner.
        Self::handle_resize(
            context,
            user_interface,
            Rect::from_min_max(pos2(rect.min.x, rect.max.y - resize_thickness), pos2(rect.min.x + resize_thickness, rect.max.y)),
            "resize_bottom_left",
            ResizeDirection::SouthWest,
        );

        // Bottom-right corner.
        Self::handle_resize(
            context,
            user_interface,
            Rect::from_min_max(pos2(rect.max.x - resize_thickness, rect.max.y - resize_thickness), rect.max),
            "resize_bottom_right",
            ResizeDirection::SouthEast,
        );

        // Left side (skip corners).
        Self::handle_resize(
            context,
            user_interface,
            Rect::from_min_max(
                pos2(rect.min.x, rect.min.y + resize_thickness),
                pos2(rect.min.x + resize_thickness, rect.max.y - resize_thickness),
            ),
            "resize_left",
            ResizeDirection::West,
        );

        // Right side (skip corners).
        Self::handle_resize(
            context,
            user_interface,
            Rect::from_min_max(
                pos2(rect.max.x - resize_thickness, rect.min.y + resize_thickness),
                pos2(rect.max.x, rect.max.y - resize_thickness),
            ),
            "resize_right",
            ResizeDirection::East,
        );

        // Top side (skip corners).
        Self::handle_resize(
            context,
            user_interface,
            Rect::from_min_max(
                pos2(rect.min.x + resize_thickness, rect.min.y),
                pos2(rect.max.x - resize_thickness, rect.min.y + resize_thickness),
            ),
            "resize_top",
            ResizeDirection::North,
        );

        // Bottom side (skip corners).
        Self::handle_resize(
            context,
            user_interface,
            Rect::from_min_max(
                pos2(rect.min.x + resize_thickness, rect.max.y - resize_thickness),
                pos2(rect.max.x - resize_thickness, rect.max.y),
            ),
            "resize_bottom",
            ResizeDirection::South,
        );
    }

    fn handle_resize(
        context: &Context,
        user_interface: &mut Ui,
        rect: Rect,
        id: &str,
        dir: ResizeDirection,
    ) {
        use eframe::egui::CursorIcon;

        let response: Response = user_interface.interact(rect, Id::new(id), Sense::click_and_drag());
        let drag_started = response.drag_started();

        // Show the appropriate cursor when hovering
        match dir {
            ResizeDirection::North | ResizeDirection::South => {
                response.on_hover_cursor(CursorIcon::ResizeVertical);
            }
            ResizeDirection::East | ResizeDirection::West => {
                response.on_hover_cursor(CursorIcon::ResizeHorizontal);
            }
            ResizeDirection::NorthEast | ResizeDirection::SouthWest => {
                response.on_hover_cursor(CursorIcon::ResizeNeSw);
            }
            ResizeDirection::NorthWest | ResizeDirection::SouthEast => {
                response.on_hover_cursor(CursorIcon::ResizeNwSe);
            }
        }

        if drag_started {
            context.send_viewport_cmd(ViewportCommand::BeginResize(dir));
        }
    }
}

impl eframe::App for App {
    fn clear_color(
        &self,
        _visuals: &Visuals,
    ) -> [f32; 4] {
        Rgba::TRANSPARENT.to_array()
    }

    fn update(
        &mut self,
        context: &Context,
        _frame: &mut eframe::Frame,
    ) {
        let app_frame = Frame::new()
            .corner_radius(self.corner_radius)
            .stroke(context.style().visuals.widgets.noninteractive.fg_stroke)
            .outer_margin(2.0);
        let resize_thickness = self.resize_thickness;

        CentralPanel::default()
            .frame(app_frame)
            .show(context, move |user_interface| {
                user_interface.style_mut().spacing.item_spacing = vec2(0.0, 0.0);
                user_interface.add(self.main_window_view.clone());

                Self::add_resize_handles(context, user_interface, resize_thickness);
            });
    }
}
