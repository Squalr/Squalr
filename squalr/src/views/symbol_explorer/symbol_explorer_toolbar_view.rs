use crate::{
    app_context::AppContext,
    ui::{draw::icon_draw::IconDraw, theme::Theme, widgets::controls::button::Button},
};
use eframe::egui::{Align, Layout, Sense, Ui, UiBuilder};
use epaint::{Color32, CornerRadius, vec2};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolExplorerToolbarAction {
    CreateModuleRoot,
    RenameSelectedEntry,
    DeleteSelectedEntry,
    OpenSelectedInCodeViewer,
    OpenSelectedInMemoryViewer,
}

#[derive(Clone)]
pub struct SymbolExplorerToolbarView {
    app_context: Arc<AppContext>,
    can_create_module_root: bool,
    can_rename_selected_entry: bool,
    can_delete_selected_entry: bool,
    can_open_in_code_viewer: bool,
    can_open_in_memory_viewer: bool,
}

impl SymbolExplorerToolbarView {
    const TOOLBAR_HEIGHT: f32 = 28.0;
    const TOOLBAR_BUTTON_SIZE: f32 = 36.0;

    pub fn new(app_context: Arc<AppContext>) -> Self {
        Self {
            app_context,
            can_create_module_root: false,
            can_rename_selected_entry: false,
            can_delete_selected_entry: false,
            can_open_in_code_viewer: false,
            can_open_in_memory_viewer: false,
        }
    }

    pub fn can_create_module_root(
        mut self,
        can_create_module_root: bool,
    ) -> Self {
        self.can_create_module_root = can_create_module_root;

        self
    }

    pub fn can_rename_selected_entry(
        mut self,
        can_rename_selected_entry: bool,
    ) -> Self {
        self.can_rename_selected_entry = can_rename_selected_entry;

        self
    }

    pub fn can_delete_selected_entry(
        mut self,
        can_delete_selected_entry: bool,
    ) -> Self {
        self.can_delete_selected_entry = can_delete_selected_entry;

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

        toolbar_user_interface.with_layout(Layout::right_to_left(Align::Center), |toolbar_user_interface| {
            if Self::draw_icon_button(
                toolbar_user_interface,
                theme,
                &theme.icon_library.icon_handle_common_delete,
                "Delete selected module, symbol, or field.",
                self.can_delete_selected_entry,
            )
            .clicked()
            {
                clicked_action = Some(SymbolExplorerToolbarAction::DeleteSelectedEntry);
            }

            if Self::draw_icon_button(
                toolbar_user_interface,
                theme,
                &theme.icon_library.icon_handle_common_edit,
                "Rename selected module, symbol, or field.",
                self.can_rename_selected_entry,
            )
            .clicked()
            {
                clicked_action = Some(SymbolExplorerToolbarAction::RenameSelectedEntry);
            }

            if Self::draw_icon_button(
                toolbar_user_interface,
                theme,
                &theme.icon_library.icon_handle_project_cpu_instruction,
                "Open selected symbol or field in Code Viewer.",
                self.can_open_in_code_viewer,
            )
            .clicked()
            {
                clicked_action = Some(SymbolExplorerToolbarAction::OpenSelectedInCodeViewer);
            }

            if Self::draw_icon_button(
                toolbar_user_interface,
                theme,
                &theme.icon_library.icon_handle_scan_collect_values,
                "Open selected symbol or field in Memory Viewer.",
                self.can_open_in_memory_viewer,
            )
            .clicked()
            {
                clicked_action = Some(SymbolExplorerToolbarAction::OpenSelectedInMemoryViewer);
            }

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
                .background_color(Color32::TRANSPARENT)
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
