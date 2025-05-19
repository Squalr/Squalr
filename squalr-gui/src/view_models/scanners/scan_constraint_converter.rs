use crate::ScanConstraintTypeView;
use slint_mvvm::convert_from_view_data::ConvertFromViewData;
use slint_mvvm::convert_to_view_data::ConvertToViewData;
use squalr_engine_api::structures::scanning::comparisons::{
    scan_compare_type::ScanCompareType, scan_compare_type_delta::ScanCompareTypeDelta, scan_compare_type_immediate::ScanCompareTypeImmediate,
    scan_compare_type_relative::ScanCompareTypeRelative,
};

pub struct ScanConstraintConverter {}

impl ScanConstraintConverter {
    pub fn new() -> Self {
        Self {}
    }
}

impl ConvertToViewData<ScanCompareType, ScanConstraintTypeView> for ScanConstraintConverter {
    fn convert_collection(
        &self,
        scan_compare_type_list: &Vec<ScanCompareType>,
    ) -> Vec<ScanConstraintTypeView> {
        return scan_compare_type_list
            .into_iter()
            .map(|item| self.convert_to_view_data(item))
            .collect();
    }

    fn convert_to_view_data(
        &self,
        scan_compare_type: &ScanCompareType,
    ) -> ScanConstraintTypeView {
        match scan_compare_type {
            ScanCompareType::Immediate(scan_compare_type_immediate) => match scan_compare_type_immediate {
                ScanCompareTypeImmediate::Equal => ScanConstraintTypeView::Equal,
                ScanCompareTypeImmediate::NotEqual => ScanConstraintTypeView::NotEqual,
                ScanCompareTypeImmediate::GreaterThan => ScanConstraintTypeView::GreaterThan,
                ScanCompareTypeImmediate::GreaterThanOrEqual => ScanConstraintTypeView::GreaterThanOrEqual,
                ScanCompareTypeImmediate::LessThan => ScanConstraintTypeView::LessThan,
                ScanCompareTypeImmediate::LessThanOrEqual => ScanConstraintTypeView::LessThanOrEqual,
            },
            ScanCompareType::Relative(scan_compare_type_relative) => match scan_compare_type_relative {
                ScanCompareTypeRelative::Changed => ScanConstraintTypeView::Changed,
                ScanCompareTypeRelative::Unchanged => ScanConstraintTypeView::Unchanged,
                ScanCompareTypeRelative::Increased => ScanConstraintTypeView::Increased,
                ScanCompareTypeRelative::Decreased => ScanConstraintTypeView::Decreased,
            },
            ScanCompareType::Delta(scan_compare_type_delta) => match scan_compare_type_delta {
                ScanCompareTypeDelta::IncreasedByX => ScanConstraintTypeView::IncreasedByX,
                ScanCompareTypeDelta::DecreasedByX => ScanConstraintTypeView::DecreasedByX,
                ScanCompareTypeDelta::MultipliedByX => ScanConstraintTypeView::MultipliedByX,
                ScanCompareTypeDelta::DividedByX => ScanConstraintTypeView::DividedByX,
                ScanCompareTypeDelta::ModuloByX => ScanConstraintTypeView::ModuloByX,
                ScanCompareTypeDelta::ShiftLeftByX => ScanConstraintTypeView::ShiftLeftByX,
                ScanCompareTypeDelta::ShiftRightByX => ScanConstraintTypeView::ShiftRightByX,
                ScanCompareTypeDelta::LogicalAndByX => ScanConstraintTypeView::LogicalAndByX,
                ScanCompareTypeDelta::LogicalOrByX => ScanConstraintTypeView::LogicalOrByX,
                ScanCompareTypeDelta::LogicalXorByX => ScanConstraintTypeView::LogicalXorByX,
            },
        }
    }
}

impl ConvertFromViewData<ScanCompareType, ScanConstraintTypeView> for ScanConstraintConverter {
    fn convert_from_view_data(
        &self,
        scan_compare_type: &ScanConstraintTypeView,
    ) -> ScanCompareType {
        match scan_compare_type {
            ScanConstraintTypeView::Equal => ScanCompareType::Immediate(ScanCompareTypeImmediate::Equal),
            ScanConstraintTypeView::NotEqual => ScanCompareType::Immediate(ScanCompareTypeImmediate::NotEqual),
            ScanConstraintTypeView::GreaterThan => ScanCompareType::Immediate(ScanCompareTypeImmediate::GreaterThan),
            ScanConstraintTypeView::GreaterThanOrEqual => ScanCompareType::Immediate(ScanCompareTypeImmediate::GreaterThanOrEqual),
            ScanConstraintTypeView::LessThan => ScanCompareType::Immediate(ScanCompareTypeImmediate::LessThan),
            ScanConstraintTypeView::LessThanOrEqual => ScanCompareType::Immediate(ScanCompareTypeImmediate::LessThanOrEqual),
            ScanConstraintTypeView::Changed => ScanCompareType::Relative(ScanCompareTypeRelative::Changed),
            ScanConstraintTypeView::Unchanged => ScanCompareType::Relative(ScanCompareTypeRelative::Unchanged),
            ScanConstraintTypeView::Increased => ScanCompareType::Relative(ScanCompareTypeRelative::Increased),
            ScanConstraintTypeView::Decreased => ScanCompareType::Relative(ScanCompareTypeRelative::Decreased),
            ScanConstraintTypeView::IncreasedByX => ScanCompareType::Delta(ScanCompareTypeDelta::IncreasedByX),
            ScanConstraintTypeView::DecreasedByX => ScanCompareType::Delta(ScanCompareTypeDelta::DecreasedByX),
            ScanConstraintTypeView::MultipliedByX => ScanCompareType::Delta(ScanCompareTypeDelta::MultipliedByX),
            ScanConstraintTypeView::DividedByX => ScanCompareType::Delta(ScanCompareTypeDelta::DividedByX),
            ScanConstraintTypeView::ModuloByX => ScanCompareType::Delta(ScanCompareTypeDelta::ModuloByX),
            ScanConstraintTypeView::ShiftLeftByX => ScanCompareType::Delta(ScanCompareTypeDelta::ShiftLeftByX),
            ScanConstraintTypeView::ShiftRightByX => ScanCompareType::Delta(ScanCompareTypeDelta::ShiftRightByX),
            ScanConstraintTypeView::LogicalAndByX => ScanCompareType::Delta(ScanCompareTypeDelta::LogicalAndByX),
            ScanConstraintTypeView::LogicalOrByX => ScanCompareType::Delta(ScanCompareTypeDelta::LogicalOrByX),
            ScanConstraintTypeView::LogicalXorByX => ScanCompareType::Delta(ScanCompareTypeDelta::LogicalXorByX),
        }
    }
}
