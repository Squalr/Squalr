use crate::registries::symbols::symbol_registry_error::SymbolRegistryError;
use crate::registries::symbols::{
    data_type_descriptor::DataTypeDescriptor, privileged_registry_catalog::PrivilegedRegistryCatalog, struct_layout_descriptor::StructLayoutDescriptor,
};
use crate::structures::data_types::generics::vector_function::GetVectorFunction;
use crate::structures::data_values::container_type::ContainerType;
use crate::structures::scanning::comparisons::scan_function_vector::{VectorCompareFnDelta, VectorCompareFnImmediate, VectorCompareFnRelative};
use crate::structures::scanning::constraints::scan_constraint::ScanConstraint;
use crate::structures::structs::symbol_resolver::SymbolResolver;
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
        data_type_scan_preference::DataTypeScanPreference,
        floating_point_tolerance::FloatingPointTolerance,
        generics::vector_comparer::VectorComparer,
        generics::vector_lane_count::VectorLaneCount,
    },
    data_values::{anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, data_value::DataValue},
    scanning::comparisons::{
        scan_compare_type::ScanCompareType,
        scan_compare_type_delta::ScanCompareTypeDelta,
        scan_compare_type_immediate::ScanCompareTypeImmediate,
        scan_compare_type_relative::ScanCompareTypeRelative,
        scan_function_scalar::{ScalarCompareFnImmediate, ScalarCompareFnRelative},
    },
    scanning::constraints::{anonymous_scan_constraint::AnonymousScanConstraint, scan_constraint_builder::ScanConstraintBuilder},
};
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    sync::{Arc, RwLock},
};

/// Manages a symbolic struct registry and a data type registry. All registered data types are also registered into the symbolic struct
/// registry, since each data type is considered to be a symbol. The struct contains a single anonymous field for the corresponding type.
pub struct SymbolRegistry {
    symbolic_struct_registry: RwLock<HashMap<String, Arc<SymbolicStructDefinition>>>,
    project_symbolic_struct_registry: RwLock<HashMap<String, Arc<SymbolicStructDefinition>>>,
    data_type_registry: RwLock<HashMap<String, Arc<dyn DataType>>>,
    built_in_data_type_ids: HashSet<String>,
    data_type_descriptor_registry: RwLock<HashMap<String, Arc<DataTypeDescriptor>>>,
}

impl SymbolRegistry {
    pub fn new() -> Self {
        let (symbolic_struct_registry, data_type_registry, data_type_descriptor_registry) = Self::create_built_in_registries();
        let built_in_data_type_ids = data_type_registry.keys().cloned().collect();

        Self {
            symbolic_struct_registry: RwLock::new(symbolic_struct_registry),
            project_symbolic_struct_registry: RwLock::new(HashMap::new()),
            data_type_registry: RwLock::new(data_type_registry),
            built_in_data_type_ids,
            data_type_descriptor_registry: RwLock::new(data_type_descriptor_registry),
        }
    }

    pub fn get_registry(&self) -> HashMap<String, Arc<SymbolicStructDefinition>> {
        let mut symbolic_struct_registry = self
            .symbolic_struct_registry
            .read()
            .map(|symbolic_struct_registry| symbolic_struct_registry.clone())
            .unwrap_or_default();

        if let Ok(project_symbolic_struct_registry) = self.project_symbolic_struct_registry.read() {
            symbolic_struct_registry.extend(project_symbolic_struct_registry.clone());
        }

        symbolic_struct_registry
    }

    pub fn get(
        &self,
        symbolic_struct_ref_id: &str,
    ) -> Option<Arc<SymbolicStructDefinition>> {
        match self.symbolic_struct_registry.read() {
            Ok(symbolic_struct_registry) => {
                if let Some(symbolic_struct_definition) = symbolic_struct_registry.get(symbolic_struct_ref_id.trim()) {
                    Some(symbolic_struct_definition.clone())
                } else if let Ok(project_symbolic_struct_registry) = self.project_symbolic_struct_registry.read() {
                    project_symbolic_struct_registry
                        .get(symbolic_struct_ref_id.trim())
                        .cloned()
                        .or_else(|| {
                            SymbolicStructDefinition::from_str(symbolic_struct_ref_id.trim())
                                .ok()
                                .map(Arc::new)
                        })
                } else {
                    SymbolicStructDefinition::from_str(symbolic_struct_ref_id.trim())
                        .ok()
                        .map(Arc::new)
                }
            }
            Err(error) => {
                log::error!("Failed to acquire symbol registry read lock: {}", error);
                None
            }
        }
    }

    pub fn get_data_type_registry(&self) -> HashMap<String, Arc<dyn DataType>> {
        self.data_type_registry
            .read()
            .map(|data_type_registry| data_type_registry.clone())
            .unwrap_or_default()
    }

    pub fn get_data_type(
        &self,
        data_type_id: &str,
    ) -> Option<Arc<dyn DataType>> {
        match self.data_type_registry.read() {
            Ok(data_type_registry) => {
                if let Some(data_type) = data_type_registry.get(data_type_id.trim()) {
                    Some(data_type.clone())
                } else {
                    if self.get_data_type_descriptor(data_type_id).is_none() {
                        log::warn!("Failed to find data type in registry: {}", data_type_id);
                    }

                    None
                }
            }
            Err(error) => {
                log::error!("Failed to acquire data type registry read lock: {}", error);
                None
            }
        }
    }

    pub fn register_data_type(
        &self,
        data_type: Arc<dyn DataType>,
    ) -> bool {
        let data_type_id = data_type.get_data_type_id().trim().to_string();

        if self.built_in_data_type_ids.contains(&data_type_id) {
            log::warn!("Refusing to overwrite built-in data type: {}", data_type_id);
            return false;
        }

        match self.data_type_registry.write() {
            Ok(mut data_type_registry) => {
                if data_type_registry.contains_key(&data_type_id) {
                    log::warn!("Refusing to overwrite registered data type: {}", data_type_id);
                    return false;
                }

                data_type_registry.insert(data_type_id.clone(), data_type.clone());
            }
            Err(error) => {
                log::error!("Failed to acquire data type registry write lock: {}", error);
                return false;
            }
        }

        let descriptor_inserted = match self.data_type_descriptor_registry.write() {
            Ok(mut data_type_descriptor_registry) => {
                data_type_descriptor_registry.insert(data_type_id.clone(), Arc::new(DataTypeDescriptor::from_data_type(data_type.as_ref())));
                true
            }
            Err(error) => {
                log::error!("Failed to acquire data type descriptor registry write lock: {}", error);
                false
            }
        };

        if !descriptor_inserted {
            if let Ok(mut data_type_registry) = self.data_type_registry.write() {
                data_type_registry.remove(&data_type_id);
            }

            return false;
        }

        match self.symbolic_struct_registry.write() {
            Ok(mut symbolic_struct_registry) => {
                symbolic_struct_registry.insert(data_type_id.clone(), Arc::new(Self::create_anonymous_data_type_symbolic_struct(&data_type_id)));
                true
            }
            Err(error) => {
                log::error!("Failed to acquire symbol registry write lock: {}", error);

                if let Ok(mut data_type_registry) = self.data_type_registry.write() {
                    data_type_registry.remove(&data_type_id);
                }

                if let Ok(mut data_type_descriptor_registry) = self.data_type_descriptor_registry.write() {
                    data_type_descriptor_registry.remove(&data_type_id);
                }

                false
            }
        }
    }

    pub fn unregister_data_type(
        &self,
        data_type_id: &str,
    ) -> bool {
        let trimmed_data_type_id = data_type_id.trim();

        if self.built_in_data_type_ids.contains(trimmed_data_type_id) {
            log::warn!("Refusing to unregister built-in data type: {}", data_type_id);
            return false;
        }

        let data_type_removed = match self.data_type_registry.write() {
            Ok(mut data_type_registry) => data_type_registry.remove(trimmed_data_type_id).is_some(),
            Err(error) => {
                log::error!("Failed to acquire data type registry write lock: {}", error);
                false
            }
        };

        if !data_type_removed {
            return false;
        }

        if let Ok(mut data_type_descriptor_registry) = self.data_type_descriptor_registry.write() {
            data_type_descriptor_registry.remove(trimmed_data_type_id);
        } else {
            log::error!(
                "Failed to acquire data type descriptor registry write lock while unregistering data type: {}",
                data_type_id
            );
        }

        if let Ok(mut symbolic_struct_registry) = self.symbolic_struct_registry.write() {
            symbolic_struct_registry.remove(trimmed_data_type_id);
        } else {
            log::error!("Failed to acquire symbol registry write lock while unregistering data type: {}", data_type_id);
        }

        true
    }

    pub fn get_registered_data_type_refs(&self) -> Vec<DataTypeRef> {
        let mut data_type_refs = self
            .data_type_descriptor_registry
            .read()
            .map(|data_type_descriptor_registry| {
                data_type_descriptor_registry
                    .keys()
                    .map(|data_type_id| DataTypeRef::new(data_type_id))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        data_type_refs.sort_by(|left_data_type_ref, right_data_type_ref| {
            left_data_type_ref
                .get_data_type_id()
                .cmp(right_data_type_ref.get_data_type_id())
        });

        data_type_refs
    }

    pub fn register_data_type_descriptor(
        &self,
        data_type_descriptor: DataTypeDescriptor,
    ) -> bool {
        let data_type_id = data_type_descriptor.get_data_type_id().to_string();

        match self.data_type_registry.read() {
            Ok(data_type_registry) => {
                if data_type_registry.contains_key(&data_type_id) {
                    log::warn!("Refusing to overwrite registered data type descriptor: {}", data_type_id);
                    return false;
                }
            }
            Err(error) => {
                log::error!("Failed to acquire data type registry read lock: {}", error);
                return false;
            }
        }

        let descriptor_inserted = match self.data_type_descriptor_registry.write() {
            Ok(mut data_type_descriptor_registry) => {
                data_type_descriptor_registry.insert(data_type_id.clone(), Arc::new(data_type_descriptor));
                true
            }
            Err(error) => {
                log::error!("Failed to acquire data type descriptor registry write lock: {}", error);
                false
            }
        };

        if !descriptor_inserted {
            return false;
        }

        match self.symbolic_struct_registry.write() {
            Ok(mut symbolic_struct_registry) => {
                symbolic_struct_registry.insert(data_type_id.clone(), Arc::new(Self::create_anonymous_data_type_symbolic_struct(&data_type_id)));
                true
            }
            Err(error) => {
                log::error!("Failed to acquire symbol registry write lock: {}", error);
                false
            }
        }
    }

    pub fn unregister_data_type_descriptor(
        &self,
        data_type_id: &str,
    ) -> bool {
        match self.data_type_registry.read() {
            Ok(data_type_registry) => {
                if data_type_registry.contains_key(data_type_id.trim()) {
                    log::warn!("Refusing to unregister registered data type descriptor: {}", data_type_id);
                    return false;
                }
            }
            Err(error) => {
                log::error!("Failed to acquire data type registry read lock: {}", error);
                return false;
            }
        }

        let descriptor_removed = match self.data_type_descriptor_registry.write() {
            Ok(mut data_type_descriptor_registry) => data_type_descriptor_registry
                .remove(data_type_id.trim())
                .is_some(),
            Err(error) => {
                log::error!("Failed to acquire data type descriptor registry write lock: {}", error);
                false
            }
        };

        if !descriptor_removed {
            return false;
        }

        match self.symbolic_struct_registry.write() {
            Ok(mut symbolic_struct_registry) => {
                symbolic_struct_registry.remove(data_type_id.trim());
                true
            }
            Err(error) => {
                log::error!("Failed to acquire symbol registry write lock: {}", error);
                false
            }
        }
    }

    pub fn register_symbolic_struct(
        &self,
        symbolic_struct_id: String,
        symbolic_struct_definition: SymbolicStructDefinition,
    ) -> bool {
        match self.data_type_registry.read() {
            Ok(data_type_registry) => {
                if data_type_registry.contains_key(symbolic_struct_id.trim()) {
                    log::warn!("Refusing to overwrite registered data type symbolic struct: {}", symbolic_struct_id);
                    return false;
                }
            }
            Err(error) => {
                log::error!("Failed to acquire data type registry read lock: {}", error);
                return false;
            }
        }

        match self.symbolic_struct_registry.write() {
            Ok(mut symbolic_struct_registry) => {
                symbolic_struct_registry.insert(symbolic_struct_id, Arc::new(symbolic_struct_definition));
                true
            }
            Err(error) => {
                log::error!("Failed to acquire symbol registry write lock: {}", error);
                false
            }
        }
    }

    pub fn unregister_symbolic_struct(
        &self,
        symbolic_struct_id: &str,
    ) -> bool {
        match self.data_type_registry.read() {
            Ok(data_type_registry) => {
                if data_type_registry.contains_key(symbolic_struct_id.trim()) {
                    log::warn!("Refusing to unregister registered data type symbolic struct: {}", symbolic_struct_id);
                    return false;
                }
            }
            Err(error) => {
                log::error!("Failed to acquire data type registry read lock: {}", error);
                return false;
            }
        }

        match self.symbolic_struct_registry.write() {
            Ok(mut symbolic_struct_registry) => symbolic_struct_registry
                .remove(symbolic_struct_id.trim())
                .is_some(),
            Err(error) => {
                log::error!("Failed to acquire symbol registry write lock: {}", error);
                false
            }
        }
    }

    pub fn set_project_symbol_catalog(
        &self,
        project_struct_layout_descriptors: &[StructLayoutDescriptor],
    ) -> bool {
        let registered_data_type_ids = self
            .data_type_registry
            .read()
            .map(|data_type_registry| data_type_registry.keys().cloned().collect::<HashSet<_>>())
            .unwrap_or_default();
        let base_symbolic_struct_ids = self
            .symbolic_struct_registry
            .read()
            .map(|symbolic_struct_registry| symbolic_struct_registry.keys().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        let mut next_project_symbolic_struct_registry = HashMap::new();

        for struct_layout_descriptor in project_struct_layout_descriptors {
            let symbolic_struct_id = struct_layout_descriptor.get_struct_layout_id().trim();

            if symbolic_struct_id.is_empty() {
                log::warn!("Ignoring empty project-authored symbolic struct id.");
                continue;
            }

            if registered_data_type_ids.contains(symbolic_struct_id)
                || base_symbolic_struct_ids
                    .iter()
                    .any(|registered_symbolic_struct_id| registered_symbolic_struct_id == symbolic_struct_id)
            {
                log::warn!(
                    "Ignoring project-authored symbol that collides with privileged symbol registry entry: {}",
                    symbolic_struct_id
                );
                continue;
            }

            next_project_symbolic_struct_registry.insert(
                symbolic_struct_id.to_string(),
                Arc::new(struct_layout_descriptor.get_struct_layout_definition().clone()),
            );
        }

        match self.project_symbolic_struct_registry.write() {
            Ok(mut project_symbolic_struct_registry) => {
                *project_symbolic_struct_registry = next_project_symbolic_struct_registry;
                true
            }
            Err(error) => {
                log::error!("Failed to acquire project symbol registry write lock: {}", error);
                false
            }
        }
    }

    pub fn create_registry_catalog(
        &self,
        generation: u64,
    ) -> PrivilegedRegistryCatalog {
        let mut data_type_descriptors = self
            .data_type_descriptor_registry
            .read()
            .map(|data_type_descriptor_registry| {
                data_type_descriptor_registry
                    .values()
                    .map(|data_type_descriptor| data_type_descriptor.as_ref().clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        data_type_descriptors.sort_by(|left_descriptor, right_descriptor| {
            left_descriptor
                .get_data_type_id()
                .cmp(right_descriptor.get_data_type_id())
        });

        let mut struct_layout_descriptors = self
            .get_registry()
            .iter()
            .map(|(symbolic_struct_id, symbolic_struct_definition)| {
                StructLayoutDescriptor::new(symbolic_struct_id.clone(), symbolic_struct_definition.as_ref().clone())
            })
            .collect::<Vec<_>>();
        struct_layout_descriptors.sort_by(|left_descriptor, right_descriptor| {
            left_descriptor
                .get_struct_layout_id()
                .cmp(right_descriptor.get_struct_layout_id())
        });

        PrivilegedRegistryCatalog::new(generation, data_type_descriptors, struct_layout_descriptors)
    }

    pub fn apply_registry_catalog(
        &self,
        privileged_registry_catalog: &PrivilegedRegistryCatalog,
    ) {
        let base_symbolic_struct_ids = self
            .symbolic_struct_registry
            .read()
            .map(|symbolic_struct_registry| symbolic_struct_registry.keys().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        let project_symbolic_struct_registry = privileged_registry_catalog
            .get_struct_layout_descriptors()
            .iter()
            .filter(|symbolic_struct_descriptor| {
                !base_symbolic_struct_ids
                    .iter()
                    .any(|registered_symbolic_struct_id| registered_symbolic_struct_id == symbolic_struct_descriptor.get_struct_layout_id())
            })
            .map(|symbolic_struct_descriptor| {
                (
                    symbolic_struct_descriptor.get_struct_layout_id().to_string(),
                    Arc::new(
                        symbolic_struct_descriptor
                            .get_struct_layout_definition()
                            .clone(),
                    ),
                )
            })
            .collect::<HashMap<_, _>>();
        let data_type_descriptor_registry = privileged_registry_catalog
            .get_data_type_descriptors()
            .iter()
            .map(|data_type_descriptor| (data_type_descriptor.get_data_type_id().to_string(), Arc::new(data_type_descriptor.clone())))
            .collect::<HashMap<_, _>>();

        if let Ok(mut project_symbolic_struct_registry_guard) = self.project_symbolic_struct_registry.write() {
            *project_symbolic_struct_registry_guard = project_symbolic_struct_registry;
        } else {
            log::error!("Failed to acquire project symbol registry write lock while applying privileged registry catalog.");
        }

        if let Ok(mut data_type_descriptor_registry_guard) = self.data_type_descriptor_registry.write() {
            *data_type_descriptor_registry_guard = data_type_descriptor_registry;
        } else {
            log::error!("Failed to acquire data type descriptor registry write lock while applying privileged registry catalog.");
        }
    }

    fn get_data_type_descriptor(
        &self,
        data_type_id: &str,
    ) -> Option<Arc<DataTypeDescriptor>> {
        match self.data_type_descriptor_registry.read() {
            Ok(data_type_descriptor_registry) => data_type_descriptor_registry.get(data_type_id.trim()).cloned(),
            Err(error) => {
                log::error!("Failed to acquire data type descriptor registry read lock: {}", error);
                None
            }
        }
    }

    /// Determines if the `DataType` this struct represents is currently registered and available.
    pub fn is_valid(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> bool {
        self.get_data_type(data_type_ref.get_data_type_id()).is_some()
            || self
                .get_data_type_descriptor(data_type_ref.get_data_type_id())
                .is_some()
    }

    pub fn get_icon_id(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> String {
        match self.get_data_type(data_type_ref.get_data_type_id()) {
            Some(data_type) => data_type.get_icon_id().to_string(),
            None => self
                .get_data_type_descriptor(data_type_ref.get_data_type_id())
                .map(|data_type_descriptor| data_type_descriptor.get_icon_id().to_string())
                .unwrap_or_default(),
        }
    }

    pub fn get_unit_size_in_bytes(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> u64 {
        match self.get_data_type(data_type_ref.get_data_type_id()) {
            Some(data_type) => data_type.get_unit_size_in_bytes(),
            None => self
                .get_data_type_descriptor(data_type_ref.get_data_type_id())
                .map(|data_type_descriptor| data_type_descriptor.get_unit_size_in_bytes())
                .unwrap_or_default(),
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

    pub fn validate_scan_constraint(
        &self,
        data_type_ref: &DataTypeRef,
        scan_compare_type: ScanCompareType,
        anonymous_value_string: &AnonymousValueString,
    ) -> bool {
        let scan_constraint_builder = ScanConstraintBuilder::new(self, FloatingPointTolerance::default());
        let anonymous_scan_constraint = AnonymousScanConstraint::new(scan_compare_type, Some(anonymous_value_string.clone()));

        scan_constraint_builder
            .build(&anonymous_scan_constraint, data_type_ref)
            .is_ok_and(|scan_constraint| scan_constraint.is_some())
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
            .or_else(|| {
                self.get_data_type_descriptor(data_type_ref.get_data_type_id())
                    .map(|data_type_descriptor| {
                        data_type_descriptor
                            .get_supported_anonymous_value_string_formats()
                            .to_vec()
                    })
            })
            .unwrap_or_default()
    }

    pub fn get_default_anonymous_value_string_format(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> AnonymousValueStringFormat {
        self.get_data_type(data_type_ref.get_data_type_id())
            .map(|data_type| data_type.get_default_anonymous_value_string_format())
            .or_else(|| {
                self.get_data_type_descriptor(data_type_ref.get_data_type_id())
                    .map(|data_type_descriptor| data_type_descriptor.get_default_anonymous_value_string_format())
            })
            .unwrap_or_default()
    }

    /// Gets a value indicating whether this value is signed, ie can be negative.
    pub fn is_signed(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> bool {
        match self.get_data_type(data_type_ref.get_data_type_id()) {
            Some(data_type) => data_type.is_signed(),
            None => self
                .get_data_type_descriptor(data_type_ref.get_data_type_id())
                .map(|data_type_descriptor| data_type_descriptor.get_is_signed())
                .unwrap_or(false),
        }
    }

    /// Gets a value indicating whether this value is discrete, ie non-floating point.
    pub fn is_floating_point(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> bool {
        match self.get_data_type(data_type_ref.get_data_type_id()) {
            Some(data_type) => data_type.is_floating_point(),
            None => self
                .get_data_type_descriptor(data_type_ref.get_data_type_id())
                .map(|data_type_descriptor| data_type_descriptor.get_is_floating_point())
                .unwrap_or(false),
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

    pub fn get_scan_preference(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> DataTypeScanPreference {
        match self.get_data_type(data_type_ref.get_data_type_id()) {
            Some(data_type) => data_type.get_scan_preference(),
            None => DataTypeScanPreference::UseDefault,
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
        VectorLaneCount<N>: VectorComparer<N> + GetVectorFunction<N>,
    {
        match self.get_data_type(scan_constraint.get_data_value().get_data_type_id()) {
            Some(data_type) => {
                <VectorLaneCount<N> as VectorComparer<N>>::get_vector_compare_func_immediate(&data_type, &scan_compare_type_immediate, scan_constraint)
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
        VectorLaneCount<N>: VectorComparer<N> + GetVectorFunction<N>,
    {
        match self.get_data_type(scan_constraint.get_data_value().get_data_type_id()) {
            Some(data_type) => {
                <VectorLaneCount<N> as VectorComparer<N>>::get_vector_compare_func_relative(&data_type, &scan_compare_type_relative, scan_constraint)
            }
            None => None,
        }
    }

    pub fn get_vector_compare_func_delta<const N: usize>(
        &self,
        scan_compare_type_delta: &ScanCompareTypeDelta,

        scan_constraint: &ScanConstraint,
    ) -> Option<VectorCompareFnDelta<N>>
    where
        VectorLaneCount<N>: VectorComparer<N> + GetVectorFunction<N>,
    {
        match self.get_data_type(scan_constraint.get_data_value().get_data_type_id()) {
            Some(data_type) => <VectorLaneCount<N> as VectorComparer<N>>::get_vector_compare_func_delta(&data_type, &scan_compare_type_delta, scan_constraint),
            None => None,
        }
    }

    fn create_built_in_registries() -> (
        HashMap<String, Arc<SymbolicStructDefinition>>,
        HashMap<String, Arc<dyn DataType>>,
        HashMap<String, Arc<DataTypeDescriptor>>,
    ) {
        let mut symbolic_struct_registry: HashMap<String, Arc<SymbolicStructDefinition>> = HashMap::new();
        let mut data_type_registry: HashMap<String, Arc<dyn DataType>> = HashMap::new();
        let mut data_type_descriptor_registry: HashMap<String, Arc<DataTypeDescriptor>> = HashMap::new();

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
            let built_in_data_type_descriptor = Arc::new(DataTypeDescriptor::from_data_type(built_in_data_type.as_ref()));

            // Create a single field symbolic struct for every registered data type.
            symbolic_struct_registry.insert(
                data_type_id.clone(),
                Arc::new(SymbolicStructDefinition::new_anonymous(vec![SymbolicFieldDefinition::new(
                    DataTypeRef::new(&data_type_id),
                    ContainerType::None,
                )])),
            );
            data_type_registry.insert(data_type_id, built_in_data_type);
            data_type_descriptor_registry.insert(built_in_data_type_descriptor.get_data_type_id().to_string(), built_in_data_type_descriptor);
        }

        (symbolic_struct_registry, data_type_registry, data_type_descriptor_registry)
    }

    fn create_anonymous_data_type_symbolic_struct(data_type_id: &str) -> SymbolicStructDefinition {
        SymbolicStructDefinition::new_anonymous(vec![SymbolicFieldDefinition::new(
            DataTypeRef::new(data_type_id),
            ContainerType::None,
        )])
    }
}

impl SymbolResolver for SymbolRegistry {
    fn get_default_value(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> Option<DataValue> {
        self.get_default_value(data_type_ref)
    }

    fn get_unit_size_in_bytes(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> u64 {
        self.get_unit_size_in_bytes(data_type_ref)
    }

    fn get_struct_layout(
        &self,
        symbolic_struct_namespace: &str,
    ) -> Option<Arc<SymbolicStructDefinition>> {
        self.get(symbolic_struct_namespace)
    }
}

#[cfg(test)]
mod tests {
    use super::SymbolRegistry;
    use crate::registries::symbols::{data_type_descriptor::DataTypeDescriptor, struct_layout_descriptor::StructLayoutDescriptor};
    use crate::structures::{
        data_types::{
            comparisons::{scalar_comparable::ScalarComparable, vector_comparable::VectorComparable},
            data_type::DataType,
            data_type_error::DataTypeError,
            data_type_ref::DataTypeRef,
        },
        data_values::{anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType},
        memory::endian::Endian,
        structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition},
    };
    use std::sync::Arc;

    #[derive(Clone, Debug)]
    struct TestRuntimeDataType;

    impl ScalarComparable for TestRuntimeDataType {
        fn get_compare_equal(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnImmediate> {
            None
        }
        fn get_compare_not_equal(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnImmediate> {
            None
        }
        fn get_compare_greater_than(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnImmediate> {
            None
        }
        fn get_compare_greater_than_or_equal(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnImmediate> {
            None
        }
        fn get_compare_less_than(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnImmediate> {
            None
        }
        fn get_compare_less_than_or_equal(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnImmediate> {
            None
        }
        fn get_compare_changed(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnRelative> {
            None
        }
        fn get_compare_unchanged(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnRelative> {
            None
        }
        fn get_compare_increased(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnRelative> {
            None
        }
        fn get_compare_decreased(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnRelative> {
            None
        }
        fn get_compare_increased_by(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnDelta> {
            None
        }
        fn get_compare_decreased_by(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnDelta> {
            None
        }
        fn get_compare_multiplied_by(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnDelta> {
            None
        }
        fn get_compare_divided_by(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnDelta> {
            None
        }
        fn get_compare_modulo_by(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnDelta> {
            None
        }
        fn get_compare_shift_left_by(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnDelta> {
            None
        }
        fn get_compare_shift_right_by(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnDelta> {
            None
        }
        fn get_compare_logical_and_by(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnDelta> {
            None
        }
        fn get_compare_logical_or_by(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnDelta> {
            None
        }
        fn get_compare_logical_xor_by(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_scalar::ScalarCompareFnDelta> {
            None
        }
    }

    impl VectorComparable for TestRuntimeDataType {
        fn get_vector_compare_equal_64(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate64> {
            None
        }
        fn get_vector_compare_equal_32(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate32> {
            None
        }
        fn get_vector_compare_equal_16(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate16> {
            None
        }
        fn get_vector_compare_not_equal_64(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate64> {
            None
        }
        fn get_vector_compare_not_equal_32(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate32> {
            None
        }
        fn get_vector_compare_not_equal_16(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate16> {
            None
        }
        fn get_vector_compare_greater_than_64(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate64> {
            None
        }
        fn get_vector_compare_greater_than_32(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate32> {
            None
        }
        fn get_vector_compare_greater_than_16(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate16> {
            None
        }
        fn get_vector_compare_greater_than_or_equal_64(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate64> {
            None
        }
        fn get_vector_compare_greater_than_or_equal_32(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate32> {
            None
        }
        fn get_vector_compare_greater_than_or_equal_16(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate16> {
            None
        }
        fn get_vector_compare_less_than_64(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate64> {
            None
        }
        fn get_vector_compare_less_than_32(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate32> {
            None
        }
        fn get_vector_compare_less_than_16(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate16> {
            None
        }
        fn get_vector_compare_less_than_or_equal_64(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate64> {
            None
        }
        fn get_vector_compare_less_than_or_equal_32(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate32> {
            None
        }
        fn get_vector_compare_less_than_or_equal_16(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnImmediate16> {
            None
        }
        fn get_vector_compare_changed_64(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnRelative64> {
            None
        }
        fn get_vector_compare_changed_32(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnRelative32> {
            None
        }
        fn get_vector_compare_changed_16(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnRelative16> {
            None
        }
        fn get_vector_compare_unchanged_64(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnRelative64> {
            None
        }
        fn get_vector_compare_unchanged_32(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnRelative32> {
            None
        }
        fn get_vector_compare_unchanged_16(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnRelative16> {
            None
        }
        fn get_vector_compare_increased_64(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnRelative64> {
            None
        }
        fn get_vector_compare_increased_32(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnRelative32> {
            None
        }
        fn get_vector_compare_increased_16(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnRelative16> {
            None
        }
        fn get_vector_compare_decreased_64(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnRelative64> {
            None
        }
        fn get_vector_compare_decreased_32(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnRelative32> {
            None
        }
        fn get_vector_compare_decreased_16(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnRelative16> {
            None
        }
        fn get_vector_compare_increased_by_64(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta64> {
            None
        }
        fn get_vector_compare_increased_by_32(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta32> {
            None
        }
        fn get_vector_compare_increased_by_16(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta16> {
            None
        }
        fn get_vector_compare_decreased_by_64(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta64> {
            None
        }
        fn get_vector_compare_decreased_by_32(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta32> {
            None
        }
        fn get_vector_compare_decreased_by_16(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta16> {
            None
        }
        fn get_vector_compare_multiplied_by_64(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta64> {
            None
        }
        fn get_vector_compare_multiplied_by_32(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta32> {
            None
        }
        fn get_vector_compare_multiplied_by_16(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta16> {
            None
        }
        fn get_vector_compare_divided_by_64(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta64> {
            None
        }
        fn get_vector_compare_divided_by_32(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta32> {
            None
        }
        fn get_vector_compare_divided_by_16(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta16> {
            None
        }
        fn get_vector_compare_modulo_by_64(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta64> {
            None
        }
        fn get_vector_compare_modulo_by_32(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta32> {
            None
        }
        fn get_vector_compare_modulo_by_16(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta16> {
            None
        }
        fn get_vector_compare_shift_left_by_64(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta64> {
            None
        }
        fn get_vector_compare_shift_left_by_32(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta32> {
            None
        }
        fn get_vector_compare_shift_left_by_16(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta16> {
            None
        }
        fn get_vector_compare_shift_right_by_64(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta64> {
            None
        }
        fn get_vector_compare_shift_right_by_32(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta32> {
            None
        }
        fn get_vector_compare_shift_right_by_16(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta16> {
            None
        }
        fn get_vector_compare_logical_and_by_64(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta64> {
            None
        }
        fn get_vector_compare_logical_and_by_32(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta32> {
            None
        }
        fn get_vector_compare_logical_and_by_16(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta16> {
            None
        }
        fn get_vector_compare_logical_or_by_64(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta64> {
            None
        }
        fn get_vector_compare_logical_or_by_32(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta32> {
            None
        }
        fn get_vector_compare_logical_or_by_16(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta16> {
            None
        }
        fn get_vector_compare_logical_xor_by_64(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta64> {
            None
        }
        fn get_vector_compare_logical_xor_by_32(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta32> {
            None
        }
        fn get_vector_compare_logical_xor_by_16(
            &self,
            _scan_constraint: &crate::structures::scanning::constraints::scan_constraint::ScanConstraint,
        ) -> Option<crate::structures::scanning::comparisons::scan_function_vector::VectorCompareFnDelta16> {
            None
        }
    }

    impl DataType for TestRuntimeDataType {
        fn get_data_type_id(&self) -> &str {
            "remote.runtime.type"
        }
        fn get_icon_id(&self) -> &str {
            "remote.runtime.type"
        }
        fn get_unit_size_in_bytes(&self) -> u64 {
            3
        }
        fn validate_value_string(
            &self,
            _anonymous_value_string: &crate::structures::data_values::anonymous_value_string::AnonymousValueString,
        ) -> bool {
            true
        }
        fn deanonymize_value_string(
            &self,
            _anonymous_value_string: &crate::structures::data_values::anonymous_value_string::AnonymousValueString,
        ) -> Result<crate::structures::data_values::data_value::DataValue, DataTypeError> {
            Ok(crate::structures::data_values::data_value::DataValue::new(
                DataTypeRef::new("remote.runtime.type"),
                vec![0, 0, 0],
            ))
        }
        fn anonymize_value_bytes(
            &self,
            _value_bytes: &[u8],
            anonymous_value_string_format: AnonymousValueStringFormat,
        ) -> Result<crate::structures::data_values::anonymous_value_string::AnonymousValueString, DataTypeError> {
            Ok(crate::structures::data_values::anonymous_value_string::AnonymousValueString::new(
                String::from("0"),
                anonymous_value_string_format,
                ContainerType::None,
            ))
        }
        fn get_supported_anonymous_value_string_formats(&self) -> Vec<AnonymousValueStringFormat> {
            vec![AnonymousValueStringFormat::Decimal]
        }
        fn get_default_anonymous_value_string_format(&self) -> AnonymousValueStringFormat {
            AnonymousValueStringFormat::Decimal
        }
        fn get_endian(&self) -> Endian {
            Endian::Little
        }
        fn is_floating_point(&self) -> bool {
            false
        }
        fn is_signed(&self) -> bool {
            false
        }
        fn get_default_value(
            &self,
            data_type_ref: DataTypeRef,
        ) -> crate::structures::data_values::data_value::DataValue {
            crate::structures::data_values::data_value::DataValue::new(data_type_ref, vec![0, 0, 0])
        }
    }

    #[test]
    fn built_in_symbol_registry_excludes_unregistered_runtime_data_types() {
        let symbol_registry = SymbolRegistry::new();

        assert!(symbol_registry.get_data_type("remote.runtime.type").is_none());
        assert!(
            !symbol_registry
                .get_registered_data_type_refs()
                .iter()
                .any(|data_type_ref| data_type_ref.get_data_type_id() == "remote.runtime.type")
        );
    }

    #[test]
    fn runtime_registered_data_types_are_available_after_registration() {
        let symbol_registry = SymbolRegistry::new();

        assert!(symbol_registry.register_data_type(Arc::new(TestRuntimeDataType)));
        assert!(symbol_registry.get_data_type("remote.runtime.type").is_some());
        assert!(
            symbol_registry
                .get_registered_data_type_refs()
                .iter()
                .any(|data_type_ref| data_type_ref.get_data_type_id() == "remote.runtime.type")
        );
    }

    #[test]
    fn register_data_type_descriptor_adds_descriptor_and_symbolic_struct() {
        let symbol_registry = SymbolRegistry::new();

        assert!(symbol_registry.register_data_type_descriptor(create_test_data_type_descriptor("remote.test.type", 16)));
        assert!(symbol_registry.is_valid(&DataTypeRef::new("remote.test.type")));
        assert!(symbol_registry.get("remote.test.type").is_some());
        assert_eq!(symbol_registry.get_unit_size_in_bytes(&DataTypeRef::new("remote.test.type")), 16);
    }

    #[test]
    fn unregister_data_type_descriptor_removes_descriptor_and_symbolic_struct() {
        let symbol_registry = SymbolRegistry::new();

        assert!(symbol_registry.register_data_type_descriptor(create_test_data_type_descriptor("remote.test.type", 16)));
        assert!(symbol_registry.unregister_data_type_descriptor("remote.test.type"));
        assert!(!symbol_registry.is_valid(&DataTypeRef::new("remote.test.type")));
        assert!(symbol_registry.get("remote.test.type").is_none());
    }

    #[test]
    fn register_symbolic_struct_adds_custom_definition() {
        let symbol_registry = SymbolRegistry::new();

        assert!(symbol_registry.register_symbolic_struct(
            "remote.test.struct".to_string(),
            SymbolicStructDefinition::new(
                "remote.test.struct".to_string(),
                vec![SymbolicFieldDefinition::new(
                    DataTypeRef::new("u32"),
                    ContainerType::None
                )],
            ),
        ));
        assert!(symbol_registry.get("remote.test.struct").is_some());
    }

    #[test]
    fn unregister_symbolic_struct_removes_custom_definition() {
        let symbol_registry = SymbolRegistry::new();

        assert!(symbol_registry.register_symbolic_struct(
            "remote.test.struct".to_string(),
            SymbolicStructDefinition::new(
                "remote.test.struct".to_string(),
                vec![SymbolicFieldDefinition::new(
                    DataTypeRef::new("u32"),
                    ContainerType::None
                )],
            ),
        ));
        assert!(symbol_registry.unregister_symbolic_struct("remote.test.struct"));
        assert!(symbol_registry.get("remote.test.struct").is_none());
    }

    #[test]
    fn register_data_type_descriptor_rejects_built_in_data_type_id() {
        let symbol_registry = SymbolRegistry::new();

        assert!(!symbol_registry.register_data_type_descriptor(create_test_data_type_descriptor("u32", 16)));
        assert_eq!(symbol_registry.get_unit_size_in_bytes(&DataTypeRef::new("u32")), 4);
    }

    #[test]
    fn set_project_symbol_catalog_replaces_project_authored_symbols() {
        let symbol_registry = SymbolRegistry::new();
        let initial_project_symbols = vec![StructLayoutDescriptor::new(
            String::from("player.stats"),
            SymbolicStructDefinition::new(
                String::from("player.stats"),
                vec![SymbolicFieldDefinition::new(
                    DataTypeRef::new("u32"),
                    ContainerType::None,
                )],
            ),
        )];
        let replacement_project_symbols = vec![StructLayoutDescriptor::new(
            String::from("player.inventory"),
            SymbolicStructDefinition::new(
                String::from("player.inventory"),
                vec![SymbolicFieldDefinition::new(
                    DataTypeRef::new("u16"),
                    ContainerType::None,
                )],
            ),
        )];

        assert!(symbol_registry.set_project_symbol_catalog(&initial_project_symbols));
        assert!(symbol_registry.get("player.stats").is_some());

        assert!(symbol_registry.set_project_symbol_catalog(&replacement_project_symbols));
        assert!(symbol_registry.get("player.stats").is_none());
        assert!(symbol_registry.get("player.inventory").is_some());
    }

    #[test]
    fn set_project_symbol_catalog_rejects_builtin_symbol_ids() {
        let symbol_registry = SymbolRegistry::new();
        let registry_size_before_project_symbols = symbol_registry.get_registry().len();
        let colliding_project_symbols = vec![StructLayoutDescriptor::new(
            String::from("u32"),
            SymbolicStructDefinition::new(
                String::from("u32"),
                vec![SymbolicFieldDefinition::new(
                    DataTypeRef::new("u32"),
                    ContainerType::None,
                )],
            ),
        )];

        assert!(symbol_registry.set_project_symbol_catalog(&colliding_project_symbols));
        assert_eq!(symbol_registry.get_registry().len(), registry_size_before_project_symbols);
    }

    fn create_test_data_type_descriptor(
        data_type_id: &str,
        unit_size_in_bytes: u64,
    ) -> DataTypeDescriptor {
        DataTypeDescriptor::new(
            data_type_id.to_string(),
            "remote-icon".to_string(),
            unit_size_in_bytes,
            vec![AnonymousValueStringFormat::Hexadecimal],
            AnonymousValueStringFormat::Hexadecimal,
            Endian::Little,
            false,
            false,
        )
    }
}
