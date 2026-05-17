use crate::app_context::AppContext;
use crate::ui::draw::icon_draw::IconDraw;
use crate::ui::widgets::controls::button::Button as ThemeButton;
use crate::views::symbol_layout_editor::view_data::symbol_layout_editor_view_data::SymbolLayoutEditorViewData;
use eframe::egui::{Align, Color32, Layout, Response, Sense, TextureHandle, Ui, UiBuilder, Widget, vec2};
use epaint::CornerRadius;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::structures::{data_types::data_type_ref::DataTypeRef, projects::project_symbol_catalog::ProjectSymbolCatalog};
use std::sync::Arc;

pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) struct SymbolLayoutListToolbarView<'view> {
    app_context: Arc<AppContext>,
    symbol_layout_editor_view_data: Dependency<SymbolLayoutEditorViewData>,
    project_symbol_catalog: &'view ProjectSymbolCatalog,
    default_data_type_ref: DataTypeRef,
    is_take_over_active: bool,
    height: f32,
    icon_button_width: f32,
    icon_button_height: f32,
}

impl<'view> SymbolLayoutListToolbarView<'view> {
    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn new(
        app_context: Arc<AppContext>,
        symbol_layout_editor_view_data: Dependency<SymbolLayoutEditorViewData>,
        project_symbol_catalog: &'view ProjectSymbolCatalog,
        default_data_type_ref: DataTypeRef,
        is_take_over_active: bool,
    ) -> Self {
        Self {
            app_context,
            symbol_layout_editor_view_data,
            project_symbol_catalog,
            default_data_type_ref,
            is_take_over_active,
            height: 28.0,
            icon_button_width: 36.0,
            icon_button_height: 28.0,
        }
    }

    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn height(
        mut self,
        height: f32,
    ) -> Self {
        self.height = height;
        self
    }

    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn icon_button_size(
        mut self,
        icon_button_width: f32,
        icon_button_height: f32,
    ) -> Self {
        self.icon_button_width = icon_button_width;
        self.icon_button_height = icon_button_height;
        self
    }

    fn render_icon_button(
        &self,
        user_interface: &mut Ui,
        icon_handle: &TextureHandle,
        tooltip_text: &str,
        is_disabled: bool,
    ) -> Response {
        let theme = &self.app_context.theme;
        let button_response = user_interface.add_sized(
            vec2(self.icon_button_width, self.icon_button_height),
            ThemeButton::new_from_theme(theme)
                .with_tooltip_text(tooltip_text)
                .background_color(Color32::TRANSPARENT)
                .disabled(is_disabled),
        );

        IconDraw::draw_tinted(
            user_interface,
            button_response.rect,
            icon_handle,
            if is_disabled { theme.foreground_preview } else { theme.foreground },
        );

        button_response
    }
}

impl Widget for SymbolLayoutListToolbarView<'_> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let (toolbar_rect, toolbar_response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), self.height), Sense::empty());

        user_interface
            .painter()
            .rect_filled(toolbar_rect, CornerRadius::ZERO, theme.background_primary);

        let mut toolbar_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(toolbar_rect)
                .layout(Layout::left_to_right(Align::Center)),
        );
        toolbar_user_interface.set_clip_rect(toolbar_rect);

        let new_layout_response = self.render_icon_button(
            &mut toolbar_user_interface,
            &theme.icon_library.icon_handle_common_add,
            "Create a new reusable symbol layout.",
            self.is_take_over_active,
        );

        if new_layout_response.clicked() {
            SymbolLayoutEditorViewData::begin_create_symbol_layout(
                self.symbol_layout_editor_view_data,
                self.project_symbol_catalog,
                self.default_data_type_ref,
            );
        }

        toolbar_response
    }
}
