use crate::views::struct_viewer::summary::build_struct_viewer_summary_lines;
use squalr_engine_api::registries::symbols::symbol_registry::SymbolRegistry;
use squalr_engine_api::structures::data_values::anonymous_value_string::AnonymousValueString;
use squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
use squalr_engine_api::structures::data_values::container_type::ContainerType;
use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
use squalr_engine_api::structures::scan_results::scan_result::ScanResult;
use squalr_engine_api::structures::scan_results::scan_result_ref::ScanResultRef;
use squalr_engine_api::structures::structs::valued_struct::ValuedStruct;
use squalr_engine_api::structures::structs::valued_struct_field::{ValuedStructField, ValuedStructFieldData};
use std::collections::HashMap;
use std::path::PathBuf;

/// Tracks where the struct currently being viewed originated from.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum StructViewerSource {
    #[default]
    None,
    ScanResults,
    ProjectItems,
}

/// Stores state for viewing and editing selected structures.
#[derive(Clone, Debug)]
pub struct StructViewerPaneState {
    pub selected_struct_name: Option<String>,
    pub selected_field_name: Option<String>,
    pub has_uncommitted_edit: bool,
    pub source: StructViewerSource,
    pub focused_struct: Option<ValuedStruct>,
    pub selected_field_position: Option<usize>,
    pub pending_edit_text: String,
    pub selected_scan_result_refs: Vec<ScanResultRef>,
    pub selected_project_item_paths: Vec<PathBuf>,
    pub is_committing_edit: bool,
    pub status_message: String,
    pub field_display_values: HashMap<String, Vec<AnonymousValueString>>,
    pub field_active_display_value_indices: HashMap<String, usize>,
}

impl StructViewerPaneState {
    pub fn clear_focus(
        &mut self,
        status_message: &str,
    ) {
        self.selected_struct_name = None;
        self.selected_field_name = None;
        self.has_uncommitted_edit = false;
        self.source = StructViewerSource::None;
        self.focused_struct = None;
        self.selected_field_position = None;
        self.pending_edit_text.clear();
        self.selected_scan_result_refs.clear();
        self.selected_project_item_paths.clear();
        self.is_committing_edit = false;
        self.status_message = status_message.to_string();
        self.field_display_values.clear();
        self.field_active_display_value_indices.clear();
    }

    pub fn focus_scan_results(
        &mut self,
        selected_scan_results: &[ScanResult],
        selected_scan_result_refs: Vec<ScanResultRef>,
    ) {
        if selected_scan_results.is_empty() || selected_scan_result_refs.is_empty() {
            self.clear_focus("No scan result selection is available for struct viewer.");
            return;
        }

        let selected_scan_result_structs = selected_scan_results
            .iter()
            .map(ScanResult::as_valued_struct)
            .collect::<Vec<_>>();
        let combined_struct = ValuedStruct::combine_exclusive(&selected_scan_result_structs);
        self.source = StructViewerSource::ScanResults;
        self.focused_struct = Some(combined_struct);
        self.selected_field_position = self
            .focused_struct
            .as_ref()
            .and_then(|focused_struct| (!focused_struct.get_fields().is_empty()).then_some(0));
        self.selected_struct_name = Some(format!("ScanResultSelection({})", selected_scan_result_refs.len()));
        self.selected_scan_result_refs = selected_scan_result_refs;
        self.selected_project_item_paths.clear();
        self.sync_selected_field_metadata();
        self.status_message = "Focused struct viewer on selected scan result entries.".to_string();
    }

    pub fn focus_project_items(
        &mut self,
        selected_project_items: Vec<(PathBuf, ProjectItem)>,
    ) {
        if selected_project_items.is_empty() {
            self.clear_focus("No project item selection is available for struct viewer.");
            return;
        }

        let selected_project_item_paths = selected_project_items
            .iter()
            .map(|(project_item_path, _)| project_item_path.clone())
            .collect::<Vec<_>>();
        let selected_project_item_structs = selected_project_items
            .iter()
            .map(|(_, project_item)| project_item.get_properties().clone())
            .collect::<Vec<_>>();
        let combined_struct = ValuedStruct::combine_exclusive(&selected_project_item_structs);
        self.source = StructViewerSource::ProjectItems;
        self.focused_struct = Some(combined_struct);
        self.selected_field_position = self
            .focused_struct
            .as_ref()
            .and_then(|focused_struct| (!focused_struct.get_fields().is_empty()).then_some(0));
        self.selected_struct_name = Some(format!("ProjectItemSelection({})", selected_project_item_paths.len()));
        self.selected_project_item_paths = selected_project_item_paths;
        self.selected_scan_result_refs.clear();
        self.sync_selected_field_metadata();
        self.status_message = "Focused struct viewer on selected project item entries.".to_string();
    }

    pub fn select_next_field(&mut self) {
        let Some(focused_struct) = self.focused_struct.as_ref() else {
            self.selected_field_position = None;
            return;
        };
        if focused_struct.get_fields().is_empty() {
            self.selected_field_position = None;
            return;
        }

        let selected_field_position = self.selected_field_position.unwrap_or(0);
        let next_field_position = (selected_field_position + 1) % focused_struct.get_fields().len();
        self.selected_field_position = Some(next_field_position);
        self.sync_selected_field_metadata();
    }

    pub fn select_previous_field(&mut self) {
        let Some(focused_struct) = self.focused_struct.as_ref() else {
            self.selected_field_position = None;
            return;
        };
        if focused_struct.get_fields().is_empty() {
            self.selected_field_position = None;
            return;
        }

        let selected_field_position = self.selected_field_position.unwrap_or(0);
        let previous_field_position = if selected_field_position == 0 {
            focused_struct.get_fields().len() - 1
        } else {
            selected_field_position - 1
        };
        self.selected_field_position = Some(previous_field_position);
        self.sync_selected_field_metadata();
    }

    pub fn append_pending_edit_character(
        &mut self,
        pending_character: char,
    ) {
        if pending_character.is_control() {
            return;
        }
        if !self.is_selected_field_editable() {
            return;
        }

        self.pending_edit_text.push(pending_character);
        self.has_uncommitted_edit = true;
    }

    pub fn backspace_pending_edit(&mut self) {
        if !self.is_selected_field_editable() {
            return;
        }

        self.pending_edit_text.pop();
        self.has_uncommitted_edit = true;
    }

    pub fn clear_pending_edit(&mut self) {
        if !self.is_selected_field_editable() {
            return;
        }

        self.pending_edit_text.clear();
        self.has_uncommitted_edit = true;
    }

    pub fn build_edited_field_from_pending_text(&self) -> Result<ValuedStructField, String> {
        let selected_field = self
            .selected_field()
            .ok_or_else(|| "No struct field is selected.".to_string())?;
        if selected_field.get_is_read_only() {
            return Err(format!("Field '{}' is read-only.", selected_field.get_name()));
        }

        let selected_field_data_value = selected_field
            .get_data_value()
            .ok_or_else(|| "Nested struct edits are not supported in the TUI yet.".to_string())?;
        let pending_edit_text = self.pending_edit_text.trim();
        if pending_edit_text.is_empty() {
            return Err("Edit value is empty.".to_string());
        }

        let symbol_registry = SymbolRegistry::get_instance();
        let selected_data_type_ref = selected_field_data_value.get_data_type_ref();
        let default_edit_format = symbol_registry.get_default_anonymous_value_string_format(selected_data_type_ref);
        let pending_edit_value = AnonymousValueString::new(pending_edit_text.to_string(), default_edit_format, ContainerType::None);
        let edited_data_value = symbol_registry
            .deanonymize_value_string(selected_data_type_ref, &pending_edit_value)
            .map_err(|error| format!("Failed to parse edited value: {}", error))?;

        Ok(ValuedStructField::new(
            selected_field.get_name().to_string(),
            ValuedStructFieldData::Value(edited_data_value),
            false,
        ))
    }

    pub fn apply_committed_field(
        &mut self,
        committed_field: &ValuedStructField,
    ) {
        if let Some(focused_struct) = self.focused_struct.as_mut() {
            focused_struct.set_field_data(
                committed_field.get_name(),
                committed_field.get_field_data().clone(),
                committed_field.get_is_read_only(),
            );
        }
        self.has_uncommitted_edit = false;
        self.sync_selected_field_metadata();
    }

    pub fn cycle_selected_field_display_format_forward(&mut self) -> Result<AnonymousValueStringFormat, String> {
        self.cycle_selected_field_display_format(true)
    }

    pub fn cycle_selected_field_display_format_backward(&mut self) -> Result<AnonymousValueStringFormat, String> {
        self.cycle_selected_field_display_format(false)
    }

    pub fn summary_lines(
        &self,
        focused_field_preview_capacity: usize,
    ) -> Vec<String> {
        build_struct_viewer_summary_lines(self, focused_field_preview_capacity)
    }

    pub(crate) fn focused_field_count(&self) -> usize {
        self.focused_struct
            .as_ref()
            .map(|focused_struct| focused_struct.get_fields().len())
            .unwrap_or(0)
    }

    fn selected_field(&self) -> Option<&ValuedStructField> {
        let selected_field_position = self.selected_field_position?;
        self.focused_struct
            .as_ref()
            .and_then(|focused_struct| focused_struct.get_fields().get(selected_field_position))
    }

    pub fn selected_field_edit_block_reason(&self) -> Option<String> {
        let selected_field = self.selected_field()?;
        if selected_field.get_is_read_only() {
            return Some(format!("Field '{}' is read-only.", selected_field.get_name()));
        }
        if matches!(selected_field.get_field_data(), ValuedStructFieldData::NestedStruct(_)) {
            return Some(format!(
                "Field '{}' is nested; nested field edits are not supported in TUI yet.",
                selected_field.get_name()
            ));
        }

        None
    }

    pub fn is_selected_field_editable(&self) -> bool {
        self.selected_field_edit_block_reason().is_none()
    }

    fn sync_selected_field_metadata(&mut self) {
        let Some(selected_field_position) = self.selected_field_position else {
            self.selected_field_name = None;
            self.pending_edit_text.clear();
            self.has_uncommitted_edit = false;
            return;
        };

        let Some(focused_struct) = self.focused_struct.as_ref() else {
            self.selected_field_name = None;
            self.pending_edit_text.clear();
            self.has_uncommitted_edit = false;
            return;
        };
        let Some(selected_field) = focused_struct
            .get_fields()
            .get(selected_field_position)
            .cloned()
        else {
            self.selected_field_name = None;
            self.pending_edit_text.clear();
            self.has_uncommitted_edit = false;
            return;
        };

        let selected_field_name = selected_field.get_name().to_string();
        self.selected_field_name = Some(selected_field_name);
        self.sync_selected_field_display_values(&selected_field);
        if self.has_uncommitted_edit {
            return;
        }

        if let Some(active_display_value) = self.selected_field_active_display_value() {
            self.pending_edit_text = active_display_value.get_anonymous_value_string().to_string();
            return;
        }

        let Some(selected_field_data_value) = selected_field.get_data_value() else {
            self.pending_edit_text.clear();
            return;
        };

        let symbol_registry = SymbolRegistry::get_instance();
        let selected_data_type_ref = selected_field_data_value.get_data_type_ref();
        let default_edit_format = symbol_registry.get_default_anonymous_value_string_format(selected_data_type_ref);
        let default_edit_value = symbol_registry
            .anonymize_value(selected_field_data_value, default_edit_format)
            .map(|anonymous_value_string| anonymous_value_string.get_anonymous_value_string().to_string())
            .unwrap_or_default();
        self.pending_edit_text = default_edit_value;
    }

    fn cycle_selected_field_display_format(
        &mut self,
        is_forward_direction: bool,
    ) -> Result<AnonymousValueStringFormat, String> {
        if self.has_uncommitted_edit {
            return Err("Cannot cycle display format while an uncommitted edit exists.".to_string());
        }

        let selected_field_name = self
            .selected_field_name
            .clone()
            .ok_or_else(|| "No struct field is selected.".to_string())?;
        let field_display_values = self
            .field_display_values
            .get(&selected_field_name)
            .ok_or_else(|| format!("No display formats are available for field '{}'.", selected_field_name))?;
        if field_display_values.is_empty() {
            return Err(format!("No display formats are available for field '{}'.", selected_field_name));
        }

        let current_display_value_index = self
            .field_active_display_value_indices
            .get(&selected_field_name)
            .copied()
            .unwrap_or(0)
            .min(field_display_values.len() - 1);
        let next_display_value_index = if is_forward_direction {
            (current_display_value_index + 1) % field_display_values.len()
        } else if current_display_value_index == 0 {
            field_display_values.len() - 1
        } else {
            current_display_value_index - 1
        };
        let next_display_value = field_display_values[next_display_value_index].clone();

        self.field_active_display_value_indices
            .insert(selected_field_name, next_display_value_index);
        self.pending_edit_text = next_display_value.get_anonymous_value_string().to_string();
        self.has_uncommitted_edit = false;

        Ok(next_display_value.get_anonymous_value_string_format())
    }

    fn sync_selected_field_display_values(
        &mut self,
        selected_field: &ValuedStructField,
    ) {
        let selected_field_name = selected_field.get_name().to_string();
        let Some(selected_field_data_value) = selected_field.get_data_value() else {
            self.field_display_values.remove(&selected_field_name);
            self.field_active_display_value_indices
                .remove(&selected_field_name);
            return;
        };

        let symbol_registry = SymbolRegistry::get_instance();
        let display_values = symbol_registry
            .anonymize_value_to_supported_formats(selected_field_data_value)
            .unwrap_or_else(|_| {
                let selected_data_type_ref = selected_field_data_value.get_data_type_ref();
                let default_edit_format = symbol_registry.get_default_anonymous_value_string_format(selected_data_type_ref);
                vec![
                    symbol_registry
                        .anonymize_value(selected_field_data_value, default_edit_format)
                        .unwrap_or_else(|_| AnonymousValueString::new(String::new(), default_edit_format, ContainerType::None)),
                ]
            });
        let default_display_format = symbol_registry.get_default_anonymous_value_string_format(selected_field_data_value.get_data_type_ref());
        let default_display_value_index = display_values
            .iter()
            .position(|display_value| display_value.get_anonymous_value_string_format() == default_display_format)
            .unwrap_or(0);
        let previous_display_value_index = self
            .field_active_display_value_indices
            .get(&selected_field_name)
            .copied()
            .unwrap_or(default_display_value_index);
        let clamped_display_value_index = previous_display_value_index.min(display_values.len().saturating_sub(1));

        self.field_display_values
            .insert(selected_field_name.clone(), display_values);
        self.field_active_display_value_indices
            .insert(selected_field_name, clamped_display_value_index);
    }

    pub(crate) fn active_display_value_for_field(
        &self,
        field_name: &str,
    ) -> Option<&AnonymousValueString> {
        let field_display_values = self.field_display_values.get(field_name)?;
        if field_display_values.is_empty() {
            return None;
        }

        let active_display_value_index = self
            .field_active_display_value_indices
            .get(field_name)
            .copied()
            .unwrap_or(0)
            .min(field_display_values.len() - 1);

        field_display_values.get(active_display_value_index)
    }

    fn selected_field_active_display_value(&self) -> Option<&AnonymousValueString> {
        let selected_field_name = self.selected_field_name.as_ref()?;
        self.active_display_value_for_field(selected_field_name)
    }

    pub(crate) fn selected_field_active_display_format(&self) -> Option<AnonymousValueStringFormat> {
        self.selected_field_active_display_value()
            .map(AnonymousValueString::get_anonymous_value_string_format)
    }

    pub(crate) fn selected_field_display_format_progress(&self) -> Option<(usize, usize)> {
        let selected_field_name = self.selected_field_name.as_ref()?;
        let field_display_values = self.field_display_values.get(selected_field_name)?;
        if field_display_values.is_empty() {
            return None;
        }

        let active_display_value_index = self
            .field_active_display_value_indices
            .get(selected_field_name)
            .copied()
            .unwrap_or(0)
            .min(field_display_values.len() - 1);
        Some((active_display_value_index, field_display_values.len()))
    }

    pub(crate) fn selected_field_edit_state_label(&self) -> String {
        if let Some(block_reason) = self.selected_field_edit_block_reason() {
            return block_reason;
        }

        "Editable value field.".to_string()
    }

    pub(crate) fn field_kind_marker(valued_struct_field: &ValuedStructField) -> &'static str {
        match valued_struct_field.get_field_data() {
            ValuedStructFieldData::Value(_) => "VAL",
            ValuedStructFieldData::NestedStruct(_) => "NEST",
        }
    }

    pub(crate) fn field_editability_marker(valued_struct_field: &ValuedStructField) -> &'static str {
        if valued_struct_field.get_is_read_only() { "RO" } else { "RW" }
    }
}

impl Default for StructViewerPaneState {
    fn default() -> Self {
        Self {
            selected_struct_name: None,
            selected_field_name: None,
            has_uncommitted_edit: false,
            source: StructViewerSource::None,
            focused_struct: None,
            selected_field_position: None,
            pending_edit_text: String::new(),
            selected_scan_result_refs: Vec::new(),
            selected_project_item_paths: Vec::new(),
            is_committing_edit: false,
            status_message: "Ready.".to_string(),
            field_display_values: HashMap::new(),
            field_active_display_value_indices: HashMap::new(),
        }
    }
}
