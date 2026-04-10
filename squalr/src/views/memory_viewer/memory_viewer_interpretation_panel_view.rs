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
}

#[derive(Clone)]
pub struct MemoryViewerInterpretationPanelView {
    app_context: Arc<AppContext>,
    memory_viewer_view_data: Dependency<MemoryViewerViewData>,
    project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
}

impl MemoryViewerInterpretationPanelView {
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
    ) {
        let target_directory_path = ProjectHierarchyViewData::get_selected_directory_path(self.project_hierarchy_view_data.clone());
        let Some(project_items_create_request) = MemoryViewerViewData::build_address_project_item_create_request(
            self.memory_viewer_view_data.clone(),
            selection_summary.selection_start_address,
            target_directory_path,
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
        let raw_hex_string = selection_summary
            .selected_bytes
            .iter()
            .map(|selected_byte| {
                selected_byte
                    .map(|selected_byte| format!("{:02X}", selected_byte))
                    .unwrap_or_else(|| String::from("??"))
            })
            .collect::<Vec<_>>()
            .join(" ");

        interpretation_entries.push(MemoryViewerInterpretationEntry {
            label: String::from("Bytes"),
            value: format!("{}", selected_byte_count),
        });
        interpretation_entries.push(MemoryViewerInterpretationEntry {
            label: String::from("Hex"),
            value: raw_hex_string,
        });

        if let Some(readable_bytes) = readable_bytes {
            interpretation_entries.push(MemoryViewerInterpretationEntry {
                label: format!("u8[{}]", selected_byte_count),
                value: readable_bytes
                    .iter()
                    .map(|selected_byte| selected_byte.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            });

            let utf8_display_text = String::from_utf8(readable_bytes.clone())
                .map(|utf8_text| {
                    utf8_text
                        .replace('\n', "\\n")
                        .replace('\r', "\\r")
                        .replace('\t', "\\t")
                })
                .unwrap_or_else(|_| String::from("(Invalid UTF-8)"));

            interpretation_entries.push(MemoryViewerInterpretationEntry {
                label: String::from("UTF-8"),
                value: utf8_display_text,
            });

            if let Some(interpreted_value) = Self::try_read_u8(&readable_bytes) {
                interpretation_entries.push(MemoryViewerInterpretationEntry {
                    label: String::from("u8"),
                    value: interpreted_value.to_string(),
                });
                interpretation_entries.push(MemoryViewerInterpretationEntry {
                    label: String::from("i8"),
                    value: (interpreted_value as i8).to_string(),
                });
            }

            if let Some(interpreted_value) = Self::try_read_u16(&readable_bytes) {
                interpretation_entries.push(MemoryViewerInterpretationEntry {
                    label: String::from("u16 (LE)"),
                    value: interpreted_value.to_string(),
                });
                interpretation_entries.push(MemoryViewerInterpretationEntry {
                    label: String::from("i16 (LE)"),
                    value: (interpreted_value as i16).to_string(),
                });
            }

            if let Some(interpreted_value) = Self::try_read_u32(&readable_bytes) {
                interpretation_entries.push(MemoryViewerInterpretationEntry {
                    label: String::from("u32 (LE)"),
                    value: interpreted_value.to_string(),
                });
                interpretation_entries.push(MemoryViewerInterpretationEntry {
                    label: String::from("i32 (LE)"),
                    value: (interpreted_value as i32).to_string(),
                });
                interpretation_entries.push(MemoryViewerInterpretationEntry {
                    label: String::from("f32 (LE)"),
                    value: f32::from_le_bytes(interpreted_value.to_le_bytes()).to_string(),
                });
            }

            if let Some(interpreted_value) = Self::try_read_u64(&readable_bytes) {
                interpretation_entries.push(MemoryViewerInterpretationEntry {
                    label: String::from("u64 (LE)"),
                    value: interpreted_value.to_string(),
                });
                interpretation_entries.push(MemoryViewerInterpretationEntry {
                    label: String::from("i64 (LE)"),
                    value: (interpreted_value as i64).to_string(),
                });
                interpretation_entries.push(MemoryViewerInterpretationEntry {
                    label: String::from("f64 (LE)"),
                    value: f64::from_le_bytes(interpreted_value.to_le_bytes()).to_string(),
                });
                interpretation_entries.push(MemoryViewerInterpretationEntry {
                    label: String::from("Pointer (LE)"),
                    value: format!("0x{:X}", interpreted_value),
                });
            }
        } else {
            interpretation_entries.push(MemoryViewerInterpretationEntry {
                label: String::from("Interpretation"),
                value: String::from("Selected bytes are not fully readable yet."),
            });
        }

        interpretation_entries
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

    fn build_add_button_label(selection_summary: &MemoryViewerSelectionSummary) -> String {
        match selection_summary.selected_bytes.len() {
            0 => String::from("Add Selection"),
            1 => String::from("Add As u8"),
            selected_byte_count => format!("Add As u8[{}]", selected_byte_count),
        }
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

                let add_button_label = Self::build_add_button_label(&selection_summary);
                let add_button = panel_user_interface.add_sized(
                    vec2(panel_user_interface.available_width(), 30.0),
                    Button::new_from_theme(theme)
                        .background_color(theme.background_control_secondary)
                        .with_tooltip_text("Create a project item from the current selection."),
                );

                panel_user_interface.painter().text(
                    add_button.rect.center(),
                    eframe::egui::Align2::CENTER_CENTER,
                    add_button_label,
                    theme.font_library.font_noto_sans.font_normal.clone(),
                    theme.foreground,
                );

                if add_button.clicked() {
                    self.dispatch_add_selection_to_project(&selection_summary);
                }

                panel_user_interface.add_space(12.0);
                let interpretation_entries = Self::build_interpretation_entries(&selection_summary);

                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(&mut panel_user_interface, |user_interface| {
                        for interpretation_entry in interpretation_entries {
                            user_interface.label(
                                RichText::new(interpretation_entry.label)
                                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                                    .color(theme.foreground_preview),
                            );
                            user_interface.label(
                                RichText::new(interpretation_entry.value)
                                    .font(theme.font_library.font_ubuntu_mono_bold.font_normal.clone())
                                    .color(theme.foreground),
                            );
                            user_interface.add_space(10.0);
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
    use super::MemoryViewerInterpretationPanelView;
    use crate::views::memory_viewer::view_data::memory_viewer_view_data::MemoryViewerSelectionSummary;

    #[test]
    fn build_add_button_label_uses_u8_array_for_multi_byte_selection() {
        let selection_summary = MemoryViewerSelectionSummary {
            selection_start_address: 0x1000,
            selection_end_address: 0x1003,
            selection_display_text: String::from("00001000"),
            selected_bytes: vec![Some(0x11), Some(0x22), Some(0x33), Some(0x44)],
        };

        let add_button_label = MemoryViewerInterpretationPanelView::build_add_button_label(&selection_summary);

        assert_eq!(add_button_label, String::from("Add As u8[4]"));
    }
}
