use crate::ScanConstraintTypeView;
use slint_mvvm::view_data_converter::ViewDataConverter;
use squalr_engine_scanning::scanners::parameters::scan_compare_type::ScanCompareType;

pub struct ScanConstraintConverter;

impl ScanConstraintConverter {
    pub fn new() -> Self {
        Self {}
    }
}

impl ViewDataConverter<ScanCompareType, ScanConstraintTypeView> for ScanConstraintConverter {
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
            ScanCompareType::Equal => ScanConstraintTypeView::Equal,
            ScanCompareType::NotEqual => ScanConstraintTypeView::NotEqual,
            ScanCompareType::Changed => ScanConstraintTypeView::Changed,
            ScanCompareType::Unchanged => ScanConstraintTypeView::Unchanged,
            ScanCompareType::Increased => ScanConstraintTypeView::Increased,
            ScanCompareType::Decreased => ScanConstraintTypeView::Decreased,
            ScanCompareType::IncreasedByX => ScanConstraintTypeView::IncreasedBy,
            ScanCompareType::DecreasedByX => ScanConstraintTypeView::DecreasedBy,
            ScanCompareType::GreaterThan => ScanConstraintTypeView::GreaterThan,
            ScanCompareType::GreaterThanOrEqual => ScanConstraintTypeView::GreaterThanOrEqualTo,
            ScanCompareType::LessThan => ScanConstraintTypeView::LessThan,
            ScanCompareType::LessThanOrEqual => ScanConstraintTypeView::LessThanOrEqualTo,
        }
    }

    fn convert_from_view_data(
        &self,
        scan_compare_type: &ScanConstraintTypeView,
    ) -> ScanCompareType {
        match scan_compare_type {
            ScanConstraintTypeView::Equal => ScanCompareType::Equal,
            ScanConstraintTypeView::NotEqual => ScanCompareType::NotEqual,
            ScanConstraintTypeView::Changed => ScanCompareType::Changed,
            ScanConstraintTypeView::Unchanged => ScanCompareType::Unchanged,
            ScanConstraintTypeView::Increased => ScanCompareType::Increased,
            ScanConstraintTypeView::Decreased => ScanCompareType::Decreased,
            ScanConstraintTypeView::IncreasedBy => ScanCompareType::IncreasedByX,
            ScanConstraintTypeView::DecreasedBy => ScanCompareType::DecreasedByX,
            ScanConstraintTypeView::GreaterThan => ScanCompareType::GreaterThan,
            ScanConstraintTypeView::GreaterThanOrEqualTo => ScanCompareType::GreaterThanOrEqual,
            ScanConstraintTypeView::LessThan => ScanCompareType::LessThan,
            ScanConstraintTypeView::LessThanOrEqualTo => ScanCompareType::LessThanOrEqual,
        }
    }
}
