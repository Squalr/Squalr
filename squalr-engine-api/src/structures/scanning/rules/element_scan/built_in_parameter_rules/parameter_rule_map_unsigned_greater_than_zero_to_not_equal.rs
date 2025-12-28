use crate::registries::symbols::symbol_registry::SymbolRegistry;
use crate::structures::scanning::comparisons::{scan_compare_type::ScanCompareType, scan_compare_type_immediate::ScanCompareTypeImmediate};
use crate::structures::scanning::plans::element_scan::element_scan_parameters::ElementScanParameters;
use crate::structures::scanning::rules::element_scan_parameters_rule::ElementScanParametersRule;

/// Defines a mapping rule that converts > 0 scans for unsigned non-floating-point values into != 0.
/// This optimization allows for better vectorization, resulting in faster scans.
pub struct RuleMapUnsignedGreaterThanZeroToNotEqual {}

impl RuleMapUnsignedGreaterThanZeroToNotEqual {
    pub const RULE_ID: &str = "map_unsigned_greater_than_zero_to_not_equal";
}

impl ElementScanParametersRule for RuleMapUnsignedGreaterThanZeroToNotEqual {
    fn get_id(&self) -> &str {
        &Self::RULE_ID
    }

    fn map_parameters(
        &self,
        symbol_registry: &SymbolRegistry,
        element_scan_parameters: &mut ElementScanParameters,
    ) {
        for scan_constraint in element_scan_parameters.get_scan_constraints_mut() {
            let data_type_ref = scan_constraint.get_data_value().get_data_type_ref();
            let is_signed = symbol_registry.is_signed(data_type_ref);
            let is_floating_point = symbol_registry.is_floating_point(data_type_ref);
            let is_all_zero = scan_constraint
                .get_data_value()
                .get_value_bytes()
                .iter()
                .all(|byte| *byte == 0);

            // Remap unsigned scans that are checking "> 0" to be "!= 0" since this is actually faster internally.
            if !is_signed && !is_floating_point && is_all_zero {
                match scan_constraint.get_scan_compare_type() {
                    ScanCompareType::Immediate(scan_compare_type_immediate) => match scan_compare_type_immediate {
                        ScanCompareTypeImmediate::GreaterThan => {
                            scan_constraint.set_scan_compare_type(ScanCompareType::Immediate(ScanCompareTypeImmediate::NotEqual));
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
    }
}
