use crate::ui::theme::Theme;
use crate::ui::widgets::controls::toolbar_menu::data_model::toolbar_menu_item_check_state::ToolbarMenuItemCheckState;
use crate::ui::widgets::controls::toolbar_menu::data_model::toolbar_menu_item_data::ToolbarMenuItemData;
use eframe::egui::{Align, Id, Key, Response, Sense, Ui, Widget};
use std::rc::Rc;

pub struct ToolbarButtonView<'a> {
    theme: Rc<Theme>,
    label: &'a String,
    items: &'a Vec<ToolbarMenuItemData>,
    height: f32,
    horizontal_padding: f32,
}

impl<'a> ToolbarButtonView<'a> {
    pub fn new(
        theme: Rc<Theme>,
        label: &'a String,
        items: &'a Vec<ToolbarMenuItemData>,
        height: f32,
        horizontal_padding: f32,
    ) -> Self {
        Self {
            theme,
            label,
            items,
            height,
            horizontal_padding,
        }
    }
}

impl<'a> Widget for ToolbarButtonView<'a> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        // The top-level button (header)
        let desired = eframe::egui::vec2(user_interface.spacing().interact_size.x + self.horizontal_padding * 2.0, self.height);
        let (rect, response) = user_interface.allocate_exact_size(desired, Sense::click());
        let is_open_id = Id::new(("toolbar_menu_open", user_interface.id().value(), &self.label));

        // Background hover/pressed
        let painter = user_interface.painter();
        if response.hovered() {
            painter.rect_filled(rect, epaint::CornerRadius::ZERO, self.theme.hover_tint);
        }
        if response.is_pointer_button_down_on() {
            painter.rect_filled(rect, epaint::CornerRadius::ZERO, self.theme.pressed_tint);
        }

        // Label
        painter.text(
            eframe::egui::pos2(rect.center().x, rect.center().y),
            eframe::egui::Align2::CENTER_CENTER,
            &self.label,
            user_interface
                .style()
                .text_styles
                .get(&eframe::egui::TextStyle::Button)
                .unwrap()
                .clone(),
            self.theme.foreground,
        );

        // Open / close logic (click or keyboard navigation).
        let mut open = user_interface.memory(|memory| memory.data.get_temp::<bool>(is_open_id).unwrap_or(false));

        if response.clicked() {
            open = !open;
        }

        if user_interface.input(|input_state| input_state.key_pressed(Key::Escape)) && open {
            open = false;
        }

        user_interface.memory_mut(|memory| memory.data.insert_temp(is_open_id, open));

        // Popup.
        if !open {
            return response;
        }

        // A simple popup aligned below the header rect.
        let popup_id = Id::new(("toolbar_menu_popup", &self.label, user_interface.id().value()));

        eframe::egui::Area::new(popup_id)
            .order(eframe::egui::Order::Foreground)
            .fixed_pos(eframe::egui::pos2(rect.min.x, rect.max.y))
            .show(user_interface.ctx(), |ui_popup| {
                let width = 200.0_f32.max(rect.width()); // min width
                eframe::egui::Frame::popup(user_interface.style())
                    .fill(self.theme.background_primary)
                    .show(ui_popup, |ui_popup| {
                        ui_popup.set_width(width);
                        ui_popup.with_layout(eframe::egui::Layout::top_down(Align::Min), |ui_popup| {
                            for (idx, item) in self.items.iter().enumerate() {
                                if item.has_separator && idx != 0 {
                                    ui_popup.separator();
                                }

                                match item.check_state {
                                    ToolbarMenuItemCheckState::None => {
                                        if ui_popup.button(&item.text).clicked() {
                                            // Close the menu.
                                            user_interface.memory_mut(|memory| memory.data.insert_temp(is_open_id, false));
                                        }
                                    }
                                    _ => {
                                        let mut checked = item.check_state == ToolbarMenuItemCheckState::Checked;
                                        let response = ui_popup.checkbox(&mut checked, &item.text);

                                        // Treat any click as "activate" (not toggling state here, that lives in caller).
                                        if response.clicked() {
                                            user_interface.memory_mut(|memory| memory.data.insert_temp(is_open_id, false));
                                        }
                                    }
                                }
                            }
                        });
                    });
            });

        // Clicking outside closes it.
        if user_interface.input(|i| i.pointer.any_click() && !rect.contains(i.pointer.interact_pos().unwrap_or(rect.center()))) {
            user_interface.memory_mut(|m| m.data.insert_temp(is_open_id, false));
        }

        response
    }
}
