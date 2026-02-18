use crate::views::entry_row_viewport::build_selection_relative_viewport_range;
use crate::views::settings::pane_state::{SettingsCategory, SettingsPaneState};
use squalr_engine_api::structures::data_types::floating_point_tolerance::FloatingPointTolerance;
use squalr_engine_api::structures::memory::memory_alignment::MemoryAlignment;
use squalr_engine_api::structures::scanning::memory_read_mode::MemoryReadMode;

pub fn build_settings_summary_lines_with_capacity(
    settings_pane_state: &SettingsPaneState,
    line_capacity: usize,
) -> Vec<String> {
    if line_capacity == 0 {
        return Vec::new();
    }

    let category_lines = selected_category_lines(settings_pane_state);
    let additional_field_capacity = line_capacity.saturating_sub(7);
    let additional_field_lines = selected_field_window_lines(settings_pane_state, &category_lines, additional_field_capacity);
    let mut prioritized_lines = vec![
        "[NAV] Left/Right category | Up/Down move | Home/End jump.".to_string(),
        "[ACT] Space toggle | +/- step | </> enum | Type number | Enter apply | r refresh-all | Ctrl+R reset-tab.".to_string(),
    ];
    prioritized_lines.push(format!("[TAB] {}.", render_category_tabs(settings_pane_state.selected_category)));
    prioritized_lines.push(format!(
        "[CAT] {} | field={}.",
        settings_pane_state.selected_category.title(),
        settings_pane_state.selected_field_index
    ));
    prioritized_lines.push(format!(
        "[LOAD] pending={} | refreshing={} | applying={}.",
        settings_pane_state.has_pending_changes, settings_pane_state.is_refreshing_settings, settings_pane_state.is_applying_settings
    ));
    prioritized_lines.push(format!("[STAT] {}.", settings_pane_state.status_message));
    prioritized_lines.extend(additional_field_lines);

    prioritized_lines.into_iter().take(line_capacity).collect()
}

fn selected_field_window_lines(
    settings_pane_state: &SettingsPaneState,
    category_lines: &[String],
    line_capacity: usize,
) -> Vec<String> {
    if line_capacity == 0 {
        return Vec::new();
    }

    let selection_window_range = build_selection_relative_viewport_range(category_lines.len(), Some(settings_pane_state.selected_field_index), line_capacity);
    category_lines[selection_window_range].to_vec()
}

fn selected_category_lines(settings_pane_state: &SettingsPaneState) -> Vec<String> {
    match settings_pane_state.selected_category {
        SettingsCategory::General => general_summary_lines(settings_pane_state),
        SettingsCategory::Memory => memory_summary_lines(settings_pane_state),
        SettingsCategory::Scan => scan_summary_lines(settings_pane_state),
    }
}

fn general_summary_lines(settings_pane_state: &SettingsPaneState) -> Vec<String> {
    vec![format!(
        "{} debug_engine_request_delay_ms={}{}.",
        selection_marker(settings_pane_state.selected_field_index, 0),
        settings_pane_state
            .general_settings
            .debug_engine_request_delay_ms,
        pending_numeric_edit_suffix(settings_pane_state, 0)
    )]
}

fn memory_summary_lines(settings_pane_state: &SettingsPaneState) -> Vec<String> {
    vec![
        format!(
            "{} type.none={}{}.",
            selection_marker(settings_pane_state.selected_field_index, 0),
            bool_indicator(settings_pane_state.memory_settings.memory_type_none),
            settings_pane_state.memory_settings.memory_type_none
        ),
        format!(
            "{} type.private={}{}.",
            selection_marker(settings_pane_state.selected_field_index, 1),
            bool_indicator(settings_pane_state.memory_settings.memory_type_private),
            settings_pane_state.memory_settings.memory_type_private
        ),
        format!(
            "{} type.image={}{}.",
            selection_marker(settings_pane_state.selected_field_index, 2),
            bool_indicator(settings_pane_state.memory_settings.memory_type_image),
            settings_pane_state.memory_settings.memory_type_image
        ),
        format!(
            "{} type.mapped={}{}.",
            selection_marker(settings_pane_state.selected_field_index, 3),
            bool_indicator(settings_pane_state.memory_settings.memory_type_mapped),
            settings_pane_state.memory_settings.memory_type_mapped
        ),
        format!(
            "{} require.write={}{}.",
            selection_marker(settings_pane_state.selected_field_index, 4),
            bool_indicator(settings_pane_state.memory_settings.required_write),
            settings_pane_state.memory_settings.required_write
        ),
        format!(
            "{} require.execute={}{}.",
            selection_marker(settings_pane_state.selected_field_index, 5),
            bool_indicator(settings_pane_state.memory_settings.required_execute),
            settings_pane_state.memory_settings.required_execute
        ),
        format!(
            "{} require.copy_on_write={}{}.",
            selection_marker(settings_pane_state.selected_field_index, 6),
            bool_indicator(settings_pane_state.memory_settings.required_copy_on_write),
            settings_pane_state.memory_settings.required_copy_on_write
        ),
        format!(
            "{} exclude.write={}{}.",
            selection_marker(settings_pane_state.selected_field_index, 7),
            bool_indicator(settings_pane_state.memory_settings.excluded_write),
            settings_pane_state.memory_settings.excluded_write
        ),
        format!(
            "{} exclude.execute={}{}.",
            selection_marker(settings_pane_state.selected_field_index, 8),
            bool_indicator(settings_pane_state.memory_settings.excluded_execute),
            settings_pane_state.memory_settings.excluded_execute
        ),
        format!(
            "{} exclude.copy_on_write={}{}.",
            selection_marker(settings_pane_state.selected_field_index, 9),
            bool_indicator(settings_pane_state.memory_settings.excluded_copy_on_write),
            settings_pane_state.memory_settings.excluded_copy_on_write
        ),
        format!(
            "{} start_address=0x{:X}{}.",
            selection_marker(settings_pane_state.selected_field_index, 10),
            settings_pane_state.memory_settings.start_address,
            pending_numeric_edit_suffix(settings_pane_state, 10)
        ),
        format!(
            "{} end_address=0x{:X}{}.",
            selection_marker(settings_pane_state.selected_field_index, 11),
            settings_pane_state.memory_settings.end_address,
            pending_numeric_edit_suffix(settings_pane_state, 11)
        ),
        format!(
            "{} usermode_only={}{}.",
            selection_marker(settings_pane_state.selected_field_index, 12),
            bool_indicator(settings_pane_state.memory_settings.only_query_usermode),
            settings_pane_state.memory_settings.only_query_usermode
        ),
    ]
}

fn scan_summary_lines(settings_pane_state: &SettingsPaneState) -> Vec<String> {
    vec![
        format!(
            "{} page_size={}{}.",
            selection_marker(settings_pane_state.selected_field_index, 0),
            settings_pane_state.scan_settings.results_page_size,
            pending_numeric_edit_suffix(settings_pane_state, 0)
        ),
        format!(
            "{} freeze_ms={}{}.",
            selection_marker(settings_pane_state.selected_field_index, 1),
            settings_pane_state.scan_settings.freeze_interval_ms,
            pending_numeric_edit_suffix(settings_pane_state, 1)
        ),
        format!(
            "{} project_read_ms={}{}.",
            selection_marker(settings_pane_state.selected_field_index, 2),
            settings_pane_state.scan_settings.project_read_interval_ms,
            pending_numeric_edit_suffix(settings_pane_state, 2)
        ),
        format!(
            "{} result_read_ms={}{}.",
            selection_marker(settings_pane_state.selected_field_index, 3),
            settings_pane_state.scan_settings.results_read_interval_ms,
            pending_numeric_edit_suffix(settings_pane_state, 3)
        ),
        format!(
            "{} memory_alignment={}.",
            selection_marker(settings_pane_state.selected_field_index, 4),
            memory_alignment_label(settings_pane_state.scan_settings.memory_alignment)
        ),
        format!(
            "{} memory_read_mode={}.",
            selection_marker(settings_pane_state.selected_field_index, 5),
            memory_read_mode_label(settings_pane_state.scan_settings.memory_read_mode)
        ),
        format!(
            "{} float_tolerance={}.",
            selection_marker(settings_pane_state.selected_field_index, 6),
            floating_point_tolerance_label(settings_pane_state.scan_settings.floating_point_tolerance)
        ),
        format!(
            "{} single_threaded={}{}.",
            selection_marker(settings_pane_state.selected_field_index, 7),
            bool_indicator(settings_pane_state.scan_settings.is_single_threaded_scan),
            settings_pane_state.scan_settings.is_single_threaded_scan
        ),
        format!(
            "{} debug_validate_scan={}{}.",
            selection_marker(settings_pane_state.selected_field_index, 8),
            bool_indicator(settings_pane_state.scan_settings.debug_perform_validation_scan),
            settings_pane_state.scan_settings.debug_perform_validation_scan
        ),
    ]
}

fn selection_marker(
    selected_field_index: usize,
    field_position: usize,
) -> &'static str {
    if selected_field_index == field_position { ">" } else { " " }
}

fn bool_indicator(boolean_value: bool) -> &'static str {
    if boolean_value { "[*] " } else { "[ ] " }
}

fn memory_alignment_label(memory_alignment: Option<MemoryAlignment>) -> &'static str {
    match memory_alignment {
        Some(MemoryAlignment::Alignment1) => "1",
        Some(MemoryAlignment::Alignment2) => "2",
        Some(MemoryAlignment::Alignment4) => "4",
        Some(MemoryAlignment::Alignment8) => "8",
        None => "none",
    }
}

fn memory_read_mode_label(memory_read_mode: MemoryReadMode) -> &'static str {
    match memory_read_mode {
        MemoryReadMode::Skip => "skip",
        MemoryReadMode::ReadBeforeScan => "before_scan",
        MemoryReadMode::ReadInterleavedWithScan => "interleaved",
    }
}

fn floating_point_tolerance_label(floating_point_tolerance: FloatingPointTolerance) -> &'static str {
    match floating_point_tolerance {
        FloatingPointTolerance::Tolerance10E1 => "0.1",
        FloatingPointTolerance::Tolerance10E2 => "0.01",
        FloatingPointTolerance::Tolerance10E3 => "0.001",
        FloatingPointTolerance::Tolerance10E4 => "0.0001",
        FloatingPointTolerance::Tolerance10E5 => "0.00001",
        FloatingPointTolerance::ToleranceEpsilon => "epsilon",
    }
}

fn render_category_tabs(selected_category: SettingsCategory) -> String {
    SettingsCategory::all_categories()
        .iter()
        .map(|settings_category| {
            if *settings_category == selected_category {
                format!("[{}]", settings_category.title())
            } else {
                settings_category.title().to_string()
            }
        })
        .collect::<Vec<String>>()
        .join(" | ")
}

fn pending_numeric_edit_suffix(
    settings_pane_state: &SettingsPaneState,
    field_position: usize,
) -> String {
    if settings_pane_state.selected_field_index != field_position {
        return String::new();
    }

    let Some(pending_numeric_edit_buffer) = settings_pane_state.pending_numeric_edit_buffer.as_ref() else {
        return String::new();
    };
    format!(" | edit='{}'", pending_numeric_edit_buffer)
}
