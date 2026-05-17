use crate::ui::list_navigation::ListNavigationDirection;
use crate::views::symbol_layout_editor::view_data::symbol_layout_editor_view_data::{
    SymbolLayoutEditDraft, SymbolLayoutFieldOffsetMode, SymbolLayoutUnassignedSelection,
};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SymbolLayoutFieldSpan {
    pub field_position: usize,
    pub offset_in_bytes: u64,
    pub size_in_bytes: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SymbolLayoutUnassignedAdjacentField {
    pub field_position: usize,
    pub offset_in_bytes: u64,
    pub size_in_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolLayoutUnassignedRowContext {
    pub offset_in_bytes: u64,
    pub size_in_bytes: u64,
    pub move_up_field: Option<SymbolLayoutUnassignedAdjacentField>,
    pub move_down_field: Option<SymbolLayoutUnassignedAdjacentField>,
    pub move_up_unassigned_span: Option<SymbolLayoutUnassignedSelection>,
    pub move_down_unassigned_span: Option<SymbolLayoutUnassignedSelection>,
    pub merge_above_span: Option<SymbolLayoutUnassignedSelection>,
    pub merge_below_span: Option<SymbolLayoutUnassignedSelection>,
}

pub struct SymbolLayoutDraftOps;

impl SymbolLayoutDraftOps {
    pub fn field_insert_index_for_offset(
        field_spans: &[SymbolLayoutFieldSpan],
        field_count: usize,
        offset_in_bytes: u64,
    ) -> usize {
        field_spans
            .iter()
            .filter(|field_span| field_span.offset_in_bytes < offset_in_bytes)
            .map(|field_span| field_span.field_position.saturating_add(1))
            .max()
            .unwrap_or(0)
            .min(field_count)
    }

    pub fn set_field_static_offset(
        draft: &mut SymbolLayoutEditDraft,
        field_position: usize,
        offset_in_bytes: u64,
    ) -> bool {
        let Some(field_draft) = draft.field_drafts.get_mut(field_position) else {
            return false;
        };

        field_draft.offset_mode = SymbolLayoutFieldOffsetMode::Static;
        field_draft.static_offset_in_bytes = offset_in_bytes.to_string();
        true
    }

    pub fn build_unique_field_name(
        draft: &SymbolLayoutEditDraft,
        base_name: &str,
    ) -> String {
        let trimmed_base_name = base_name.trim();
        let base_name = if trimmed_base_name.is_empty() {
            if draft.layout_kind.is_union() { "variant" } else { "field" }
        } else {
            trimmed_base_name
        };
        if !draft
            .field_drafts
            .iter()
            .any(|field_draft| field_draft.field_name.trim() == base_name)
        {
            return base_name.to_string();
        }

        let mut suffix_index = 2_u64;
        loop {
            let candidate_name = format!("{}_{}", base_name, suffix_index);
            if !draft
                .field_drafts
                .iter()
                .any(|field_draft| field_draft.field_name.trim() == candidate_name)
            {
                return candidate_name;
            }
            suffix_index = suffix_index.saturating_add(1);
        }
    }

    pub fn resolve_field_span_by_position(
        field_spans: &[SymbolLayoutFieldSpan],
        field_position: usize,
    ) -> Option<SymbolLayoutFieldSpan> {
        field_spans
            .iter()
            .copied()
            .find(|field_span| field_span.field_position == field_position)
    }

    pub fn resolve_adjacent_field_span(
        field_spans: &[SymbolLayoutFieldSpan],
        current_field_span: SymbolLayoutFieldSpan,
        direction: ListNavigationDirection,
    ) -> Option<SymbolLayoutFieldSpan> {
        let current_sort_key = (current_field_span.offset_in_bytes, current_field_span.field_position);
        match direction {
            ListNavigationDirection::Up => field_spans
                .iter()
                .copied()
                .filter(|field_span| field_span.field_position != current_field_span.field_position)
                .filter(|field_span| (field_span.offset_in_bytes, field_span.field_position) < current_sort_key)
                .max_by_key(|field_span| (field_span.offset_in_bytes, field_span.field_position)),
            ListNavigationDirection::Down => field_spans
                .iter()
                .copied()
                .filter(|field_span| field_span.field_position != current_field_span.field_position)
                .filter(|field_span| (field_span.offset_in_bytes, field_span.field_position) > current_sort_key)
                .min_by_key(|field_span| (field_span.offset_in_bytes, field_span.field_position)),
        }
    }

    pub fn resolve_unassigned_row_before_field(
        current_field_span: SymbolLayoutFieldSpan,
        previous_field_span: Option<SymbolLayoutFieldSpan>,
        split_offsets: &BTreeSet<u64>,
    ) -> Option<SymbolLayoutUnassignedSelection> {
        let gap_start = previous_field_span
            .map(|previous_field_span| {
                previous_field_span
                    .offset_in_bytes
                    .saturating_add(previous_field_span.size_in_bytes)
            })
            .unwrap_or(0);
        let gap_end = current_field_span.offset_in_bytes;

        if gap_end <= gap_start {
            return None;
        }

        let row_start = split_offsets
            .iter()
            .copied()
            .filter(|split_offset| *split_offset > gap_start && *split_offset < gap_end)
            .max()
            .unwrap_or(gap_start);

        Some(SymbolLayoutUnassignedSelection::new(row_start, gap_end.saturating_sub(row_start)))
    }

    pub fn resolve_unassigned_row_after_field(
        current_field_span: SymbolLayoutFieldSpan,
        next_field_span: Option<SymbolLayoutFieldSpan>,
        layout_size_in_bytes: u64,
        split_offsets: &BTreeSet<u64>,
    ) -> Option<SymbolLayoutUnassignedSelection> {
        let gap_start = current_field_span
            .offset_in_bytes
            .saturating_add(current_field_span.size_in_bytes);
        let gap_end = next_field_span
            .map(|next_field_span| next_field_span.offset_in_bytes)
            .unwrap_or(layout_size_in_bytes);

        if gap_end <= gap_start {
            return None;
        }

        let row_end = split_offsets
            .iter()
            .copied()
            .filter(|split_offset| *split_offset > gap_start && *split_offset < gap_end)
            .min()
            .unwrap_or(gap_end);

        Some(SymbolLayoutUnassignedSelection::new(gap_start, row_end.saturating_sub(gap_start)))
    }

    pub fn move_struct_field_up(
        draft: &mut SymbolLayoutEditDraft,
        field_spans: &[SymbolLayoutFieldSpan],
        split_offsets: &BTreeSet<u64>,
        field_index: usize,
    ) -> bool {
        let Some(current_field_span) = Self::resolve_field_span_by_position(field_spans, field_index) else {
            return false;
        };
        let previous_field_span = Self::resolve_adjacent_field_span(field_spans, current_field_span, ListNavigationDirection::Up);

        if let Some(previous_unassigned_row) = Self::resolve_unassigned_row_before_field(current_field_span, previous_field_span, split_offsets) {
            return Self::set_field_static_offset(draft, field_index, previous_unassigned_row.get_offset_in_bytes());
        }

        let Some(previous_field_span) = previous_field_span else {
            return false;
        };
        let moved_current_offset = previous_field_span.offset_in_bytes;
        let moved_previous_offset = previous_field_span
            .offset_in_bytes
            .saturating_add(current_field_span.size_in_bytes);
        Self::set_field_static_offset(draft, field_index, moved_current_offset)
            && Self::set_field_static_offset(draft, previous_field_span.field_position, moved_previous_offset)
    }

    pub fn split_offset_to_preserve_field_move_up(
        field_spans: &[SymbolLayoutFieldSpan],
        layout_size_in_bytes: u64,
        split_offsets: &BTreeSet<u64>,
        field_index: usize,
    ) -> Option<u64> {
        let current_field_span = Self::resolve_field_span_by_position(field_spans, field_index)?;
        let previous_field_span = Self::resolve_adjacent_field_span(field_spans, current_field_span, ListNavigationDirection::Up);
        Self::resolve_unassigned_row_before_field(current_field_span, previous_field_span, split_offsets)?;
        let next_field_span = Self::resolve_adjacent_field_span(field_spans, current_field_span, ListNavigationDirection::Down);
        Self::resolve_unassigned_row_after_field(current_field_span, next_field_span, layout_size_in_bytes, split_offsets)?;

        current_field_span
            .offset_in_bytes
            .checked_add(current_field_span.size_in_bytes)
    }

    pub fn move_struct_field_down(
        draft: &mut SymbolLayoutEditDraft,
        field_spans: &[SymbolLayoutFieldSpan],
        layout_size_in_bytes: u64,
        split_offsets: &BTreeSet<u64>,
        field_index: usize,
    ) -> bool {
        let Some(current_field_span) = Self::resolve_field_span_by_position(field_spans, field_index) else {
            return false;
        };
        let next_field_span = Self::resolve_adjacent_field_span(field_spans, current_field_span, ListNavigationDirection::Down);

        if let Some(next_unassigned_row) = Self::resolve_unassigned_row_after_field(current_field_span, next_field_span, layout_size_in_bytes, split_offsets) {
            return Self::set_field_static_offset(
                draft,
                field_index,
                current_field_span
                    .offset_in_bytes
                    .saturating_add(next_unassigned_row.get_size_in_bytes()),
            );
        }

        let Some(next_field_span) = next_field_span else {
            return false;
        };
        let moved_next_offset = current_field_span.offset_in_bytes;
        let moved_current_offset = current_field_span
            .offset_in_bytes
            .saturating_add(next_field_span.size_in_bytes);
        Self::set_field_static_offset(draft, next_field_span.field_position, moved_next_offset)
            && Self::set_field_static_offset(draft, field_index, moved_current_offset)
    }

    pub fn split_offset_to_preserve_field_move_down(
        field_spans: &[SymbolLayoutFieldSpan],
        layout_size_in_bytes: u64,
        split_offsets: &BTreeSet<u64>,
        field_index: usize,
    ) -> Option<u64> {
        let current_field_span = Self::resolve_field_span_by_position(field_spans, field_index)?;
        let next_field_span = Self::resolve_adjacent_field_span(field_spans, current_field_span, ListNavigationDirection::Down);
        Self::resolve_unassigned_row_after_field(current_field_span, next_field_span, layout_size_in_bytes, split_offsets)?;
        let previous_field_span = Self::resolve_adjacent_field_span(field_spans, current_field_span, ListNavigationDirection::Up);
        Self::resolve_unassigned_row_before_field(current_field_span, previous_field_span, split_offsets)?;

        (current_field_span.offset_in_bytes > 0).then_some(current_field_span.offset_in_bytes)
    }

    pub fn can_move_struct_field_up(
        field_spans: &[SymbolLayoutFieldSpan],
        split_offsets: &BTreeSet<u64>,
        field_index: usize,
    ) -> bool {
        let Some(current_field_span) = Self::resolve_field_span_by_position(field_spans, field_index) else {
            return false;
        };
        let previous_field_span = Self::resolve_adjacent_field_span(field_spans, current_field_span, ListNavigationDirection::Up);

        Self::resolve_unassigned_row_before_field(current_field_span, previous_field_span, split_offsets).is_some() || previous_field_span.is_some()
    }

    pub fn can_move_struct_field_down(
        field_spans: &[SymbolLayoutFieldSpan],
        layout_size_in_bytes: u64,
        split_offsets: &BTreeSet<u64>,
        field_index: usize,
    ) -> bool {
        let Some(current_field_span) = Self::resolve_field_span_by_position(field_spans, field_index) else {
            return false;
        };
        let next_field_span = Self::resolve_adjacent_field_span(field_spans, current_field_span, ListNavigationDirection::Down);

        Self::resolve_unassigned_row_after_field(current_field_span, next_field_span, layout_size_in_bytes, split_offsets).is_some()
            || next_field_span.is_some()
    }

    pub fn resolve_first_unassigned_offset(
        field_spans: &[SymbolLayoutFieldSpan],
        layout_size_in_bytes: u64,
    ) -> Option<u64> {
        let mut sorted_field_spans = field_spans.to_vec();
        sorted_field_spans.sort_by_key(|field_span| (field_span.offset_in_bytes, field_span.field_position));

        let mut next_visible_offset = 0_u64;
        for field_span in sorted_field_spans {
            if field_span.offset_in_bytes > next_visible_offset {
                return Some(next_visible_offset);
            }
            next_visible_offset = next_visible_offset.max(
                field_span
                    .offset_in_bytes
                    .saturating_add(field_span.size_in_bytes),
            );
        }

        (next_visible_offset < layout_size_in_bytes).then_some(next_visible_offset)
    }

    pub fn resolve_tail_unassigned_offset(
        field_spans: &[SymbolLayoutFieldSpan],
        layout_size_in_bytes: u64,
    ) -> Option<u64> {
        let tail_offset = field_spans
            .iter()
            .map(|field_span| {
                field_span
                    .offset_in_bytes
                    .saturating_add(field_span.size_in_bytes)
            })
            .max()
            .unwrap_or(0);

        (tail_offset < layout_size_in_bytes).then_some(tail_offset)
    }

    pub fn move_unassigned_span_up(
        draft: &mut SymbolLayoutEditDraft,
        row_context: SymbolLayoutUnassignedRowContext,
    ) -> Option<SymbolLayoutUnassignedSelection> {
        let adjacent_field = row_context.move_up_field?;
        let moved_field_offset = adjacent_field
            .offset_in_bytes
            .checked_add(row_context.size_in_bytes)?;

        Self::set_field_static_offset(draft, adjacent_field.field_position, moved_field_offset)
            .then(|| SymbolLayoutUnassignedSelection::new(adjacent_field.offset_in_bytes, row_context.size_in_bytes))
    }

    pub fn move_unassigned_span_down(
        draft: &mut SymbolLayoutEditDraft,
        row_context: SymbolLayoutUnassignedRowContext,
    ) -> Option<SymbolLayoutUnassignedSelection> {
        let adjacent_field = row_context.move_down_field?;
        let moved_unassigned_offset = row_context
            .offset_in_bytes
            .checked_add(adjacent_field.size_in_bytes)?;

        Self::set_field_static_offset(draft, adjacent_field.field_position, row_context.offset_in_bytes)
            .then(|| SymbolLayoutUnassignedSelection::new(moved_unassigned_offset, row_context.size_in_bytes))
    }

    pub fn build_unassigned_row_contexts(
        offset_in_bytes: u64,
        size_in_bytes: u64,
        split_offsets: &BTreeSet<u64>,
        move_up_field: Option<SymbolLayoutUnassignedAdjacentField>,
        move_down_field: Option<SymbolLayoutUnassignedAdjacentField>,
    ) -> Vec<SymbolLayoutUnassignedRowContext> {
        if size_in_bytes == 0 {
            return Vec::new();
        }

        let Some(end_offset_in_bytes) = offset_in_bytes.checked_add(size_in_bytes) else {
            return Vec::new();
        };
        let mut row_boundaries = Vec::new();

        row_boundaries.push(offset_in_bytes);
        row_boundaries.extend(
            split_offsets
                .iter()
                .copied()
                .filter(|split_offset| *split_offset > offset_in_bytes && *split_offset < end_offset_in_bytes),
        );
        row_boundaries.push(end_offset_in_bytes);

        row_boundaries
            .windows(2)
            .enumerate()
            .filter_map(|(segment_position, row_boundary_pair)| {
                let row_offset = *row_boundary_pair.first()?;
                let row_end = *row_boundary_pair.get(1)?;
                let row_size = row_end.checked_sub(row_offset)?;
                if row_size == 0 {
                    return None;
                }

                let previous_span = (segment_position > 0).then(|| {
                    let previous_offset = row_boundaries[segment_position - 1];
                    SymbolLayoutUnassignedSelection::new(previous_offset, row_offset.saturating_sub(previous_offset))
                });
                let next_span = (segment_position + 2 < row_boundaries.len()).then(|| {
                    let next_end = row_boundaries[segment_position + 2];
                    SymbolLayoutUnassignedSelection::new(row_end, next_end.saturating_sub(row_end))
                });
                let merge_above_span = previous_span.as_ref().map(|previous_span| {
                    SymbolLayoutUnassignedSelection::new(previous_span.get_offset_in_bytes(), previous_span.get_size_in_bytes().saturating_add(row_size))
                });
                let merge_below_span = next_span
                    .as_ref()
                    .map(|next_span| SymbolLayoutUnassignedSelection::new(row_offset, row_size.saturating_add(next_span.get_size_in_bytes())));

                Some(SymbolLayoutUnassignedRowContext {
                    offset_in_bytes: row_offset,
                    size_in_bytes: row_size,
                    move_up_field: (segment_position == 0).then_some(move_up_field).flatten(),
                    move_down_field: (segment_position + 2 == row_boundaries.len())
                        .then_some(move_down_field)
                        .flatten(),
                    move_up_unassigned_span: previous_span,
                    move_down_unassigned_span: next_span,
                    merge_above_span,
                    merge_below_span,
                })
            })
            .collect()
    }
}
