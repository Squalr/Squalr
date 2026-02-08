use crate::registries::symbols::symbol_registry_error::SymbolRegistryError;
use crate::structures::data_types::generics::vector_function::GetVectorFunction;
use crate::structures::data_values::container_type::ContainerType;
use crate::structures::scanning::comparisons::scan_function_vector::{VectorCompareFnDelta, VectorCompareFnImmediate, VectorCompareFnRelative};
use crate::structures::scanning::constraints::scan_constraint::ScanConstraint;
use crate::structures::structs::symbolic_field_definition::SymbolicFieldDefinition;
use crate::structures::structs::symbolic_struct_definition::SymbolicStructDefinition;
use crate::structures::{
    data_types::{
        built_in_types::{
            bool8::data_type_bool8::DataTypeBool8, bool32::data_type_bool32::DataTypeBool32, f32::data_type_f32::DataTypeF32,
            f32be::data_type_f32be::DataTypeF32be, f64::data_type_f64::DataTypeF64, f64be::data_type_f64be::DataTypeF64be, i8::data_type_i8::DataTypeI8,
            i16::data_type_i16::DataTypeI16, i16be::data_type_i16be::DataTypeI16be, i32::data_type_i32::DataTypeI32, i32be::data_type_i32be::DataTypeI32be,
            i64::data_type_i64::DataTypeI64, i64be::data_type_i64be::DataTypeI64be, string::utf8::data_type_string_utf8::DataTypeStringUtf8,
            u8::data_type_u8::DataTypeU8, u16::data_type_u16::DataTypeU16, u16be::data_type_u16be::DataTypeU16be, u32::data_type_u32::DataTypeU32,
            u32be::data_type_u32be::DataTypeU32be, u64::data_type_u64::DataTypeU64, u64be::data_type_u64be::DataTypeU64be,
        },
        data_type::DataType,
        data_type_ref::DataTypeRef,
        generics::vector_comparer::VectorComparer,
    },
    data_values::{anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, data_value::DataValue},
    scanning::comparisons::{
        scan_compare_type_delta::ScanCompareTypeDelta,
        scan_compare_type_immediate::ScanCompareTypeImmediate,
        scan_compare_type_relative::ScanCompareTypeRelative,
        scan_function_scalar::{ScalarCompareFnImmediate, ScalarCompareFnRelative},
    },
};
use std::sync::Once;
use std::{
    collections::HashMap,
    simd::{LaneCount, SupportedLaneCount},
    sync::Arc,
};

/// Manages a symbolic struct registry and a data type registry. All registered data types are also registered into the symbolic struct
/// registry, since each data type is considered to be a symbol. The struct contains a single anonymous field for the corresponding type.
pub struct SymbolRegistry {
    symbolic_struct_registry: HashMap<String, Arc<SymbolicStructDefinition>>,
    data_type_registry: HashMap<String, Arc<dyn DataType>>,
}

impl SymbolRegistry {
    // JIRA: Deprecate this. Needs to support mutability, mirroring from client to server for non-standalone builds, etc.
    pub fn get_instance() -> &'static SymbolRegistry {
        static mut INSTANCE: Option<SymbolRegistry> = None;
        static ONCE: Once = Once::new();

        unsafe {
            ONCE.call_once(|| {
                let instance = SymbolRegistry::new();
                INSTANCE = Some(instance);
            });

            #[allow(static_mut_refs)]
            INSTANCE.as_ref().unwrap_unchecked()
        }
    }

    pub fn new() -> Self {
        let (symbolic_struct_registry, data_type_registry) = Self::create_built_in_registries();

        Self {
            symbolic_struct_registry,
            data_type_registry,
        }
    }

    pub fn get_registry(&self) -> &HashMap<String, Arc<SymbolicStructDefinition>> {
        &self.symbolic_struct_registry
    }

    fn register_data_type(
        &mut self,
        _data_type: Arc<dyn DataType>,
    ) {
        // JIRA
    }

    fn unregister_data_type(
        &mut self,
        _data_type: Arc<dyn DataType>,
    ) {
        // JIRA
    }

    pub fn get(
        &self,
        symbolic_struct_ref_id: &str,
    ) -> Option<Arc<SymbolicStructDefinition>> {
        if let Some(symbolic_struct_definition) = self.symbolic_struct_registry.get(symbolic_struct_ref_id.trim()) {
            Some(symbolic_struct_definition.clone())
        } else {
            log::warn!("Failed to find symbolic struct in registry: {}", symbolic_struct_ref_id);
            None
        }
    }

    pub fn get_data_type_registry(&self) -> &HashMap<String, Arc<dyn DataType>> {
        &self.data_type_registry
    }

    pub fn get_data_type(
        &self,
        data_type_id: &str,
    ) -> Option<Arc<dyn DataType>> {
        if let Some(data_type) = self.data_type_registry.get(data_type_id.trim()) {
            Some(data_type.clone())
        } else {
            log::warn!("Failed to find data type in registry: {}", data_type_id);
            None
        }
    }

    /// Determines if the `DataType` this struct represents is currently registered and available.
    pub fn is_valid(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> bool {
        self.get_data_type(data_type_ref.get_data_type_id()).is_some()
    }

    pub fn get_icon_id(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> String {
        match self.get_data_type(data_type_ref.get_data_type_id()) {
            Some(data_type) => data_type.get_icon_id().to_string(),
            None => String::new(),
        }
    }

    pub fn get_unit_size_in_bytes(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> u64 {
        match self.get_data_type(data_type_ref.get_data_type_id()) {
            Some(data_type) => data_type.get_unit_size_in_bytes(),
            None => 0,
        }
    }

    pub fn validate_value_string(
        &self,
        data_type_ref: &DataTypeRef,
        anonymous_value_string: &AnonymousValueString,
    ) -> bool {
        match self.get_data_type(data_type_ref.get_data_type_id()) {
            Some(data_type) => {
                if !data_type.validate_value_string(anonymous_value_string) {
                    return false;
                }
            }
            None => return false,
        }

        true
    }

    pub fn deanonymize_value_string(
        &self,
        data_type_ref: &DataTypeRef,
        anonymous_value_string: &AnonymousValueString,
    ) -> Result<DataValue, SymbolRegistryError> {
        match self.get_data_type(data_type_ref.get_data_type_id()) {
            Some(data_type) => data_type
                .deanonymize_value_string(anonymous_value_string)
                .map_err(|error| SymbolRegistryError::data_type_operation_failed("deanonymize value string", error)),
            None => Err(SymbolRegistryError::data_type_not_registered(
                "deanonymize value string",
                data_type_ref.get_data_type_id(),
            )),
        }
    }

    pub fn anonymize_value(
        &self,
        data_value: &DataValue,
        anonymous_value_string_format: AnonymousValueStringFormat,
    ) -> Result<AnonymousValueString, SymbolRegistryError> {
        match self.get_data_type(data_value.get_data_type_id()) {
            Some(data_type) => data_type
                .anonymize_value_bytes(data_value.get_value_bytes(), anonymous_value_string_format)
                .map_err(|error| SymbolRegistryError::data_type_operation_failed("anonymize value", error)),
            None => Err(SymbolRegistryError::data_type_not_registered("anonymize value", data_value.get_data_type_id())),
        }
    }

    pub fn anonymize_value_to_supported_formats(
        &self,
        data_value: &DataValue,
    ) -> Result<Vec<AnonymousValueString>, SymbolRegistryError> {
        match self.get_data_type(data_value.get_data_type_id()) {
            Some(data_type) => {
                let mut anonymized_values = Vec::new();

                for anonymous_value_string_format in data_type.get_supported_anonymous_value_string_formats() {
                    let anonymous_value_string = data_type
                        .anonymize_value_bytes(data_value.get_value_bytes(), anonymous_value_string_format)
                        .map_err(|error| SymbolRegistryError::data_type_operation_failed("anonymize value", error))?;

                    anonymized_values.push(anonymous_value_string);
                }

                Ok(anonymized_values)
            }
            None => Err(SymbolRegistryError::data_type_not_registered("anonymize value", data_value.get_data_type_id())),
        }
    }

    pub fn get_supported_anonymous_value_string_formats(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> Vec<AnonymousValueStringFormat> {
        self.get_data_type(data_type_ref.get_data_type_id())
            .map(|data_type| data_type.get_supported_anonymous_value_string_formats())
            .unwrap_or_default()
    }

    pub fn get_default_anonymous_value_string_format(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> AnonymousValueStringFormat {
        self.get_data_type(data_type_ref.get_data_type_id())
            .map(|data_type| data_type.get_default_anonymous_value_string_format())
            .unwrap_or_default()
    }

    /// Gets a value indicating whether this value is signed, ie can be negative.
    pub fn is_signed(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> bool {
        match self.get_data_type(data_type_ref.get_data_type_id()) {
            Some(data_type) => data_type.is_signed(),
            None => false,
        }
    }

    /// Gets a value indicating whether this value is discrete, ie non-floating point.
    pub fn is_floating_point(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> bool {
        match self.get_data_type(data_type_ref.get_data_type_id()) {
            Some(data_type) => data_type.is_floating_point(),
            None => false,
        }
    }

    pub fn get_default_value(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> Option<DataValue> {
        match self.get_data_type(data_type_ref.get_data_type_id()) {
            Some(data_type) => Some(data_type.get_default_value(data_type_ref.clone())),
            None => None,
        }
    }

    pub fn get_scalar_compare_func_immediate(
        &self,
        scan_compare_type: &ScanCompareTypeImmediate,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnImmediate> {
        match self.get_data_type(scan_constraint.get_data_value().get_data_type_id()) {
            Some(data_type) => match scan_compare_type {
                ScanCompareTypeImmediate::Equal => data_type.get_compare_equal(scan_constraint),
                ScanCompareTypeImmediate::NotEqual => data_type.get_compare_not_equal(scan_constraint),
                ScanCompareTypeImmediate::GreaterThan => data_type.get_compare_greater_than(scan_constraint),
                ScanCompareTypeImmediate::GreaterThanOrEqual => data_type.get_compare_greater_than_or_equal(scan_constraint),
                ScanCompareTypeImmediate::LessThan => data_type.get_compare_less_than(scan_constraint),
                ScanCompareTypeImmediate::LessThanOrEqual => data_type.get_compare_less_than_or_equal(scan_constraint),
            },
            None => None,
        }
    }

    pub fn get_scalar_compare_func_relative(
        &self,
        scan_compare_type: &ScanCompareTypeRelative,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnRelative> {
        match self.get_data_type(scan_constraint.get_data_value().get_data_type_id()) {
            Some(data_type) => match scan_compare_type {
                ScanCompareTypeRelative::Changed => data_type.get_compare_changed(scan_constraint),
                ScanCompareTypeRelative::Unchanged => data_type.get_compare_unchanged(scan_constraint),
                ScanCompareTypeRelative::Increased => data_type.get_compare_increased(scan_constraint),
                ScanCompareTypeRelative::Decreased => data_type.get_compare_decreased(scan_constraint),
            },
            None => None,
        }
    }

    pub fn get_scalar_compare_func_delta(
        &self,
        scan_compare_type: &ScanCompareTypeDelta,
        scan_constraint: &ScanConstraint,
    ) -> Option<ScalarCompareFnRelative> {
        match self.get_data_type(scan_constraint.get_data_value().get_data_type_id()) {
            Some(data_type) => match scan_compare_type {
                ScanCompareTypeDelta::IncreasedByX => data_type.get_compare_increased_by(scan_constraint),
                ScanCompareTypeDelta::DecreasedByX => data_type.get_compare_decreased_by(scan_constraint),
                ScanCompareTypeDelta::MultipliedByX => data_type.get_compare_multiplied_by(scan_constraint),
                ScanCompareTypeDelta::DividedByX => data_type.get_compare_divided_by(scan_constraint),
                ScanCompareTypeDelta::ModuloByX => data_type.get_compare_modulo_by(scan_constraint),
                ScanCompareTypeDelta::ShiftLeftByX => data_type.get_compare_shift_left_by(scan_constraint),
                ScanCompareTypeDelta::ShiftRightByX => data_type.get_compare_shift_right_by(scan_constraint),
                ScanCompareTypeDelta::LogicalAndByX => data_type.get_compare_logical_and_by(scan_constraint),
                ScanCompareTypeDelta::LogicalOrByX => data_type.get_compare_logical_or_by(scan_constraint),
                ScanCompareTypeDelta::LogicalXorByX => data_type.get_compare_logical_xor_by(scan_constraint),
            },
            None => None,
        }
    }

    pub fn get_vector_compare_func_immediate<const N: usize>(
        &self,
        scan_compare_type_immediate: &ScanCompareTypeImmediate,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnImmediate<N>>
    where
        LaneCount<N>: SupportedLaneCount + VectorComparer<N> + GetVectorFunction<N>,
    {
        match self.get_data_type(scan_constraint.get_data_value().get_data_type_id()) {
            Some(data_type) => {
                <LaneCount<N> as VectorComparer<N>>::get_vector_compare_func_immediate(&data_type, &scan_compare_type_immediate, scan_constraint)
            }
            None => None,
        }
    }

    pub fn get_vector_compare_func_relative<const N: usize>(
        &self,
        scan_compare_type_relative: &ScanCompareTypeRelative,
        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnRelative<N>>
    where
        LaneCount<N>: SupportedLaneCount + VectorComparer<N> + GetVectorFunction<N>,
    {
        match self.get_data_type(scan_constraint.get_data_value().get_data_type_id()) {
            Some(data_type) => <LaneCount<N> as VectorComparer<N>>::get_vector_compare_func_relative(&data_type, &scan_compare_type_relative, scan_constraint),
            None => None,
        }
    }

    pub fn get_vector_compare_func_delta<const N: usize>(
        &self,
        scan_compare_type_delta: &ScanCompareTypeDelta,

        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta<N>>
    where
        LaneCount<N>: SupportedLaneCount + VectorComparer<N> + GetVectorFunction<N>,
    {
        match self.get_data_type(scan_constraint.get_data_value().get_data_type_id()) {
            Some(data_type) => <LaneCount<N> as VectorComparer<N>>::get_vector_compare_func_delta(&data_type, &scan_compare_type_delta, scan_constraint),
            None => None,
        }
    }

    fn create_built_in_registries() -> (HashMap<String, Arc<SymbolicStructDefinition>>, HashMap<String, Arc<dyn DataType>>) {
        let mut symbolic_struct_registry: HashMap<String, Arc<SymbolicStructDefinition>> = HashMap::new();
        let mut data_type_registry: HashMap<String, Arc<dyn DataType>> = HashMap::new();

        let built_in_data_types: Vec<Arc<dyn DataType>> = vec![
            Arc::new(DataTypeBool8 {}),
            Arc::new(DataTypeBool32 {}),
            Arc::new(DataTypeI8 {}),
            Arc::new(DataTypeI16 {}),
            Arc::new(DataTypeI16be {}),
            Arc::new(DataTypeI32 {}),
            Arc::new(DataTypeI32be {}),
            Arc::new(DataTypeI64 {}),
            Arc::new(DataTypeI64be {}),
            Arc::new(DataTypeU8 {}),
            Arc::new(DataTypeU16 {}),
            Arc::new(DataTypeU16be {}),
            Arc::new(DataTypeU32 {}),
            Arc::new(DataTypeU32be {}),
            Arc::new(DataTypeU64 {}),
            Arc::new(DataTypeU64be {}),
            Arc::new(DataTypeF32 {}),
            Arc::new(DataTypeF32be {}),
            Arc::new(DataTypeF64 {}),
            Arc::new(DataTypeF64be {}),
            Arc::new(DataTypeStringUtf8 {}),
        ];

        for built_in_data_type in built_in_data_types.into_iter() {
            let data_type_id = built_in_data_type.get_data_type_id().to_string();

            // Create a single field symbolic struct for every registered data type.
            symbolic_struct_registry.insert(
                data_type_id.clone(),
                Arc::new(SymbolicStructDefinition::new_anonymous(vec![SymbolicFieldDefinition::new(
                    DataTypeRef::new(&data_type_id),
                    ContainerType::None,
                )])),
            );
            data_type_registry.insert(data_type_id, built_in_data_type);
        }

        (symbolic_struct_registry, data_type_registry)
    }
}
