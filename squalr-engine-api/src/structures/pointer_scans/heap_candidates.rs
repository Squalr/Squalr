use super::{pointer_scan_candidate::PointerScanCandidate, pointer_scan_heap_slot::PointerScanHeapSlot, pointer_scan_node_type::PointerScanNodeType};
use serde::{Deserialize, Serialize, Serializer};
use std::{ops::Range, sync::OnceLock};

#[derive(Clone, Debug, Deserialize)]
#[serde(from = "Vec<PointerScanCandidate>")]
pub(super) enum HeapCandidates {
    Packed {
        depth: u64,
        first_id: u64,
        slots: Vec<PointerScanHeapSlot>,
        // Only the public accessors that promise a borrowed full record need this view.
        expanded: OnceLock<Vec<PointerScanCandidate>>,
    },
    Explicit(Vec<PointerScanCandidate>),
}

impl HeapCandidates {
    pub fn packed(
        depth: u64,
        first_id: u64,
        slots: Vec<PointerScanHeapSlot>,
    ) -> Self {
        Self::Packed {
            depth,
            first_id,
            slots,
            expanded: OnceLock::new(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Packed { slots, .. } => slots.len(),
            Self::Explicit(records) => records.len(),
        }
    }

    pub fn candidate(
        &self,
        slot_index: usize,
    ) -> PointerScanCandidate {
        match self {
            Self::Packed { first_id, depth, slots, .. } => {
                let slot = slots[slot_index];
                PointerScanCandidate::new(
                    first_id.saturating_add(slot_index as u64),
                    *depth,
                    PointerScanNodeType::Heap,
                    slot.pointer_address,
                    slot.pointer_value,
                    0,
                    0,
                )
            }
            Self::Explicit(records) => records[slot_index].clone(),
        }
    }

    pub fn full_records(&self) -> &Vec<PointerScanCandidate> {
        match self {
            Self::Packed { expanded, .. } => expanded.get_or_init(|| {
                (0..self.len())
                    .map(|slot_index| self.candidate(slot_index))
                    .collect()
            }),
            Self::Explicit(records) => records,
        }
    }

    pub fn sort_by_address(&mut self) {
        // Packed records are created in address order, so their IDs remain tied to their positions.
        if let Self::Explicit(records) = self {
            records.sort_unstable_by_key(PointerScanCandidate::get_pointer_address);
        }
    }

    pub fn address(
        &self,
        slot_index: usize,
    ) -> u64 {
        match self {
            Self::Packed { slots, .. } => slots[slot_index].pointer_address,
            Self::Explicit(records) => records[slot_index].get_pointer_address(),
        }
    }

    pub fn address_range(
        &self,
        lower_bound: u64,
        upper_bound: u64,
    ) -> Range<usize> {
        match self {
            Self::Packed { slots, .. } => {
                slots.partition_point(|slot| slot.pointer_address < lower_bound)..slots.partition_point(|slot| slot.pointer_address <= upper_bound)
            }
            Self::Explicit(records) => {
                records.partition_point(|record| record.get_pointer_address() < lower_bound)
                    ..records.partition_point(|record| record.get_pointer_address() <= upper_bound)
            }
        }
    }
}

impl From<Vec<PointerScanCandidate>> for HeapCandidates {
    fn from(records: Vec<PointerScanCandidate>) -> Self {
        let Some(first) = records.first() else { return Self::Explicit(records) };
        let depth = first.get_discovery_depth();
        let first_id = first.get_candidate_id();
        if records.is_sorted_by_key(PointerScanCandidate::get_pointer_address)
            && records.iter().enumerate().all(|(slot_index, record)| {
                record.get_candidate_id() == first_id.saturating_add(slot_index as u64)
                    && record.get_discovery_depth() == depth
                    && record.get_pointer_scan_node_type() == PointerScanNodeType::Heap
                    && record.get_module_index() == 0
                    && record.get_module_offset() == 0
            })
        {
            Self::packed(
                depth,
                first_id,
                records
                    .into_iter()
                    .map(|record| PointerScanHeapSlot {
                        pointer_address: record.get_pointer_address(),
                        pointer_value: record.get_pointer_value(),
                    })
                    .collect(),
            )
        } else {
            Self::Explicit(records)
        }
    }
}

impl Default for HeapCandidates {
    fn default() -> Self {
        Self::Explicit(Vec::new())
    }
}

impl PartialEq for HeapCandidates {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.len() == other.len() && (0..self.len()).all(|slot_index| self.candidate(slot_index) == other.candidate(slot_index))
    }
}
impl Eq for HeapCandidates {}

impl Serialize for HeapCandidates {
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.collect_seq((0..self.len()).map(|slot_index| self.candidate(slot_index)))
    }
}
