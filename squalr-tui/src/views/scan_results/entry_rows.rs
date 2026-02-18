use crate::state::pane_entry_row::PaneEntryRow;
use crate::views::entry_row_viewport::build_selection_relative_viewport_range;
use crate::views::scan_results::pane_state::ScanResultsPaneState;
use squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
use squalr_engine_api::structures::scan_results::scan_result::ScanResult;
use std::ops::RangeInclusive;

pub fn build_visible_scan_result_rows(
    scan_results_pane_state: &ScanResultsPaneState,
    viewport_capacity: usize,
) -> Vec<PaneEntryRow> {
    let selected_result_range = build_selected_result_range(scan_results_pane_state);
    let visible_scan_result_range = build_selection_relative_viewport_range(
        scan_results_pane_state.scan_results.len(),
        scan_results_pane_state.selected_result_index,
        viewport_capacity,
    );
    let mut entry_rows = Vec::with_capacity(visible_scan_result_range.len());

    for visible_scan_result_position in visible_scan_result_range {
        if let Some(scan_result) = scan_results_pane_state
            .scan_results
            .get(visible_scan_result_position)
        {
            let is_selected_scan_result = scan_results_pane_state.selected_result_index == Some(visible_scan_result_position);
            let is_in_selected_range = selected_result_range
                .as_ref()
                .map(|selected_range| selected_range.contains(&visible_scan_result_position))
                .unwrap_or(false);
            let freeze_marker = if scan_result.get_is_frozen() { "F" } else { " " };
            let value_preview = display_value_text(scan_result).unwrap_or_else(|| "?".to_string());
            let previous_value_preview = previous_display_value_text(scan_result);
            let marker_text = format!("{}{}", if is_in_selected_range { "*" } else { " " }, freeze_marker);
            let primary_text = format!(
                "idx={} global={} addr=0x{:X}",
                visible_scan_result_position,
                scan_result
                    .get_base_result()
                    .get_scan_result_ref()
                    .get_scan_result_global_index(),
                scan_result.get_address()
            );
            let secondary_text = Some(match previous_value_preview {
                Some(previous_value_preview) => format!(
                    "type={} value={} previous={}",
                    scan_result.get_data_type_ref().get_data_type_id(),
                    value_preview,
                    previous_value_preview
                ),
                None => format!("type={} value={}", scan_result.get_data_type_ref().get_data_type_id(), value_preview),
            });

            if is_selected_scan_result {
                entry_rows.push(PaneEntryRow::selected(marker_text, primary_text, secondary_text));
            } else if value_preview == "?" {
                entry_rows.push(PaneEntryRow::disabled(marker_text, primary_text, secondary_text));
            } else {
                entry_rows.push(PaneEntryRow::normal(marker_text, primary_text, secondary_text));
            }
        }
    }

    entry_rows
}

fn build_selected_result_range(scan_results_pane_state: &ScanResultsPaneState) -> Option<RangeInclusive<usize>> {
    let selection_anchor_position = scan_results_pane_state.selected_result_index?;
    let selection_end_position = scan_results_pane_state
        .selected_range_end_index
        .unwrap_or(selection_anchor_position);
    let (range_start_position, range_end_position) = if selection_anchor_position <= selection_end_position {
        (selection_anchor_position, selection_end_position)
    } else {
        (selection_end_position, selection_anchor_position)
    };

    Some(range_start_position..=range_end_position)
}

fn display_value_text(scan_result: &ScanResult) -> Option<String> {
    if let Some(display_value) = scan_result.get_recently_read_display_value_resolved(AnonymousValueStringFormat::Decimal) {
        return Some(display_value.get_anonymous_value_string().to_string());
    }

    if let Some(display_value) = scan_result.get_recently_read_display_values().first() {
        return Some(display_value.get_anonymous_value_string().to_string());
    }

    None
}

fn previous_display_value_text(scan_result: &ScanResult) -> Option<String> {
    if let Some(previous_display_value) = scan_result.get_previous_display_value(AnonymousValueStringFormat::Decimal) {
        return Some(previous_display_value.get_anonymous_value_string().to_string());
    }

    scan_result
        .get_previous_display_values()
        .first()
        .map(|previous_display_value| previous_display_value.get_anonymous_value_string().to_string())
}
