use crate::views::element_scanner::pane_state::{ElementScannerConstraintState, ElementScannerFocusTarget, ElementScannerPaneState};
use crate::views::entry_row_viewport::build_selection_relative_viewport_range;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_delta::ScanCompareTypeDelta;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_immediate::ScanCompareTypeImmediate;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_relative::ScanCompareTypeRelative;

pub fn build_element_scanner_summary_lines_with_capacity(
    element_scanner_pane_state: &ElementScannerPaneState,
    line_capacity: usize,
) -> Vec<String> {
    if line_capacity == 0 {
        return Vec::new();
    }

    let data_type_grid_lines = build_data_type_grid_lines(element_scanner_pane_state);
    let constraint_row_lines = build_constraint_row_lines(element_scanner_pane_state);
    let constraint_line_capacity = line_capacity.saturating_sub(4 + data_type_grid_lines.len());
    let visible_constraint_lines = selected_constraint_window_lines(element_scanner_pane_state, &constraint_row_lines, constraint_line_capacity);

    let mut prioritized_lines = vec![
        "[ACT] s scan | n reset | c collect | a add | x remove.".to_string(),
        "[CTRL] arrows move | Space/Enter toggle-type (types) | [/] row (constraints) | m/M compare | type value.".to_string(),
        format!(
            "[SCAN] constraints={} | selected_row={} | pending={} | has_results={}.",
            element_scanner_pane_state.active_constraint_count(),
            element_scanner_pane_state.selected_constraint_row_index + 1,
            element_scanner_pane_state.has_pending_scan_request,
            element_scanner_pane_state.has_scan_results
        ),
        format!("[STAT] {}.", element_scanner_pane_state.status_message),
    ];
    prioritized_lines.extend(data_type_grid_lines);
    prioritized_lines.extend(visible_constraint_lines);

    prioritized_lines.into_iter().take(line_capacity).collect()
}

fn build_data_type_grid_lines(element_scanner_pane_state: &ElementScannerPaneState) -> Vec<String> {
    let supported_data_type_names = ElementScannerPaneState::supported_data_type_names();
    let grid_column_count = ElementScannerPaneState::data_type_grid_column_count();
    let mut data_type_grid_lines = vec![format!(
        "[TYPE] selected={}.",
        element_scanner_pane_state.selected_data_type_ids().join(",")
    )];

    for (data_type_row_index, data_type_name_row) in supported_data_type_names.chunks(grid_column_count).enumerate() {
        let mut row_cells = Vec::with_capacity(data_type_name_row.len());
        for (data_type_name_column_index, data_type_name) in data_type_name_row.iter().enumerate() {
            let data_type_index = (data_type_row_index * grid_column_count) + data_type_name_column_index;
            let hovered_marker = if element_scanner_pane_state.focus_target == ElementScannerFocusTarget::DataTypes
                && element_scanner_pane_state.is_data_type_hovered(data_type_index)
            {
                ">"
            } else {
                " "
            };
            let selected_marker = if element_scanner_pane_state.is_data_type_selected(data_type_index) {
                "[x]"
            } else {
                "[ ]"
            };
            row_cells.push(format!("{}{} {}", hovered_marker, selected_marker, data_type_name));
        }

        data_type_grid_lines.push(format!("      {}.", row_cells.join("  ")));
    }

    data_type_grid_lines
}

fn build_constraint_row_lines(element_scanner_pane_state: &ElementScannerPaneState) -> Vec<String> {
    element_scanner_pane_state
        .constraint_rows
        .iter()
        .enumerate()
        .map(|(constraint_row_index, constraint_row)| {
            build_constraint_row_line(
                constraint_row,
                constraint_row_index,
                element_scanner_pane_state.focus_target == ElementScannerFocusTarget::Constraints
                    && element_scanner_pane_state.selected_constraint_row_index == constraint_row_index,
            )
        })
        .collect()
}

fn build_constraint_row_line(
    constraint_row: &ElementScannerConstraintState,
    constraint_index: usize,
    is_selected: bool,
) -> String {
    let selected_marker = if is_selected { ">" } else { " " };
    let compare_expression = scan_compare_expression(constraint_row);
    format!("{} [CONSTRAINT #{}] {}.", selected_marker, constraint_index + 1, compare_expression)
}

fn selected_constraint_window_lines(
    element_scanner_pane_state: &ElementScannerPaneState,
    constraint_row_lines: &[String],
    line_capacity: usize,
) -> Vec<String> {
    if line_capacity == 0 {
        return Vec::new();
    }

    let selection_window_range = build_selection_relative_viewport_range(
        constraint_row_lines.len(),
        Some(element_scanner_pane_state.selected_constraint_row_index),
        line_capacity,
    );
    constraint_row_lines[selection_window_range].to_vec()
}

fn scan_compare_type_label(scan_compare_type: ScanCompareType) -> &'static str {
    match scan_compare_type {
        ScanCompareType::Immediate(ScanCompareTypeImmediate::Equal) => "==",
        ScanCompareType::Immediate(ScanCompareTypeImmediate::NotEqual) => "!=",
        ScanCompareType::Immediate(ScanCompareTypeImmediate::GreaterThan) => ">",
        ScanCompareType::Immediate(ScanCompareTypeImmediate::GreaterThanOrEqual) => ">=",
        ScanCompareType::Immediate(ScanCompareTypeImmediate::LessThan) => "<",
        ScanCompareType::Immediate(ScanCompareTypeImmediate::LessThanOrEqual) => "<=",
        ScanCompareType::Relative(ScanCompareTypeRelative::Changed) => "changed",
        ScanCompareType::Relative(ScanCompareTypeRelative::Unchanged) => "unchanged",
        ScanCompareType::Relative(ScanCompareTypeRelative::Increased) => "increased",
        ScanCompareType::Relative(ScanCompareTypeRelative::Decreased) => "decreased",
        ScanCompareType::Delta(ScanCompareTypeDelta::IncreasedByX) => "+x",
        ScanCompareType::Delta(ScanCompareTypeDelta::DecreasedByX) => "-x",
        ScanCompareType::Delta(ScanCompareTypeDelta::MultipliedByX) => "*x",
        ScanCompareType::Delta(ScanCompareTypeDelta::DividedByX) => "/x",
        ScanCompareType::Delta(ScanCompareTypeDelta::ModuloByX) => "%x",
        ScanCompareType::Delta(ScanCompareTypeDelta::ShiftLeftByX) => "<<x",
        ScanCompareType::Delta(ScanCompareTypeDelta::ShiftRightByX) => ">>x",
        ScanCompareType::Delta(ScanCompareTypeDelta::LogicalAndByX) => "&x",
        ScanCompareType::Delta(ScanCompareTypeDelta::LogicalOrByX) => "|x",
        ScanCompareType::Delta(ScanCompareTypeDelta::LogicalXorByX) => "^x",
    }
}

fn scan_compare_expression(constraint_row: &ElementScannerConstraintState) -> String {
    let compare_label = scan_compare_type_label(constraint_row.scan_compare_type);
    if matches!(constraint_row.scan_compare_type, ScanCompareType::Immediate(_)) {
        format!("x {} {}", compare_label, constraint_row.scan_value_text)
    } else {
        format!("{} {}", compare_label, constraint_row.scan_value_text)
    }
}
