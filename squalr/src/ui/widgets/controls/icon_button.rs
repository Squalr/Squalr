use crate::ui::{draw::icon_draw::IconDraw, theme::Theme, widgets::controls::button::Button};
use eframe::egui::{Color32, Response, Ui, Widget};
use epaint::{CornerRadius, TextureHandle, Vec2};

pub struct IconButtonView<'view> {
    theme: &'view Theme,
    icon_handle: &'view TextureHandle,
    tooltip_text: &'view str,
    is_disabled: bool,
    background_color: Color32,
    border_color: Color32,
    border_width: f32,
    corner_radius: CornerRadius,
    icon_size: Option<Vec2>,
    enabled_icon_tint: Color32,
    disabled_icon_tint: Color32,
}

impl<'view> IconButtonView<'view> {
    pub fn new(
        theme: &'view Theme,
        icon_handle: &'view TextureHandle,
        tooltip_text: &'view str,
    ) -> Self {
        Self {
            theme,
            icon_handle,
            tooltip_text,
            is_disabled: false,
            background_color: Color32::TRANSPARENT,
            border_color: Color32::TRANSPARENT,
            border_width: 0.0,
            corner_radius: CornerRadius::ZERO,
            icon_size: None,
            enabled_icon_tint: theme.foreground,
            disabled_icon_tint: theme.foreground_preview,
        }
    }

    pub fn disabled(
        mut self,
        is_disabled: bool,
    ) -> Self {
        self.is_disabled = is_disabled;
        self
    }

    pub fn background_color(
        mut self,
        background_color: Color32,
    ) -> Self {
        self.background_color = background_color;
        self
    }

    pub fn border_color(
        mut self,
        border_color: Color32,
    ) -> Self {
        self.border_color = border_color;
        self
    }

    pub fn border_width(
        mut self,
        border_width: f32,
    ) -> Self {
        self.border_width = border_width;
        self
    }

    pub fn corner_radius(
        mut self,
        corner_radius: CornerRadius,
    ) -> Self {
        self.corner_radius = corner_radius;
        self
    }

    pub fn icon_size(
        mut self,
        icon_size: Vec2,
    ) -> Self {
        self.icon_size = Some(icon_size);
        self
    }

    pub fn enabled_icon_tint(
        mut self,
        enabled_icon_tint: Color32,
    ) -> Self {
        self.enabled_icon_tint = enabled_icon_tint;
        self
    }
}

impl Widget for IconButtonView<'_> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let button_response = user_interface.add(
            Button::new_from_theme(self.theme)
                .with_tooltip_text(self.tooltip_text)
                .background_color(self.background_color)
                .border_color(self.border_color)
                .border_width(self.border_width)
                .corner_radius(self.corner_radius)
                .disabled(self.is_disabled),
        );
        let icon_tint = if self.is_disabled { self.disabled_icon_tint } else { self.enabled_icon_tint };

        if let Some(icon_size) = self.icon_size {
            IconDraw::draw_sized_tinted(user_interface, button_response.rect.center(), icon_size, self.icon_handle, icon_tint);
        } else {
            IconDraw::draw_tinted(user_interface, button_response.rect, self.icon_handle, icon_tint);
        }

        button_response
    }
}
