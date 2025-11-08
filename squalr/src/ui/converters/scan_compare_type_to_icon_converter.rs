use crate::ui::icon_library::IconLibrary;
use epaint::TextureHandle;
use squalr_engine_api::structures::scanning::comparisons::{
    scan_compare_type::ScanCompareType, scan_compare_type_delta::ScanCompareTypeDelta, scan_compare_type_immediate::ScanCompareTypeImmediate,
    scan_compare_type_relative::ScanCompareTypeRelative,
};

pub struct ScanCompareTypeToIconConverter {}

impl ScanCompareTypeToIconConverter {
    pub fn convert_scan_compare_type_to_icon(
        scan_compare_type: &ScanCompareType,
        icon_library: &IconLibrary,
    ) -> TextureHandle {
        match scan_compare_type {
            ScanCompareType::Delta(scan_compare_type_delta) => match scan_compare_type_delta {
                ScanCompareTypeDelta::IncreasedByX => icon_library.icon_handle_scan_delta_increased_by_x.clone(),
                ScanCompareTypeDelta::DecreasedByX => icon_library.icon_handle_scan_delta_decreased_by_x.clone(),
                ScanCompareTypeDelta::MultipliedByX => icon_library.icon_handle_scan_delta_multiplied_by_x.clone(),
                ScanCompareTypeDelta::DividedByX => icon_library.icon_handle_scan_delta_divided_by_x.clone(),
                ScanCompareTypeDelta::ModuloByX => icon_library.icon_handle_scan_delta_modulo_by_x.clone(),
                ScanCompareTypeDelta::ShiftLeftByX => icon_library.icon_handle_scan_delta_shift_left_by_x.clone(),
                ScanCompareTypeDelta::ShiftRightByX => icon_library.icon_handle_scan_delta_shift_right_by_x.clone(),
                ScanCompareTypeDelta::LogicalAndByX => icon_library.icon_handle_scan_delta_logical_and_by_x.clone(),
                ScanCompareTypeDelta::LogicalOrByX => icon_library.icon_handle_scan_delta_logical_or_by_x.clone(),
                ScanCompareTypeDelta::LogicalXorByX => icon_library.icon_handle_scan_delta_logical_xor_by_x.clone(),
            },
            ScanCompareType::Immediate(scan_compare_type_immediate) => match scan_compare_type_immediate {
                ScanCompareTypeImmediate::Equal => icon_library.icon_handle_scan_immediate_equal.clone(),
                ScanCompareTypeImmediate::NotEqual => icon_library.icon_handle_scan_immediate_not_equal.clone(),
                ScanCompareTypeImmediate::GreaterThan => icon_library.icon_handle_scan_immediate_greater_than.clone(),
                ScanCompareTypeImmediate::GreaterThanOrEqual => icon_library
                    .icon_handle_scan_immediate_greater_than_or_equal
                    .clone(),
                ScanCompareTypeImmediate::LessThan => icon_library.icon_handle_scan_immediate_less_than.clone(),
                ScanCompareTypeImmediate::LessThanOrEqual => icon_library
                    .icon_handle_scan_immediate_less_than_or_equal
                    .clone(),
            },
            ScanCompareType::Relative(scan_compare_type_relative) => match scan_compare_type_relative {
                ScanCompareTypeRelative::Changed => icon_library.icon_handle_scan_relative_changed.clone(),
                ScanCompareTypeRelative::Unchanged => icon_library.icon_handle_scan_relative_unchanged.clone(),
                ScanCompareTypeRelative::Increased => icon_library.icon_handle_scan_relative_increased.clone(),
                ScanCompareTypeRelative::Decreased => icon_library.icon_handle_scan_relative_decreased.clone(),
            },
        }
    }
}
