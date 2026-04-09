use crate::conversions::conversions_from_hex_pattern::ConversionsFromHexPattern;
use crate::registries::symbols::symbol_registry::SymbolRegistry;
use crate::registries::symbols::symbol_registry_error::SymbolRegistryError;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_types::floating_point_tolerance::FloatingPointTolerance;
use crate::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use crate::structures::scanning::comparisons::scan_compare_type_immediate::ScanCompareTypeImmediate;
use crate::structures::scanning::constraints::anonymous_scan_constraint::AnonymousScanConstraint;
use crate::structures::scanning::constraints::scan_constraint::ScanConstraint;
use thiserror::Error;

pub struct ScanConstraintBuilder<'a> {
    symbol_registry: &'a SymbolRegistry,
    floating_point_tolerance: FloatingPointTolerance,
}

impl<'a> ScanConstraintBuilder<'a> {
    pub fn new(
        symbol_registry: &'a SymbolRegistry,
        floating_point_tolerance: FloatingPointTolerance,
    ) -> Self {
        Self {
            symbol_registry,
            floating_point_tolerance,
        }
    }

    pub fn build(
        &self,
        anonymous_scan_constraint: &AnonymousScanConstraint,
        data_type_ref: &DataTypeRef,
    ) -> Result<Option<ScanConstraint>, ScanConstraintBuilderError> {
        let Some(anonymous_value_string) = anonymous_scan_constraint.get_anonymous_value_string() else {
            return Ok(None);
        };

        if anonymous_value_string.get_anonymous_value_string_format() == AnonymousValueStringFormat::HexPattern {
            return self.build_hex_pattern_constraint(anonymous_scan_constraint, data_type_ref);
        }

        let data_value = self
            .symbol_registry
            .deanonymize_value_string(data_type_ref, anonymous_value_string)?;

        self.validate_scan_shape(anonymous_scan_constraint, data_type_ref, &data_value)?;
        let mut scan_constraint = ScanConstraint::new(anonymous_scan_constraint.get_scan_compare_type(), data_value, self.floating_point_tolerance);
        scan_constraint.set_result_container_type(anonymous_value_string.get_container_type());

        Ok(Some(scan_constraint))
    }

    fn build_hex_pattern_constraint(
        &self,
        anonymous_scan_constraint: &AnonymousScanConstraint,
        data_type_ref: &DataTypeRef,
    ) -> Result<Option<ScanConstraint>, ScanConstraintBuilderError> {
        let anonymous_value_string = anonymous_scan_constraint
            .get_anonymous_value_string()
            .as_ref()
            .ok_or_else(|| ScanConstraintBuilderError::build_failed("missing anonymous value string"))?;
        let (pattern_bytes, mask_bytes) = ConversionsFromHexPattern::parse(anonymous_value_string.get_anonymous_value_string())
            .map_err(|error| ScanConstraintBuilderError::build_failed(error.as_str()))?;
        let data_value = DataValue::new(data_type_ref.clone(), pattern_bytes);

        self.validate_scan_shape(anonymous_scan_constraint, data_type_ref, &data_value)?;

        if ConversionsFromHexPattern::has_wildcards(&mask_bytes) {
            let mut scan_constraint = ScanConstraint::new_masked(
                anonymous_scan_constraint.get_scan_compare_type(),
                data_value,
                self.floating_point_tolerance,
                mask_bytes,
            );
            scan_constraint.set_result_container_type(anonymous_value_string.get_container_type());

            Ok(Some(scan_constraint))
        } else {
            let mut scan_constraint = ScanConstraint::new(anonymous_scan_constraint.get_scan_compare_type(), data_value, self.floating_point_tolerance);
            scan_constraint.set_result_container_type(anonymous_value_string.get_container_type());

            Ok(Some(scan_constraint))
        }
    }

    fn validate_scan_shape(
        &self,
        anonymous_scan_constraint: &AnonymousScanConstraint,
        data_type_ref: &DataTypeRef,
        data_value: &DataValue,
    ) -> Result<(), ScanConstraintBuilderError> {
        let data_type_unit_size_in_bytes = self.symbol_registry.get_unit_size_in_bytes(data_type_ref);
        let is_multi_element_value = data_value.get_size_in_bytes() > data_type_unit_size_in_bytes;

        if is_multi_element_value {
            match anonymous_scan_constraint.get_scan_compare_type() {
                ScanCompareType::Immediate(ScanCompareTypeImmediate::Equal) => {}
                _ => {
                    return Err(ScanConstraintBuilderError::build_failed(
                        "Array and pattern scans currently only support equality comparisons",
                    ));
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ScanConstraintBuilderError {
    #[error("Failed while creating scan constraint: {reason}.")]
    BuildFailed { reason: String },
    #[error(transparent)]
    SymbolRegistry(#[from] SymbolRegistryError),
}

impl ScanConstraintBuilderError {
    pub fn build_failed(reason: impl Into<String>) -> Self {
        Self::BuildFailed { reason: reason.into() }
    }
}

#[cfg(test)]
mod tests {
    use super::ScanConstraintBuilder;
    use crate::registries::symbols::symbol_registry::SymbolRegistry;
    use crate::structures::data_types::data_type_ref::DataTypeRef;
    use crate::structures::data_types::floating_point_tolerance::FloatingPointTolerance;
    use crate::structures::data_values::anonymous_value_string::AnonymousValueString;
    use crate::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
    use crate::structures::data_values::container_type::ContainerType;
    use crate::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
    use crate::structures::scanning::comparisons::scan_compare_type_immediate::ScanCompareTypeImmediate;
    use crate::structures::scanning::comparisons::scan_compare_type_relative::ScanCompareTypeRelative;
    use crate::structures::scanning::constraints::anonymous_scan_constraint::AnonymousScanConstraint;

    #[test]
    fn build_creates_literal_constraint_via_data_type() {
        let symbol_registry = SymbolRegistry::new();
        let builder = ScanConstraintBuilder::new(&symbol_registry, FloatingPointTolerance::default());
        let data_type_ref = DataTypeRef::new("u8");
        let anonymous_scan_constraint = AnonymousScanConstraint::new(
            ScanCompareType::Immediate(ScanCompareTypeImmediate::Equal),
            Some(AnonymousValueString::new(
                "7F".to_string(),
                AnonymousValueStringFormat::Hexadecimal,
                ContainerType::None,
            )),
        );

        let scan_constraint = builder
            .build(&anonymous_scan_constraint, &data_type_ref)
            .expect("scan constraint creation should succeed")
            .expect("scan constraint should be produced");

        assert_eq!(scan_constraint.get_data_value().get_value_bytes(), &[0x7F]);
        assert!(!scan_constraint.has_mask());
    }

    #[test]
    fn build_creates_masked_hex_pattern_constraint() {
        let symbol_registry = SymbolRegistry::new();
        let builder = ScanConstraintBuilder::new(&symbol_registry, FloatingPointTolerance::default());
        let data_type_ref = DataTypeRef::new("u8");
        let anonymous_scan_constraint = AnonymousScanConstraint::new(
            ScanCompareType::Immediate(ScanCompareTypeImmediate::Equal),
            Some(AnonymousValueString::new(
                "00 xx 7x".to_string(),
                AnonymousValueStringFormat::HexPattern,
                ContainerType::None,
            )),
        );

        let scan_constraint = builder
            .build(&anonymous_scan_constraint, &data_type_ref)
            .expect("scan constraint creation should succeed")
            .expect("scan constraint should be produced");

        assert_eq!(scan_constraint.get_data_value().get_value_bytes(), &[0x00, 0x00, 0x70]);
        assert_eq!(scan_constraint.get_mask().cloned(), Some(vec![0xFF, 0x00, 0xF0]));
    }

    #[test]
    fn build_rejects_non_equality_array_scans() {
        let symbol_registry = SymbolRegistry::new();
        let builder = ScanConstraintBuilder::new(&symbol_registry, FloatingPointTolerance::default());
        let data_type_ref = DataTypeRef::new("u32");
        let anonymous_scan_constraint = AnonymousScanConstraint::new(
            ScanCompareType::Relative(ScanCompareTypeRelative::Changed),
            Some(AnonymousValueString::new(
                "1, 2".to_string(),
                AnonymousValueStringFormat::Decimal,
                ContainerType::None,
            )),
        );

        let result = builder.build(&anonymous_scan_constraint, &data_type_ref);

        assert!(result.is_err());
    }
}
