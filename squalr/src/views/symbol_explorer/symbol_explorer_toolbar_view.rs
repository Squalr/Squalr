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
    DeleteSelectedSymbolClaim,
    OpenSelectedInCodeViewer,
    OpenSelectedInMemoryViewer,
    PromoteSelectedDerivedSymbol,
    CancelCreateModuleRoot,
    CommitCreateModuleRoot,
}

#[derive(Clone)]
pub struct SymbolExplorerToolbarView {
    app_context: Arc<AppContext>,
    show_actions: bool,
    can_create_module_root: bool,
    can_rename_selected_entry: bool,
    can_delete_symbol_claim: bool,
    can_open_in_code_viewer: bool,
    can_open_in_memory_viewer: bool,
    can_promote_derived_symbol: bool,
    can_cancel_create_module_root: bool,
    can_commit_create_module_root: bool,
}

impl SymbolExplorerToolbarView {
    const TOOLBAR_HEIGHT: f32 = 28.0;
    const TOOLBAR_BUTTON_SIZE: f32 = 36.0;

    pub fn new(app_context: Arc<AppContext>) -> Self {
        Self {
            app_context,
            show_actions: true,
            can_create_module_root: false,
            can_rename_selected_entry: false,
            can_delete_symbol_claim: false,
            can_open_in_code_viewer: false,
            can_open_in_memory_viewer: false,
            can_promote_derived_symbol: false,
            can_cancel_create_module_root: false,
            can_commit_create_module_root: false,
        }
    }

    pub fn show_actions(
        mut self,
        show_actions: bool,
    ) -> Self {
        self.show_actions = show_actions;

        self
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

    pub fn can_delete_symbol_claim(
        mut self,
        can_delete_symbol_claim: bool,
    ) -> Self {
        self.can_delete_symbol_claim = can_delete_symbol_claim;

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

    pub fn can_promote_derived_symbol(
        mut self,
        can_promote_derived_symbol: bool,
    ) -> Self {
        self.can_promote_derived_symbol = can_promote_derived_symbol;

        self
    }

    pub fn can_cancel_create_module_root(
        mut self,
        can_cancel_create_module_root: bool,
    ) -> Self {
        self.can_cancel_create_module_root = can_cancel_create_module_root;

        self
    }

    pub fn can_commit_create_module_root(
        mut self,
        can_commit_create_module_root: bool,
    ) -> Self {
        self.can_commit_create_module_root = can_commit_create_module_root;

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

        if !self.show_actions {
            return clicked_action;
        }

        toolbar_user_interface.with_layout(Layout::right_to_left(Align::Center), |toolbar_user_interface| {
            if self.can_commit_create_module_root || self.can_cancel_create_module_root {
                if Self::draw_icon_button(
                    toolbar_user_interface,
                    theme,
                    &theme.icon_library.icon_handle_common_check_mark,
                    "Create module.",
                    self.can_commit_create_module_root,
                )
                .clicked()
                {
                    clicked_action = Some(SymbolExplorerToolbarAction::CommitCreateModuleRoot);
                }

                if Self::draw_icon_button(
                    toolbar_user_interface,
                    theme,
                    &theme.icon_library.icon_handle_navigation_cancel,
                    "Cancel module creation.",
                    self.can_cancel_create_module_root,
                )
                .clicked()
                {
                    clicked_action = Some(SymbolExplorerToolbarAction::CancelCreateModuleRoot);
                }

                return;
            }

            if self.can_delete_symbol_claim
                && Self::draw_icon_button(
                    toolbar_user_interface,
                    theme,
                    &theme.icon_library.icon_handle_common_delete,
                    "Delete selected symbol.",
                    true,
                )
                .clicked()
            {
                clicked_action = Some(SymbolExplorerToolbarAction::DeleteSelectedSymbolClaim);
            }

            if self.can_promote_derived_symbol
                && Self::draw_icon_button(
                    toolbar_user_interface,
                    theme,
                    &theme.icon_library.icon_handle_common_add,
                    "Promote selected derived field to a symbol.",
                    true,
                )
                .clicked()
            {
                clicked_action = Some(SymbolExplorerToolbarAction::PromoteSelectedDerivedSymbol);
            }

            if self.can_rename_selected_entry
                && Self::draw_icon_button(
                    toolbar_user_interface,
                    theme,
                    &theme.icon_library.icon_handle_common_edit,
                    "Rename selected module or symbol.",
                    true,
                )
                .clicked()
            {
                clicked_action = Some(SymbolExplorerToolbarAction::RenameSelectedEntry);
            }

            if self.can_open_in_code_viewer
                && Self::draw_icon_button(
                    toolbar_user_interface,
                    theme,
                    &theme.icon_library.icon_handle_project_cpu_instruction,
                    "Open selected symbol in Code Viewer.",
                    true,
                )
                .clicked()
            {
                clicked_action = Some(SymbolExplorerToolbarAction::OpenSelectedInCodeViewer);
            }

            if self.can_open_in_memory_viewer
                && Self::draw_icon_button(
                    toolbar_user_interface,
                    theme,
                    &theme.icon_library.icon_handle_scan_collect_values,
                    "Open selected symbol in Memory Viewer.",
                    true,
                )
                .clicked()
            {
                clicked_action = Some(SymbolExplorerToolbarAction::OpenSelectedInMemoryViewer);
            }

            if self.can_create_module_root
                && Self::draw_icon_button(toolbar_user_interface, theme, &theme.icon_library.icon_handle_common_add, "Add module.", true).clicked()
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
