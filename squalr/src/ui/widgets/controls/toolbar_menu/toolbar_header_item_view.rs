use crate::models::toolbar::toolbar_menu_item_data::ToolbarMenuItemData;
use crate::ui::widgets::controls::state_layer::StateLayer;
use crate::{app_context::AppContext, ui::widgets::controls::toolbar_menu::toolbar_menu_item_view::ToolbarMenuItemView};
use eframe::egui::{Align, Area, Frame, Id, Layout, Order, Response, Sense, Ui, Widget};
use epaint::{CornerRadius, Rect, pos2, vec2};
use smallvec::SmallVec;
use std::sync::Arc;

pub struct ToolbarHeaderItemView<'lifetime> {
    app_context: Arc<AppContext>,
    header: &'lifetime String,
    items: &'lifetime SmallVec<[ToolbarMenuItemData; 24]>,
    width: f32,
    height: f32,
    horizontal_padding: f32,
    on_select: &'lifetime dyn Fn(&'lifetime str),
}

impl<'lifetime> ToolbarHeaderItemView<'lifetime> {
    pub fn new(
        app_context: Arc<AppContext>,
        header: &'lifetime String,
        items: &'lifetime SmallVec<[ToolbarMenuItemData; 24]>,
        width: f32,
        height: f32,
        horizontal_padding: f32,
        on_select: &'lifetime dyn Fn(&'lifetime str),
    ) -> Self {
        Self {
            app_context,
            header,
            items,
            width,
            height,
            horizontal_padding,
            on_select,
        }
    }
}

impl<'lifetime> Widget for ToolbarHeaderItemView<'lifetime> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        // Basic drawing & layout
        let theme = &self.app_context.theme;
        let font_id = theme.font_library.font_noto_sans.font_header.clone();
        let text_color = theme.foreground;
        let header_galley = user_interface
            .ctx()
            .fonts_mut(|fonts| fonts.layout_no_wrap(self.header.clone(), font_id.clone(), text_color));
        let text_size = header_galley.size();
        let style = user_interface.style().clone();
        let padding_v = style.spacing.button_padding.y.max(4.0);
        let padding_h = self.horizontal_padding.max(style.spacing.button_padding.x);
        let desired = vec2(text_size.x + 2.0 * padding_h, self.height.max(text_size.y + 2.0 * padding_v));
        let (allocated_size_rectangle, response) = user_interface.allocate_exact_size(desired, Sense::click());

        StateLayer {
            bounds_min: allocated_size_rectangle.min,
            bounds_max: allocated_size_rectangle.max,
            enabled: true,
            pressed: response.is_pointer_button_down_on(),
            has_hover: response.hovered(),
            has_focus: response.has_focus(),
            corner_radius: CornerRadius::ZERO,
            border_width: 0.0,
            hover_color: theme.hover_tint,
            pressed_color: theme.pressed_tint,
            border_color: theme.background_control_primary_light,
            border_color_focused: theme.background_control_primary_light,
        }
        .ui(user_interface);

        let text_pos = pos2(
            allocated_size_rectangle.min.x + padding_h,
            allocated_size_rectangle.center().y - text_size.y * 0.5,
        );
        user_interface
            .painter()
            .galley(text_pos, header_galley, text_color);

        // Menu open state logic.
        let open_menu_id = Id::new("toolbar_current_open_menu");

        // Get the currently open header (if any).
        let mut open_header = user_interface.memory(|mem| mem.data.get_temp::<String>(open_menu_id));
        let is_this_open = open_header.as_deref() == Some(self.header);

        // Toggle open/close on click.
        if response.clicked() {
            if is_this_open {
                user_interface.memory_mut(|memory| memory.data.remove::<String>(open_menu_id));
                return response;
            } else {
                user_interface.memory_mut(|memory| memory.data.insert_temp(open_menu_id, self.header.clone()));
            }
        }

        // Hovering while another is open switches the open one.
        if response.hovered() && open_header.is_some() && !is_this_open {
            user_interface.memory_mut(|memory| memory.data.insert_temp(open_menu_id, self.header.clone()));
        }

        // Refresh open_header after possible update.
        open_header = user_interface.memory(|memory| memory.data.get_temp::<String>(open_menu_id));

        let is_open = open_header.as_deref() == Some(self.header);

        if !is_open {
            return response;
        }

        // Compute popup width.
        let mut widest = allocated_size_rectangle.width();

        user_interface.ctx().fonts_mut(|fonts| {
            for item in self.items.iter() {
                let galley = fonts.layout_no_wrap(
                    item.text.clone(),
                    theme.font_library.font_noto_sans.font_normal.clone(),
                    style.visuals.text_color(),
                );
                widest = widest.max(galley.size().x + 2.0 * style.spacing.button_padding.x);
            }
        });

        let widest = widest
            + style
                .spacing
                .icon_width_inner
                .max(style.spacing.interact_size.y * 0.6);

        // Popup drawing.
        let popup_id = Id::new(("toolbar_menu_popup", &self.header, user_interface.id().value()));
        let mut popup_rectangle: Option<Rect> = None;

        Area::new(popup_id)
            .order(Order::Foreground)
            .fixed_pos(pos2(allocated_size_rectangle.min.x, allocated_size_rectangle.max.y))
            .show(user_interface.ctx(), |popup_user_interface| {
                Frame::popup(user_interface.style())
                    .fill(theme.background_primary)
                    .corner_radius(CornerRadius::ZERO)
                    .inner_margin(0)
                    .show(popup_user_interface, |popup_user_interface| {
                        popup_user_interface.set_min_width(widest);
                        popup_user_interface.with_layout(Layout::top_down(Align::Min), |popup_popup_user_interface| {
                            for (index, item) in self.items.iter().enumerate() {
                                if item.has_separator && index != 0 {
                                    popup_popup_user_interface.separator();
                                }

                                let item_response = popup_popup_user_interface.add(ToolbarMenuItemView::new(
                                    self.app_context.clone(),
                                    &item.text,
                                    &item.id,
                                    &item.check_state,
                                    self.width,
                                ));

                                if item_response.clicked() {
                                    user_interface.memory_mut(|memory| memory.data.remove::<String>(open_menu_id));
                                    (self.on_select)(&item.id);
                                }
                            }
                        });

                        popup_rectangle = Some(popup_user_interface.min_rect());
                    });
            });

        // Close when clicking outside.
        if user_interface.input(|input| {
            if !input.pointer.any_click() {
                return false;
            }

            let position = input
                .pointer
                .interact_pos()
                .unwrap_or(allocated_size_rectangle.center());
            let outside_header = !allocated_size_rectangle.contains(position);
            let outside_popup = popup_rectangle.map_or(true, |r| !r.contains(position));
            outside_header && outside_popup
        }) {
            user_interface.memory_mut(|memory| memory.data.remove::<String>(open_menu_id));
        }

        response
    }
}
