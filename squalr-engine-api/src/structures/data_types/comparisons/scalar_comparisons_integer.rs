use crate::structures::data_types::comparisons::scalar_comparable::{ScalarCompareFnDelta, ScalarCompareFnImmediate, ScalarCompareFnRelative};
use crate::structures::scanning::parameters::mapped::mapped_scan_parameters::MappedScanParameters;
use num_traits::{WrappingAdd, WrappingSub};
use std::ops::{Add, Sub};
use std::ptr;

pub struct ScalarComparisonsInteger {}

impl ScalarComparisonsInteger {
    pub fn get_compare_equal<PrimitiveType: PartialEq + 'static>(scan_parameters: &MappedScanParameters) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = scan_parameters.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) };

        Some(Box::new(move |current_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };

            current_value == immediate_value
        }))
    }

    pub fn get_compare_not_equal<PrimitiveType: PartialEq + 'static>(scan_parameters: &MappedScanParameters) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = scan_parameters.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) };

        Some(Box::new(move |current_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };

            current_value != immediate_value
        }))
    }

    pub fn get_compare_greater_than<PrimitiveType: PartialOrd + 'static>(scan_parameters: &MappedScanParameters) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = scan_parameters.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) };

        Some(Box::new(move |current_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };

            current_value > immediate_value
        }))
    }

    pub fn get_compare_greater_than_or_equal<PrimitiveType: PartialOrd + 'static>(scan_parameters: &MappedScanParameters) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = scan_parameters.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) };

        Some(Box::new(move |current_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };

            current_value >= immediate_value
        }))
    }

    pub fn get_compare_less_than<PrimitiveType: PartialOrd + 'static>(scan_parameters: &MappedScanParameters) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = scan_parameters.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) };

        Some(Box::new(move |current_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };

            current_value < immediate_value
        }))
    }

    pub fn get_compare_less_than_or_equal<PrimitiveType: PartialOrd + 'static>(scan_parameters: &MappedScanParameters) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = scan_parameters.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) };

        Some(Box::new(move |current_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };

            current_value <= immediate_value
        }))
    }

    pub fn get_compare_changed<PrimitiveType: PartialEq + 'static>(_scan_parameters: &MappedScanParameters) -> Option<ScalarCompareFnRelative> {
        Some(Box::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };
            let previous_value = unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) };

            current_value != previous_value
        }))
    }

    pub fn get_compare_unchanged<PrimitiveType: PartialEq + 'static>(_scan_parameters: &MappedScanParameters) -> Option<ScalarCompareFnRelative> {
        Some(Box::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };
            let previous_value = unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) };

            current_value == previous_value
        }))
    }

    pub fn get_compare_increased<PrimitiveType: PartialOrd + 'static>(_scan_parameters: &MappedScanParameters) -> Option<ScalarCompareFnRelative> {
        Some(Box::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };
            let previous_value = unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) };

            current_value > previous_value
        }))
    }

    pub fn get_compare_decreased<PrimitiveType: PartialOrd + 'static>(_scan_parameters: &MappedScanParameters) -> Option<ScalarCompareFnRelative> {
        Some(Box::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };
            let previous_value = unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) };

            current_value < previous_value
        }))
    }

    pub fn get_compare_increased_by<PrimitiveType: Copy + PartialEq + Add<Output = PrimitiveType> + WrappingAdd + 'static>(
        scan_parameters: &MappedScanParameters
    ) -> Option<ScalarCompareFnDelta> {
        let immediate_value = scan_parameters.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value: PrimitiveType = unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) };

        Some(Box::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };
            let previous_value = unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) };
            let target_value = previous_value.wrapping_add(&delta_value);

            current_value == target_value
        }))
    }

    pub fn get_compare_decreased_by<PrimitiveType: Copy + PartialEq + Sub<Output = PrimitiveType> + WrappingSub + 'static>(
        scan_parameters: &MappedScanParameters
    ) -> Option<ScalarCompareFnDelta> {
        let immediate_value = scan_parameters.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value: PrimitiveType = unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) };

        Some(Box::new(move |current_value_ptr, previous_value_ptr| {
            let current_value = unsafe { ptr::read_unaligned(current_value_ptr as *const PrimitiveType) };
            let previous_value = unsafe { ptr::read_unaligned(previous_value_ptr as *const PrimitiveType) };
            let target_value = previous_value.wrapping_sub(&delta_value);

            current_value == target_value
        }))
    }
}
