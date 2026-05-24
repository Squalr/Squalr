use crate::structures::structs::symbolic_struct_definition::SymbolicLayoutKind;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SymbolLayoutMoveDirection {
    Up,
    Down,
}

pub trait SymbolLayoutDraftMutationTarget {
    fn get_layout_kind(&self) -> SymbolicLayoutKind;
    fn get_field_count(&self) -> usize;
    fn get_field_name(
        &self,
        field_position: usize,
    ) -> Option<&str>;
    fn set_field_static_offset(
        &mut self,
        field_position: usize,
        offset_in_bytes: u64,
    ) -> bool;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SymbolLayoutFieldSpan {
    pub field_position: usize,
    pub offset_in_bytes: u64,
    pub size_in_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolLayoutUnassignedSelection {
    layout_id: Option<String>,
    offset_in_bytes: u64,
    size_in_bytes: u64,
}

impl SymbolLayoutUnassignedSelection {
    pub fn new(
        offset_in_bytes: u64,
        size_in_bytes: u64,
    ) -> Self {
        Self {
            layout_id: None,
            offset_in_bytes,
            size_in_bytes,
        }
    }

    pub fn new_for_layout(
        layout_id: String,
        offset_in_bytes: u64,
        size_in_bytes: u64,
    ) -> Self {
        Self {
            layout_id: Some(layout_id),
            offset_in_bytes,
            size_in_bytes,
        }
    }

    pub fn get_layout_id(&self) -> Option<&str> {
        self.layout_id.as_deref()
    }

    pub fn matches(
        &self,
        layout_id: Option<&str>,
        offset_in_bytes: u64,
        size_in_bytes: u64,
    ) -> bool {
        self.get_layout_id() == layout_id && self.offset_in_bytes == offset_in_bytes && self.size_in_bytes == size_in_bytes
    }

    pub fn get_offset_in_bytes(&self) -> u64 {
        self.offset_in_bytes
    }

    pub fn get_size_in_bytes(&self) -> u64 {
        self.size_in_bytes
    }
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

    pub fn build_unique_field_name<Draft>(
        draft: &Draft,
        base_name: &str,
    ) -> String
    where
        Draft: SymbolLayoutDraftMutationTarget,
    {
        let trimmed_base_name = base_name.trim();
        let base_name = if trimmed_base_name.is_empty() {
            if draft.get_layout_kind().is_union() { "variant" } else { "field" }
        } else {
            trimmed_base_name
        };
        if !(0..draft.get_field_count()).any(|field_position| {
            draft
                .get_field_name(field_position)
                .is_some_and(|field_name| field_name.trim() == base_name)
        }) {
            return base_name.to_string();
        }

        let mut suffix_index = 2_u64;
        loop {
            let candidate_name = format!("{}_{}", base_name, suffix_index);
            if !(0..draft.get_field_count()).any(|field_position| {
                draft
                    .get_field_name(field_position)
                    .is_some_and(|field_name| field_name.trim() == candidate_name)
            }) {
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

    fn resolve_adjacent_field_span(
        field_spans: &[SymbolLayoutFieldSpan],
        current_field_span: SymbolLayoutFieldSpan,
        direction: SymbolLayoutMoveDirection,
    ) -> Option<SymbolLayoutFieldSpan> {
        let current_sort_key = (current_field_span.offset_in_bytes, current_field_span.field_position);
        match direction {
            SymbolLayoutMoveDirection::Up => field_spans
                .iter()
                .copied()
                .filter(|field_span| field_span.field_position != current_field_span.field_position)
                .filter(|field_span| (field_span.offset_in_bytes, field_span.field_position) < current_sort_key)
                .max_by_key(|field_span| (field_span.offset_in_bytes, field_span.field_position)),
            SymbolLayoutMoveDirection::Down => field_spans
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

    pub fn move_struct_field_up<Draft>(
        draft: &mut Draft,
        field_spans: &[SymbolLayoutFieldSpan],
        split_offsets: &BTreeSet<u64>,
        field_index: usize,
    ) -> bool
    where
        Draft: SymbolLayoutDraftMutationTarget,
    {
        let Some(current_field_span) = Self::resolve_field_span_by_position(field_spans, field_index) else {
            return false;
        };
        let previous_field_span = Self::resolve_adjacent_field_span(field_spans, current_field_span, SymbolLayoutMoveDirection::Up);

        if let Some(previous_unassigned_row) = Self::resolve_unassigned_row_before_field(current_field_span, previous_field_span, split_offsets) {
            return draft.set_field_static_offset(field_index, previous_unassigned_row.get_offset_in_bytes());
        }

        let Some(previous_field_span) = previous_field_span else {
            return false;
        };
        let moved_current_offset = previous_field_span.offset_in_bytes;
        let moved_previous_offset = previous_field_span
            .offset_in_bytes
            .saturating_add(current_field_span.size_in_bytes);
        draft.set_field_static_offset(field_index, moved_current_offset)
            && draft.set_field_static_offset(previous_field_span.field_position, moved_previous_offset)
    }

    pub fn split_offset_to_preserve_field_move_up(
        field_spans: &[SymbolLayoutFieldSpan],
        layout_size_in_bytes: u64,
        split_offsets: &BTreeSet<u64>,
        field_index: usize,
    ) -> Option<u64> {
        let current_field_span = Self::resolve_field_span_by_position(field_spans, field_index)?;
        let previous_field_span = Self::resolve_adjacent_field_span(field_spans, current_field_span, SymbolLayoutMoveDirection::Up);
        Self::resolve_unassigned_row_before_field(current_field_span, previous_field_span, split_offsets)?;
        let next_field_span = Self::resolve_adjacent_field_span(field_spans, current_field_span, SymbolLayoutMoveDirection::Down);
        Self::resolve_unassigned_row_after_field(current_field_span, next_field_span, layout_size_in_bytes, split_offsets)?;

        current_field_span
            .offset_in_bytes
            .checked_add(current_field_span.size_in_bytes)
    }

    pub fn move_struct_field_down<Draft>(
        draft: &mut Draft,
        field_spans: &[SymbolLayoutFieldSpan],
        layout_size_in_bytes: u64,
        split_offsets: &BTreeSet<u64>,
        field_index: usize,
    ) -> bool
    where
        Draft: SymbolLayoutDraftMutationTarget,
    {
        let Some(current_field_span) = Self::resolve_field_span_by_position(field_spans, field_index) else {
            return false;
        };
        let next_field_span = Self::resolve_adjacent_field_span(field_spans, current_field_span, SymbolLayoutMoveDirection::Down);

        if let Some(next_unassigned_row) = Self::resolve_unassigned_row_after_field(current_field_span, next_field_span, layout_size_in_bytes, split_offsets) {
            return draft.set_field_static_offset(
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
        draft.set_field_static_offset(next_field_span.field_position, moved_next_offset) && draft.set_field_static_offset(field_index, moved_current_offset)
    }

    pub fn split_offset_to_preserve_field_move_down(
        field_spans: &[SymbolLayoutFieldSpan],
        layout_size_in_bytes: u64,
        split_offsets: &BTreeSet<u64>,
        field_index: usize,
    ) -> Option<u64> {
        let current_field_span = Self::resolve_field_span_by_position(field_spans, field_index)?;
        let next_field_span = Self::resolve_adjacent_field_span(field_spans, current_field_span, SymbolLayoutMoveDirection::Down);
        Self::resolve_unassigned_row_after_field(current_field_span, next_field_span, layout_size_in_bytes, split_offsets)?;
        let previous_field_span = Self::resolve_adjacent_field_span(field_spans, current_field_span, SymbolLayoutMoveDirection::Up);
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
        let previous_field_span = Self::resolve_adjacent_field_span(field_spans, current_field_span, SymbolLayoutMoveDirection::Up);

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
        let next_field_span = Self::resolve_adjacent_field_span(field_spans, current_field_span, SymbolLayoutMoveDirection::Down);

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

    pub fn move_unassigned_span_up<Draft>(
        draft: &mut Draft,
        row_context: SymbolLayoutUnassignedRowContext,
    ) -> Option<SymbolLayoutUnassignedSelection>
    where
        Draft: SymbolLayoutDraftMutationTarget,
    {
        let adjacent_field = row_context.move_up_field?;
        let moved_field_offset = adjacent_field
            .offset_in_bytes
            .checked_add(row_context.size_in_bytes)?;

        draft
            .set_field_static_offset(adjacent_field.field_position, moved_field_offset)
            .then(|| SymbolLayoutUnassignedSelection::new(adjacent_field.offset_in_bytes, row_context.size_in_bytes))
    }

    pub fn move_unassigned_span_down<Draft>(
        draft: &mut Draft,
        row_context: SymbolLayoutUnassignedRowContext,
    ) -> Option<SymbolLayoutUnassignedSelection>
    where
        Draft: SymbolLayoutDraftMutationTarget,
    {
        let adjacent_field = row_context.move_down_field?;
        let moved_unassigned_offset = row_context
            .offset_in_bytes
            .checked_add(adjacent_field.size_in_bytes)?;

        draft
            .set_field_static_offset(adjacent_field.field_position, row_context.offset_in_bytes)
            .then(|| SymbolLayoutUnassignedSelection::new(moved_unassigned_offset, row_context.size_in_bytes))
    }

    pub fn split_offset_to_preserve_unassigned_move_up(updated_unassigned_selection: &SymbolLayoutUnassignedSelection) -> Option<u64> {
        let split_offset_in_bytes = updated_unassigned_selection.get_offset_in_bytes();

        (split_offset_in_bytes > 0).then_some(split_offset_in_bytes)
    }

    pub fn split_offset_to_preserve_unassigned_move_down(updated_unassigned_selection: &SymbolLayoutUnassignedSelection) -> Option<u64> {
        updated_unassigned_selection
            .get_offset_in_bytes()
            .checked_add(updated_unassigned_selection.get_size_in_bytes())
    }

    pub fn split_offsets_to_preserve_unassigned_field(
        field_span: SymbolLayoutFieldSpan,
        layout_size_in_bytes: u64,
    ) -> Vec<u64> {
        let mut split_offsets = Vec::with_capacity(2);

        if field_span.offset_in_bytes > 0 {
            split_offsets.push(field_span.offset_in_bytes);
        }

        if let Some(field_end_offset_in_bytes) = field_span.offset_in_bytes.checked_add(field_span.size_in_bytes)
            && field_end_offset_in_bytes < layout_size_in_bytes
        {
            split_offsets.push(field_end_offset_in_bytes);
        }

        split_offsets
    }

    pub fn field_offset_to_preserve_after_unassign(
        field_spans: &[SymbolLayoutFieldSpan],
        field_index: usize,
    ) -> Option<(usize, u64)> {
        let current_field_span = Self::resolve_field_span_by_position(field_spans, field_index)?;
        let next_field_span = Self::resolve_adjacent_field_span(field_spans, current_field_span, SymbolLayoutMoveDirection::Down)?;
        let preserved_field_index = if next_field_span.field_position > field_index {
            next_field_span.field_position.saturating_sub(1)
        } else {
            next_field_span.field_position
        };

        Some((preserved_field_index, next_field_span.offset_in_bytes))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    struct TestDraft {
        layout_kind: SymbolicLayoutKind,
        fields: Vec<TestField>,
    }

    #[derive(Clone, Debug)]
    struct TestField {
        field_name: String,
        static_offset_in_bytes: u64,
    }

    impl SymbolLayoutDraftMutationTarget for TestDraft {
        fn get_layout_kind(&self) -> SymbolicLayoutKind {
            self.layout_kind
        }

        fn get_field_count(&self) -> usize {
            self.fields.len()
        }

        fn get_field_name(
            &self,
            field_position: usize,
        ) -> Option<&str> {
            self.fields
                .get(field_position)
                .map(|field| field.field_name.as_str())
        }

        fn set_field_static_offset(
            &mut self,
            field_position: usize,
            offset_in_bytes: u64,
        ) -> bool {
            let Some(field) = self.fields.get_mut(field_position) else {
                return false;
            };

            field.static_offset_in_bytes = offset_in_bytes;
            true
        }
    }

    #[test]
    fn move_unassigned_span_down_preserves_next_unassigned_boundary() {
        let mut draft = TestDraft {
            layout_kind: SymbolicLayoutKind::Struct,
            fields: vec![
                TestField {
                    field_name: String::from("health"),
                    static_offset_in_bytes: 0,
                },
                TestField {
                    field_name: String::from("mana"),
                    static_offset_in_bytes: 16,
                },
            ],
        };
        let row_context = SymbolLayoutUnassignedRowContext {
            offset_in_bytes: 4,
            size_in_bytes: 12,
            move_up_field: None,
            move_down_field: Some(SymbolLayoutUnassignedAdjacentField {
                field_position: 1,
                offset_in_bytes: 16,
                size_in_bytes: 4,
            }),
            move_up_unassigned_span: None,
            move_down_unassigned_span: None,
            merge_above_span: None,
            merge_below_span: None,
        };
        let updated_unassigned_selection = SymbolLayoutDraftOps::move_unassigned_span_down(&mut draft, row_context).expect("Expected span to move.");

        assert_eq!(draft.fields[1].static_offset_in_bytes, 4);
        assert_eq!(updated_unassigned_selection.get_offset_in_bytes(), 8);
        assert_eq!(updated_unassigned_selection.get_size_in_bytes(), 12);
        assert_eq!(
            SymbolLayoutDraftOps::split_offset_to_preserve_unassigned_move_down(&updated_unassigned_selection),
            Some(20)
        );
    }

    #[test]
    fn build_unique_field_name_uses_union_variant_base() {
        let draft = TestDraft {
            layout_kind: SymbolicLayoutKind::Union,
            fields: vec![TestField {
                field_name: String::from("variant"),
                static_offset_in_bytes: 0,
            }],
        };

        assert_eq!(SymbolLayoutDraftOps::build_unique_field_name(&draft, ""), "variant_2");
    }

    #[test]
    fn split_offsets_to_preserve_unassigned_field_keeps_middle_field_boundaries() {
        let field_span = SymbolLayoutFieldSpan {
            field_position: 1,
            offset_in_bytes: 4,
            size_in_bytes: 4,
        };

        assert_eq!(SymbolLayoutDraftOps::split_offsets_to_preserve_unassigned_field(field_span, 12), vec![4, 8]);
    }

    #[test]
    fn split_offsets_to_preserve_unassigned_field_omits_layout_edges() {
        let field_span = SymbolLayoutFieldSpan {
            field_position: 0,
            offset_in_bytes: 0,
            size_in_bytes: 4,
        };

        assert_eq!(
            SymbolLayoutDraftOps::split_offsets_to_preserve_unassigned_field(field_span, 4),
            Vec::<u64>::new()
        );
    }

    #[test]
    fn field_offset_to_preserve_after_unassign_keeps_next_field_in_place() {
        let field_spans = vec![
            SymbolLayoutFieldSpan {
                field_position: 0,
                offset_in_bytes: 0,
                size_in_bytes: 4,
            },
            SymbolLayoutFieldSpan {
                field_position: 1,
                offset_in_bytes: 4,
                size_in_bytes: 4,
            },
            SymbolLayoutFieldSpan {
                field_position: 2,
                offset_in_bytes: 8,
                size_in_bytes: 4,
            },
        ];

        assert_eq!(SymbolLayoutDraftOps::field_offset_to_preserve_after_unassign(&field_spans, 1), Some((1, 8)));
    }

    #[test]
    fn field_offset_to_preserve_after_unassign_handles_render_order_different_from_draft_order() {
        let field_spans = vec![
            SymbolLayoutFieldSpan {
                field_position: 0,
                offset_in_bytes: 8,
                size_in_bytes: 4,
            },
            SymbolLayoutFieldSpan {
                field_position: 1,
                offset_in_bytes: 0,
                size_in_bytes: 4,
            },
        ];

        assert_eq!(SymbolLayoutDraftOps::field_offset_to_preserve_after_unassign(&field_spans, 1), Some((0, 8)));
    }
}
