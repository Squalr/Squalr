use crate::ui::widgets::controls::toolbar_menu::toolbar_menu_check_state::ToolbarMenuCheckState;
use crate::ui::widgets::controls::toolbar_menu::toolbar_menu_item_data::ToolbarMenuItemData;
use crate::ui::{theme::Theme, widgets::controls::toolbar_menu::toolbar_menu_data::ToolbarMenuData};
use eframe::egui::{Align, Id, Key, Response, Sense, Ui, Widget};
use std::rc::Rc;

pub struct ToolbarMenuButton<'a> {
    pub theme: Rc<Theme>,
    pub label: &'a String,
    pub items: &'a Vec<ToolbarMenuItemData>,
    /// When a menu item is clicked, we write its id here.
    pub clicked_out: Option<*mut Option<String>>, // raw ptr to &mut Option<String> (safe with care)
    pub height: f32,
    pub horizontal_padding: f32,
}

impl<'a> ToolbarMenuButton<'a> {
    pub fn from_menu(
        theme: Rc<Theme>,
        menu: &'a ToolbarMenuData,
        height: f32,
    ) -> Self {
        Self {
            theme,
            label: &menu.header,
            items: &menu.items,
            clicked_out: None,
            height,
            horizontal_padding: 8.0,
        }
    }

    fn write_clicked(
        &self,
        id: &str,
    ) {
        if let Some(ptr) = self.clicked_out {
            // Safety: we only set this to a stack &mut Option<String> from the parent show method per frame.
            unsafe {
                *ptr = Some(id.to_owned());
            }
        }
    }
}

impl<'a> Widget for ToolbarMenuButton<'a> {
    fn ui(
        self,
        ui: &mut Ui,
    ) -> Response {
        // The top-level button (header)
        let desired = eframe::egui::vec2(ui.spacing().interact_size.x + self.horizontal_padding * 2.0, self.height);
        let (rect, resp) = ui.allocate_exact_size(desired, Sense::click());
        let is_open_id = Id::new(("toolbar_menu_open", ui.id().value(), &self.label));

        // Background hover/pressed
        let painter = ui.painter();
        if resp.hovered() {
            painter.rect_filled(rect, epaint::CornerRadius::ZERO, self.theme.hover_tint);
        }
        if resp.is_pointer_button_down_on() {
            painter.rect_filled(rect, epaint::CornerRadius::ZERO, self.theme.pressed_tint);
        }

        // Label
        painter.text(
            eframe::egui::pos2(rect.center().x, rect.center().y),
            eframe::egui::Align2::CENTER_CENTER,
            &self.label,
            ui.style()
                .text_styles
                .get(&eframe::egui::TextStyle::Button)
                .unwrap()
                .clone(),
            self.theme.foreground,
        );

        // Open / close logic (click or keyboard navigation)
        let mut open = ui.memory(|m| m.data.get_temp::<bool>(is_open_id).unwrap_or(false));
        if resp.clicked() {
            open = !open;
        }
        if ui.input(|i| i.key_pressed(Key::Escape)) && open {
            open = false;
        }
        ui.memory_mut(|m| m.data.insert_temp(is_open_id, open));

        // Popup
        if open {
            // A simple popup aligned below the header rect:
            let popup_id = Id::new(("toolbar_menu_popup", &self.label, ui.id().value()));
            eframe::egui::Area::new(popup_id)
                .order(eframe::egui::Order::Foreground)
                .fixed_pos(eframe::egui::pos2(rect.min.x, rect.max.y))
                .show(ui.ctx(), |ui_popup| {
                    let width = 200.0_f32.max(rect.width()); // min width
                    eframe::egui::Frame::popup(ui.style())
                        .fill(self.theme.background_primary)
                        .show(ui_popup, |ui_popup| {
                            ui_popup.set_width(width);
                            ui_popup.with_layout(eframe::egui::Layout::top_down(Align::Min), |ui_popup| {
                                for (idx, item) in self.items.iter().enumerate() {
                                    if item.has_separator && idx != 0 {
                                        ui_popup.separator();
                                    }

                                    match item.check_state {
                                        ToolbarMenuCheckState::None => {
                                            if ui_popup.button(&item.text).clicked() {
                                                self.write_clicked(&item.id);
                                                // close the menu
                                                ui.memory_mut(|m| m.data.insert_temp(is_open_id, false));
                                            }
                                        }
                                        _ => {
                                            let mut checked = item.check_state == ToolbarMenuCheckState::Checked;
                                            let resp = ui_popup.checkbox(&mut checked, &item.text);
                                            // treat any click as "activate" (not toggling state here, that lives in caller)
                                            if resp.clicked() {
                                                self.write_clicked(&item.id);
                                                ui.memory_mut(|m| m.data.insert_temp(is_open_id, false));
                                            }
                                        }
                                    }
                                }
                            });
                        });
                });
            // Clicking outside closes it:
            if ui.input(|i| i.pointer.any_click() && !rect.contains(i.pointer.interact_pos().unwrap_or(rect.center()))) {
                ui.memory_mut(|m| m.data.insert_temp(is_open_id, false));
            }
        }

        resp
    }
}
