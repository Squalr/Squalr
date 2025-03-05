use crate::structures::data_types::built_in_types::i64be::data_type_i64be::DataTypeI64be;
use crate::structures::data_types::comparisons::vector_comparable::VectorComparable;
use crate::structures::data_types::comparisons::vector_comparable::{
    VectorCompareFnDelta16, VectorCompareFnDelta32, VectorCompareFnDelta64, VectorCompareFnImmediate16, VectorCompareFnImmediate32, VectorCompareFnImmediate64,
    VectorCompareFnRelative16, VectorCompareFnRelative32, VectorCompareFnRelative64,
};
use crate::structures::data_types::comparisons::vector_generics::VectorGenerics;

type PrimitiveType = i64;

// JIRA
impl VectorComparable for DataTypeI64be {
    fn get_vector_compare_equal_64(&self) -> VectorCompareFnImmediate64 {
        VectorGenerics::get_vector_compare_equal::<PrimitiveType, 64>()
    }

    fn get_vector_compare_equal_32(&self) -> VectorCompareFnImmediate32 {
        VectorGenerics::get_vector_compare_equal::<PrimitiveType, 32>()
    }

    fn get_vector_compare_equal_16(&self) -> VectorCompareFnImmediate16 {
        VectorGenerics::get_vector_compare_equal::<PrimitiveType, 16>()
    }

    fn get_vector_compare_not_equal_64(&self) -> VectorCompareFnImmediate64 {
        VectorGenerics::get_vector_compare_not_equal::<PrimitiveType, 64>()
    }

    fn get_vector_compare_not_equal_32(&self) -> VectorCompareFnImmediate32 {
        VectorGenerics::get_vector_compare_not_equal::<PrimitiveType, 32>()
    }

    fn get_vector_compare_not_equal_16(&self) -> VectorCompareFnImmediate16 {
        VectorGenerics::get_vector_compare_not_equal::<PrimitiveType, 16>()
    }

    fn get_vector_compare_greater_than_64(&self) -> VectorCompareFnImmediate64 {
        VectorGenerics::get_vector_compare_greater_than::<PrimitiveType, 64>()
    }

    fn get_vector_compare_greater_than_32(&self) -> VectorCompareFnImmediate32 {
        VectorGenerics::get_vector_compare_greater_than::<PrimitiveType, 32>()
    }

    fn get_vector_compare_greater_than_16(&self) -> VectorCompareFnImmediate16 {
        VectorGenerics::get_vector_compare_greater_than::<PrimitiveType, 16>()
    }

    fn get_vector_compare_greater_than_or_equal_64(&self) -> VectorCompareFnImmediate64 {
        VectorGenerics::get_vector_compare_greater_than_or_equal::<PrimitiveType, 64>()
    }

    fn get_vector_compare_greater_than_or_equal_32(&self) -> VectorCompareFnImmediate32 {
        VectorGenerics::get_vector_compare_greater_than_or_equal::<PrimitiveType, 32>()
    }

    fn get_vector_compare_greater_than_or_equal_16(&self) -> VectorCompareFnImmediate16 {
        VectorGenerics::get_vector_compare_greater_than_or_equal::<PrimitiveType, 16>()
    }

    fn get_vector_compare_less_than_64(&self) -> VectorCompareFnImmediate64 {
        VectorGenerics::get_vector_compare_less_than::<PrimitiveType, 64>()
    }

    fn get_vector_compare_less_than_32(&self) -> VectorCompareFnImmediate32 {
        VectorGenerics::get_vector_compare_less_than::<PrimitiveType, 32>()
    }

    fn get_vector_compare_less_than_16(&self) -> VectorCompareFnImmediate16 {
        VectorGenerics::get_vector_compare_less_than::<PrimitiveType, 16>()
    }

    fn get_vector_compare_less_than_or_equal_64(&self) -> VectorCompareFnImmediate64 {
        VectorGenerics::get_vector_compare_less_than_or_equal::<PrimitiveType, 64>()
    }

    fn get_vector_compare_less_than_or_equal_32(&self) -> VectorCompareFnImmediate32 {
        VectorGenerics::get_vector_compare_less_than_or_equal::<PrimitiveType, 32>()
    }

    fn get_vector_compare_less_than_or_equal_16(&self) -> VectorCompareFnImmediate16 {
        VectorGenerics::get_vector_compare_less_than_or_equal::<PrimitiveType, 16>()
    }

    fn get_vector_compare_changed_64(&self) -> VectorCompareFnRelative64 {
        VectorGenerics::get_vector_compare_changed::<PrimitiveType, 64>()
    }

    fn get_vector_compare_changed_32(&self) -> VectorCompareFnRelative32 {
        VectorGenerics::get_vector_compare_changed::<PrimitiveType, 32>()
    }

    fn get_vector_compare_changed_16(&self) -> VectorCompareFnRelative16 {
        VectorGenerics::get_vector_compare_changed::<PrimitiveType, 16>()
    }

    fn get_vector_compare_unchanged_64(&self) -> VectorCompareFnRelative64 {
        VectorGenerics::get_vector_compare_unchanged::<PrimitiveType, 64>()
    }

    fn get_vector_compare_unchanged_32(&self) -> VectorCompareFnRelative32 {
        VectorGenerics::get_vector_compare_unchanged::<PrimitiveType, 32>()
    }

    fn get_vector_compare_unchanged_16(&self) -> VectorCompareFnRelative16 {
        VectorGenerics::get_vector_compare_unchanged::<PrimitiveType, 16>()
    }

    fn get_vector_compare_increased_64(&self) -> VectorCompareFnRelative64 {
        VectorGenerics::get_vector_compare_increased::<PrimitiveType, 64>()
    }

    fn get_vector_compare_increased_32(&self) -> VectorCompareFnRelative32 {
        VectorGenerics::get_vector_compare_increased::<PrimitiveType, 32>()
    }

    fn get_vector_compare_increased_16(&self) -> VectorCompareFnRelative16 {
        VectorGenerics::get_vector_compare_increased::<PrimitiveType, 16>()
    }

    fn get_vector_compare_decreased_64(&self) -> VectorCompareFnRelative64 {
        VectorGenerics::get_vector_compare_decreased::<PrimitiveType, 64>()
    }

    fn get_vector_compare_decreased_32(&self) -> VectorCompareFnRelative32 {
        VectorGenerics::get_vector_compare_decreased::<PrimitiveType, 32>()
    }

    fn get_vector_compare_decreased_16(&self) -> VectorCompareFnRelative16 {
        VectorGenerics::get_vector_compare_decreased::<PrimitiveType, 16>()
    }

    fn get_vector_compare_increased_by_64(&self) -> VectorCompareFnDelta64 {
        VectorGenerics::get_vector_compare_increased_by::<PrimitiveType, 64>()
    }

    fn get_vector_compare_increased_by_32(&self) -> VectorCompareFnDelta32 {
        VectorGenerics::get_vector_compare_increased_by::<PrimitiveType, 32>()
    }

    fn get_vector_compare_increased_by_16(&self) -> VectorCompareFnDelta16 {
        VectorGenerics::get_vector_compare_increased_by::<PrimitiveType, 16>()
    }

    fn get_vector_compare_decreased_by_64(&self) -> VectorCompareFnDelta64 {
        VectorGenerics::get_vector_compare_decreased_by::<PrimitiveType, 64>()
    }

    fn get_vector_compare_decreased_by_32(&self) -> VectorCompareFnDelta32 {
        VectorGenerics::get_vector_compare_decreased_by::<PrimitiveType, 32>()
    }

    fn get_vector_compare_decreased_by_16(&self) -> VectorCompareFnDelta16 {
        VectorGenerics::get_vector_compare_decreased_by::<PrimitiveType, 16>()
    }
}
