use crate::app_context::AppContext;
use eframe::egui::{Align, Area, Frame, Id, Key, Layout, Order, Ui};
use epaint::{CornerRadius, Margin, Rect, Vec2};
use std::sync::Arc;

/// A generic context menu popup that displays arbitrary content.
/// The caller fully controls open/close via `open: &mut bool`.
pub struct ContextMenu<'a, F: FnOnce(&mut Ui, &mut bool)> {
    app_context: Arc<AppContext>,
    menu_id: &'a str,
    /// Top-left position of the popup (caller decides).
    pos: epaint::Pos2,
    add_contents: F,
    width: f32,
    corner_radius: u8,
    close_on_escape: bool,
    close_on_click_outside: bool,
}

impl<'a, F: FnOnce(&mut Ui, &mut bool)> ContextMenu<'a, F> {
    pub fn new(
        app_context: Arc<AppContext>,
        menu_id: &'a str,
        pos: epaint::Pos2,
        add_contents: F,
    ) -> Self {
        Self {
            app_context,
            menu_id,
            pos,
            add_contents,
            width: 192.0,
            corner_radius: 0,
            close_on_escape: true,
            close_on_click_outside: true,
        }
    }

    pub fn width(
        mut self,
        width: f32,
    ) -> Self {
        self.width = width;
        self
    }

    pub fn corner_radius(
        mut self,
        corner_radius: u8,
    ) -> Self {
        self.corner_radius = corner_radius;
        self
    }

    pub fn close_on_escape(
        mut self,
        enabled: bool,
    ) -> Self {
        self.close_on_escape = enabled;
        self
    }

    pub fn close_on_click_outside(
        mut self,
        enabled: bool,
    ) -> Self {
        self.close_on_click_outside = enabled;
        self
    }

    /// Show the context menu if `open` is true. The caller owns `open`.
    ///
    /// Returns the popup rectangle if shown.
    pub fn show(
        self,
        ui: &mut Ui,
        open: &mut bool,
    ) -> Option<Rect> {
        if !*open {
            return None;
        }

        let theme = &self.app_context.theme;

        // Close on escape key.
        if self.close_on_escape && ui.input(|i| i.key_pressed(Key::Escape)) {
            *open = false;
            return None;
        }

        let popup_area_id = Id::new((self.menu_id, ui.id().value(), "context_menu_area"));

        let mut should_close = false;

        let area_response = Area::new(popup_area_id)
            .order(Order::Foreground)
            .fixed_pos(self.pos)
            .show(ui.ctx(), |popup_ui| {
                Frame::popup(ui.style())
                    .fill(theme.background_primary)
                    .inner_margin(Margin::ZERO)
                    .corner_radius(CornerRadius::same(self.corner_radius))
                    .show(popup_ui, |popup_ui| {
                        popup_ui.spacing_mut().menu_margin = Margin::ZERO;
                        popup_ui.spacing_mut().window_margin = Margin::ZERO;
                        popup_ui.spacing_mut().menu_spacing = 0.0;
                        popup_ui.spacing_mut().item_spacing = Vec2::ZERO;
                        popup_ui.set_min_width(self.width);

                        popup_ui.with_layout(Layout::top_down(Align::Min), |inner_ui| {
                            (self.add_contents)(inner_ui, &mut should_close);
                        });
                    });
            });

        let popup_rectangle = area_response.response.rect;

        // Close on click outside.
        if self.close_on_click_outside {
            let clicked_outside = ui.input(|input| {
                if !input.pointer.any_click() {
                    return false;
                }
                let click_pos = input.pointer.interact_pos().unwrap_or(self.pos);
                !popup_rectangle.contains(click_pos)
            });

            if clicked_outside {
                should_close = true;
            }
        }

        if should_close {
            *open = false;
            None
        } else {
            Some(popup_rectangle)
        }
    }
}
