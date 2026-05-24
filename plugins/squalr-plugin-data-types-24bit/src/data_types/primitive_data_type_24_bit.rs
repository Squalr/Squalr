use squalr_engine_api::structures::data_types::data_type_error::DataTypeError;
use squalr_engine_api::structures::data_values::anonymous_value_string::AnonymousValueString;
use squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
use squalr_engine_api::structures::data_values::container_type::ContainerType;
use squalr_engine_api::structures::memory::endian::Endian;
use squalr_engine_api::structures::scanning::comparisons::scan_function_scalar::{ScalarCompareFnDelta, ScalarCompareFnImmediate, ScalarCompareFnRelative};
use squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint;
use std::sync::Arc;

const TWENTY_FOUR_BIT_BYTE_COUNT: usize = 3;
const TWENTY_FOUR_BIT_MASK: u32 = 0x00FF_FFFF;
const TWENTY_FOUR_BIT_SIGN_BIT: u32 = 0x0080_0000;
const TWENTY_FOUR_BIT_SIGNED_MIN: i32 = -(1 << 23);
const TWENTY_FOUR_BIT_SIGNED_MAX: i32 = (1 << 23) - 1;

type ScalarUnsignedOperation = fn(u32, u32) -> Option<u32>;
type ScalarSignedOperation = fn(i32, i32) -> Option<i32>;

pub struct PrimitiveDataType24Bit;

impl PrimitiveDataType24Bit {
    pub fn get_supported_anonymous_value_string_formats() -> Vec<AnonymousValueStringFormat> {
        vec![
            AnonymousValueStringFormat::Binary,
            AnonymousValueStringFormat::Decimal,
            AnonymousValueStringFormat::Hexadecimal,
        ]
    }

    pub fn get_unsigned_value_bytes(
        value: u32,
        endian: Endian,
    ) -> Vec<u8> {
        match endian {
            Endian::Little => Self::unsigned_to_little_endian_bytes(value).to_vec(),
            Endian::Big => Self::unsigned_to_big_endian_bytes(value).to_vec(),
        }
    }

    pub fn get_signed_value_bytes(
        value: i32,
        endian: Endian,
    ) -> Vec<u8> {
        Self::get_unsigned_value_bytes(Self::signed_to_raw(value), endian)
    }

    pub fn deanonymize_unsigned(
        anonymous_value_string: &AnonymousValueString,
        endian: Endian,
    ) -> Result<Vec<u8>, DataTypeError> {
        let value = match anonymous_value_string.get_anonymous_value_string_format() {
            AnonymousValueStringFormat::Binary => Self::parse_unsigned_radix(anonymous_value_string.get_anonymous_value_string(), 2)?,
            AnonymousValueStringFormat::Address | AnonymousValueStringFormat::Hexadecimal => {
                Self::parse_unsigned_radix(anonymous_value_string.get_anonymous_value_string(), 16)?
            }
            _ => anonymous_value_string
                .get_anonymous_value_string()
                .parse::<u32>()
                .map_err(|error| DataTypeError::ParseError(format!("Failed to parse u24 value: {error}")))?,
        };

        if value > TWENTY_FOUR_BIT_MASK {
            return Err(DataTypeError::ParseError(format!(
                "Value '{}' is out of range for u24.",
                anonymous_value_string.get_anonymous_value_string()
            )));
        }

        Ok(Self::get_unsigned_value_bytes(value, endian))
    }

    pub fn deanonymize_signed(
        anonymous_value_string: &AnonymousValueString,
        endian: Endian,
    ) -> Result<Vec<u8>, DataTypeError> {
        let value = match anonymous_value_string.get_anonymous_value_string_format() {
            AnonymousValueStringFormat::Binary => {
                let raw_value = Self::parse_unsigned_radix(anonymous_value_string.get_anonymous_value_string(), 2)?;
                if raw_value > TWENTY_FOUR_BIT_MASK {
                    return Err(DataTypeError::ParseError(format!(
                        "Value '{}' is out of range for i24.",
                        anonymous_value_string.get_anonymous_value_string()
                    )));
                }

                Self::raw_to_signed(raw_value)
            }
            AnonymousValueStringFormat::Address | AnonymousValueStringFormat::Hexadecimal => {
                let raw_value = Self::parse_unsigned_radix(anonymous_value_string.get_anonymous_value_string(), 16)?;
                if raw_value > TWENTY_FOUR_BIT_MASK {
                    return Err(DataTypeError::ParseError(format!(
                        "Value '{}' is out of range for i24.",
                        anonymous_value_string.get_anonymous_value_string()
                    )));
                }

                Self::raw_to_signed(raw_value)
            }
            _ => {
                let parsed_value = anonymous_value_string
                    .get_anonymous_value_string()
                    .parse::<i32>()
                    .map_err(|error| DataTypeError::ParseError(format!("Failed to parse i24 value: {error}")))?;

                if !(TWENTY_FOUR_BIT_SIGNED_MIN..=TWENTY_FOUR_BIT_SIGNED_MAX).contains(&parsed_value) {
                    return Err(DataTypeError::ParseError(format!(
                        "Value '{}' is out of range for i24.",
                        anonymous_value_string.get_anonymous_value_string()
                    )));
                }

                parsed_value
            }
        };

        Ok(Self::get_signed_value_bytes(value, endian))
    }

    pub fn anonymize_unsigned(
        value_bytes: &[u8],
        endian: Endian,
        anonymous_value_string_format: AnonymousValueStringFormat,
    ) -> Result<AnonymousValueString, DataTypeError> {
        let values = Self::read_unsigned_values(value_bytes, endian)?;
        let value_strings = values
            .iter()
            .map(|value| match anonymous_value_string_format {
                AnonymousValueStringFormat::Binary => format!("{:b}", value),
                AnonymousValueStringFormat::Address | AnonymousValueStringFormat::Hexadecimal => format!("{:X}", value),
                _ => value.to_string(),
            })
            .collect::<Vec<_>>();

        Ok(AnonymousValueString::new(
            value_strings.join(", "),
            anonymous_value_string_format,
            Self::container_type_for_value_count(value_strings.len()),
        ))
    }

    pub fn anonymize_signed(
        value_bytes: &[u8],
        endian: Endian,
        anonymous_value_string_format: AnonymousValueStringFormat,
    ) -> Result<AnonymousValueString, DataTypeError> {
        let values = Self::read_signed_values(value_bytes, endian)?;
        let value_strings = values
            .iter()
            .map(|value| match anonymous_value_string_format {
                AnonymousValueStringFormat::Binary => format!("{:b}", Self::signed_to_raw(*value)),
                AnonymousValueStringFormat::Address | AnonymousValueStringFormat::Hexadecimal => format!("{:X}", Self::signed_to_raw(*value)),
                _ => value.to_string(),
            })
            .collect::<Vec<_>>();

        Ok(AnonymousValueString::new(
            value_strings.join(", "),
            anonymous_value_string_format,
            Self::container_type_for_value_count(value_strings.len()),
        ))
    }

    pub fn read_unsigned(
        value_bytes: &[u8],
        endian: Endian,
    ) -> Result<u32, DataTypeError> {
        if value_bytes.len() != TWENTY_FOUR_BIT_BYTE_COUNT {
            return Err(DataTypeError::InvalidByteCount {
                expected: TWENTY_FOUR_BIT_BYTE_COUNT as u64,
                actual: value_bytes.len() as u64,
            });
        }

        Ok(match endian {
            Endian::Little => Self::unsigned_from_little_endian_bytes([value_bytes[0], value_bytes[1], value_bytes[2]]),
            Endian::Big => Self::unsigned_from_big_endian_bytes([value_bytes[0], value_bytes[1], value_bytes[2]]),
        })
    }

    pub fn read_signed(
        value_bytes: &[u8],
        endian: Endian,
    ) -> Result<i32, DataTypeError> {
        Self::read_unsigned(value_bytes, endian).map(Self::raw_to_signed)
    }

    pub unsafe fn read_unsigned_unchecked(
        value_ptr: *const u8,
        endian: Endian,
    ) -> u32 {
        match endian {
            Endian::Little => Self::unsigned_from_little_endian_bytes([unsafe { *value_ptr }, unsafe { *value_ptr.add(1) }, unsafe {
                *value_ptr.add(2)
            }]),
            Endian::Big => Self::unsigned_from_big_endian_bytes([unsafe { *value_ptr }, unsafe { *value_ptr.add(1) }, unsafe {
                *value_ptr.add(2)
            }]),
        }
    }

    pub unsafe fn read_signed_unchecked(
        value_ptr: *const u8,
        endian: Endian,
    ) -> i32 {
        Self::raw_to_signed(unsafe { Self::read_unsigned_unchecked(value_ptr, endian) })
    }

    pub fn get_compare_equal_unsigned(
        scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = Self::read_unsigned(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

        Some(Arc::new(move |current_value_ptr| unsafe {
            Self::read_unsigned_unchecked(current_value_ptr, endian) == immediate_value
        }))
    }

    pub fn get_compare_not_equal_unsigned(
        scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = Self::read_unsigned(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

        Some(Arc::new(move |current_value_ptr| unsafe {
            Self::read_unsigned_unchecked(current_value_ptr, endian) != immediate_value
        }))
    }

    pub fn get_compare_greater_than_unsigned(
        scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = Self::read_unsigned(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

        Some(Arc::new(move |current_value_ptr| unsafe {
            Self::read_unsigned_unchecked(current_value_ptr, endian) > immediate_value
        }))
    }

    pub fn get_compare_greater_than_or_equal_unsigned(
        scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = Self::read_unsigned(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

        Some(Arc::new(move |current_value_ptr| unsafe {
            Self::read_unsigned_unchecked(current_value_ptr, endian) >= immediate_value
        }))
    }

    pub fn get_compare_less_than_unsigned(
        scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = Self::read_unsigned(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

        Some(Arc::new(move |current_value_ptr| unsafe {
            Self::read_unsigned_unchecked(current_value_ptr, endian) < immediate_value
        }))
    }

    pub fn get_compare_less_than_or_equal_unsigned(
        scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = Self::read_unsigned(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

        Some(Arc::new(move |current_value_ptr| unsafe {
            Self::read_unsigned_unchecked(current_value_ptr, endian) <= immediate_value
        }))
    }

    pub fn get_compare_changed_unsigned(
        _scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnRelative> {
        Some(Arc::new(move |current_value_ptr, previous_value_ptr| unsafe {
            Self::read_unsigned_unchecked(current_value_ptr, endian) != Self::read_unsigned_unchecked(previous_value_ptr, endian)
        }))
    }

    pub fn get_compare_unchanged_unsigned(
        _scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnRelative> {
        Some(Arc::new(move |current_value_ptr, previous_value_ptr| unsafe {
            Self::read_unsigned_unchecked(current_value_ptr, endian) == Self::read_unsigned_unchecked(previous_value_ptr, endian)
        }))
    }

    pub fn get_compare_increased_unsigned(
        _scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnRelative> {
        Some(Arc::new(move |current_value_ptr, previous_value_ptr| unsafe {
            Self::read_unsigned_unchecked(current_value_ptr, endian) > Self::read_unsigned_unchecked(previous_value_ptr, endian)
        }))
    }

    pub fn get_compare_decreased_unsigned(
        _scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnRelative> {
        Some(Arc::new(move |current_value_ptr, previous_value_ptr| unsafe {
            Self::read_unsigned_unchecked(current_value_ptr, endian) < Self::read_unsigned_unchecked(previous_value_ptr, endian)
        }))
    }

    pub fn get_compare_increased_by_unsigned(
        scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnDelta> {
        Self::build_unsigned_delta_compare(scan_constraint, endian, |previous_value, delta_value| {
            Some(previous_value.wrapping_add(delta_value) & TWENTY_FOUR_BIT_MASK)
        })
    }

    pub fn get_compare_decreased_by_unsigned(
        scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnDelta> {
        Self::build_unsigned_delta_compare(scan_constraint, endian, |previous_value, delta_value| {
            Some(previous_value.wrapping_sub(delta_value) & TWENTY_FOUR_BIT_MASK)
        })
    }

    pub fn get_compare_multiplied_by_unsigned(
        scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnDelta> {
        Self::build_unsigned_delta_compare(scan_constraint, endian, |previous_value, delta_value| {
            Some(previous_value.wrapping_mul(delta_value) & TWENTY_FOUR_BIT_MASK)
        })
    }

    pub fn get_compare_divided_by_unsigned(
        scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnDelta> {
        Self::build_unsigned_delta_compare(scan_constraint, endian, |previous_value, delta_value| {
            if delta_value == 0 { None } else { Some(previous_value / delta_value) }
        })
    }

    pub fn get_compare_modulo_by_unsigned(
        scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnDelta> {
        Self::build_unsigned_delta_compare(scan_constraint, endian, |previous_value, delta_value| {
            if delta_value == 0 { None } else { Some(previous_value % delta_value) }
        })
    }

    pub fn get_compare_shift_left_by_unsigned(
        scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnDelta> {
        Self::build_unsigned_delta_compare(scan_constraint, endian, |previous_value, delta_value| {
            if delta_value >= 24 {
                None
            } else {
                Some((previous_value << delta_value) & TWENTY_FOUR_BIT_MASK)
            }
        })
    }

    pub fn get_compare_shift_right_by_unsigned(
        scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnDelta> {
        Self::build_unsigned_delta_compare(scan_constraint, endian, |previous_value, delta_value| {
            if delta_value >= 24 { None } else { Some(previous_value >> delta_value) }
        })
    }

    pub fn get_compare_logical_and_by_unsigned(
        scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnDelta> {
        Self::build_unsigned_delta_compare(scan_constraint, endian, |previous_value, delta_value| Some(previous_value & delta_value))
    }

    pub fn get_compare_logical_or_by_unsigned(
        scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnDelta> {
        Self::build_unsigned_delta_compare(scan_constraint, endian, |previous_value, delta_value| {
            Some((previous_value | delta_value) & TWENTY_FOUR_BIT_MASK)
        })
    }

    pub fn get_compare_logical_xor_by_unsigned(
        scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnDelta> {
        Self::build_unsigned_delta_compare(scan_constraint, endian, |previous_value, delta_value| {
            Some((previous_value ^ delta_value) & TWENTY_FOUR_BIT_MASK)
        })
    }

    pub fn get_compare_equal_signed(
        scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = Self::read_signed(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

        Some(Arc::new(move |current_value_ptr| unsafe {
            Self::read_signed_unchecked(current_value_ptr, endian) == immediate_value
        }))
    }

    pub fn get_compare_not_equal_signed(
        scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = Self::read_signed(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

        Some(Arc::new(move |current_value_ptr| unsafe {
            Self::read_signed_unchecked(current_value_ptr, endian) != immediate_value
        }))
    }

    pub fn get_compare_greater_than_signed(
        scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = Self::read_signed(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

        Some(Arc::new(move |current_value_ptr| unsafe {
            Self::read_signed_unchecked(current_value_ptr, endian) > immediate_value
        }))
    }

    pub fn get_compare_greater_than_or_equal_signed(
        scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = Self::read_signed(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

        Some(Arc::new(move |current_value_ptr| unsafe {
            Self::read_signed_unchecked(current_value_ptr, endian) >= immediate_value
        }))
    }

    pub fn get_compare_less_than_signed(
        scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = Self::read_signed(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

        Some(Arc::new(move |current_value_ptr| unsafe {
            Self::read_signed_unchecked(current_value_ptr, endian) < immediate_value
        }))
    }

    pub fn get_compare_less_than_or_equal_signed(
        scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnImmediate> {
        let immediate_value = Self::read_signed(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

        Some(Arc::new(move |current_value_ptr| unsafe {
            Self::read_signed_unchecked(current_value_ptr, endian) <= immediate_value
        }))
    }

    pub fn get_compare_changed_signed(
        _scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnRelative> {
        Some(Arc::new(move |current_value_ptr, previous_value_ptr| unsafe {
            Self::read_signed_unchecked(current_value_ptr, endian) != Self::read_signed_unchecked(previous_value_ptr, endian)
        }))
    }

    pub fn get_compare_unchanged_signed(
        _scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnRelative> {
        Some(Arc::new(move |current_value_ptr, previous_value_ptr| unsafe {
            Self::read_signed_unchecked(current_value_ptr, endian) == Self::read_signed_unchecked(previous_value_ptr, endian)
        }))
    }

    pub fn get_compare_increased_signed(
        _scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnRelative> {
        Some(Arc::new(move |current_value_ptr, previous_value_ptr| unsafe {
            Self::read_signed_unchecked(current_value_ptr, endian) > Self::read_signed_unchecked(previous_value_ptr, endian)
        }))
    }

    pub fn get_compare_decreased_signed(
        _scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnRelative> {
        Some(Arc::new(move |current_value_ptr, previous_value_ptr| unsafe {
            Self::read_signed_unchecked(current_value_ptr, endian) < Self::read_signed_unchecked(previous_value_ptr, endian)
        }))
    }

    pub fn get_compare_increased_by_signed(
        scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnDelta> {
        Self::build_signed_delta_compare(scan_constraint, endian, |previous_value, delta_value| {
            Some(Self::normalize_signed(previous_value.wrapping_add(delta_value)))
        })
    }

    pub fn get_compare_decreased_by_signed(
        scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnDelta> {
        Self::build_signed_delta_compare(scan_constraint, endian, |previous_value, delta_value| {
            Some(Self::normalize_signed(previous_value.wrapping_sub(delta_value)))
        })
    }

    pub fn get_compare_multiplied_by_signed(
        scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnDelta> {
        Self::build_signed_delta_compare(scan_constraint, endian, |previous_value, delta_value| {
            Some(Self::normalize_signed(previous_value.wrapping_mul(delta_value)))
        })
    }

    pub fn get_compare_divided_by_signed(
        scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnDelta> {
        Self::build_signed_delta_compare(scan_constraint, endian, |previous_value, delta_value| {
            if delta_value == 0 {
                None
            } else {
                Some(Self::normalize_signed(previous_value / delta_value))
            }
        })
    }

    pub fn get_compare_modulo_by_signed(
        scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnDelta> {
        Self::build_signed_delta_compare(scan_constraint, endian, |previous_value, delta_value| {
            if delta_value == 0 {
                None
            } else {
                Some(Self::normalize_signed(previous_value % delta_value))
            }
        })
    }

    pub fn get_compare_shift_left_by_signed(
        scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnDelta> {
        Self::build_signed_delta_compare(scan_constraint, endian, |previous_value, delta_value| {
            if delta_value < 0 || delta_value >= 24 {
                None
            } else {
                Some(Self::normalize_signed(previous_value << delta_value))
            }
        })
    }

    pub fn get_compare_shift_right_by_signed(
        scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnDelta> {
        Self::build_signed_delta_compare(scan_constraint, endian, |previous_value, delta_value| {
            if delta_value < 0 || delta_value >= 24 {
                None
            } else {
                Some(Self::normalize_signed(previous_value >> delta_value))
            }
        })
    }

    pub fn get_compare_logical_and_by_signed(
        scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnDelta> {
        Self::build_signed_delta_compare(scan_constraint, endian, |previous_value, delta_value| {
            Some(Self::raw_to_signed(Self::signed_to_raw(previous_value) & Self::signed_to_raw(delta_value)))
        })
    }

    pub fn get_compare_logical_or_by_signed(
        scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnDelta> {
        Self::build_signed_delta_compare(scan_constraint, endian, |previous_value, delta_value| {
            Some(Self::raw_to_signed(
                (Self::signed_to_raw(previous_value) | Self::signed_to_raw(delta_value)) & TWENTY_FOUR_BIT_MASK,
            ))
        })
    }

    pub fn get_compare_logical_xor_by_signed(
        scan_constraint: &ScanConstraint,
        endian: Endian,
    ) -> Option<ScalarCompareFnDelta> {
        Self::build_signed_delta_compare(scan_constraint, endian, |previous_value, delta_value| {
            Some(Self::raw_to_signed(
                (Self::signed_to_raw(previous_value) ^ Self::signed_to_raw(delta_value)) & TWENTY_FOUR_BIT_MASK,
            ))
        })
    }

    fn build_unsigned_delta_compare(
        scan_constraint: &ScanConstraint,
        endian: Endian,
        operation: ScalarUnsignedOperation,
    ) -> Option<ScalarCompareFnDelta> {
        let delta_value = Self::read_unsigned(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

        Some(Arc::new(move |current_value_ptr, previous_value_ptr| unsafe {
            let current_value = Self::read_unsigned_unchecked(current_value_ptr, endian);
            let previous_value = Self::read_unsigned_unchecked(previous_value_ptr, endian);

            operation(previous_value, delta_value)
                .map(|target_value| current_value == target_value)
                .unwrap_or(false)
        }))
    }

    fn build_signed_delta_compare(
        scan_constraint: &ScanConstraint,
        endian: Endian,
        operation: ScalarSignedOperation,
    ) -> Option<ScalarCompareFnDelta> {
        let delta_value = Self::read_signed(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

        Some(Arc::new(move |current_value_ptr, previous_value_ptr| unsafe {
            let current_value = Self::read_signed_unchecked(current_value_ptr, endian);
            let previous_value = Self::read_signed_unchecked(previous_value_ptr, endian);

            operation(previous_value, delta_value)
                .map(|target_value| current_value == target_value)
                .unwrap_or(false)
        }))
    }

    fn parse_unsigned_radix(
        value: &str,
        radix: u32,
    ) -> Result<u32, DataTypeError> {
        let trimmed_value = value.trim();
        let normalized_value = match radix {
            2 => trimmed_value
                .strip_prefix("0b")
                .or_else(|| trimmed_value.strip_prefix("0B"))
                .unwrap_or(trimmed_value),
            16 => trimmed_value
                .strip_prefix("0x")
                .or_else(|| trimmed_value.strip_prefix("0X"))
                .unwrap_or(trimmed_value),
            _ => trimmed_value,
        };

        u32::from_str_radix(normalized_value, radix).map_err(|error| DataTypeError::ParseError(format!("Failed to parse 24-bit value: {error}")))
    }

    fn read_unsigned_values(
        value_bytes: &[u8],
        endian: Endian,
    ) -> Result<Vec<u32>, DataTypeError> {
        if value_bytes.is_empty() {
            return Err(DataTypeError::NoBytes);
        }

        if value_bytes.len() % TWENTY_FOUR_BIT_BYTE_COUNT != 0 {
            return Err(DataTypeError::InvalidByteCount {
                expected: TWENTY_FOUR_BIT_BYTE_COUNT as u64,
                actual: value_bytes.len() as u64,
            });
        }

        value_bytes
            .chunks_exact(TWENTY_FOUR_BIT_BYTE_COUNT)
            .map(|chunk| Self::read_unsigned(chunk, endian))
            .collect()
    }

    fn read_signed_values(
        value_bytes: &[u8],
        endian: Endian,
    ) -> Result<Vec<i32>, DataTypeError> {
        if value_bytes.is_empty() {
            return Err(DataTypeError::NoBytes);
        }

        if value_bytes.len() % TWENTY_FOUR_BIT_BYTE_COUNT != 0 {
            return Err(DataTypeError::InvalidByteCount {
                expected: TWENTY_FOUR_BIT_BYTE_COUNT as u64,
                actual: value_bytes.len() as u64,
            });
        }

        value_bytes
            .chunks_exact(TWENTY_FOUR_BIT_BYTE_COUNT)
            .map(|chunk| Self::read_signed(chunk, endian))
            .collect()
    }

    fn container_type_for_value_count(value_count: usize) -> ContainerType {
        if value_count > 1 {
            ContainerType::ArrayFixed(value_count as u64)
        } else {
            ContainerType::None
        }
    }

    fn unsigned_to_little_endian_bytes(value: u32) -> [u8; TWENTY_FOUR_BIT_BYTE_COUNT] {
        let masked_value = value & TWENTY_FOUR_BIT_MASK;

        [
            masked_value as u8,
            (masked_value >> 8) as u8,
            (masked_value >> 16) as u8,
        ]
    }

    fn unsigned_to_big_endian_bytes(value: u32) -> [u8; TWENTY_FOUR_BIT_BYTE_COUNT] {
        let masked_value = value & TWENTY_FOUR_BIT_MASK;

        [
            (masked_value >> 16) as u8,
            (masked_value >> 8) as u8,
            masked_value as u8,
        ]
    }

    fn unsigned_from_little_endian_bytes(value_bytes: [u8; TWENTY_FOUR_BIT_BYTE_COUNT]) -> u32 {
        u32::from_le_bytes([value_bytes[0], value_bytes[1], value_bytes[2], 0]) & TWENTY_FOUR_BIT_MASK
    }

    fn unsigned_from_big_endian_bytes(value_bytes: [u8; TWENTY_FOUR_BIT_BYTE_COUNT]) -> u32 {
        u32::from_be_bytes([0, value_bytes[0], value_bytes[1], value_bytes[2]]) & TWENTY_FOUR_BIT_MASK
    }

    fn raw_to_signed(raw_value: u32) -> i32 {
        let masked_value = raw_value & TWENTY_FOUR_BIT_MASK;

        if masked_value & TWENTY_FOUR_BIT_SIGN_BIT != 0 {
            (masked_value | !TWENTY_FOUR_BIT_MASK) as i32
        } else {
            masked_value as i32
        }
    }

    fn signed_to_raw(value: i32) -> u32 {
        (value as u32) & TWENTY_FOUR_BIT_MASK
    }

    fn normalize_signed(value: i32) -> i32 {
        Self::raw_to_signed(Self::signed_to_raw(value))
    }
}

#[cfg(test)]
mod tests {
    use super::PrimitiveDataType24Bit;
    use squalr_engine_api::structures::data_values::anonymous_value_string::AnonymousValueString;
    use squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
    use squalr_engine_api::structures::data_values::container_type::ContainerType;
    use squalr_engine_api::structures::memory::endian::Endian;

    #[test]
    fn deanonymize_signed_hexadecimal_sign_extends_values() {
        let anonymous_value_string = AnonymousValueString::new(String::from("FF8000"), AnonymousValueStringFormat::Hexadecimal, ContainerType::None);
        let value_bytes = PrimitiveDataType24Bit::deanonymize_signed(&anonymous_value_string, Endian::Big).expect("Expected i24 hexadecimal value to parse.");

        assert_eq!(
            PrimitiveDataType24Bit::read_signed(&value_bytes, Endian::Big).expect("Expected i24 bytes to decode."),
            -32768
        );
    }

    #[test]
    fn anonymize_unsigned_reads_three_byte_chunks() {
        let value_bytes = vec![0x34, 0x12, 0x00, 0x78, 0x56, 0x00];
        let anonymous_value_string = PrimitiveDataType24Bit::anonymize_unsigned(&value_bytes, Endian::Little, AnonymousValueStringFormat::Decimal)
            .expect("Expected u24 values to anonymize.");

        assert_eq!(anonymous_value_string.get_anonymous_value_string(), "4660, 22136");
    }
}
