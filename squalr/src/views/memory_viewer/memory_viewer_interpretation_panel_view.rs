use crate::{
    app_context::AppContext,
    ui::widgets::controls::button::Button,
    views::{
        memory_viewer::view_data::memory_viewer_view_data::{MemoryViewerSelectionSummary, MemoryViewerViewData},
        project_explorer::project_hierarchy::view_data::project_hierarchy_view_data::ProjectHierarchyViewData,
    },
};
use eframe::egui::{Align, Layout, Response, RichText, ScrollArea, Sense, Ui, UiBuilder, Widget, vec2};
use epaint::{CornerRadius, Stroke, StrokeKind};
use squalr_engine_api::{commands::unprivileged_command_request::UnprivilegedCommandRequest, dependency_injection::dependency::Dependency};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
struct MemoryViewerInterpretationEntry {
    label: String,
    value: String,
    add_data_type_id: Option<String>,
}

#[derive(Clone)]
pub struct MemoryViewerInterpretationPanelView {
    app_context: Arc<AppContext>,
    memory_viewer_view_data: Dependency<MemoryViewerViewData>,
    project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
}

impl MemoryViewerInterpretationPanelView {
    const MAX_ARRAY_PREVIEW_ITEMS: usize = 8;
    const MAX_HEX_PREVIEW_BYTES: usize = 16;
    const MAX_UTF8_PREVIEW_CHARACTERS: usize = 80;
    const ACTION_BUTTON_WIDTH: f32 = 120.0;
    const ACTION_BUTTON_HEIGHT: f32 = 24.0;

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let memory_viewer_view_data = app_context
            .dependency_container
            .get_dependency::<MemoryViewerViewData>();
        let project_hierarchy_view_data = app_context
            .dependency_container
            .get_dependency::<ProjectHierarchyViewData>();

        Self {
            app_context,
            memory_viewer_view_data,
            project_hierarchy_view_data,
        }
    }

    fn dispatch_add_selection_to_project(
        &self,
        selection_summary: &MemoryViewerSelectionSummary,
        data_type_id: String,
    ) {
        let target_directory_path = ProjectHierarchyViewData::get_selected_directory_path(self.project_hierarchy_view_data.clone());
        let Some(project_items_create_request) = MemoryViewerViewData::build_address_project_item_create_request_with_data_type(
            self.memory_viewer_view_data.clone(),
            selection_summary.selection_start_address,
            target_directory_path,
            Some(data_type_id),
        ) else {
            log::warn!("Failed to build memory viewer selection project item create request.");
            return;
        };

        project_items_create_request.send(&self.app_context.engine_unprivileged_state, |project_items_create_response| {
            if !project_items_create_response.success {
                log::warn!("Memory viewer add-selection-to-project command failed.");
            }
        });
    }

    fn build_interpretation_entries(selection_summary: &MemoryViewerSelectionSummary) -> Vec<MemoryViewerInterpretationEntry> {
        let mut interpretation_entries = Vec::new();
        let readable_bytes = selection_summary
            .selected_bytes
            .iter()
            .copied()
            .collect::<Option<Vec<u8>>>();
        let selected_byte_count = selection_summary.selected_bytes.len();

        interpretation_entries.push(MemoryViewerInterpretationEntry {
            label: String::from("Bytes"),
            value: selected_byte_count.to_string(),
            add_data_type_id: None,
        });
        interpretation_entries.push(MemoryViewerInterpretationEntry {
            label: String::from("Hex"),
            value: Self::format_hex_preview(&selection_summary.selected_bytes),
            add_data_type_id: None,
        });

        let Some(readable_bytes) = readable_bytes else {
            interpretation_entries.push(MemoryViewerInterpretationEntry {
                label: String::from("Interpretation"),
                value: String::from("Selected bytes are not fully readable yet."),
                add_data_type_id: None,
            });

            return interpretation_entries;
        };

        if selected_byte_count > 1 {
            interpretation_entries.push(Self::build_typed_entry(
                &format!("u8[{}]", selected_byte_count),
                Self::format_u8_array_preview(&readable_bytes),
                Some(format!("u8[{}]", selected_byte_count)),
            ));
            interpretation_entries.push(Self::build_typed_entry(
                &format!("i8[{}]", selected_byte_count),
                Self::format_i8_array_preview(&readable_bytes),
                Some(format!("i8[{}]", selected_byte_count)),
            ));
        }

        interpretation_entries.push(MemoryViewerInterpretationEntry {
            label: String::from("UTF-8"),
            value: Self::format_utf8_preview(&readable_bytes),
            add_data_type_id: None,
        });

        if Self::has_exact_byte_count(&readable_bytes, 1) {
            let interpreted_value = Self::try_read_u8(&readable_bytes).unwrap_or_default();
            interpretation_entries.push(Self::build_typed_entry("u8", interpreted_value.to_string(), Some(String::from("u8"))));
            interpretation_entries.push(Self::build_typed_entry("i8", (interpreted_value as i8).to_string(), Some(String::from("i8"))));
        }

        if Self::has_exact_byte_count(&readable_bytes, 2) {
            let interpreted_value = Self::try_read_u16(&readable_bytes).unwrap_or_default();
            interpretation_entries.push(Self::build_typed_entry("u16 (LE)", interpreted_value.to_string(), Some(String::from("u16"))));
            interpretation_entries.push(Self::build_typed_entry(
                "i16 (LE)",
                (interpreted_value as i16).to_string(),
                Some(String::from("i16")),
            ));
        }

        if Self::has_exact_byte_count(&readable_bytes, 4) {
            let interpreted_value = Self::try_read_u32(&readable_bytes).unwrap_or_default();
            interpretation_entries.push(Self::build_typed_entry("u32 (LE)", interpreted_value.to_string(), Some(String::from("u32"))));
            interpretation_entries.push(Self::build_typed_entry(
                "i32 (LE)",
                (interpreted_value as i32).to_string(),
                Some(String::from("i32")),
            ));
            interpretation_entries.push(Self::build_typed_entry(
                "f32 (LE)",
                f32::from_le_bytes(interpreted_value.to_le_bytes()).to_string(),
                Some(String::from("f32")),
            ));
        }

        if Self::has_exact_byte_count(&readable_bytes, 8) {
            let interpreted_value = Self::try_read_u64(&readable_bytes).unwrap_or_default();
            interpretation_entries.push(Self::build_typed_entry("u64 (LE)", interpreted_value.to_string(), Some(String::from("u64"))));
            interpretation_entries.push(Self::build_typed_entry(
                "i64 (LE)",
                (interpreted_value as i64).to_string(),
                Some(String::from("i64")),
            ));
            interpretation_entries.push(Self::build_typed_entry(
                "f64 (LE)",
                f64::from_le_bytes(interpreted_value.to_le_bytes()).to_string(),
                Some(String::from("f64")),
            ));
            interpretation_entries.push(MemoryViewerInterpretationEntry {
                label: String::from("Pointer (LE)"),
                value: format!("0x{:X}", interpreted_value),
                add_data_type_id: None,
            });
        }

        Self::append_array_entry(&mut interpretation_entries, &readable_bytes, 2, "u16", |value_bytes| {
            u16::from_le_bytes([value_bytes[0], value_bytes[1]]).to_string()
        });
        Self::append_array_entry(&mut interpretation_entries, &readable_bytes, 2, "i16", |value_bytes| {
            i16::from_le_bytes([value_bytes[0], value_bytes[1]]).to_string()
        });
        Self::append_array_entry(&mut interpretation_entries, &readable_bytes, 4, "u32", |value_bytes| {
            u32::from_le_bytes([value_bytes[0], value_bytes[1], value_bytes[2], value_bytes[3]]).to_string()
        });
        Self::append_array_entry(&mut interpretation_entries, &readable_bytes, 4, "i32", |value_bytes| {
            i32::from_le_bytes([value_bytes[0], value_bytes[1], value_bytes[2], value_bytes[3]]).to_string()
        });
        Self::append_array_entry(&mut interpretation_entries, &readable_bytes, 4, "f32", |value_bytes| {
            f32::from_le_bytes([value_bytes[0], value_bytes[1], value_bytes[2], value_bytes[3]]).to_string()
        });
        Self::append_array_entry(&mut interpretation_entries, &readable_bytes, 8, "u64", |value_bytes| {
            u64::from_le_bytes([
                value_bytes[0],
                value_bytes[1],
                value_bytes[2],
                value_bytes[3],
                value_bytes[4],
                value_bytes[5],
                value_bytes[6],
                value_bytes[7],
            ])
            .to_string()
        });
        Self::append_array_entry(&mut interpretation_entries, &readable_bytes, 8, "i64", |value_bytes| {
            i64::from_le_bytes([
                value_bytes[0],
                value_bytes[1],
                value_bytes[2],
                value_bytes[3],
                value_bytes[4],
                value_bytes[5],
                value_bytes[6],
                value_bytes[7],
            ])
            .to_string()
        });
        Self::append_array_entry(&mut interpretation_entries, &readable_bytes, 8, "f64", |value_bytes| {
            f64::from_le_bytes([
                value_bytes[0],
                value_bytes[1],
                value_bytes[2],
                value_bytes[3],
                value_bytes[4],
                value_bytes[5],
                value_bytes[6],
                value_bytes[7],
            ])
            .to_string()
        });

        interpretation_entries
    }

    fn build_typed_entry(
        label: &str,
        value: String,
        add_data_type_id: Option<String>,
    ) -> MemoryViewerInterpretationEntry {
        MemoryViewerInterpretationEntry {
            label: label.to_string(),
            value,
            add_data_type_id,
        }
    }

    fn append_array_entry<Formatter>(
        interpretation_entries: &mut Vec<MemoryViewerInterpretationEntry>,
        readable_bytes: &[u8],
        element_size_in_bytes: usize,
        data_type_id: &str,
        formatter: Formatter,
    ) where
        Formatter: Fn(&[u8]) -> String,
    {
        if readable_bytes.len() <= element_size_in_bytes || readable_bytes.len() % element_size_in_bytes != 0 {
            return;
        }

        let element_count = readable_bytes.len() / element_size_in_bytes;

        if element_count <= 1 {
            return;
        }

        let preview_values = readable_bytes
            .chunks_exact(element_size_in_bytes)
            .take(Self::MAX_ARRAY_PREVIEW_ITEMS)
            .map(formatter)
            .collect::<Vec<_>>();

        interpretation_entries.push(Self::build_typed_entry(
            &format!("{}[{}]", data_type_id, element_count),
            Self::format_truncated_list(preview_values, element_count),
            Some(format!("{}[{}]", data_type_id, element_count)),
        ));
    }

    fn try_read_u8(readable_bytes: &[u8]) -> Option<u8> {
        readable_bytes.first().copied()
    }

    fn try_read_u16(readable_bytes: &[u8]) -> Option<u16> {
        let array_bytes: [u8; 2] = readable_bytes.get(..2)?.try_into().ok()?;

        Some(u16::from_le_bytes(array_bytes))
    }

    fn try_read_u32(readable_bytes: &[u8]) -> Option<u32> {
        let array_bytes: [u8; 4] = readable_bytes.get(..4)?.try_into().ok()?;

        Some(u32::from_le_bytes(array_bytes))
    }

    fn try_read_u64(readable_bytes: &[u8]) -> Option<u64> {
        let array_bytes: [u8; 8] = readable_bytes.get(..8)?.try_into().ok()?;

        Some(u64::from_le_bytes(array_bytes))
    }

    fn has_exact_byte_count(
        readable_bytes: &[u8],
        expected_byte_count: usize,
    ) -> bool {
        readable_bytes.len() == expected_byte_count
    }

    fn format_hex_preview(selected_bytes: &[Option<u8>]) -> String {
        let preview_hex_values = selected_bytes
            .iter()
            .take(Self::MAX_HEX_PREVIEW_BYTES)
            .map(|selected_byte| {
                selected_byte
                    .map(|selected_byte| format!("{:02X}", selected_byte))
                    .unwrap_or_else(|| String::from("??"))
            })
            .collect::<Vec<_>>();
        let mut preview_text = preview_hex_values.join(" ");

        if selected_bytes.len() > Self::MAX_HEX_PREVIEW_BYTES {
            preview_text.push_str(" ...");
        }

        preview_text
    }

    fn format_u8_array_preview(readable_bytes: &[u8]) -> String {
        let preview_values = readable_bytes
            .iter()
            .take(Self::MAX_ARRAY_PREVIEW_ITEMS)
            .map(|value_byte| value_byte.to_string())
            .collect::<Vec<_>>();

        Self::format_truncated_list(preview_values, readable_bytes.len())
    }

    fn format_i8_array_preview(readable_bytes: &[u8]) -> String {
        let preview_values = readable_bytes
            .iter()
            .take(Self::MAX_ARRAY_PREVIEW_ITEMS)
            .map(|value_byte| (*value_byte as i8).to_string())
            .collect::<Vec<_>>();

        Self::format_truncated_list(preview_values, readable_bytes.len())
    }

    fn format_utf8_preview(readable_bytes: &[u8]) -> String {
        let utf8_display_text = String::from_utf8(readable_bytes.to_vec())
            .map(|utf8_text| {
                utf8_text
                    .replace('\n', "\\n")
                    .replace('\r', "\\r")
                    .replace('\t', "\\t")
            })
            .unwrap_or_else(|_| String::from("(Invalid UTF-8)"));

        Self::truncate_text(&utf8_display_text, Self::MAX_UTF8_PREVIEW_CHARACTERS)
    }

    fn format_truncated_list(
        preview_values: Vec<String>,
        total_value_count: usize,
    ) -> String {
        if preview_values.is_empty() {
            return String::from("[]");
        }

        let mut list_text = format!("[{}]", preview_values.join(", "));

        if total_value_count > preview_values.len() {
            list_text.truncate(list_text.len().saturating_sub(1));
            list_text.push_str(", ...]");
        }

        list_text
    }

    fn truncate_text(
        value_text: &str,
        max_character_count: usize,
    ) -> String {
        let mut truncated_text = value_text.chars().take(max_character_count).collect::<String>();

        if value_text.chars().count() > max_character_count {
            truncated_text.push_str("...");
        }

        truncated_text
    }

    fn build_add_button_label(data_type_id: &str) -> String {
        format!("Add {}", data_type_id)
    }

    fn render_entry(
        &self,
        user_interface: &mut Ui,
        selection_summary: &MemoryViewerSelectionSummary,
        interpretation_entry: &MemoryViewerInterpretationEntry,
    ) {
        let theme = &self.app_context.theme;

        user_interface.horizontal(|user_interface| {
            user_interface.label(
                RichText::new(&interpretation_entry.label)
                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                    .color(theme.foreground_preview),
            );

            user_interface.with_layout(Layout::right_to_left(Align::Center), |user_interface| {
                if let Some(add_data_type_id) = interpretation_entry.add_data_type_id.clone() {
                    let add_button_label = Self::build_add_button_label(&add_data_type_id);
                    let add_button = user_interface.add_sized(
                        vec2(
                            Self::ACTION_BUTTON_WIDTH.min(user_interface.available_width().max(1.0)),
                            Self::ACTION_BUTTON_HEIGHT,
                        ),
                        Button::new_from_theme(theme)
                            .background_color(theme.background_control_secondary)
                            .with_tooltip_text(&format!("Create a project item from the current selection as `{}`.", add_data_type_id)),
                    );

                    user_interface.painter().text(
                        add_button.rect.center(),
                        eframe::egui::Align2::CENTER_CENTER,
                        add_button_label,
                        theme.font_library.font_noto_sans.font_small.clone(),
                        theme.foreground,
                    );

                    if add_button.clicked() {
                        self.dispatch_add_selection_to_project(selection_summary, add_data_type_id);
                    }
                }
            });
        });
        user_interface.label(
            RichText::new(&interpretation_entry.value)
                .font(theme.font_library.font_ubuntu_mono_bold.font_normal.clone())
                .color(theme.foreground),
        );
    }
}

impl Widget for MemoryViewerInterpretationPanelView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let (panel_rect, response) = user_interface.allocate_exact_size(user_interface.available_size(), Sense::click());

        user_interface
            .painter()
            .rect_filled(panel_rect, CornerRadius::ZERO, theme.background_primary);
        user_interface
            .painter()
            .rect_stroke(panel_rect, CornerRadius::ZERO, Stroke::new(1.0, theme.submenu_border), StrokeKind::Inside);

        let mut panel_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(panel_rect.shrink2(vec2(10.0, 10.0)))
                .layout(Layout::top_down(Align::Min)),
        );
        let selection_summary = MemoryViewerViewData::get_selection_summary(self.memory_viewer_view_data.clone());

        panel_user_interface.label(
            RichText::new("Interpretation")
                .font(theme.font_library.font_noto_sans.font_header.clone())
                .color(theme.foreground),
        );
        panel_user_interface.add_space(8.0);

        match selection_summary {
            Some(selection_summary) => {
                panel_user_interface.label(
                    RichText::new(format!(
                        "{} | {} byte{}",
                        selection_summary.selection_display_text,
                        selection_summary.selected_bytes.len(),
                        if selection_summary.selected_bytes.len() == 1 { "" } else { "s" }
                    ))
                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                    .color(theme.foreground_preview),
                );
                panel_user_interface.add_space(10.0);

                let interpretation_entries = Self::build_interpretation_entries(&selection_summary);

                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(&mut panel_user_interface, |user_interface| {
                        for (entry_index, interpretation_entry) in interpretation_entries.iter().enumerate() {
                            self.render_entry(user_interface, &selection_summary, interpretation_entry);

                            if entry_index + 1 < interpretation_entries.len() {
                                user_interface.add_space(8.0);
                                user_interface.separator();
                                user_interface.add_space(8.0);
                            }
                        }
                    });
            }
            None => {
                panel_user_interface.label(
                    RichText::new("Select bytes in the memory viewer to inspect them here.")
                        .font(theme.font_library.font_noto_sans.font_normal.clone())
                        .color(theme.foreground_preview),
                );
            }
        }

        response
    }
}

#[cfg(test)]
mod tests {
    use super::{MemoryViewerInterpretationEntry, MemoryViewerInterpretationPanelView};
    use crate::views::memory_viewer::view_data::memory_viewer_view_data::MemoryViewerSelectionSummary;

    #[test]
    fn build_add_button_label_uses_data_type_id() {
        let add_button_label = MemoryViewerInterpretationPanelView::build_add_button_label("u16[4]");

        assert_eq!(add_button_label, String::from("Add u16[4]"));
    }

    #[test]
    fn format_truncated_list_appends_ellipsis_when_preview_is_trimmed() {
        let preview_text = MemoryViewerInterpretationPanelView::format_truncated_list(
            vec![
                String::from("1"),
                String::from("2"),
                String::from("3"),
                String::from("4"),
            ],
            6,
        );

        assert_eq!(preview_text, String::from("[1, 2, 3, 4, ...]"));
    }

    #[test]
    fn build_interpretation_entries_adds_aligned_u16_array_entry() {
        let selection_summary = MemoryViewerSelectionSummary {
            selection_start_address: 0x1000,
            selection_end_address: 0x1007,
            selection_display_text: String::from("00001000"),
            selected_bytes: vec![
                Some(0x01),
                Some(0x00),
                Some(0x02),
                Some(0x00),
                Some(0x03),
                Some(0x00),
                Some(0x04),
                Some(0x00),
            ],
        };

        let interpretation_entries = MemoryViewerInterpretationPanelView::build_interpretation_entries(&selection_summary);

        assert!(interpretation_entries.contains(&MemoryViewerInterpretationEntry {
            label: String::from("u16[4]"),
            value: String::from("[1, 2, 3, 4]"),
            add_data_type_id: Some(String::from("u16[4]")),
        }));
    }

    #[test]
    fn build_interpretation_entries_omits_scalar_types_for_non_matching_selection_sizes() {
        let selection_summary = MemoryViewerSelectionSummary {
            selection_start_address: 0x1000,
            selection_end_address: 0x1007,
            selection_display_text: String::from("00001000"),
            selected_bytes: vec![
                Some(0x01),
                Some(0x00),
                Some(0x02),
                Some(0x00),
                Some(0x03),
                Some(0x00),
                Some(0x04),
                Some(0x00),
            ],
        };

        let interpretation_entries = MemoryViewerInterpretationPanelView::build_interpretation_entries(&selection_summary);

        assert!(
            !interpretation_entries
                .iter()
                .any(|entry| entry.label == "u16 (LE)")
        );
        assert!(
            !interpretation_entries
                .iter()
                .any(|entry| entry.label == "u32 (LE)")
        );
        assert!(
            interpretation_entries
                .iter()
                .any(|entry| entry.label == "u64 (LE)")
        );
        assert!(
            interpretation_entries
                .iter()
                .any(|entry| entry.label == "u16[4]")
        );
    }
}
