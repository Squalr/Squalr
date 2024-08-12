use crate::scanners::comparers::standard::snapshot_element_scanner_standard::SnapshotElementRangeScannerStandard;
use crate::scanners::constraints::scan_constraints::ScanConstraints;
use crate::snapshots::snapshot_element_range::SnapshotElementRange;

pub struct SnapshotElementRangeScannerIterative<'a> {
    base_scanner: SnapshotElementRangeScannerStandard<'a>,
    element_compare: Option<Box<dyn Fn() -> bool + 'a>>,
    current_value_pointer: *const u8,
    previous_value_pointer: *const u8,
}

impl<'a> SnapshotElementRangeScannerIterative<'a> {
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
        element_range: &SnapshotElementRange,
        constraints: &ScanConstraints,
    ) -> &Vec<SnapshotElementRange<'a>> {
        self.initialize(element_range, constraints);

        let aligned_element_count = element_range.get_aligned_element_count(constraints.get_alignment());

        for _ in 0..aligned_element_count {
            if (self.element_compare.as_ref().unwrap())() {
                self.base_scanner
                    .get_run_length_encoder()
                    .encode_range(constraints.get_alignment() as usize);
            } else {
                self.base_scanner
                    .get_run_length_encoder()
                    .finalize_current_encode_unchecked(constraints.get_alignment() as usize);
            }

            unsafe {
                self.current_value_pointer = self.current_value_pointer.add(constraints.get_alignment() as usize);
                self.previous_value_pointer = self.previous_value_pointer.add(constraints.get_alignment() as usize);
            }
        }

        self.base_scanner
            .get_run_length_encoder()
            .finalize_current_encode_unchecked();

        return self.base_scanner.get_run_length_encoder().get_collected_regions();
    }

    fn initialize(&mut self, element_range: &SnapshotElementRange, constraints: &ScanConstraints) {
        self.base_scanner.initialize(&element_range.clone(), constraints);
        self.element_compare = Some(self.base_scanner.build_compare_actions(constraints));
        self.initialize_pointers(element_range);
    }

    fn initialize_pointers(&mut self, element_range: &SnapshotElementRange) {
        let current_values = element_range.parent_region.current_values.as_ptr();
        let previous_values = element_range.parent_region.previous_values.as_ptr();

        unsafe {
            self.current_value_pointer = current_values.add(element_range.region_offset);
            self.previous_value_pointer = previous_values.add(element_range.region_offset);
        }
    }
}
