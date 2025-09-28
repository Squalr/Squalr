use crate::ui::theme::Theme;
use crate::ui::widgets::controls::toolbar_menu::data_model::toolbar_menu_item_check_state::ToolbarMenuItemCheckState;
use crate::ui::widgets::controls::toolbar_menu::data_model::toolbar_menu_item_data::ToolbarMenuItemData;
use eframe::egui::{Align, Area, Frame, Id, Key, Layout, Order, Response, Sense, Ui, Widget};
use epaint::{Color32, Rect, pos2, vec2};
use std::rc::Rc;

pub struct ToolbarHeaderItemView<'a> {
    theme: Rc<Theme>,
    header: &'a String,
    items: &'a Vec<ToolbarMenuItemData>,
    height: f32,
    horizontal_padding: f32,
}

impl<'a> ToolbarHeaderItemView<'a> {
    pub fn new(
        theme: Rc<Theme>,
        header: &'a String,
        items: &'a Vec<ToolbarMenuItemData>,
        height: f32,
        horizontal_padding: f32,
    ) -> Self {
        Self {
            theme,
            header,
            items,
            height,
            horizontal_padding,
        }
    }
}

impl<'a> Widget for ToolbarHeaderItemView<'a> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        // Measure header text and compute padded size.
        let font_id = self.theme.font_library.font_noto_sans.font_header.clone();
        let text_color = self.theme.foreground;
        let header_galley = user_interface
            .ctx()
            .fonts(|fonts| fonts.layout_no_wrap(self.header.clone(), font_id.clone(), text_color));
        let text_size = header_galley.size();
        let style = user_interface.style().clone();
        let padding_vertical = style.spacing.button_padding.y.max(4.0);
        let padding_horizontal = self.horizontal_padding.max(style.spacing.button_padding.x);
        let desired = vec2(text_size.x + 2.0 * padding_horizontal, text_size.y + 2.0 * padding_vertical);
        let (available_size, response) = user_interface.allocate_exact_size(desired, Sense::click());
        let is_open_id = Id::new(("toolbar_menu_open", user_interface.id().value(), &self.header));
        let painter = user_interface.painter();

        if response.hovered() {
            painter.rect_filled(available_size, epaint::CornerRadius::ZERO, self.theme.hover_tint);
        }
        if response.is_pointer_button_down_on() {
            painter.rect_filled(available_size, epaint::CornerRadius::ZERO, self.theme.pressed_tint);
        }

        // Header label with a top left position for the galley.
        let text_pos = pos2(available_size.min.x + padding_horizontal, available_size.center().y - text_size.y * 0.5);

        painter.galley(text_pos, header_galley, Color32::RED);

        // Open / close logic.
        let mut open = user_interface.memory(|m| m.data.get_temp::<bool>(is_open_id).unwrap_or(false));

        if response.clicked() {
            open = !open;
        }
        if user_interface.input(|i| i.key_pressed(Key::Escape)) && open {
            open = false;
        }

        user_interface.memory_mut(|memory| memory.data.insert_temp(is_open_id, open));

        if !open {
            return response;
        }

        // Compute popup width from content (widest item text), with padding
        // Include header width so popup isn't narrower than the button.
        let mut widest = available_size.width();

        user_interface.ctx().fonts(|fonts| {
            for item in self.items.iter() {
                let galley = fonts.layout_no_wrap(
                    item.text.clone(),
                    self.theme.font_library.font_noto_sans.font_normal.clone(),
                    style.visuals.text_color(),
                );
                widest = widest.max(galley.size().x + 2.0 * style.spacing.button_padding.x);
            }
        });

        // Account for checkbox icon space (roughly the height of a row) when present.
        let checkbox_extra = style.spacing.icon_width_inner;
        let widest = widest + checkbox_extra.max(style.spacing.interact_size.y * 0.6);

        // Popup area just below header.
        let popup_id = Id::new(("toolbar_menu_popup", &self.header, user_interface.id().value()));
        let mut popup_rect: Option<Rect> = None;

        Area::new(popup_id)
            .order(Order::Foreground)
            .fixed_pos(pos2(available_size.min.x, available_size.max.y))
            .show(user_interface.ctx(), |popup_user_interface| {
                Frame::popup(user_interface.style())
                    .fill(self.theme.background_primary)
                    .show(popup_user_interface, |popup_user_interface| {
                        popup_user_interface.set_min_width(widest);
                        popup_user_interface.with_layout(Layout::top_down(Align::Min), |popup_user_interface| {
                            for (idx, item) in self.items.iter().enumerate() {
                                if item.has_separator && idx != 0 {
                                    popup_user_interface.separator();
                                }

                                match item.check_state {
                                    ToolbarMenuItemCheckState::None => {
                                        if popup_user_interface.button(&item.text).clicked() {
                                            user_interface.memory_mut(|m| m.data.insert_temp(is_open_id, false));
                                        }
                                    }
                                    _ => {
                                        let mut checked = item.check_state == ToolbarMenuItemCheckState::Checked;
                                        let resp = popup_user_interface.checkbox(&mut checked, &item.text);
                                        if resp.clicked() {
                                            user_interface.memory_mut(|m| m.data.insert_temp(is_open_id, false));
                                        }
                                    }
                                }
                            }
                        });

                        // Capture the full popup rect for click-outside logic.
                        popup_rect = Some(popup_user_interface.min_rect());
                    });
            });

        // Close when clicking outside both the header and the popup.
        if user_interface.input(|input_state| {
            if !input_state.pointer.any_click() {
                return false;
            }

            let pos = input_state
                .pointer
                .interact_pos()
                .unwrap_or(available_size.center());
            let outside_header = !available_size.contains(pos);
            let outside_popup = popup_rect.map_or(true, |r| !r.contains(pos));
            outside_header && outside_popup
        }) {
            user_interface.memory_mut(|memory| memory.data.insert_temp(is_open_id, false));
        }

        response
    }
}
