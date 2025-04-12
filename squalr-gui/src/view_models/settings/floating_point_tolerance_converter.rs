use slint_mvvm::view_data_converter::ViewDataConverter;
use squalr_engine_api::structures::data_types::floating_point_tolerance::FloatingPointTolerance;

use crate::FloatingPointToleranceView;

pub struct FloatingPointToleranceConverter;

impl FloatingPointToleranceConverter {
    pub fn new() -> Self {
        Self {}
    }
}

impl ViewDataConverter<FloatingPointTolerance, FloatingPointToleranceView> for FloatingPointToleranceConverter {
    fn convert_collection(
        &self,
        floating_point_tolerance_list: &Vec<FloatingPointTolerance>,
    ) -> Vec<FloatingPointToleranceView> {
        floating_point_tolerance_list
            .into_iter()
            .map(|item| self.convert_to_view_data(item))
            .collect()
    }

    fn convert_to_view_data(
        &self,
        floating_point_tolerance: &FloatingPointTolerance,
    ) -> FloatingPointToleranceView {
        match floating_point_tolerance {
            FloatingPointTolerance::Tolerance10E1 => FloatingPointToleranceView::Tolerance10e1,
            FloatingPointTolerance::Tolerance10E2 => FloatingPointToleranceView::Tolerance10e2,
            FloatingPointTolerance::Tolerance10E3 => FloatingPointToleranceView::Tolerance10e3,
            FloatingPointTolerance::Tolerance10E4 => FloatingPointToleranceView::Tolerance10e4,
            FloatingPointTolerance::Tolerance10E5 => FloatingPointToleranceView::Tolerance10e5,
            FloatingPointTolerance::ToleranceEpsilon => FloatingPointToleranceView::ToleranceEpsilon,
        }
    }

    fn convert_from_view_data(
        &self,
        floating_point_tolerance_view: &FloatingPointToleranceView,
    ) -> FloatingPointTolerance {
        match floating_point_tolerance_view {
            FloatingPointToleranceView::Tolerance10e1 => FloatingPointTolerance::Tolerance10E1,
            FloatingPointToleranceView::Tolerance10e2 => FloatingPointTolerance::Tolerance10E2,
            FloatingPointToleranceView::Tolerance10e3 => FloatingPointTolerance::Tolerance10E3,
            FloatingPointToleranceView::Tolerance10e4 => FloatingPointTolerance::Tolerance10E4,
            FloatingPointToleranceView::Tolerance10e5 => FloatingPointTolerance::Tolerance10E5,
            FloatingPointToleranceView::ToleranceEpsilon => FloatingPointTolerance::ToleranceEpsilon,
        }
    }
}
