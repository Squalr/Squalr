use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::anonymous_value_string::AnonymousValueString;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PointerScanTargetDescriptor {
    Address {
        target_address: u64,
    },
    Value {
        target_value: AnonymousValueString,
        data_type_ref: DataTypeRef,
        target_address_count: u64,
    },
}

impl Default for PointerScanTargetDescriptor {
    fn default() -> Self {
        Self::Address { target_address: 0 }
    }
}

impl PointerScanTargetDescriptor {
    pub fn address(target_address: u64) -> Self {
        Self::Address { target_address }
    }

    pub fn value(
        target_value: AnonymousValueString,
        data_type_ref: DataTypeRef,
        target_address_count: u64,
    ) -> Self {
        Self::Value {
            target_value,
            data_type_ref,
            target_address_count,
        }
    }

    pub fn get_target_address(&self) -> Option<u64> {
        match self {
            Self::Address { target_address } => Some(*target_address),
            Self::Value { .. } => None,
        }
    }

    pub fn get_target_value(&self) -> Option<&AnonymousValueString> {
        match self {
            Self::Address { .. } => None,
            Self::Value { target_value, .. } => Some(target_value),
        }
    }

    pub fn get_data_type_ref(&self) -> Option<&DataTypeRef> {
        match self {
            Self::Address { .. } => None,
            Self::Value { data_type_ref, .. } => Some(data_type_ref),
        }
    }

    pub fn get_target_address_count(&self) -> u64 {
        match self {
            Self::Address { .. } => 1,
            Self::Value { target_address_count, .. } => *target_address_count,
        }
    }
}

impl fmt::Display for PointerScanTargetDescriptor {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Self::Address { target_address } => write!(formatter, "0x{:X}", target_address),
            Self::Value {
                target_value,
                data_type_ref,
                target_address_count,
            } => write!(
                formatter,
                "{} ({}, {} matches)",
                target_value.get_anonymous_value_string(),
                data_type_ref.get_data_type_id(),
                target_address_count
            ),
        }
    }
}
