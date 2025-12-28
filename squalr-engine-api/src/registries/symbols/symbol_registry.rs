use crate::structures::scanning::plans::element_scan::snapshot_filter_element_scan_plan::SnapshotFilterElementScanPlan;
use crate::structures::structs::container_type::ContainerType;
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
    data_values::{
        anonymous_value::AnonymousValue, anonymous_value_container::AnonymousValueContainer, data_value::DataValue, display_value_type::DisplayValueType,
        display_values::DisplayValues,
    },
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
    simd::{LaneCount, Simd, SupportedLaneCount},
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
        data_type: Arc<dyn DataType>,
    ) {
        // JIRA
    }

    fn unregister_data_type(
        &mut self,
        data_type: Arc<dyn DataType>,
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

    pub fn validate_value(
        &self,
        data_type_ref: &DataTypeRef,
        anonymous_value: &AnonymousValue,
    ) -> bool {
        let anonymous_value_container = anonymous_value.get_value();

        match self.get_data_type(data_type_ref.get_data_type_id()) {
            Some(data_type) => {
                if !data_type.validate_value(anonymous_value_container) {
                    return false;
                }
            }
            None => return false,
        }

        true
    }

    pub fn deanonymize_value(
        &self,
        data_type_ref: &DataTypeRef,
        anonymous_value_container: &AnonymousValueContainer,
    ) -> Result<DataValue, String> {
        match self.get_data_type(data_type_ref.get_data_type_id()) {
            Some(data_type) => {
                let deanonymized_value = data_type.deanonymize_value(anonymous_value_container);

                match deanonymized_value {
                    Ok(value) => Ok(value),
                    Err(error) => Err(error.to_string()),
                }
            }
            None => Err("Cannot deanonymize value: data type is not registered.".into()),
        }
    }

    pub fn get_supported_display_value_types(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> Vec<DisplayValueType> {
        self.get_data_type(data_type_ref.get_data_type_id())
            .map(|data_type| data_type.get_supported_display_types())
            .unwrap_or_default()
    }

    pub fn get_default_display_type(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> DisplayValueType {
        self.get_data_type(data_type_ref.get_data_type_id())
            .map(|data_type| data_type.get_default_display_type())
            .unwrap_or_default()
    }

    pub fn create_display_values(
        &self,
        data_type_ref: &DataTypeRef,
        value_bytes: &[u8],
    ) -> DisplayValues {
        self.get_data_type(data_type_ref.get_data_type_id())
            .and_then(|data_type| data_type.create_display_values(value_bytes).ok())
            .unwrap_or_else(|| DisplayValues::new(vec![], DisplayValueType::String))
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
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnImmediate> {
        match self.get_data_type(
            snapshot_filter_element_scan_plan
                .get_data_value()
                .get_data_type_id(),
        ) {
            Some(data_type) => match scan_compare_type {
                ScanCompareTypeImmediate::Equal => data_type.get_compare_equal(snapshot_filter_element_scan_plan),
                ScanCompareTypeImmediate::NotEqual => data_type.get_compare_not_equal(snapshot_filter_element_scan_plan),
                ScanCompareTypeImmediate::GreaterThan => data_type.get_compare_greater_than(snapshot_filter_element_scan_plan),
                ScanCompareTypeImmediate::GreaterThanOrEqual => data_type.get_compare_greater_than_or_equal(snapshot_filter_element_scan_plan),
                ScanCompareTypeImmediate::LessThan => data_type.get_compare_less_than(snapshot_filter_element_scan_plan),
                ScanCompareTypeImmediate::LessThanOrEqual => data_type.get_compare_less_than_or_equal(snapshot_filter_element_scan_plan),
            },
            None => None,
        }
    }

    pub fn get_scalar_compare_func_relative(
        &self,
        scan_compare_type: &ScanCompareTypeRelative,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnRelative> {
        match self.get_data_type(
            snapshot_filter_element_scan_plan
                .get_data_value()
                .get_data_type_id(),
        ) {
            Some(data_type) => match scan_compare_type {
                ScanCompareTypeRelative::Changed => data_type.get_compare_changed(snapshot_filter_element_scan_plan),
                ScanCompareTypeRelative::Unchanged => data_type.get_compare_unchanged(snapshot_filter_element_scan_plan),
                ScanCompareTypeRelative::Increased => data_type.get_compare_increased(snapshot_filter_element_scan_plan),
                ScanCompareTypeRelative::Decreased => data_type.get_compare_decreased(snapshot_filter_element_scan_plan),
            },
            None => None,
        }
    }

    pub fn get_scalar_compare_func_delta(
        &self,
        scan_compare_type: &ScanCompareTypeDelta,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<ScalarCompareFnRelative> {
        match self.get_data_type(
            snapshot_filter_element_scan_plan
                .get_data_value()
                .get_data_type_id(),
        ) {
            Some(data_type) => match scan_compare_type {
                ScanCompareTypeDelta::IncreasedByX => data_type.get_compare_increased_by(snapshot_filter_element_scan_plan),
                ScanCompareTypeDelta::DecreasedByX => data_type.get_compare_decreased_by(snapshot_filter_element_scan_plan),
                ScanCompareTypeDelta::MultipliedByX => data_type.get_compare_multiplied_by(snapshot_filter_element_scan_plan),
                ScanCompareTypeDelta::DividedByX => data_type.get_compare_divided_by(snapshot_filter_element_scan_plan),
                ScanCompareTypeDelta::ModuloByX => data_type.get_compare_modulo_by(snapshot_filter_element_scan_plan),
                ScanCompareTypeDelta::ShiftLeftByX => data_type.get_compare_shift_left_by(snapshot_filter_element_scan_plan),
                ScanCompareTypeDelta::ShiftRightByX => data_type.get_compare_shift_right_by(snapshot_filter_element_scan_plan),
                ScanCompareTypeDelta::LogicalAndByX => data_type.get_compare_logical_and_by(snapshot_filter_element_scan_plan),
                ScanCompareTypeDelta::LogicalOrByX => data_type.get_compare_logical_or_by(snapshot_filter_element_scan_plan),
                ScanCompareTypeDelta::LogicalXorByX => data_type.get_compare_logical_xor_by(snapshot_filter_element_scan_plan),
            },
            None => None,
        }
    }

    pub fn get_vector_compare_func_immediate<const N: usize>(
        &self,
        scan_compare_type_immediate: &ScanCompareTypeImmediate,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<Box<dyn Fn(*const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
    {
        match self.get_data_type(
            snapshot_filter_element_scan_plan
                .get_data_value()
                .get_data_type_id(),
        ) {
            Some(data_type) => <LaneCount<N> as VectorComparer<N>>::get_vector_compare_func_immediate(
                &data_type,
                &scan_compare_type_immediate,
                snapshot_filter_element_scan_plan,
            ),
            None => None,
        }
    }

    pub fn get_vector_compare_func_relative<const N: usize>(
        &self,
        scan_compare_type_relative: &ScanCompareTypeRelative,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
    {
        match self.get_data_type(
            snapshot_filter_element_scan_plan
                .get_data_value()
                .get_data_type_id(),
        ) {
            Some(data_type) => <LaneCount<N> as VectorComparer<N>>::get_vector_compare_func_relative(
                &data_type,
                &scan_compare_type_relative,
                snapshot_filter_element_scan_plan,
            ),
            None => None,
        }
    }

    pub fn get_vector_compare_func_delta<const N: usize>(
        &self,
        scan_compare_type_delta: &ScanCompareTypeDelta,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
    {
        match self.get_data_type(
            snapshot_filter_element_scan_plan
                .get_data_value()
                .get_data_type_id(),
        ) {
            Some(data_type) => {
                <LaneCount<N> as VectorComparer<N>>::get_vector_compare_func_delta(&data_type, &scan_compare_type_delta, snapshot_filter_element_scan_plan)
            }
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
