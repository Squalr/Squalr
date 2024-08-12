use crate::scanners::comparers::standard::snapshot_element_scanner_standard::SnapshotElementRangeScannerStandard;
use crate::scanners::comparers::snapshot_element_run_length_encoder::SnapshotElementRunLengthEncoder;
use crate::scanners::constraints::scan_constraints::ScanConstraints;
use crate::snapshots::snapshot_element_range::SnapshotElementRange;

pub struct SnapshotElementRangeScannerIterative<'a> {
    base_scanner: SnapshotElementRangeScannerStandard<'a>,
    element_compare: Option<Box<dyn Fn() -> bool + 'a>>,
    current_value_pointer: *const u8,
    previous_value_pointer: *const u8,
}

impl<'a> SnapshotElementRangeScannerIterative<'a> {
    fn initialize(&mut self, element_range: &'a SnapshotElementRange<'a>, constraints: &'a ScanConstraints) {
        self.base_scanner.initialize(element_range, constraints);

        if let Some(root_constraint) = constraints.get_root_constraint() {
            let scan_constraint = root_constraint.borrow();
            self.element_compare = Some(self.base_scanner.build_compare_actions(&scan_constraint));
        }

        self.initialize_pointers(element_range);
    }

    fn initialize_pointers(&mut self, element_range: &SnapshotElementRange) {
        let current_values = element_range.parent_region.borrow().current_values.as_ptr();
        let previous_values = element_range.parent_region.borrow().previous_values.as_ptr();

        unsafe {
            self.current_value_pointer = current_values.add(element_range.region_offset);
            self.previous_value_pointer = previous_values.add(element_range.region_offset);
        }
    }

    pub fn scan_region(
        &mut self,
        element_range: &'a SnapshotElementRange<'a>,
        constraints: &'a ScanConstraints,
    ) -> Vec<SnapshotElementRange<'a>> {
        self.initialize(element_range, constraints);
    
        let aligned_element_count = element_range.get_aligned_element_count(constraints.get_alignment());
        let encoder: &mut SnapshotElementRunLengthEncoder<'a> = self.base_scanner.get_run_length_encoder();
    
        for _ in 0..aligned_element_count {
            let should_encode = (self.element_compare.as_ref().unwrap())();
    
            if should_encode {
                encoder.encode_range(constraints.get_alignment() as usize);
            } else {
                encoder.finalize_current_encode_unchecked(constraints.get_alignment() as usize);
            }
    
            unsafe {
                self.current_value_pointer = self.current_value_pointer.add(constraints.get_alignment() as usize);
                self.previous_value_pointer = self.previous_value_pointer.add(constraints.get_alignment() as usize);
            }
        }
    
        encoder.finalize_current_encode_unchecked(0);
        return encoder.get_collected_regions().clone();
    }
}    
