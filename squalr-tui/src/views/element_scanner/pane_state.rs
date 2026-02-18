use crate::views::element_scanner::summary::build_element_scanner_summary_lines_with_capacity;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_values::anonymous_value_string::AnonymousValueString;
use squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
use squalr_engine_api::structures::data_values::container_type::ContainerType;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_delta::ScanCompareTypeDelta;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_immediate::ScanCompareTypeImmediate;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_relative::ScanCompareTypeRelative;
use squalr_engine_api::structures::scanning::constraints::anonymous_scan_constraint::AnonymousScanConstraint;
use std::collections::BTreeSet;

/// Stores one editable scanner constraint row.
#[derive(Clone, Debug)]
pub struct ElementScannerConstraintState {
    pub scan_compare_type: ScanCompareType,
    pub scan_value_text: String,
}

impl Default for ElementScannerConstraintState {
    fn default() -> Self {
        Self {
            scan_compare_type: ScanCompareType::Immediate(ScanCompareTypeImmediate::Equal),
            scan_value_text: "0".to_string(),
        }
    }
}

/// Stores UI state for element scanner controls.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ElementScannerFocusTarget {
    DataTypes,
    Constraints,
}

/// Stores UI state for element scanner controls.
#[derive(Clone, Debug)]
pub struct ElementScannerPaneState {
    pub focus_target: ElementScannerFocusTarget,
    pub selected_data_type_index: usize,
    pub selected_data_type_indices: BTreeSet<usize>,
    pub constraint_rows: Vec<ElementScannerConstraintState>,
    pub selected_constraint_row_index: usize,
    pub has_pending_scan_request: bool,
    pub has_scan_results: bool,
    pub last_result_count: u64,
    pub last_total_size_in_bytes: u64,
    pub status_message: String,
}

impl ElementScannerPaneState {
    const MAX_CONSTRAINT_COUNT: usize = 5;
    const DATA_TYPE_GRID_COLUMN_COUNT: usize = 5;
    const SUPPORTED_DATA_TYPE_IDS: [&'static str; 10] = [
        "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64",
    ];
    const SUPPORTED_COMPARE_TYPES: [ScanCompareType; 20] = [
        ScanCompareType::Immediate(ScanCompareTypeImmediate::Equal),
        ScanCompareType::Immediate(ScanCompareTypeImmediate::NotEqual),
        ScanCompareType::Immediate(ScanCompareTypeImmediate::GreaterThan),
        ScanCompareType::Immediate(ScanCompareTypeImmediate::GreaterThanOrEqual),
        ScanCompareType::Immediate(ScanCompareTypeImmediate::LessThan),
        ScanCompareType::Immediate(ScanCompareTypeImmediate::LessThanOrEqual),
        ScanCompareType::Relative(ScanCompareTypeRelative::Changed),
        ScanCompareType::Relative(ScanCompareTypeRelative::Unchanged),
        ScanCompareType::Relative(ScanCompareTypeRelative::Increased),
        ScanCompareType::Relative(ScanCompareTypeRelative::Decreased),
        ScanCompareType::Delta(ScanCompareTypeDelta::IncreasedByX),
        ScanCompareType::Delta(ScanCompareTypeDelta::DecreasedByX),
        ScanCompareType::Delta(ScanCompareTypeDelta::MultipliedByX),
        ScanCompareType::Delta(ScanCompareTypeDelta::DividedByX),
        ScanCompareType::Delta(ScanCompareTypeDelta::ModuloByX),
        ScanCompareType::Delta(ScanCompareTypeDelta::ShiftLeftByX),
        ScanCompareType::Delta(ScanCompareTypeDelta::ShiftRightByX),
        ScanCompareType::Delta(ScanCompareTypeDelta::LogicalAndByX),
        ScanCompareType::Delta(ScanCompareTypeDelta::LogicalOrByX),
        ScanCompareType::Delta(ScanCompareTypeDelta::LogicalXorByX),
    ];

    pub fn selected_data_type_refs(&self) -> Vec<DataTypeRef> {
        self.selected_data_type_ids()
            .iter()
            .map(|data_type_id| DataTypeRef::new(data_type_id))
            .collect()
    }

    pub fn selected_data_type_ids(&self) -> Vec<&'static str> {
        self.selected_data_type_indices
            .iter()
            .filter_map(|selected_data_type_index| {
                Self::SUPPORTED_DATA_TYPE_IDS
                    .get(*selected_data_type_index)
                    .copied()
            })
            .collect()
    }

    pub fn is_data_type_selected(
        &self,
        data_type_index: usize,
    ) -> bool {
        self.selected_data_type_indices.contains(&data_type_index)
    }

    pub fn is_data_type_hovered(
        &self,
        data_type_index: usize,
    ) -> bool {
        self.selected_data_type_index == data_type_index
    }

    pub fn toggle_hovered_data_type_selection(&mut self) -> bool {
        if self
            .selected_data_type_indices
            .contains(&self.selected_data_type_index)
        {
            self.selected_data_type_indices
                .remove(&self.selected_data_type_index);
            true
        } else {
            self.selected_data_type_indices
                .insert(self.selected_data_type_index);
            true
        }
    }

    pub fn supported_data_type_names() -> &'static [&'static str] {
        &Self::SUPPORTED_DATA_TYPE_IDS
    }

    pub fn data_type_grid_column_count() -> usize {
        Self::DATA_TYPE_GRID_COLUMN_COUNT
    }

    pub fn active_constraint_count(&self) -> usize {
        self.constraint_rows.len()
    }

    pub fn select_data_type_right(&mut self) {
        self.selected_data_type_index = (self.selected_data_type_index + 1) % Self::SUPPORTED_DATA_TYPE_IDS.len();
    }

    pub fn select_data_type_left(&mut self) {
        self.selected_data_type_index = if self.selected_data_type_index == 0 {
            Self::SUPPORTED_DATA_TYPE_IDS.len() - 1
        } else {
            self.selected_data_type_index - 1
        };
    }

    pub fn select_data_type_down(&mut self) {
        self.select_data_type_vertical(true);
    }

    pub fn select_data_type_up(&mut self) {
        self.select_data_type_vertical(false);
    }

    pub fn move_focus_down(&mut self) {
        match self.focus_target {
            ElementScannerFocusTarget::DataTypes => {
                if self.is_selected_data_type_on_bottom_row() {
                    self.focus_target = ElementScannerFocusTarget::Constraints;
                } else {
                    self.select_data_type_down();
                }
            }
            ElementScannerFocusTarget::Constraints => self.select_next_constraint(),
        }
    }

    pub fn move_focus_up(&mut self) {
        match self.focus_target {
            ElementScannerFocusTarget::Constraints => {
                if self.selected_constraint_row_index == 0 {
                    self.focus_target = ElementScannerFocusTarget::DataTypes;
                } else {
                    self.select_previous_constraint();
                }
            }
            ElementScannerFocusTarget::DataTypes => self.select_data_type_up(),
        }
    }

    pub fn select_next_constraint(&mut self) {
        self.selected_constraint_row_index = (self.selected_constraint_row_index + 1) % self.constraint_rows.len();
    }

    pub fn select_previous_constraint(&mut self) {
        self.selected_constraint_row_index = if self.selected_constraint_row_index == 0 {
            self.constraint_rows.len() - 1
        } else {
            self.selected_constraint_row_index - 1
        };
    }

    pub fn add_constraint(&mut self) -> bool {
        if self.constraint_rows.len() >= Self::MAX_CONSTRAINT_COUNT {
            return false;
        }

        self.constraint_rows.push(if self.constraint_rows.len() == 1 {
            ElementScannerConstraintState {
                scan_compare_type: ScanCompareType::Immediate(ScanCompareTypeImmediate::LessThanOrEqual),
                ..ElementScannerConstraintState::default()
            }
        } else {
            ElementScannerConstraintState::default()
        });
        self.selected_constraint_row_index = self.constraint_rows.len() - 1;
        true
    }

    pub fn remove_selected_constraint(&mut self) -> bool {
        if self.constraint_rows.len() <= 1 {
            return false;
        }

        self.constraint_rows.remove(self.selected_constraint_row_index);
        if self.selected_constraint_row_index >= self.constraint_rows.len() {
            self.selected_constraint_row_index = self.constraint_rows.len() - 1;
        }
        true
    }

    pub fn cycle_selected_constraint_compare_type_forward(&mut self) {
        let selected_compare_type = self.constraint_rows[self.selected_constraint_row_index].scan_compare_type;
        let current_compare_type_index = Self::SUPPORTED_COMPARE_TYPES
            .iter()
            .position(|compare_type_candidate| *compare_type_candidate == selected_compare_type)
            .unwrap_or(0);
        let next_compare_type_index = (current_compare_type_index + 1) % Self::SUPPORTED_COMPARE_TYPES.len();
        self.constraint_rows[self.selected_constraint_row_index].scan_compare_type = Self::SUPPORTED_COMPARE_TYPES[next_compare_type_index];
    }

    pub fn cycle_selected_constraint_compare_type_backward(&mut self) {
        let selected_compare_type = self.constraint_rows[self.selected_constraint_row_index].scan_compare_type;
        let current_compare_type_index = Self::SUPPORTED_COMPARE_TYPES
            .iter()
            .position(|compare_type_candidate| *compare_type_candidate == selected_compare_type)
            .unwrap_or(0);
        let previous_compare_type_index = if current_compare_type_index == 0 {
            Self::SUPPORTED_COMPARE_TYPES.len() - 1
        } else {
            current_compare_type_index - 1
        };
        self.constraint_rows[self.selected_constraint_row_index].scan_compare_type = Self::SUPPORTED_COMPARE_TYPES[previous_compare_type_index];
    }

    pub fn append_selected_constraint_value_character(
        &mut self,
        value_character: char,
    ) {
        if !Self::is_supported_value_character(value_character) {
            return;
        }

        self.constraint_rows[self.selected_constraint_row_index]
            .scan_value_text
            .push(value_character);
    }

    pub fn backspace_selected_constraint_value(&mut self) {
        let selected_scan_value = &mut self.constraint_rows[self.selected_constraint_row_index].scan_value_text;
        selected_scan_value.pop();

        if selected_scan_value.is_empty() {
            selected_scan_value.push('0');
        }
    }

    pub fn clear_selected_constraint_value(&mut self) {
        self.constraint_rows[self.selected_constraint_row_index].scan_value_text = "0".to_string();
    }

    pub fn build_anonymous_scan_constraints(&self) -> Vec<AnonymousScanConstraint> {
        self.constraint_rows
            .iter()
            .map(|constraint_row| {
                let should_include_value = !matches!(constraint_row.scan_compare_type, ScanCompareType::Relative(_));
                let anonymous_value_string = if should_include_value {
                    Some(AnonymousValueString::new(
                        constraint_row.scan_value_text.clone(),
                        AnonymousValueStringFormat::Decimal,
                        ContainerType::None,
                    ))
                } else {
                    None
                };

                AnonymousScanConstraint::new(constraint_row.scan_compare_type, anonymous_value_string)
            })
            .collect()
    }

    pub fn summary_lines_with_capacity(
        &self,
        line_capacity: usize,
    ) -> Vec<String> {
        build_element_scanner_summary_lines_with_capacity(self, line_capacity)
    }

    fn is_supported_value_character(value_character: char) -> bool {
        value_character.is_ascii_digit() || value_character == '-' || value_character == '.'
    }

    fn select_data_type_vertical(
        &mut self,
        move_down: bool,
    ) {
        let data_type_count = Self::SUPPORTED_DATA_TYPE_IDS.len();
        let column_count = Self::DATA_TYPE_GRID_COLUMN_COUNT;
        let row_count = data_type_count.div_ceil(column_count);
        if row_count <= 1 {
            return;
        }

        let selected_row_index = self.selected_data_type_index / column_count;
        let selected_column_index = self.selected_data_type_index % column_count;
        let target_row_index = if move_down {
            (selected_row_index + 1) % row_count
        } else if selected_row_index == 0 {
            row_count - 1
        } else {
            selected_row_index - 1
        };

        let tentative_target_data_type_index = target_row_index * column_count + selected_column_index;
        self.selected_data_type_index = tentative_target_data_type_index.min(data_type_count - 1);
    }

    fn is_selected_data_type_on_bottom_row(&self) -> bool {
        self.selected_data_type_index + Self::DATA_TYPE_GRID_COLUMN_COUNT >= Self::SUPPORTED_DATA_TYPE_IDS.len()
    }
}

impl Default for ElementScannerPaneState {
    fn default() -> Self {
        Self {
            focus_target: ElementScannerFocusTarget::DataTypes,
            selected_data_type_index: 2,
            selected_data_type_indices: BTreeSet::from([2]),
            constraint_rows: vec![ElementScannerConstraintState::default()],
            selected_constraint_row_index: 0,
            has_pending_scan_request: false,
            has_scan_results: false,
            last_result_count: 0,
            last_total_size_in_bytes: 0,
            status_message: "Ready.".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ElementScannerPaneState;

    #[test]
    fn data_type_horizontal_navigation_wraps() {
        let mut element_scanner_pane_state = ElementScannerPaneState::default();
        element_scanner_pane_state.selected_data_type_index = 0;

        element_scanner_pane_state.select_data_type_left();
        assert_eq!(element_scanner_pane_state.selected_data_type_index, 9);

        element_scanner_pane_state.select_data_type_right();
        assert_eq!(element_scanner_pane_state.selected_data_type_index, 0);
    }

    #[test]
    fn data_type_vertical_navigation_moves_between_grid_rows() {
        let mut element_scanner_pane_state = ElementScannerPaneState::default();
        element_scanner_pane_state.selected_data_type_index = 2;

        element_scanner_pane_state.select_data_type_down();
        assert_eq!(element_scanner_pane_state.selected_data_type_index, 7);

        element_scanner_pane_state.select_data_type_up();
        assert_eq!(element_scanner_pane_state.selected_data_type_index, 2);
    }

    #[test]
    fn data_type_vertical_navigation_clamps_to_last_valid_cell_on_ragged_row() {
        let mut element_scanner_pane_state = ElementScannerPaneState::default();
        element_scanner_pane_state.selected_data_type_index = 3;

        element_scanner_pane_state.select_data_type_down();
        assert_eq!(element_scanner_pane_state.selected_data_type_index, 8);

        element_scanner_pane_state.selected_data_type_index = 4;
        element_scanner_pane_state.select_data_type_down();
        assert_eq!(element_scanner_pane_state.selected_data_type_index, 9);
    }

    #[test]
    fn toggling_hovered_data_type_selection_supports_multi_select() {
        let mut element_scanner_pane_state = ElementScannerPaneState::default();
        element_scanner_pane_state.selected_data_type_index = 1;

        assert!(element_scanner_pane_state.toggle_hovered_data_type_selection());
        assert!(
            element_scanner_pane_state
                .selected_data_type_indices
                .contains(&1)
        );
        assert!(
            element_scanner_pane_state
                .selected_data_type_indices
                .contains(&2)
        );
    }

    #[test]
    fn toggling_last_selected_data_type_allows_empty_selection() {
        let mut element_scanner_pane_state = ElementScannerPaneState::default();
        element_scanner_pane_state.selected_data_type_index = 2;

        assert!(element_scanner_pane_state.toggle_hovered_data_type_selection());
        assert!(element_scanner_pane_state.selected_data_type_indices.is_empty());
        assert!(element_scanner_pane_state.selected_data_type_ids().is_empty());
    }
}
