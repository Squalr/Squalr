use crate::state::pane_entry_row::PaneEntryRow;
use crate::views::entry_row_viewport::build_selection_relative_viewport_range;
use crate::views::memory_viewer::pane_state::MemoryViewerSelectionSummary;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryInterpretationEntry {
    pub label: String,
    pub value: String,
    pub add_data_type_id: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct MemoryInterpretationPaneState {
    pub entries: Vec<MemoryInterpretationEntry>,
    pub selected_entry_index: Option<usize>,
    pub status_message: String,
}

impl MemoryInterpretationPaneState {
    const MAX_HEX_PREVIEW_BYTES: usize = 16;
    const MAX_ARRAY_PREVIEW_ITEMS: usize = 6;
    const MAX_UTF8_PREVIEW_CHARACTERS: usize = 32;

    pub fn summary_lines(
        &self,
        has_project_open: bool,
    ) -> Vec<String> {
        vec![
            String::from("[ACT] Up/Down move | Enter/a add selected typed entry."),
            format!(
                "[SEL] index={} | entries={} | project_open={}.",
                self.selected_entry_index
                    .map(|selected_entry_index| selected_entry_index + 1)
                    .unwrap_or(0),
                self.entries.len(),
                has_project_open
            ),
            format!("[STAT] {}.", self.status_message),
        ]
    }

    pub fn apply_selection_summary(
        &mut self,
        selection_summary: Option<&MemoryViewerSelectionSummary>,
    ) {
        let selected_label_before_refresh = self
            .selected_entry()
            .map(|memory_interpretation_entry| memory_interpretation_entry.label.clone());
        self.entries = selection_summary
            .map(Self::build_interpretation_entries)
            .unwrap_or_default();
        self.selected_entry_index = selected_label_before_refresh
            .as_ref()
            .and_then(|selected_label| {
                self.entries
                    .iter()
                    .position(|memory_interpretation_entry| &memory_interpretation_entry.label == selected_label)
            })
            .or_else(|| (!self.entries.is_empty()).then_some(0));
    }

    pub fn visible_entry_rows(
        &self,
        viewport_capacity: usize,
        has_project_open: bool,
    ) -> Vec<PaneEntryRow> {
        let visible_entry_range = build_selection_relative_viewport_range(self.entries.len(), self.selected_entry_index, viewport_capacity);
        let mut entry_rows = Vec::with_capacity(visible_entry_range.len());

        for visible_entry_position in visible_entry_range {
            let Some(memory_interpretation_entry) = self.entries.get(visible_entry_position) else {
                continue;
            };
            let is_selected_entry = self.selected_entry_index == Some(visible_entry_position);
            let can_add_entry_to_project = has_project_open && memory_interpretation_entry.add_data_type_id.is_some();
            let marker_text = if memory_interpretation_entry.add_data_type_id.is_some() {
                "+".to_string()
            } else {
                String::new()
            };
            let secondary_text = memory_interpretation_entry.add_data_type_id.clone();

            if !can_add_entry_to_project && memory_interpretation_entry.add_data_type_id.is_some() {
                entry_rows.push(PaneEntryRow::disabled(
                    marker_text,
                    format!("{} = {}", memory_interpretation_entry.label, memory_interpretation_entry.value),
                    secondary_text,
                ));
            } else if is_selected_entry {
                entry_rows.push(PaneEntryRow::selected(
                    marker_text,
                    format!("{} = {}", memory_interpretation_entry.label, memory_interpretation_entry.value),
                    secondary_text,
                ));
            } else {
                entry_rows.push(PaneEntryRow::normal(
                    marker_text,
                    format!("{} = {}", memory_interpretation_entry.label, memory_interpretation_entry.value),
                    secondary_text,
                ));
            }
        }

        entry_rows
    }

    pub fn selected_entry(&self) -> Option<&MemoryInterpretationEntry> {
        let selected_entry_index = self.selected_entry_index?;

        self.entries.get(selected_entry_index)
    }

    pub fn selected_add_data_type_id(&self) -> Option<String> {
        self.selected_entry()?.add_data_type_id.clone()
    }

    pub fn select_next_entry(&mut self) {
        if self.entries.is_empty() {
            self.selected_entry_index = None;
            return;
        }

        let selected_entry_index = self.selected_entry_index.unwrap_or(0);
        self.selected_entry_index = Some((selected_entry_index + 1).min(self.entries.len().saturating_sub(1)));
    }

    pub fn select_previous_entry(&mut self) {
        if self.entries.is_empty() {
            self.selected_entry_index = None;
            return;
        }

        let selected_entry_index = self.selected_entry_index.unwrap_or(0);
        self.selected_entry_index = Some(selected_entry_index.saturating_sub(1));
    }

    pub fn select_first_entry(&mut self) {
        self.selected_entry_index = (!self.entries.is_empty()).then_some(0);
    }

    pub fn select_last_entry(&mut self) {
        self.selected_entry_index = self.entries.len().checked_sub(1);
    }

    fn build_interpretation_entries(selection_summary: &MemoryViewerSelectionSummary) -> Vec<MemoryInterpretationEntry> {
        let mut interpretation_entries = Vec::new();
        let readable_bytes = selection_summary
            .selected_bytes
            .iter()
            .copied()
            .collect::<Option<Vec<u8>>>();
        let selected_byte_count = selection_summary.selected_bytes.len();

        interpretation_entries.push(MemoryInterpretationEntry {
            label: String::from("Range"),
            value: selection_summary.selection_display_text.clone(),
            add_data_type_id: None,
        });
        interpretation_entries.push(MemoryInterpretationEntry {
            label: String::from("Bytes"),
            value: selected_byte_count.to_string(),
            add_data_type_id: None,
        });
        interpretation_entries.push(MemoryInterpretationEntry {
            label: String::from("Hex"),
            value: Self::format_hex_preview(&selection_summary.selected_bytes),
            add_data_type_id: None,
        });

        let Some(readable_bytes) = readable_bytes else {
            interpretation_entries.push(MemoryInterpretationEntry {
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

        interpretation_entries.push(MemoryInterpretationEntry {
            label: String::from("UTF-8"),
            value: Self::format_utf8_preview(&readable_bytes),
            add_data_type_id: None,
        });

        if Self::has_exact_byte_count(&readable_bytes, 1) {
            let interpreted_value = readable_bytes[0];
            interpretation_entries.push(Self::build_typed_entry("u8", interpreted_value.to_string(), Some(String::from("u8"))));
            interpretation_entries.push(Self::build_typed_entry("i8", (interpreted_value as i8).to_string(), Some(String::from("i8"))));
        }

        if Self::has_exact_byte_count(&readable_bytes, 2) {
            let interpreted_value = u16::from_le_bytes([readable_bytes[0], readable_bytes[1]]);
            interpretation_entries.push(Self::build_typed_entry("u16 (LE)", interpreted_value.to_string(), Some(String::from("u16"))));
            interpretation_entries.push(Self::build_typed_entry(
                "i16 (LE)",
                (interpreted_value as i16).to_string(),
                Some(String::from("i16")),
            ));
        }

        if Self::has_exact_byte_count(&readable_bytes, 4) {
            let interpreted_value = u32::from_le_bytes([
                readable_bytes[0],
                readable_bytes[1],
                readable_bytes[2],
                readable_bytes[3],
            ]);
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
            let interpreted_value = u64::from_le_bytes([
                readable_bytes[0],
                readable_bytes[1],
                readable_bytes[2],
                readable_bytes[3],
                readable_bytes[4],
                readable_bytes[5],
                readable_bytes[6],
                readable_bytes[7],
            ]);
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
    ) -> MemoryInterpretationEntry {
        MemoryInterpretationEntry {
            label: label.to_string(),
            value,
            add_data_type_id,
        }
    }

    fn append_array_entry<Formatter>(
        interpretation_entries: &mut Vec<MemoryInterpretationEntry>,
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
}

#[cfg(test)]
mod tests {
    use super::MemoryInterpretationPaneState;
    use crate::views::memory_viewer::pane_state::MemoryViewerSelectionSummary;

    #[test]
    fn apply_selection_summary_builds_scalar_and_array_entries() {
        let mut memory_interpretation_pane_state = MemoryInterpretationPaneState::default();
        let memory_viewer_selection_summary = MemoryViewerSelectionSummary {
            selection_start_address: 0x1010,
            selection_end_address: 0x1013,
            selection_display_text: String::from("game.exe+0x10"),
            selected_bytes: vec![Some(1), Some(2), Some(3), Some(4)],
        };

        memory_interpretation_pane_state.apply_selection_summary(Some(&memory_viewer_selection_summary));

        assert!(
            memory_interpretation_pane_state
                .entries
                .iter()
                .any(|entry| entry.label == "u32 (LE)")
        );
        assert!(
            memory_interpretation_pane_state
                .entries
                .iter()
                .any(|entry| entry.label == "u8[4]")
        );
    }
}
