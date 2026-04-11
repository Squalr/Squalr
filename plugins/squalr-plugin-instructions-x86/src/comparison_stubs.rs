macro_rules! impl_data_type_comparison_stubs {
    ($target_type:ty) => {
        impl squalr_engine_api::structures::data_types::comparisons::scalar_comparable::ScalarComparable for $target_type {
            fn get_compare_equal(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnImmediate> {
                None
            }

            fn get_compare_not_equal(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnImmediate> {
                None
            }

            fn get_compare_greater_than(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnImmediate> {
                None
            }

            fn get_compare_greater_than_or_equal(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnImmediate> {
                None
            }

            fn get_compare_less_than(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnImmediate> {
                None
            }

            fn get_compare_less_than_or_equal(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnImmediate> {
                None
            }

            fn get_compare_changed(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnRelative> {
                None
            }

            fn get_compare_unchanged(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnRelative> {
                None
            }

            fn get_compare_increased(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnRelative> {
                None
            }

            fn get_compare_decreased(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnRelative> {
                None
            }

            fn get_compare_increased_by(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnDelta> {
                None
            }

            fn get_compare_decreased_by(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnDelta> {
                None
            }

            fn get_compare_multiplied_by(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnDelta> {
                None
            }

            fn get_compare_divided_by(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnDelta> {
                None
            }

            fn get_compare_modulo_by(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnDelta> {
                None
            }

            fn get_compare_shift_left_by(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnDelta> {
                None
            }

            fn get_compare_shift_right_by(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnDelta> {
                None
            }

            fn get_compare_logical_and_by(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnDelta> {
                None
            }

            fn get_compare_logical_or_by(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnDelta> {
                None
            }

            fn get_compare_logical_xor_by(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnDelta> {
                None
            }
        }

        impl squalr_engine_api::structures::data_types::comparisons::vector_comparable::VectorComparable for $target_type {
            fn get_vector_compare_equal_64(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate64> {
                None
            }

            fn get_vector_compare_equal_32(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate32> {
                None
            }

            fn get_vector_compare_equal_16(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate16> {
                None
            }

            fn get_vector_compare_not_equal_64(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate64> {
                None
            }

            fn get_vector_compare_not_equal_32(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate32> {
                None
            }

            fn get_vector_compare_not_equal_16(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate16> {
                None
            }

            fn get_vector_compare_greater_than_64(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate64> {
                None
            }

            fn get_vector_compare_greater_than_32(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate32> {
                None
            }

            fn get_vector_compare_greater_than_16(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate16> {
                None
            }

            fn get_vector_compare_greater_than_or_equal_64(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate64> {
                None
            }

            fn get_vector_compare_greater_than_or_equal_32(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate32> {
                None
            }

            fn get_vector_compare_greater_than_or_equal_16(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate16> {
                None
            }

            fn get_vector_compare_less_than_64(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate64> {
                None
            }

            fn get_vector_compare_less_than_32(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate32> {
                None
            }

            fn get_vector_compare_less_than_16(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate16> {
                None
            }

            fn get_vector_compare_less_than_or_equal_64(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate64> {
                None
            }

            fn get_vector_compare_less_than_or_equal_32(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate32> {
                None
            }

            fn get_vector_compare_less_than_or_equal_16(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate16> {
                None
            }

            fn get_vector_compare_changed_64(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnRelative64> {
                None
            }

            fn get_vector_compare_changed_32(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnRelative32> {
                None
            }

            fn get_vector_compare_changed_16(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnRelative16> {
                None
            }

            fn get_vector_compare_unchanged_64(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnRelative64> {
                None
            }

            fn get_vector_compare_unchanged_32(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnRelative32> {
                None
            }

            fn get_vector_compare_unchanged_16(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnRelative16> {
                None
            }

            fn get_vector_compare_increased_64(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnRelative64> {
                None
            }

            fn get_vector_compare_increased_32(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnRelative32> {
                None
            }

            fn get_vector_compare_increased_16(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnRelative16> {
                None
            }

            fn get_vector_compare_decreased_64(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnRelative64> {
                None
            }

            fn get_vector_compare_decreased_32(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnRelative32> {
                None
            }

            fn get_vector_compare_decreased_16(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnRelative16> {
                None
            }

            fn get_vector_compare_increased_by_64(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta64> {
                None
            }

            fn get_vector_compare_increased_by_32(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta32> {
                None
            }

            fn get_vector_compare_increased_by_16(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta16> {
                None
            }

            fn get_vector_compare_decreased_by_64(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta64> {
                None
            }

            fn get_vector_compare_decreased_by_32(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta32> {
                None
            }

            fn get_vector_compare_decreased_by_16(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta16> {
                None
            }

            fn get_vector_compare_multiplied_by_64(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta64> {
                None
            }

            fn get_vector_compare_multiplied_by_32(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta32> {
                None
            }

            fn get_vector_compare_multiplied_by_16(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta16> {
                None
            }

            fn get_vector_compare_divided_by_64(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta64> {
                None
            }

            fn get_vector_compare_divided_by_32(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta32> {
                None
            }

            fn get_vector_compare_divided_by_16(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta16> {
                None
            }

            fn get_vector_compare_modulo_by_64(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta64> {
                None
            }

            fn get_vector_compare_modulo_by_32(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta32> {
                None
            }

            fn get_vector_compare_modulo_by_16(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta16> {
                None
            }

            fn get_vector_compare_shift_left_by_64(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta64> {
                None
            }

            fn get_vector_compare_shift_left_by_32(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta32> {
                None
            }

            fn get_vector_compare_shift_left_by_16(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta16> {
                None
            }

            fn get_vector_compare_shift_right_by_64(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta64> {
                None
            }

            fn get_vector_compare_shift_right_by_32(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta32> {
                None
            }

            fn get_vector_compare_shift_right_by_16(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta16> {
                None
            }

            fn get_vector_compare_logical_and_by_64(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta64> {
                None
            }

            fn get_vector_compare_logical_and_by_32(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta32> {
                None
            }

            fn get_vector_compare_logical_and_by_16(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta16> {
                None
            }

            fn get_vector_compare_logical_or_by_64(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta64> {
                None
            }

            fn get_vector_compare_logical_or_by_32(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta32> {
                None
            }

            fn get_vector_compare_logical_or_by_16(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta16> {
                None
            }

            fn get_vector_compare_logical_xor_by_64(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta64> {
                None
            }

            fn get_vector_compare_logical_xor_by_32(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta32> {
                None
            }

            fn get_vector_compare_logical_xor_by_16(
                &self,
                _scan_constraint: &squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint,
            ) -> Option<squalr_engine_api::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta16> {
                None
            }
        }
    };
}

pub(crate) use impl_data_type_comparison_stubs;
