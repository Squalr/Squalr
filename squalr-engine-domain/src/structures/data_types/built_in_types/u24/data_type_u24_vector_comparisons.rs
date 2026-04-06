use crate::structures::data_types::built_in_types::u24::data_type_u24::DataTypeU24;
use crate::structures::data_types::comparisons::vector_comparable_none::impl_vector_comparable_none;
use crate::structures::scanning::comparisons::scan_function_vector::{
	VectorCompareFnDelta16, VectorCompareFnDelta32, VectorCompareFnDelta64, VectorCompareFnImmediate16, VectorCompareFnImmediate32, VectorCompareFnImmediate64,
	VectorCompareFnRelative16, VectorCompareFnRelative32, VectorCompareFnRelative64,
};
use crate::structures::scanning::constraints::scan_constraint::ScanConstraint;

impl_vector_comparable_none!(DataTypeU24);