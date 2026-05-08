use crate::{
    app_context::AppContext,
    ui::{draw::icon_draw::IconDraw, theme::Theme, widgets::controls::button::Button},
};
use eframe::egui::{Align, Layout, Sense, Ui, UiBuilder};
use epaint::{CornerRadius, vec2};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolExplorerToolbarAction {
    CreateModuleRoot,
}

#[derive(Clone)]
pub struct SymbolExplorerToolbarView {
    app_context: Arc<AppContext>,
    can_create_module_root: bool,
}

impl SymbolExplorerToolbarView {
    const TOOLBAR_HEIGHT: f32 = 28.0;
    const TOOLBAR_BUTTON_SIZE: f32 = 36.0;

    pub fn new(app_context: Arc<AppContext>) -> Self {
        Self {
            app_context,
            can_create_module_root: false,
        }
    }

    pub fn can_create_module_root(
        mut self,
        can_create_module_root: bool,
    ) -> Self {
        self.can_create_module_root = can_create_module_root;

        self
    }

    pub fn show(
        self,
        user_interface: &mut Ui,
    ) -> Option<SymbolExplorerToolbarAction> {
        let theme = &self.app_context.theme;
        let (toolbar_rect, _toolbar_response) =
            user_interface.allocate_exact_size(vec2(user_interface.available_width().max(1.0), Self::TOOLBAR_HEIGHT), Sense::empty());
        let mut clicked_action = None;

        user_interface
            .painter()
            .rect_filled(toolbar_rect, CornerRadius::ZERO, theme.background_primary);

        let mut toolbar_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(toolbar_rect)
                .layout(Layout::left_to_right(Align::Center)),
        );

        toolbar_user_interface.with_layout(Layout::left_to_right(Align::Center), |toolbar_user_interface| {
            if Self::draw_icon_button(
                toolbar_user_interface,
                theme,
                &theme.icon_library.icon_handle_common_add,
                "Add module.",
                self.can_create_module_root,
            )
            .clicked()
            {
                clicked_action = Some(SymbolExplorerToolbarAction::CreateModuleRoot);
            }
        });

        clicked_action
    }

    fn draw_icon_button(
        user_interface: &mut Ui,
        theme: &Theme,
        icon_handle: &epaint::TextureHandle,
        tooltip_text: &str,
        enabled: bool,
    ) -> eframe::egui::Response {
        let button_response = user_interface.add_sized(
            vec2(Self::TOOLBAR_BUTTON_SIZE, Self::TOOLBAR_HEIGHT),
            Button::new_from_theme(theme)
                .with_tooltip_text(tooltip_text)
                .background_color(epaint::Color32::TRANSPARENT)
                .disabled(!enabled),
        );

        IconDraw::draw_tinted(
            user_interface,
            button_response.rect,
            icon_handle,
            if enabled { theme.foreground } else { theme.foreground_preview },
        );

        button_response
    }
}
