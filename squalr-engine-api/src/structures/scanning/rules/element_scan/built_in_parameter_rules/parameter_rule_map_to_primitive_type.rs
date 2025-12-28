use crate::registries::symbols::symbol_registry::SymbolRegistry;
use crate::structures::data_types::built_in_types::u8::data_type_u8::DataTypeU8;
use crate::structures::data_types::built_in_types::u16be::data_type_u16be::DataTypeU16be;
use crate::structures::data_types::built_in_types::u32be::data_type_u32be::DataTypeU32be;
use crate::structures::data_types::built_in_types::u64be::data_type_u64be::DataTypeU64be;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use crate::structures::scanning::plans::element_scan::element_scan_parameters::ElementScanParameters;
use crate::structures::scanning::rules::element_scan_parameters_rule::ElementScanParametersRule;

pub struct RuleMapToPrimitiveType {}

impl RuleMapToPrimitiveType {
    pub const RULE_ID: &str = "map_to_primitive_type";
}

impl ElementScanParametersRule for RuleMapToPrimitiveType {
    fn get_id(&self) -> &str {
        &Self::RULE_ID
    }

    fn map_parameters(
        &self,
        symbol_registry: &SymbolRegistry,
        element_scan_parameters: &mut ElementScanParameters,
    ) {
        for scan_constraint in element_scan_parameters.get_scan_constraints_mut() {
            let data_value = scan_constraint.get_data_value();
            let data_type_ref = data_value.get_data_type_ref();

            // Only immediate scans can be remapped, if the scan is relative, then the original data type is crucial.
            match scan_constraint.get_scan_compare_type() {
                ScanCompareType::Relative(_) | ScanCompareType::Delta(_) => return,
                ScanCompareType::Immediate(_) => {}
            };

            // Non discrete / floating point types cannot be remapped. For example, if we have an array of two f32 values,
            // we absolutely cannot remap this to a single u64 (nor an f64) since these require tolerance comparisons.
            if symbol_registry.is_floating_point(data_type_ref) {
                return;
            }

            let data_type_size = data_value.get_size_in_bytes();
            let data_type_default_size = symbol_registry.get_unit_size_in_bytes(data_type_ref);

            // If the data type size is the default for that type, and its already a valid primitive size,
            // there is no need to perform a remapping. We do this check to avoid meaningless remappings,
            // such as remapping i16 to u16, even though this appears acceptable at a surface level.
            // These remappings must be avoided to avoid subtle problems, such as remapping a ">= 0" scan from signed to unsigned.
            // This would potentially allow for poisoned future optimizations, such as remapping ">= 0" to "!= 0",
            // which is perfectly valid for unsigned scans, but not valid at all for signed scans.
            if data_type_size == data_type_default_size {
                match data_type_size {
                    1 | 2 | 4 | 8 => return,
                    _ => {}
                };
            }

            // If applicable, try to reinterpret array of byte scans as a primitive type of the same size.
            // These are much more efficient than array of byte scans, so for scans of these sizes performance will be improved greatly.
            match data_type_size {
                8 => scan_constraint.set_data_type_in_place(DataTypeRef::new(DataTypeU64be::get_data_type_id()), symbol_registry),
                4 => scan_constraint.set_data_type_in_place(DataTypeRef::new(DataTypeU32be::get_data_type_id()), symbol_registry),
                2 => scan_constraint.set_data_type_in_place(DataTypeRef::new(DataTypeU16be::get_data_type_id()), symbol_registry),
                1 => scan_constraint.set_data_type_in_place(DataTypeRef::new(DataTypeU8::get_data_type_id()), symbol_registry),
                _ => {}
            }
        }
    }
}
