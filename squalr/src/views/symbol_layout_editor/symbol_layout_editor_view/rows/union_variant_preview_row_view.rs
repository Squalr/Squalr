use crate::app_context::AppContext;
use crate::ui::text::text_fitting::truncate_text_to_width;
use eframe::egui::{Align2, Sense, Ui, pos2, vec2};
use std::sync::Arc;

use super::super::SymbolLayoutEditorView;

pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) struct UnionVariantPreviewRowView<'view> {
    app_context: Arc<AppContext>,
    label_text: &'view str,
    preview_text: &'view str,
}

impl<'view> UnionVariantPreviewRowView<'view> {
    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn new(
        app_context: Arc<AppContext>,
        label_text: &'view str,
        preview_text: &'view str,
    ) -> Self {
        Self {
            app_context,
            label_text,
            preview_text,
        }
    }

    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn show(
        self,
        user_interface: &mut Ui,
    ) {
        let theme = &self.app_context.theme;
        let (row_rect, _) = user_interface.allocate_exact_size(
            vec2(user_interface.available_width().max(1.0), SymbolLayoutEditorView::LIST_ROW_HEIGHT),
            Sense::hover(),
        );
        let label_position = pos2(row_rect.min.x + SymbolLayoutEditorView::FIELD_ROW_LEFT_PADDING, row_rect.center().y);
        let preview_position = pos2(row_rect.max.x - SymbolLayoutEditorView::FIELD_ROW_LEFT_PADDING, row_rect.center().y);

        user_interface.painter().text(
            label_position,
            Align2::LEFT_CENTER,
            truncate_text_to_width(
                user_interface,
                self.label_text,
                &theme.font_library.font_noto_sans.font_normal,
                theme.foreground,
                (row_rect.width() * 0.6).max(0.0),
            ),
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground,
        );
        user_interface.painter().text(
            preview_position,
            Align2::RIGHT_CENTER,
            truncate_text_to_width(
                user_interface,
                self.preview_text,
                &theme.font_library.font_noto_sans.font_small,
                theme.foreground_preview,
                (row_rect.width() * 0.35).max(0.0),
            ),
            theme.font_library.font_noto_sans.font_small.clone(),
            theme.foreground_preview,
        );
    }
}
