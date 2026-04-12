use crate::conversions::conversions_from_hex_pattern::ConversionsFromHexPattern;
use crate::registries::symbols::symbol_registry::SymbolRegistry;
use crate::registries::symbols::symbol_registry_error::SymbolRegistryError;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_types::floating_point_tolerance::FloatingPointTolerance;
use crate::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
use crate::structures::data_values::container_type::ContainerType;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::memory::endian::Endian;
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

        if anonymous_scan_constraint.uses_hex_pattern_matching() {
            return self.build_hex_pattern_constraint(anonymous_scan_constraint, data_type_ref);
        }

        if let Some(scan_constraint) = self.build_wildcard_array_constraint(anonymous_scan_constraint, data_type_ref)? {
            return Ok(Some(scan_constraint));
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

    fn build_wildcard_array_constraint(
        &self,
        anonymous_scan_constraint: &AnonymousScanConstraint,
        data_type_ref: &DataTypeRef,
    ) -> Result<Option<ScanConstraint>, ScanConstraintBuilderError> {
        let anonymous_value_string = anonymous_scan_constraint
            .get_anonymous_value_string()
            .as_ref()
            .ok_or_else(|| ScanConstraintBuilderError::build_failed("missing anonymous value string"))?;
        let container_type = anonymous_value_string.get_container_type();

        if !matches!(container_type, ContainerType::Array | ContainerType::ArrayFixed(_)) {
            return Ok(None);
        }

        let tokens = Self::split_array_tokens(anonymous_value_string.get_anonymous_value_string());

        if tokens.is_empty() || !tokens.iter().any(|token| Self::token_contains_wildcard(token)) {
            return Ok(None);
        }

        let data_type = self
            .symbol_registry
            .get_data_type(data_type_ref.get_data_type_id())
            .ok_or_else(|| ScanConstraintBuilderError::build_failed("data type not registered"))?;
        let unit_size_in_bytes = data_type.get_unit_size_in_bytes() as usize;

        if unit_size_in_bytes == 0 {
            return Err(ScanConstraintBuilderError::build_failed("data type has zero-byte unit size"));
        }

        if let ContainerType::ArrayFixed(expected_value_count) = container_type {
            if tokens.len() as u64 != expected_value_count {
                return Err(ScanConstraintBuilderError::build_failed(format!(
                    "Expected {} values for array input, but found {}.",
                    expected_value_count,
                    tokens.len()
                )));
            }
        }

        let mut pattern_bytes = Vec::with_capacity(tokens.len() * unit_size_in_bytes);
        let mut mask_bytes = Vec::with_capacity(tokens.len() * unit_size_in_bytes);

        for token in &tokens {
            let (token_pattern_bytes, token_mask_bytes) = self.parse_wildcard_array_token(
                data_type_ref,
                token,
                anonymous_value_string.get_anonymous_value_string_format(),
                data_type.get_endian(),
                unit_size_in_bytes,
            )?;

            pattern_bytes.extend(token_pattern_bytes);
            mask_bytes.extend(token_mask_bytes);
        }

        self.validate_scan_shape(
            anonymous_scan_constraint,
            data_type_ref,
            &DataValue::new(data_type_ref.clone(), pattern_bytes.clone()),
        )?;

        let mut scan_constraint = ScanConstraint::new_masked(
            anonymous_scan_constraint.get_scan_compare_type(),
            DataValue::new(data_type_ref.clone(), pattern_bytes),
            self.floating_point_tolerance,
            mask_bytes,
        );
        scan_constraint.set_result_container_type(container_type);

        Ok(Some(scan_constraint))
    }

    fn parse_wildcard_array_token(
        &self,
        data_type_ref: &DataTypeRef,
        token: &str,
        value_format: AnonymousValueStringFormat,
        endian: Endian,
        unit_size_in_bytes: usize,
    ) -> Result<(Vec<u8>, Vec<u8>), ScanConstraintBuilderError> {
        if Self::token_is_full_wildcard(token) {
            return Ok((vec![0u8; unit_size_in_bytes], vec![0u8; unit_size_in_bytes]));
        }

        match value_format {
            AnonymousValueStringFormat::Hexadecimal => {
                if Self::token_contains_wildcard(token) {
                    return Self::parse_hex_wildcard_token(token, endian, unit_size_in_bytes);
                }
            }
            AnonymousValueStringFormat::Decimal | AnonymousValueStringFormat::Binary | AnonymousValueStringFormat::Address => {
                if Self::token_contains_wildcard(token) {
                    return Err(ScanConstraintBuilderError::build_failed(format!(
                        "Wildcard token '{}' must be a full-element wildcard for {} arrays.",
                        token, value_format
                    )));
                }
            }
            _ => {}
        }

        let token_value = self
            .symbol_registry
            .deanonymize_value_string(
                data_type_ref,
                &crate::structures::data_values::anonymous_value_string::AnonymousValueString::new(token.to_string(), value_format, ContainerType::None),
            )?
            .get_value_bytes()
            .clone();

        Ok((token_value, vec![0xFFu8; unit_size_in_bytes]))
    }

    fn parse_hex_wildcard_token(
        token: &str,
        endian: Endian,
        unit_size_in_bytes: usize,
    ) -> Result<(Vec<u8>, Vec<u8>), ScanConstraintBuilderError> {
        let normalized_token = token
            .trim()
            .trim_start_matches("0x")
            .trim_start_matches("0X")
            .to_ascii_lowercase();
        let expected_nibble_count = unit_size_in_bytes.saturating_mul(2);

        if normalized_token.is_empty() {
            return Err(ScanConstraintBuilderError::build_failed("empty hex wildcard token"));
        }

        if normalized_token.len() > expected_nibble_count {
            return Err(ScanConstraintBuilderError::build_failed(format!(
                "Hex token '{}' exceeds the expected width of {} nibbles.",
                token, expected_nibble_count
            )));
        }

        let padded_token = format!("{:0>width$}", normalized_token, width = expected_nibble_count);
        let mut pattern_bytes = Vec::with_capacity(unit_size_in_bytes);
        let mut mask_bytes = Vec::with_capacity(unit_size_in_bytes);

        for nibble_pair in padded_token.as_bytes().chunks_exact(2) {
            let (high_pattern_nibble, high_mask_nibble) = Self::parse_hex_wildcard_nibble(nibble_pair[0] as char, token)?;
            let (low_pattern_nibble, low_mask_nibble) = Self::parse_hex_wildcard_nibble(nibble_pair[1] as char, token)?;

            pattern_bytes.push((high_pattern_nibble << 4) | low_pattern_nibble);
            mask_bytes.push((high_mask_nibble << 4) | low_mask_nibble);
        }

        if endian == Endian::Little && unit_size_in_bytes > 1 {
            pattern_bytes.reverse();
            mask_bytes.reverse();
        }

        Ok((pattern_bytes, mask_bytes))
    }

    fn parse_hex_wildcard_nibble(
        nibble_character: char,
        token: &str,
    ) -> Result<(u8, u8), ScanConstraintBuilderError> {
        if matches!(nibble_character, 'x' | 'X') {
            return Ok((0u8, 0u8));
        }

        nibble_character
            .to_digit(16)
            .map(|nibble_value| (nibble_value as u8, 0xFu8))
            .ok_or_else(|| ScanConstraintBuilderError::build_failed(format!("Invalid hexadecimal wildcard token '{}'.", token)))
    }

    fn split_array_tokens(value_string: &str) -> Vec<&str> {
        value_string
            .split(|character: char| character == ',' || character.is_whitespace())
            .map(str::trim)
            .filter(|token| !token.is_empty())
            .collect()
    }

    fn token_contains_wildcard(token: &str) -> bool {
        token.chars().any(|character| matches!(character, 'x' | 'X'))
    }

    fn token_is_full_wildcard(token: &str) -> bool {
        let trimmed_token = token.trim();

        !trimmed_token.is_empty()
            && trimmed_token
                .chars()
                .all(|character| matches!(character, 'x' | 'X'))
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
                AnonymousValueStringFormat::Hexadecimal,
                ContainerType::None,
            )),
        )
        .with_hex_pattern_matching(true);

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

    #[test]
    fn build_creates_decimal_array_constraint_with_full_wildcard_tokens() {
        let symbol_registry = SymbolRegistry::new();
        let builder = ScanConstraintBuilder::new(&symbol_registry, FloatingPointTolerance::default());
        let data_type_ref = DataTypeRef::new("u8");
        let anonymous_scan_constraint = AnonymousScanConstraint::new(
            ScanCompareType::Immediate(ScanCompareTypeImmediate::Equal),
            Some(AnonymousValueString::new(
                "1 xx 55".to_string(),
                AnonymousValueStringFormat::Decimal,
                ContainerType::Array,
            )),
        );

        let scan_constraint = builder
            .build(&anonymous_scan_constraint, &data_type_ref)
            .expect("scan constraint creation should succeed")
            .expect("scan constraint should be produced");

        assert_eq!(scan_constraint.get_data_value().get_value_bytes(), &[1u8, 0u8, 55u8]);
        assert_eq!(scan_constraint.get_mask().cloned(), Some(vec![0xFFu8, 0x00u8, 0xFFu8]));
    }

    #[test]
    fn build_creates_hex_array_constraint_with_nibble_wildcards() {
        let symbol_registry = SymbolRegistry::new();
        let builder = ScanConstraintBuilder::new(&symbol_registry, FloatingPointTolerance::default());
        let data_type_ref = DataTypeRef::new("u8");
        let anonymous_scan_constraint = AnonymousScanConstraint::new(
            ScanCompareType::Immediate(ScanCompareTypeImmediate::Equal),
            Some(AnonymousValueString::new(
                "1 7x x5 xx 55".to_string(),
                AnonymousValueStringFormat::Hexadecimal,
                ContainerType::Array,
            )),
        );

        let scan_constraint = builder
            .build(&anonymous_scan_constraint, &data_type_ref)
            .expect("scan constraint creation should succeed")
            .expect("scan constraint should be produced");

        assert_eq!(scan_constraint.get_data_value().get_value_bytes(), &[0x01u8, 0x70u8, 0x05u8, 0x00u8, 0x55u8]);
        assert_eq!(scan_constraint.get_mask().cloned(), Some(vec![0xFFu8, 0xF0u8, 0x0Fu8, 0x00u8, 0xFFu8]));
    }

    #[test]
    fn build_accepts_space_separated_decimal_array_values_without_commas() {
        let symbol_registry = SymbolRegistry::new();
        let builder = ScanConstraintBuilder::new(&symbol_registry, FloatingPointTolerance::default());
        let data_type_ref = DataTypeRef::new("u8");
        let anonymous_scan_constraint = AnonymousScanConstraint::new(
            ScanCompareType::Immediate(ScanCompareTypeImmediate::Equal),
            Some(AnonymousValueString::new(
                "1 2 3".to_string(),
                AnonymousValueStringFormat::Decimal,
                ContainerType::Array,
            )),
        );

        let scan_constraint = builder
            .build(&anonymous_scan_constraint, &data_type_ref)
            .expect("scan constraint creation should succeed")
            .expect("scan constraint should be produced");

        assert_eq!(scan_constraint.get_data_value().get_value_bytes(), &[1u8, 2u8, 3u8]);
        assert!(!scan_constraint.has_mask());
    }
}
