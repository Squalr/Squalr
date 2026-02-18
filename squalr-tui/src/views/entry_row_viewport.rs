use std::ops::Range;

/// Builds a bounded viewport range for list rows using selection-relative windowing.
pub fn build_selection_relative_viewport_range(
    total_entry_count: usize,
    selected_entry_index: Option<usize>,
    viewport_capacity: usize,
) -> Range<usize> {
    if total_entry_count == 0 || viewport_capacity == 0 {
        return 0..0;
    }

    let visible_entry_count = total_entry_count.min(viewport_capacity);
    if visible_entry_count == total_entry_count {
        return 0..total_entry_count;
    }

    let selected_entry_index = selected_entry_index
        .filter(|selected_entry_index| *selected_entry_index < total_entry_count)
        .unwrap_or(0);
    let leading_context_count = visible_entry_count / 2;
    let mut viewport_start_index = selected_entry_index.saturating_sub(leading_context_count);
    let maximum_viewport_start_index = total_entry_count.saturating_sub(visible_entry_count);
    if viewport_start_index > maximum_viewport_start_index {
        viewport_start_index = maximum_viewport_start_index;
    }

    let viewport_end_index = viewport_start_index + visible_entry_count;
    viewport_start_index..viewport_end_index
}
