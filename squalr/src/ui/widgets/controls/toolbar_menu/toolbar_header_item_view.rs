use crate::models::toolbar::toolbar_menu_item_data::ToolbarMenuItemData;
use crate::ui::widgets::controls::state_layer::StateLayer;
use crate::{app_context::AppContext, ui::widgets::controls::toolbar_menu::toolbar_menu_item_view::ToolbarMenuItemView};
use eframe::egui::{Align, Area, Frame, Id, Key, Layout, Order, Response, Sense, Ui, Widget};
use epaint::{CornerRadius, Rect, pos2, vec2};
use smallvec::SmallVec;
use std::rc::Rc;

pub struct ToolbarHeaderItemView<'lifetime> {
    app_context: Rc<AppContext>,
    header: &'lifetime String,
    items: &'lifetime SmallVec<[ToolbarMenuItemData; 24]>,
    width: f32,
    height: f32,
    horizontal_padding: f32,
}

impl<'lifetime> ToolbarHeaderItemView<'lifetime> {
    pub fn new(
        app_context: Rc<AppContext>,
        header: &'lifetime String,
        items: &'lifetime SmallVec<[ToolbarMenuItemData; 24]>,
        width: f32,
        height: f32,
        horizontal_padding: f32,
    ) -> Self {
        Self {
            app_context,
            header,
            items,
            width,
            height,
            horizontal_padding,
        }
    }
}

impl<'lifetime> Widget for ToolbarHeaderItemView<'lifetime> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        // Measure header text and compute padded size.
        let theme = &self.app_context.theme;
        let font_id = theme.font_library.font_noto_sans.font_header.clone();
        let text_color = theme.foreground;

        let header_galley = user_interface
            .ctx()
            .fonts(|fonts| fonts.layout_no_wrap(self.header.clone(), font_id.clone(), text_color));
        let text_size = header_galley.size();

        let style = user_interface.style().clone();
        let padding_vertical = style.spacing.button_padding.y.max(4.0);
        let padding_horizontal = self.horizontal_padding.max(style.spacing.button_padding.x);

        let desired = vec2(
            text_size.x + 2.0 * padding_horizontal,
            (self.height.max(0.0)).max(text_size.y + 2.0 * padding_vertical),
        );

        let (available_size_rectangle, response) = user_interface.allocate_exact_size(desired, Sense::click());

        // Compose the StateLayer (hover/press/focus) like the Button impl.
        StateLayer {
            bounds_min: available_size_rectangle.min,
            bounds_max: available_size_rectangle.max,
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

        // Header label centered vertically.
        let text_pos = pos2(
            available_size_rectangle.min.x + padding_horizontal,
            available_size_rectangle.center().y - text_size.y * 0.5,
        );

        user_interface
            .painter()
            .galley(text_pos, header_galley, text_color);

        // Open / close logic.
        let is_open_id = Id::new(("toolbar_menu_open", user_interface.id().value(), &self.header));
        let mut open = user_interface.memory(|memory| memory.data.get_temp::<bool>(is_open_id).unwrap_or(false));

        if response.clicked() {
            open = !open;
        }
        if user_interface.input(|input_state| input_state.key_pressed(Key::Escape)) && open {
            open = false;
        }
        user_interface.memory_mut(|memory| memory.data.insert_temp(is_open_id, open));

        if !open {
            return response;
        }

        // Compute popup width from content (widest item text), with padding
        // Include header width so popup isn't narrower than the button.
        let mut widest = available_size_rectangle.width();

        user_interface.ctx().fonts(|fonts| {
            for item in self.items.iter() {
                let galley = fonts.layout_no_wrap(
                    item.text.clone(),
                    theme.font_library.font_noto_sans.font_normal.clone(),
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
        let mut popup_rectangle: Option<Rect> = None;

        Area::new(popup_id)
            .order(Order::Foreground)
            .fixed_pos(pos2(available_size_rectangle.min.x, available_size_rectangle.max.y))
            .show(user_interface.ctx(), |popup_user_interface| {
                Frame::popup(user_interface.style())
                    .fill(theme.background_primary)
                    .corner_radius(CornerRadius::ZERO)
                    .inner_margin(0)
                    .show(popup_user_interface, |popup_user_interface| {
                        popup_user_interface.set_min_width(widest);
                        popup_user_interface.with_layout(Layout::top_down(Align::Min), |popup_user_interface| {
                            for (index, item) in self.items.iter().enumerate() {
                                if item.has_separator && index != 0 {
                                    popup_user_interface.separator();
                                }

                                popup_user_interface.add(ToolbarMenuItemView::new(
                                    self.app_context.clone(),
                                    &item.text,
                                    item.check_state.clone(),
                                    self.width,
                                ));
                            }
                        });

                        // Capture the full popup rect for click-outside logic.
                        popup_rectangle = Some(popup_user_interface.min_rect());
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
                .unwrap_or(available_size_rectangle.center());
            let outside_header = !available_size_rectangle.contains(pos);
            let outside_popup = popup_rectangle.map_or(true, |rectangle| !rectangle.contains(pos));

            outside_header && outside_popup
        }) {
            user_interface.memory_mut(|memory| memory.data.insert_temp(is_open_id, false));
        }

        response
    }
}
