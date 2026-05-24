use crate::app_context::AppContext;
use crate::ui::draw::icon_draw::IconDraw;
use crate::ui::widgets::controls::button::Button;
use eframe::egui::viewport::ViewportCommand;
use eframe::egui::{Align, Id, Layout, Rect, Response, RichText, Sense, Ui, UiBuilder, Widget, pos2};
use epaint::{Color32, CornerRadius, vec2};
use std::rc::Rc;
use std::sync::Arc;

#[derive(Clone)]
pub struct MainTitleBarView {
    app_context: Arc<AppContext>,
    corner_radius: CornerRadius,
    height: f32,
    title: Rc<String>,
}

impl MainTitleBarView {
    const IS_MACOS_MIRROR_MODE: bool = cfg!(target_os = "macos");

    pub fn new(
        app_context: Arc<AppContext>,
        corner_radius: CornerRadius,
        height: f32,
        title: Rc<String>,
    ) -> Self {
        Self {
            app_context,
            corner_radius,
            height,
            title,
        }
    }
}

impl Widget for MainTitleBarView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (allocated_size_rectangle, response) =
            user_interface.allocate_exact_size(vec2(user_interface.available_width().max(1.0), self.height), Sense::empty());
        let theme = &self.app_context.theme;
        let context = &self.app_context.context;

        user_interface.painter().rect_filled(
            allocated_size_rectangle,
            CornerRadius {
                nw: self.corner_radius.nw,
                ne: self.corner_radius.ne,
                sw: 0,
                se: 0,
            },
            theme.background_primary,
        );

        // Create a child ui constrained to the title bar.
        let builder = UiBuilder::new()
            .max_rect(allocated_size_rectangle)
            .layout(Layout::left_to_right(Align::Center));
        let mut child_user_interface = user_interface.new_child(builder);
        let mut window_controls_rectangle: Option<Rect> = None;

        // Hard-clip to the titlebar.
        child_user_interface.set_clip_rect(allocated_size_rectangle);

        if Self::IS_MACOS_MIRROR_MODE {
            child_user_interface.with_layout(Layout::left_to_right(Align::Center), |user_interface| {
                window_controls_rectangle = Self::render_window_controls(user_interface, theme, context);
            });

            child_user_interface.with_layout(Layout::right_to_left(Align::Center), |user_interface| {
                Self::render_title_content(user_interface, theme, self.title.as_ref());
            });
        } else {
            Self::render_title_content(&mut child_user_interface, theme, self.title.as_ref());

            // Push the rest (buttons) to the far right within the same row.
            child_user_interface.add_space(child_user_interface.available_width());

            child_user_interface.with_layout(Layout::right_to_left(Align::Center), |user_interface| {
                window_controls_rectangle = Self::render_window_controls(user_interface, theme, context);
            });
        }

        let drag_rect = if Self::IS_MACOS_MIRROR_MODE {
            let left_edge = window_controls_rectangle
                .map(|rectangle| rectangle.max.x)
                .unwrap_or(allocated_size_rectangle.min.x);
            Rect::from_min_max(pos2(left_edge, allocated_size_rectangle.min.y), allocated_size_rectangle.max)
        } else {
            let right_edge = window_controls_rectangle
                .map(|rectangle| rectangle.min.x)
                .unwrap_or(allocated_size_rectangle.max.x);
            Rect::from_min_max(allocated_size_rectangle.min, pos2(right_edge, allocated_size_rectangle.max.y))
        };
        let drag = user_interface.interact(drag_rect, Id::new("titlebar"), Sense::click_and_drag());

        if drag.drag_started() {
            context.send_viewport_cmd(ViewportCommand::StartDrag);
        }

        if drag.double_clicked() {
            let is_max = context.input(|input_state| input_state.viewport().maximized.unwrap_or(false));
            context.send_viewport_cmd(ViewportCommand::Maximized(!is_max));
        }

        response
    }
}

impl MainTitleBarView {
    fn render_title_content(
        user_interface: &mut Ui,
        theme: &crate::ui::theme::Theme,
        title: &str,
    ) {
        user_interface.add_space(8.0);

        let [texture_width, texture_height] = theme.icon_library.icon_handle_logo.size();
        let (_id, app_icon_rect) = user_interface.allocate_space(vec2(texture_width as f32, texture_height as f32));
        IconDraw::draw(user_interface, app_icon_rect, &theme.icon_library.icon_handle_logo);

        user_interface.add_space(4.0);

        user_interface.label(
            RichText::new(title)
                .font(theme.font_library.font_noto_sans.font_window_title.clone())
                .color(theme.foreground),
        );
    }

    fn render_window_controls(
        user_interface: &mut Ui,
        theme: &crate::ui::theme::Theme,
        context: &eframe::egui::Context,
    ) -> Option<Rect> {
        let button_size = vec2(36.0, 32.0);

        let button_close = user_interface.add_sized(button_size, Button::new_from_theme(theme).background_color(Color32::TRANSPARENT));
        IconDraw::draw(user_interface, button_close.rect, &theme.icon_library.icon_handle_close);

        if button_close.clicked() {
            context.send_viewport_cmd(ViewportCommand::Close);
        }

        let button_minimize_maximize = user_interface.add_sized(button_size, Button::new_from_theme(theme).background_color(Color32::TRANSPARENT));
        IconDraw::draw(user_interface, button_minimize_maximize.rect, &theme.icon_library.icon_handle_maximize);

        if button_minimize_maximize.clicked() {
            let is_max = context.input(|input_state| input_state.viewport().maximized.unwrap_or(false));
            context.send_viewport_cmd(ViewportCommand::Maximized(!is_max));
        }

        let button_minimize = user_interface.add_sized(button_size, Button::new_from_theme(theme).background_color(Color32::TRANSPARENT));
        IconDraw::draw(user_interface, button_minimize.rect, &theme.icon_library.icon_handle_minimize);

        if button_minimize.clicked() {
            context.send_viewport_cmd(ViewportCommand::Minimized(true));
        }

        Some(
            button_close
                .rect
                .union(button_minimize_maximize.rect)
                .union(button_minimize.rect),
        )
    }
}
