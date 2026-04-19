use crate::{
    app_context::AppContext,
    ui::{draw::icon_draw::IconDraw, widgets::controls::button::Button as ThemeButton},
};
use eframe::egui::{Align, Response, Sense, Ui, UiBuilder};
use epaint::{Color32, CornerRadius, vec2};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolTableToolbarAction {
    CreateRootedSymbol,
    OpenSelectedInCodeViewer,
    OpenSelectedInMemoryViewer,
    DeleteSelectedRootedSymbol,
}

#[derive(Clone)]
pub struct SymbolTableToolbarView {
    app_context: Arc<AppContext>,
    can_create_rooted_symbol: bool,
    can_delete_rooted_symbol: bool,
    can_open_in_code_viewer: bool,
    can_open_in_memory_viewer: bool,
}

impl SymbolTableToolbarView {
    const TOOLBAR_HEIGHT: f32 = 28.0;
    const TOOLBAR_BUTTON_WIDTH: f32 = 36.0;

    pub fn new(app_context: Arc<AppContext>) -> Self {
        Self {
            app_context,
            can_create_rooted_symbol: false,
            can_delete_rooted_symbol: false,
            can_open_in_code_viewer: false,
            can_open_in_memory_viewer: false,
        }
    }

    pub fn can_create_rooted_symbol(
        mut self,
        can_create_rooted_symbol: bool,
    ) -> Self {
        self.can_create_rooted_symbol = can_create_rooted_symbol;
        self
    }

    pub fn can_delete_rooted_symbol(
        mut self,
        can_delete_rooted_symbol: bool,
    ) -> Self {
        self.can_delete_rooted_symbol = can_delete_rooted_symbol;
        self
    }

    pub fn can_open_in_code_viewer(
        mut self,
        can_open_in_code_viewer: bool,
    ) -> Self {
        self.can_open_in_code_viewer = can_open_in_code_viewer;
        self
    }

    pub fn can_open_in_memory_viewer(
        mut self,
        can_open_in_memory_viewer: bool,
    ) -> Self {
        self.can_open_in_memory_viewer = can_open_in_memory_viewer;
        self
    }

    pub fn show(
        self,
        user_interface: &mut Ui,
    ) -> Option<SymbolTableToolbarAction> {
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
                .layout(eframe::egui::Layout::left_to_right(Align::Center)),
        );

        if self.can_create_rooted_symbol
            && Self::draw_icon_button(
                &mut toolbar_user_interface,
                theme,
                &theme.icon_library.icon_handle_common_add,
                "Create a new rooted symbol.",
            )
            .clicked()
        {
            clicked_action = Some(SymbolTableToolbarAction::CreateRootedSymbol);
        }

        if self.can_open_in_memory_viewer
            && Self::draw_icon_button(
                &mut toolbar_user_interface,
                theme,
                &theme.icon_library.icon_handle_scan_collect_values,
                "Open selected symbol in Memory Viewer.",
            )
            .clicked()
        {
            clicked_action = Some(SymbolTableToolbarAction::OpenSelectedInMemoryViewer);
        }

        if self.can_open_in_code_viewer
            && Self::draw_icon_button(
                &mut toolbar_user_interface,
                theme,
                &theme.icon_library.icon_handle_project_cpu_instruction,
                "Open selected symbol in Code Viewer.",
            )
            .clicked()
        {
            clicked_action = Some(SymbolTableToolbarAction::OpenSelectedInCodeViewer);
        }

        if self.can_delete_rooted_symbol
            && Self::draw_icon_button(
                &mut toolbar_user_interface,
                theme,
                &theme.icon_library.icon_handle_common_delete,
                "Delete selected rooted symbol.",
            )
            .clicked()
        {
            clicked_action = Some(SymbolTableToolbarAction::DeleteSelectedRootedSymbol);
        }

        clicked_action
    }

    fn draw_icon_button(
        user_interface: &mut Ui,
        theme: &crate::ui::theme::Theme,
        icon_handle: &eframe::egui::TextureHandle,
        tooltip_text: &str,
    ) -> Response {
        let response = user_interface.add_sized(
            vec2(Self::TOOLBAR_BUTTON_WIDTH, Self::TOOLBAR_HEIGHT),
            ThemeButton::new_from_theme(theme)
                .with_tooltip_text(tooltip_text)
                .background_color(Color32::TRANSPARENT),
        );

        IconDraw::draw(user_interface, response.rect, icon_handle);

        response
    }
}
