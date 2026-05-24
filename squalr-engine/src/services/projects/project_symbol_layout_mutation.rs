use squalr_engine_api::commands::project_symbols::delete::project_symbols_delete_request::{
    ProjectSymbolsDeleteModuleRange, ProjectSymbolsDeleteModuleRangeMode,
};
use squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::projects::{
    project_symbol_catalog::ProjectSymbolCatalog, project_symbol_claim::ProjectSymbolClaim, project_symbol_locator::ProjectSymbolLocator,
    project_symbol_module::ProjectSymbolModule, project_symbol_module_field::ProjectSymbolModuleField,
    symbol_layouts::symbol_layout_size_resolver::SymbolLayoutSizeResolver,
};
use squalr_engine_api::structures::structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition};
use std::collections::HashSet;
use std::str::FromStr;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProjectSymbolLayoutMutationSummary {
    changed: bool,
    deleted_module_field_count: u64,
    deleted_module_range_count: u64,
}

impl ProjectSymbolLayoutMutationSummary {
    pub fn get_changed(&self) -> bool {
        self.changed
    }

    pub fn get_deleted_module_field_count(&self) -> u64 {
        self.deleted_module_field_count
    }

    pub fn get_deleted_module_range_count(&self) -> u64 {
        self.deleted_module_range_count
    }
}

pub struct ProjectSymbolLayoutMutation;

impl ProjectSymbolLayoutMutation {
    pub fn upsert_struct_layout_descriptor(
        project_symbol_catalog: &mut ProjectSymbolCatalog,
        original_struct_layout_id: Option<&str>,
        struct_layout_descriptor: StructLayoutDescriptor,
    ) -> Result<ProjectSymbolLayoutMutationSummary, String> {
        let struct_layout_id = struct_layout_descriptor.get_struct_layout_id();
        let conflicts_with_existing_layout = original_struct_layout_id.is_some_and(|original_struct_layout_id| {
            original_struct_layout_id != struct_layout_id
                && project_symbol_catalog
                    .get_struct_layout_descriptors()
                    .iter()
                    .any(|existing_struct_layout_descriptor| existing_struct_layout_descriptor.get_struct_layout_id() == struct_layout_id)
        });

        if conflicts_with_existing_layout {
            return Err(format!("Symbol layout id `{}` is already used.", struct_layout_id));
        }

        let replacement_struct_layout_id = struct_layout_id.to_string();
        let mut updated_struct_layout_descriptors = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .filter(|existing_struct_layout_descriptor| {
                Some(existing_struct_layout_descriptor.get_struct_layout_id()) != original_struct_layout_id
                    && existing_struct_layout_descriptor.get_struct_layout_id() != replacement_struct_layout_id
            })
            .cloned()
            .collect::<Vec<_>>();

        updated_struct_layout_descriptors.push(struct_layout_descriptor);
        Self::sort_struct_layout_descriptors(&mut updated_struct_layout_descriptors);
        project_symbol_catalog.set_struct_layout_descriptors(updated_struct_layout_descriptors);

        if let Some(original_struct_layout_id) = original_struct_layout_id {
            if original_struct_layout_id != replacement_struct_layout_id {
                Self::retarget_catalog_struct_layout_references(
                    project_symbol_catalog,
                    original_struct_layout_id,
                    &DataTypeRef::new(&replacement_struct_layout_id),
                );
            }
        }

        Ok(ProjectSymbolLayoutMutationSummary {
            changed: true,
            deleted_module_field_count: 0,
            deleted_module_range_count: 0,
        })
    }

    pub fn delete_struct_layout(
        project_symbol_catalog: &mut ProjectSymbolCatalog,
        struct_layout_id: &str,
        replacement_data_type_ref: DataTypeRef,
    ) -> Result<ProjectSymbolLayoutMutationSummary, String> {
        let struct_layout_count_before_delete = project_symbol_catalog.get_struct_layout_descriptors().len();
        let updated_struct_layout_descriptors = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .filter(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() != struct_layout_id)
            .cloned()
            .collect::<Vec<_>>();

        if updated_struct_layout_descriptors.len() == struct_layout_count_before_delete {
            return Err(format!("Symbol layout `{}` does not exist.", struct_layout_id));
        }

        project_symbol_catalog.set_struct_layout_descriptors(updated_struct_layout_descriptors);
        Self::retarget_catalog_struct_layout_references(project_symbol_catalog, struct_layout_id, &replacement_data_type_ref);

        Ok(ProjectSymbolLayoutMutationSummary {
            changed: true,
            deleted_module_field_count: 0,
            deleted_module_range_count: 0,
        })
    }

    pub fn resolve_struct_layout_id_size_in_bytes<ResolvePrimitiveSize, ResolveStructLayout>(
        struct_layout_id: &str,
        resolve_primitive_size_in_bytes: ResolvePrimitiveSize,
        resolve_struct_layout_definition: ResolveStructLayout,
    ) -> Option<u64>
    where
        ResolvePrimitiveSize: Fn(&DataTypeRef) -> Option<u64> + Copy,
        ResolveStructLayout: Fn(&str) -> Option<SymbolicStructDefinition> + Copy,
    {
        let symbolic_field_definition = SymbolicFieldDefinition::from_str(struct_layout_id).ok()?;
        let mut visited_struct_layout_ids = HashSet::new();

        Self::resolve_symbolic_field_size_in_bytes(
            &symbolic_field_definition,
            resolve_primitive_size_in_bytes,
            resolve_struct_layout_definition,
            &mut visited_struct_layout_ids,
        )
    }

    pub fn upsert_module_field<ResolveFieldSize>(
        project_symbol_catalog: &mut ProjectSymbolCatalog,
        module_name: &str,
        display_name: String,
        offset: u64,
        struct_layout_id: String,
        resolve_field_size_in_bytes: ResolveFieldSize,
    ) -> Result<ProjectSymbolLayoutMutationSummary, String>
    where
        ResolveFieldSize: Fn(&str) -> Option<u64> + Copy,
    {
        let field_size_in_bytes =
            resolve_field_size_in_bytes(&struct_layout_id).ok_or_else(|| format!("Cannot resolve byte size for `{}`.", struct_layout_id))?;

        if field_size_in_bytes == 0 {
            return Err(format!("`{}` has no byte size.", struct_layout_id));
        }

        let field_end_offset = offset
            .checked_add(field_size_in_bytes)
            .ok_or_else(|| String::from("Module field range is too large."))?;

        project_symbol_catalog.ensure_symbol_module(module_name, field_end_offset);

        let Some(symbol_module) = project_symbol_catalog.find_symbol_module_mut(module_name) else {
            return Err(format!("Could not resolve module `{}` after creating it.", module_name));
        };

        let new_module_field = ProjectSymbolModuleField::new(display_name, offset, struct_layout_id);
        Self::upsert_module_field_in_module(symbol_module, new_module_field, field_size_in_bytes, resolve_field_size_in_bytes)
    }

    pub fn delete_module_fields_by_locator_key(
        project_symbol_catalog: &mut ProjectSymbolCatalog,
        symbol_locator_key_set: &HashSet<String>,
    ) -> ProjectSymbolLayoutMutationSummary {
        let mut deleted_module_field_count = 0_u64;

        for symbol_module in project_symbol_catalog.get_symbol_modules_mut() {
            let module_name = symbol_module.get_module_name().to_string();
            let module_field_count_before_delete = symbol_module.get_fields().len();

            symbol_module
                .get_fields_mut()
                .retain(|module_field| !symbol_locator_key_set.contains(&module_field.get_symbol_locator_key(&module_name)));
            deleted_module_field_count =
                deleted_module_field_count.saturating_add(module_field_count_before_delete.saturating_sub(symbol_module.get_fields().len()) as u64);
        }

        ProjectSymbolLayoutMutationSummary {
            changed: deleted_module_field_count > 0,
            deleted_module_field_count,
            deleted_module_range_count: 0,
        }
    }

    pub fn delete_module_ranges_and_shift(
        project_symbol_catalog: &mut ProjectSymbolCatalog,
        module_ranges: &[ProjectSymbolsDeleteModuleRange],
        deleted_module_names: &HashSet<String>,
    ) -> ProjectSymbolLayoutMutationSummary {
        let mut deleted_module_range_count = 0_u64;

        for module_range in module_ranges {
            if deleted_module_names.contains(&module_range.module_name) {
                continue;
            }

            let Some(symbol_module) = project_symbol_catalog.find_symbol_module_mut(&module_range.module_name) else {
                continue;
            };
            let module_size = symbol_module.get_size();

            if module_range.offset >= module_size {
                continue;
            }

            let deleted_length = module_range
                .length
                .min(module_size.saturating_sub(module_range.offset));

            if deleted_length == 0 {
                continue;
            }

            symbol_module.set_size(module_size.saturating_sub(deleted_length));
            Self::delete_or_shift_module_fields(symbol_module.get_fields_mut(), module_range, deleted_length);
            Self::delete_or_shift_module_symbol_claims(project_symbol_catalog.get_symbol_claims_mut(), module_range, deleted_length);
            deleted_module_range_count = deleted_module_range_count.saturating_add(1);
        }

        ProjectSymbolLayoutMutationSummary {
            changed: deleted_module_range_count > 0,
            deleted_module_field_count: 0,
            deleted_module_range_count,
        }
    }

    pub fn delete_module_ranges_to_unassigned(
        project_symbol_catalog: &mut ProjectSymbolCatalog,
        module_ranges: &[ProjectSymbolsDeleteModuleRange],
        deleted_module_names: &HashSet<String>,
    ) -> ProjectSymbolLayoutMutationSummary {
        let mut deleted_module_range_count = 0_u64;

        for module_range in module_ranges {
            if deleted_module_names.contains(&module_range.module_name) {
                continue;
            }

            let Some((deleted_length, did_delete_range)) = Self::delete_module_range_to_unassigned(project_symbol_catalog, module_range) else {
                continue;
            };

            Self::delete_module_symbol_claims_in_range(project_symbol_catalog.get_symbol_claims_mut(), module_range, deleted_length);

            if did_delete_range {
                deleted_module_range_count = deleted_module_range_count.saturating_add(1);
            }
        }

        ProjectSymbolLayoutMutationSummary {
            changed: deleted_module_range_count > 0,
            deleted_module_field_count: 0,
            deleted_module_range_count,
        }
    }

    fn delete_module_range_to_unassigned(
        project_symbol_catalog: &mut ProjectSymbolCatalog,
        module_range: &ProjectSymbolsDeleteModuleRange,
    ) -> Option<(u64, bool)> {
        let symbol_module = project_symbol_catalog.find_symbol_module_mut(&module_range.module_name)?;
        let module_size = symbol_module.get_size();

        if module_range.offset >= module_size {
            return None;
        }

        let deleted_length = module_range
            .length
            .min(module_size.saturating_sub(module_range.offset));

        if deleted_length == 0 {
            return None;
        }

        let deleted_range_end = module_range.offset.saturating_add(deleted_length);
        let module_field_count_before_delete = symbol_module.get_fields().len();

        symbol_module.get_fields_mut().retain(|module_field| {
            let field_offset = module_field.get_offset();

            field_offset < module_range.offset || field_offset >= deleted_range_end
        });
        Self::sort_module_fields(symbol_module.get_fields_mut());

        Some((
            deleted_length,
            symbol_module.get_fields().len() != module_field_count_before_delete || deleted_length > 0,
        ))
    }

    pub fn delete_module_ranges(
        project_symbol_catalog: &mut ProjectSymbolCatalog,
        module_ranges: &[ProjectSymbolsDeleteModuleRange],
        deleted_module_names: &HashSet<String>,
    ) -> ProjectSymbolLayoutMutationSummary {
        let mut shift_left_ranges = Vec::new();
        let mut replace_with_unassigned_ranges = Vec::new();

        for module_range in module_ranges {
            match module_range.mode {
                ProjectSymbolsDeleteModuleRangeMode::ShiftLeft => shift_left_ranges.push(module_range.clone()),
                ProjectSymbolsDeleteModuleRangeMode::ReplaceWithUnassigned => replace_with_unassigned_ranges.push(module_range.clone()),
            }
        }

        let shift_left_summary = Self::delete_module_ranges_and_shift(project_symbol_catalog, &shift_left_ranges, deleted_module_names);
        let replace_with_unassigned_summary =
            Self::delete_module_ranges_to_unassigned(project_symbol_catalog, &replace_with_unassigned_ranges, deleted_module_names);

        ProjectSymbolLayoutMutationSummary {
            changed: shift_left_summary.get_changed() || replace_with_unassigned_summary.get_changed(),
            deleted_module_field_count: shift_left_summary
                .get_deleted_module_field_count()
                .saturating_add(replace_with_unassigned_summary.get_deleted_module_field_count()),
            deleted_module_range_count: shift_left_summary
                .get_deleted_module_range_count()
                .saturating_add(replace_with_unassigned_summary.get_deleted_module_range_count()),
        }
    }

    fn upsert_module_field_in_module<ResolveFieldSize>(
        symbol_module: &mut ProjectSymbolModule,
        new_module_field: ProjectSymbolModuleField,
        new_field_size_in_bytes: u64,
        resolve_field_size_in_bytes: ResolveFieldSize,
    ) -> Result<ProjectSymbolLayoutMutationSummary, String>
    where
        ResolveFieldSize: Fn(&str) -> Option<u64> + Copy,
    {
        let new_field_offset = new_module_field.get_offset();
        let new_field_end_offset = new_field_offset
            .checked_add(new_field_size_in_bytes)
            .ok_or_else(|| String::from("Module field range is too large."))?;

        if Self::has_overlapping_module_field(symbol_module.get_fields(), new_field_offset, new_field_end_offset, resolve_field_size_in_bytes) {
            return Err(format!(
                "Module field `{}` overlaps an existing field in `{}`.",
                new_module_field.get_display_name(),
                symbol_module.get_module_name()
            ));
        }

        if let Some(existing_field) = symbol_module.find_field_mut(new_field_offset) {
            existing_field.set_display_name(new_module_field.get_display_name().to_string());
            existing_field.set_struct_layout_id(new_module_field.get_struct_layout_id().to_string());
        } else {
            symbol_module.get_fields_mut().push(new_module_field);
        }

        Self::sort_module_fields(symbol_module.get_fields_mut());

        Ok(ProjectSymbolLayoutMutationSummary {
            changed: true,
            deleted_module_field_count: 0,
            deleted_module_range_count: 0,
        })
    }

    fn has_overlapping_module_field<ResolveFieldSize>(
        module_fields: &[ProjectSymbolModuleField],
        new_field_offset: u64,
        new_field_end_offset: u64,
        resolve_field_size_in_bytes: ResolveFieldSize,
    ) -> bool
    where
        ResolveFieldSize: Fn(&str) -> Option<u64> + Copy,
    {
        module_fields.iter().any(|module_field| {
            if module_field.get_offset() == new_field_offset {
                return false;
            }

            let Some(module_field_size_in_bytes) = resolve_field_size_in_bytes(module_field.get_struct_layout_id()) else {
                return false;
            };
            let Some(module_field_end_offset) = module_field
                .get_offset()
                .checked_add(module_field_size_in_bytes)
            else {
                return true;
            };

            module_field.get_offset() < new_field_end_offset && new_field_offset < module_field_end_offset
        })
    }

    fn resolve_symbolic_field_size_in_bytes<ResolvePrimitiveSize, ResolveStructLayout>(
        symbolic_field_definition: &SymbolicFieldDefinition,
        resolve_primitive_size_in_bytes: ResolvePrimitiveSize,
        resolve_struct_layout_definition: ResolveStructLayout,
        visited_struct_layout_ids: &mut HashSet<String>,
    ) -> Option<u64>
    where
        ResolvePrimitiveSize: Fn(&DataTypeRef) -> Option<u64> + Copy,
        ResolveStructLayout: Fn(&str) -> Option<SymbolicStructDefinition> + Copy,
    {
        SymbolLayoutSizeResolver::resolve_symbolic_field_size_in_bytes(
            symbolic_field_definition,
            resolve_primitive_size_in_bytes,
            resolve_struct_layout_definition,
            visited_struct_layout_ids,
        )
    }

    fn delete_or_shift_module_fields(
        module_fields: &mut Vec<ProjectSymbolModuleField>,
        module_range: &ProjectSymbolsDeleteModuleRange,
        deleted_length: u64,
    ) {
        let deleted_range_end = module_range.offset.saturating_add(deleted_length);

        module_fields.retain_mut(|module_field| {
            let offset = module_field.get_offset();

            if offset >= module_range.offset && offset < deleted_range_end {
                return false;
            }

            if offset >= deleted_range_end {
                module_field.set_offset(offset.saturating_sub(deleted_length));
            }

            true
        });
    }

    fn delete_or_shift_module_symbol_claims(
        symbol_claims: &mut Vec<ProjectSymbolClaim>,
        module_range: &ProjectSymbolsDeleteModuleRange,
        deleted_length: u64,
    ) {
        let deleted_range_end = module_range.offset.saturating_add(deleted_length);

        symbol_claims.retain_mut(|symbol_claim| {
            let ProjectSymbolLocator::ModuleOffset { module_name, offset } = symbol_claim.get_locator_mut() else {
                return true;
            };

            if module_name != &module_range.module_name {
                return true;
            }

            if *offset >= module_range.offset && *offset < deleted_range_end {
                return false;
            }

            if *offset >= deleted_range_end {
                *offset = offset.saturating_sub(deleted_length);
            }

            true
        });
    }

    fn delete_module_symbol_claims_in_range(
        symbol_claims: &mut Vec<ProjectSymbolClaim>,
        module_range: &ProjectSymbolsDeleteModuleRange,
        deleted_length: u64,
    ) {
        let deleted_range_end = module_range.offset.saturating_add(deleted_length);

        symbol_claims.retain(|symbol_claim| {
            let ProjectSymbolLocator::ModuleOffset { module_name, offset } = symbol_claim.get_locator() else {
                return true;
            };

            if module_name != &module_range.module_name {
                return true;
            }

            !(*offset >= module_range.offset && *offset < deleted_range_end)
        });
    }

    fn sort_module_fields(module_fields: &mut [ProjectSymbolModuleField]) {
        module_fields.sort_by(|left_module_field, right_module_field| {
            left_module_field
                .get_offset()
                .cmp(&right_module_field.get_offset())
                .then_with(|| {
                    left_module_field
                        .get_display_name()
                        .cmp(right_module_field.get_display_name())
                })
        });
    }

    fn sort_struct_layout_descriptors(struct_layout_descriptors: &mut [StructLayoutDescriptor]) {
        struct_layout_descriptors.sort_by(|left_layout, right_layout| {
            left_layout
                .get_struct_layout_id()
                .to_ascii_lowercase()
                .cmp(&right_layout.get_struct_layout_id().to_ascii_lowercase())
        });
    }

    fn retarget_catalog_struct_layout_references(
        project_symbol_catalog: &mut ProjectSymbolCatalog,
        source_struct_layout_id: &str,
        replacement_data_type_ref: &DataTypeRef,
    ) {
        for symbol_claim in project_symbol_catalog.get_symbol_claims_mut() {
            if symbol_claim.get_struct_layout_id() == source_struct_layout_id {
                symbol_claim.set_struct_layout_id(replacement_data_type_ref.get_data_type_id().to_string());
            }
        }

        for symbol_module in project_symbol_catalog.get_symbol_modules_mut() {
            for module_field in symbol_module.get_fields_mut() {
                if module_field.get_struct_layout_id() == source_struct_layout_id {
                    module_field.set_struct_layout_id(replacement_data_type_ref.get_data_type_id().to_string());
                }
            }
        }

        let retargeted_struct_layout_descriptors = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .map(|struct_layout_descriptor| {
                let retargeted_fields = struct_layout_descriptor
                    .get_struct_layout_definition()
                    .get_fields()
                    .iter()
                    .map(|symbolic_field_definition| {
                        Self::retarget_symbolic_field_definition_type(symbolic_field_definition, source_struct_layout_id, replacement_data_type_ref)
                    })
                    .collect();

                StructLayoutDescriptor::new(
                    struct_layout_descriptor.get_struct_layout_id().to_string(),
                    SymbolicStructDefinition::new_with_layout_kind(
                        struct_layout_descriptor
                            .get_struct_layout_definition()
                            .get_symbol_namespace()
                            .to_string(),
                        struct_layout_descriptor
                            .get_struct_layout_definition()
                            .get_layout_kind(),
                        retargeted_fields,
                    )
                    .with_declared_size_in_bytes(
                        struct_layout_descriptor
                            .get_struct_layout_definition()
                            .get_declared_size_in_bytes(),
                    ),
                )
            })
            .collect();

        project_symbol_catalog.set_struct_layout_descriptors(retargeted_struct_layout_descriptors);
    }

    fn retarget_symbolic_field_definition_type(
        symbolic_field_definition: &SymbolicFieldDefinition,
        source_struct_layout_id: &str,
        replacement_data_type_ref: &DataTypeRef,
    ) -> SymbolicFieldDefinition {
        if symbolic_field_definition.get_data_type_ref().get_data_type_id() != source_struct_layout_id {
            return symbolic_field_definition.clone();
        }

        SymbolicFieldDefinition::new_named_with_resolutions_and_display_count(
            symbolic_field_definition.get_field_name().to_string(),
            replacement_data_type_ref.clone(),
            symbolic_field_definition.get_container_type(),
            symbolic_field_definition.get_count_resolution().clone(),
            symbolic_field_definition.get_display_count_resolution().clone(),
            symbolic_field_definition.get_offset_resolution().clone(),
        )
        .with_active_when_resolver(symbolic_field_definition.get_active_when_resolver().cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectSymbolLayoutMutation;
    use squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
    use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
    use squalr_engine_api::structures::data_values::container_type::ContainerType;
    use squalr_engine_api::structures::projects::{
        project_symbol_catalog::ProjectSymbolCatalog, project_symbol_claim::ProjectSymbolClaim, project_symbol_module::ProjectSymbolModule,
        project_symbol_module_field::ProjectSymbolModuleField,
    };
    use squalr_engine_api::structures::structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition};
    use std::str::FromStr;

    fn resolve_test_field_size(struct_layout_id: &str) -> Option<u64> {
        ProjectSymbolLayoutMutation::resolve_struct_layout_id_size_in_bytes(
            struct_layout_id,
            |data_type_ref: &DataTypeRef| match data_type_ref.get_data_type_id() {
                "u8" => Some(1),
                "u16" => Some(2),
                "u32" => Some(4),
                _ => None,
            },
            |_struct_layout_id| None,
        )
    }

    fn resolve_test_field_size_with_structs(struct_layout_id: &str) -> Option<u64> {
        ProjectSymbolLayoutMutation::resolve_struct_layout_id_size_in_bytes(
            struct_layout_id,
            |data_type_ref: &DataTypeRef| match data_type_ref.get_data_type_id() {
                "u8" => Some(1),
                "u16" => Some(2),
                "u32" => Some(4),
                _ => None,
            },
            |resolved_struct_layout_id| {
                if resolved_struct_layout_id == "player.stats" {
                    return Some(SymbolicStructDefinition::new(
                        String::from("player.stats"),
                        vec![
                            SymbolicFieldDefinition::new_named(String::from("health"), DataTypeRef::new("u32"), ContainerType::None),
                            SymbolicFieldDefinition::new_named(String::from("team"), DataTypeRef::new("u16"), ContainerType::None),
                        ],
                    ));
                }

                if resolved_struct_layout_id == "player.declared" {
                    return Some(
                        SymbolicStructDefinition::new(
                            String::from("player.declared"),
                            vec![SymbolicFieldDefinition::new_named(
                                String::from("health"),
                                DataTypeRef::new("u32"),
                                ContainerType::None,
                            )],
                        )
                        .with_declared_size_in_bytes(Some(0x20)),
                    );
                }

                (resolved_struct_layout_id == "variant.payload")
                    .then(|| SymbolicStructDefinition::from_str("as_u32:u32 @ +0;raw:u8[16] @ +0").expect("Expected variant payload layout to parse."))
                    .or_else(|| {
                        (resolved_struct_layout_id == "variant.payload.union").then(|| {
                            SymbolicStructDefinition::new_union(
                                String::from("variant.payload.union"),
                                vec![
                                    SymbolicFieldDefinition::from_str("as_u32:u32").expect("Expected u32 union field to parse."),
                                    SymbolicFieldDefinition::from_str("raw:u8[16]").expect("Expected raw union field to parse."),
                                ],
                            )
                        })
                    })
            },
        )
    }

    #[test]
    fn upsert_module_field_inserts_into_unassigned_gap() {
        let symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x20);
        let mut project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![symbol_module], Vec::new(), Vec::new());

        let mutation_result = ProjectSymbolLayoutMutation::upsert_module_field(
            &mut project_symbol_catalog,
            "game.exe",
            String::from("Health"),
            0x08,
            String::from("u32"),
            resolve_test_field_size,
        )
        .expect("Expected unassigned gap mutation to succeed.");

        assert!(mutation_result.get_changed());
        let fields = project_symbol_catalog.get_symbol_modules()[0].get_fields();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].get_display_name(), "Health");
        assert_eq!(fields[0].get_offset(), 0x08);
        assert_eq!(fields[0].get_struct_layout_id(), "u32");
    }

    #[test]
    fn repeated_upsert_into_unassigned_gap_keeps_fields_ordered() {
        let symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x10);
        let mut project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![symbol_module], Vec::new(), Vec::new());

        ProjectSymbolLayoutMutation::upsert_module_field(
            &mut project_symbol_catalog,
            "game.exe",
            String::from("Head"),
            0x00,
            String::from("u8[8]"),
            resolve_test_field_size,
        )
        .expect("Expected first gap mutation to succeed.");
        ProjectSymbolLayoutMutation::upsert_module_field(
            &mut project_symbol_catalog,
            "game.exe",
            String::from("Middle"),
            0x08,
            String::from("u8[4]"),
            resolve_test_field_size,
        )
        .expect("Expected repeated gap mutation to succeed.");

        let fields = project_symbol_catalog.get_symbol_modules()[0].get_fields();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].get_offset(), 0x00);
        assert_eq!(fields[0].get_struct_layout_id(), "u8[8]");
        assert_eq!(fields[1].get_offset(), 0x08);
        assert_eq!(fields[1].get_struct_layout_id(), "u8[4]");
    }

    #[test]
    fn upsert_rejects_overlap_with_explicit_u8_array_field() {
        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x10);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("u8_00000008"), 0x08, String::from("u8[8]")));
        let mut project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![symbol_module], Vec::new(), Vec::new());

        let mutation_result = ProjectSymbolLayoutMutation::upsert_module_field(
            &mut project_symbol_catalog,
            "game.exe",
            String::from("Middle"),
            0x0C,
            String::from("u8[4]"),
            resolve_test_field_size,
        );

        assert!(mutation_result.is_err());
    }

    #[test]
    fn upsert_module_field_rejects_overlap_with_typed_field() {
        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x20);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("Health"), 0x08, String::from("u32")));
        let mut project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![symbol_module], Vec::new(), Vec::new());

        let mutation_result = ProjectSymbolLayoutMutation::upsert_module_field(
            &mut project_symbol_catalog,
            "game.exe",
            String::from("Ammo"),
            0x0A,
            String::from("u16"),
            resolve_test_field_size,
        );

        assert!(mutation_result.is_err());
    }

    #[test]
    fn resolve_struct_layout_id_size_uses_nested_struct_layout_size() {
        assert_eq!(resolve_test_field_size_with_structs("player.stats"), Some(6));
    }

    #[test]
    fn resolve_struct_layout_id_size_uses_declared_struct_size() {
        assert_eq!(resolve_test_field_size_with_structs("player.declared"), Some(0x20));
    }

    #[test]
    fn resolve_struct_layout_id_size_uses_static_field_span_for_overlapping_layouts() {
        assert_eq!(resolve_test_field_size_with_structs("variant.payload"), Some(16));
    }

    #[test]
    fn resolve_struct_layout_id_size_defaults_union_fields_to_shared_offset() {
        assert_eq!(resolve_test_field_size_with_structs("variant.payload.union"), Some(16));
    }

    #[test]
    fn resolve_struct_layout_id_size_uses_pointer_slot_size_for_pointer_to_struct() {
        assert_eq!(resolve_test_field_size_with_structs("player.stats*(u64)"), Some(8));
        assert_eq!(resolve_test_field_size_with_structs("player.stats*(u32)"), Some(4));
    }

    #[test]
    fn upsert_module_field_in_unassigned_gap_uses_struct_layout_size() {
        let symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x20);
        let mut project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![symbol_module], Vec::new(), Vec::new());

        ProjectSymbolLayoutMutation::upsert_module_field(
            &mut project_symbol_catalog,
            "game.exe",
            String::from("PlayerStats"),
            0x08,
            String::from("player.stats"),
            resolve_test_field_size_with_structs,
        )
        .expect("Expected symbol layout carve mutation to succeed.");

        let fields = project_symbol_catalog.get_symbol_modules()[0].get_fields();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].get_offset(), 0x08);
        assert_eq!(fields[0].get_struct_layout_id(), "player.stats");
    }

    #[test]
    fn upsert_struct_layout_descriptor_rename_retargets_module_fields_and_nested_layout_fields() {
        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x1000);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("Player"), 0, String::from("player.stats")));
        let mut project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(
            vec![symbol_module],
            vec![
                StructLayoutDescriptor::new(
                    String::from("player.stats"),
                    SymbolicStructDefinition::new(
                        String::from("player.stats"),
                        vec![SymbolicFieldDefinition::new_named(
                            String::from("health"),
                            DataTypeRef::new("u32"),
                            ContainerType::None,
                        )],
                    ),
                ),
                StructLayoutDescriptor::new(
                    String::from("player.container"),
                    SymbolicStructDefinition::new(
                        String::from("player.container"),
                        vec![SymbolicFieldDefinition::new_named(
                            String::from("Stats"),
                            DataTypeRef::new("player.stats"),
                            ContainerType::None,
                        )],
                    ),
                ),
            ],
            vec![ProjectSymbolClaim::new_absolute_address(
                String::from("Player"),
                0x1234,
                String::from("player.stats"),
            )],
        );

        ProjectSymbolLayoutMutation::upsert_struct_layout_descriptor(
            &mut project_symbol_catalog,
            Some("player.stats"),
            StructLayoutDescriptor::new(
                String::from("player.profile"),
                SymbolicStructDefinition::new(
                    String::from("player.profile"),
                    vec![SymbolicFieldDefinition::new_named(
                        String::from("health"),
                        DataTypeRef::new("u32"),
                        ContainerType::None,
                    )],
                ),
            ),
        )
        .expect("Expected struct layout rename to apply.");

        assert_eq!(project_symbol_catalog.get_symbol_claims()[0].get_struct_layout_id(), "player.profile");
        assert_eq!(
            project_symbol_catalog.get_symbol_modules()[0].get_fields()[0].get_struct_layout_id(),
            "player.profile"
        );
        assert_eq!(
            project_symbol_catalog.get_struct_layout_descriptors()[0]
                .get_struct_layout_definition()
                .get_fields()[0]
                .get_data_type_ref()
                .get_data_type_id(),
            "player.profile"
        );
    }

    #[test]
    fn delete_struct_layout_retargets_module_fields_and_nested_layout_fields() {
        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x1000);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("Player"), 0, String::from("player.stats")));
        let mut project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(
            vec![symbol_module],
            vec![
                StructLayoutDescriptor::new(
                    String::from("player.stats"),
                    SymbolicStructDefinition::new(
                        String::from("player.stats"),
                        vec![SymbolicFieldDefinition::new_named(
                            String::from("health"),
                            DataTypeRef::new("u32"),
                            ContainerType::None,
                        )],
                    ),
                ),
                StructLayoutDescriptor::new(
                    String::from("player.container"),
                    SymbolicStructDefinition::new(
                        String::from("player.container"),
                        vec![SymbolicFieldDefinition::new_named(
                            String::from("Stats"),
                            DataTypeRef::new("player.stats"),
                            ContainerType::None,
                        )],
                    ),
                ),
            ],
            vec![ProjectSymbolClaim::new_absolute_address(
                String::from("Player"),
                0x1234,
                String::from("player.stats"),
            )],
        );

        ProjectSymbolLayoutMutation::delete_struct_layout(&mut project_symbol_catalog, "player.stats", DataTypeRef::new("u8"))
            .expect("Expected struct layout delete to apply.");

        assert_eq!(project_symbol_catalog.get_struct_layout_descriptors().len(), 1);
        assert_eq!(project_symbol_catalog.get_symbol_claims()[0].get_struct_layout_id(), "u8");
        assert_eq!(project_symbol_catalog.get_symbol_modules()[0].get_fields()[0].get_struct_layout_id(), "u8");
        assert_eq!(
            project_symbol_catalog.get_struct_layout_descriptors()[0]
                .get_struct_layout_definition()
                .get_fields()[0]
                .get_data_type_ref()
                .get_data_type_id(),
            "u8"
        );
    }
}
