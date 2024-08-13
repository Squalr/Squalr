use crate::scanners::comparers::standard::snapshot_element_scanner_standard::SnapshotElementRangeScannerStandard;
use crate::scanners::constraints::scan_constraints::ScanConstraints;
use crate::snapshots::snapshot_element_range::SnapshotElementRange;
use std::rc::Rc;

pub struct SnapshotElementRangeScannerIterative {
    base_scanner: SnapshotElementRangeScannerStandard,
    element_compare: Option<Box<dyn Fn() -> bool>>,
    current_value_pointer: *const u8,
    previous_value_pointer: *const u8,
}

impl SnapshotElementRangeScannerIterative {
    pub fn new() -> Self {
        Self {
            base_scanner: SnapshotElementRangeScannerStandard::new(),
            element_compare: None,
            current_value_pointer: std::ptr::null(),
            previous_value_pointer: std::ptr::null(),
        }
    }

    pub fn scan_region(
        &mut self,
        element_range: Rc<SnapshotElementRange>,
        constraints: Rc<ScanConstraints>,
    ) -> Vec<Rc<SnapshotElementRange>> {
        self.initialize(element_range.clone(), constraints.clone());

        let aligned_element_count = element_range.get_aligned_element_count(constraints.get_byte_alignment());
        let root_constraint = constraints.get_root_constraint().as_ref().unwrap();
        let scan_constraint = root_constraint.read().unwrap();

        for _ in 0..aligned_element_count {
            if self.base_scanner.do_compare_action(self.current_value_pointer, self.previous_value_pointer, &scan_constraint) {
                self.base_scanner
                    .get_run_length_encoder()
                    .encode_range(constraints.get_byte_alignment() as usize);
            } else {
                self.base_scanner
                    .get_run_length_encoder()
                    .finalize_current_encode_unchecked(constraints.get_byte_alignment() as usize);
            }

            unsafe {
                self.current_value_pointer = self.current_value_pointer.add(self.base_scanner.get_byte_alignment() as usize);
                self.previous_value_pointer = self.previous_value_pointer.add(self.base_scanner.get_byte_alignment() as usize);
            }
        }

        self.base_scanner
            .get_run_length_encoder()
            .finalize_current_encode_unchecked(0);

        return self.base_scanner.get_run_length_encoder().get_collected_regions().clone();
    }

    fn initialize(&mut self, element_range: Rc<SnapshotElementRange>, constraints: Rc<ScanConstraints>) {
        self.base_scanner.initialize(element_range.clone(), constraints.clone());
        self.initialize_pointers(element_range);
    }

    fn initialize_pointers(&mut self, element_range: Rc<SnapshotElementRange>) {
        let current_values = element_range.parent_region.borrow().current_values.as_ptr();
        let previous_values = element_range.parent_region.borrow().previous_values.as_ptr();

        unsafe {
            self.current_value_pointer = current_values.add(element_range.region_offset);
            self.previous_value_pointer = previous_values.add(element_range.region_offset);
        }
    }
}
