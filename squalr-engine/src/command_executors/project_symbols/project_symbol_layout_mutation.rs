use squalr_engine_api::commands::project_symbols::delete::project_symbols_delete_request::{
    ProjectSymbolsDeleteModuleRange, ProjectSymbolsDeleteModuleRangeMode,
};
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_values::container_type::ContainerType;
use squalr_engine_api::structures::projects::{
    project_symbol_catalog::ProjectSymbolCatalog, project_symbol_claim::ProjectSymbolClaim, project_symbol_locator::ProjectSymbolLocator,
    project_symbol_module::ProjectSymbolModule, project_symbol_module_field::ProjectSymbolModuleField,
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

    pub fn delete_module_ranges_to_u8_fields(
        project_symbol_catalog: &mut ProjectSymbolCatalog,
        module_ranges: &[ProjectSymbolsDeleteModuleRange],
        deleted_module_names: &HashSet<String>,
    ) -> ProjectSymbolLayoutMutationSummary {
        let mut deleted_module_range_count = 0_u64;

        for module_range in module_ranges {
            if deleted_module_names.contains(&module_range.module_name) {
                continue;
            }

            let Some((replacement_length, did_replace_range)) = Self::replace_module_range_with_u8_field(project_symbol_catalog, module_range) else {
                continue;
            };

            Self::delete_module_symbol_claims_in_range(project_symbol_catalog.get_symbol_claims_mut(), module_range, replacement_length);

            if did_replace_range {
                deleted_module_range_count = deleted_module_range_count.saturating_add(1);
            }
        }

        ProjectSymbolLayoutMutationSummary {
            changed: deleted_module_range_count > 0,
            deleted_module_field_count: 0,
            deleted_module_range_count,
        }
    }

    fn replace_module_range_with_u8_field(
        project_symbol_catalog: &mut ProjectSymbolCatalog,
        module_range: &ProjectSymbolsDeleteModuleRange,
    ) -> Option<(u64, bool)> {
        let symbol_module = project_symbol_catalog.find_symbol_module_mut(&module_range.module_name)?;
        let module_size = symbol_module.get_size();

        if module_range.offset >= module_size {
            return None;
        }

        let replacement_length = module_range
            .length
            .min(module_size.saturating_sub(module_range.offset));

        if replacement_length == 0 {
            return None;
        }

        let deleted_range_end = module_range.offset.saturating_add(replacement_length);
        let module_field_count_before_delete = symbol_module.get_fields().len();

        symbol_module.get_fields_mut().retain(|module_field| {
            let field_offset = module_field.get_offset();

            field_offset < module_range.offset || field_offset >= deleted_range_end
        });
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(
                format!("u8_{:08X}", module_range.offset),
                module_range.offset,
                Self::u8_array_type_id(replacement_length),
            ));
        Self::sort_module_fields(symbol_module.get_fields_mut());
        Self::merge_adjacent_u8_array_fields(symbol_module.get_fields_mut());

        Some((
            replacement_length,
            symbol_module.get_fields().len() != module_field_count_before_delete || replacement_length > 0,
        ))
    }

    pub fn delete_module_ranges(
        project_symbol_catalog: &mut ProjectSymbolCatalog,
        module_ranges: &[ProjectSymbolsDeleteModuleRange],
        deleted_module_names: &HashSet<String>,
    ) -> ProjectSymbolLayoutMutationSummary {
        let mut shift_left_ranges = Vec::new();
        let mut replace_with_u8_ranges = Vec::new();

        for module_range in module_ranges {
            match module_range.mode {
                ProjectSymbolsDeleteModuleRangeMode::ShiftLeft => shift_left_ranges.push(module_range.clone()),
                ProjectSymbolsDeleteModuleRangeMode::ReplaceWithU8 => replace_with_u8_ranges.push(module_range.clone()),
            }
        }

        let shift_left_summary = Self::delete_module_ranges_and_shift(project_symbol_catalog, &shift_left_ranges, deleted_module_names);
        let replace_with_u8_summary = Self::delete_module_ranges_to_u8_fields(project_symbol_catalog, &replace_with_u8_ranges, deleted_module_names);

        ProjectSymbolLayoutMutationSummary {
            changed: shift_left_summary.get_changed() || replace_with_u8_summary.get_changed(),
            deleted_module_field_count: shift_left_summary
                .get_deleted_module_field_count()
                .saturating_add(replace_with_u8_summary.get_deleted_module_field_count()),
            deleted_module_range_count: shift_left_summary
                .get_deleted_module_range_count()
                .saturating_add(replace_with_u8_summary.get_deleted_module_range_count()),
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

        if let Some(filler_field_position) = Self::find_containing_u8_array_field_position(symbol_module.get_fields(), new_field_offset, new_field_end_offset) {
            Self::replace_u8_array_field_span(symbol_module.get_fields_mut(), filler_field_position, new_module_field, new_field_size_in_bytes)?;
            Self::sort_module_fields(symbol_module.get_fields_mut());

            return Ok(ProjectSymbolLayoutMutationSummary {
                changed: true,
                deleted_module_field_count: 0,
                deleted_module_range_count: 0,
            });
        }

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

    fn replace_u8_array_field_span(
        module_fields: &mut Vec<ProjectSymbolModuleField>,
        filler_field_position: usize,
        new_module_field: ProjectSymbolModuleField,
        new_field_size_in_bytes: u64,
    ) -> Result<(), String> {
        let Some(source_u8_field) = module_fields.get(filler_field_position).cloned() else {
            return Err(String::from("Could not resolve source u8[] field for replacement."));
        };
        let Some(source_u8_length) = Self::resolve_u8_array_length(source_u8_field.get_struct_layout_id()) else {
            return Err(String::from("Source field is no longer a fixed u8[] field."));
        };

        let source_start_offset = source_u8_field.get_offset();
        let source_end_offset = source_start_offset
            .checked_add(source_u8_length)
            .ok_or_else(|| String::from("Source u8[] field range is too large."))?;
        let new_field_offset = new_module_field.get_offset();
        let new_field_end_offset = new_field_offset
            .checked_add(new_field_size_in_bytes)
            .ok_or_else(|| String::from("Replacement module field range is too large."))?;

        if new_field_offset < source_start_offset || new_field_end_offset > source_end_offset {
            return Err(String::from("Replacement module field does not fit inside the source u8[] field."));
        }

        module_fields.remove(filler_field_position);

        if new_field_offset > source_start_offset {
            let prefix_length = new_field_offset.saturating_sub(source_start_offset);
            module_fields.push(ProjectSymbolModuleField::new(
                format!("u8_{:08X}", source_start_offset),
                source_start_offset,
                Self::u8_array_type_id(prefix_length),
            ));
        }

        module_fields.push(new_module_field);

        if new_field_end_offset < source_end_offset {
            let suffix_length = source_end_offset.saturating_sub(new_field_end_offset);
            module_fields.push(ProjectSymbolModuleField::new(
                format!("u8_{:08X}", new_field_end_offset),
                new_field_end_offset,
                Self::u8_array_type_id(suffix_length),
            ));
        }

        Ok(())
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

    fn find_containing_u8_array_field_position(
        module_fields: &[ProjectSymbolModuleField],
        field_offset: u64,
        field_end_offset: u64,
    ) -> Option<usize> {
        module_fields.iter().position(|module_field| {
            let Some(u8_array_length) = Self::resolve_u8_array_length(module_field.get_struct_layout_id()) else {
                return false;
            };
            let Some(u8_array_end_offset) = module_field.get_offset().checked_add(u8_array_length) else {
                return false;
            };

            module_field.get_offset() <= field_offset && field_end_offset <= u8_array_end_offset
        })
    }

    fn resolve_u8_array_length(struct_layout_id: &str) -> Option<u64> {
        let symbolic_field_definition = SymbolicFieldDefinition::from_str(struct_layout_id).ok()?;

        if symbolic_field_definition.get_data_type_ref().get_data_type_id() != "u8" {
            return None;
        }

        let ContainerType::ArrayFixed(length) = symbolic_field_definition.get_container_type() else {
            return None;
        };

        (length > 0).then_some(length)
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
        if let Some(pointer_size) = symbolic_field_definition
            .get_container_type()
            .get_pointer_size()
        {
            return Some(pointer_size.get_size_in_bytes());
        }

        let unit_size_in_bytes = if let Some(primitive_size_in_bytes) = resolve_primitive_size_in_bytes(symbolic_field_definition.get_data_type_ref()) {
            primitive_size_in_bytes
        } else {
            let struct_layout_id = symbolic_field_definition.get_data_type_ref().get_data_type_id();

            if !visited_struct_layout_ids.insert(struct_layout_id.to_string()) {
                return None;
            }

            let symbolic_struct_definition = resolve_struct_layout_definition(struct_layout_id)?;
            let struct_size_in_bytes = Self::resolve_symbolic_struct_size_in_bytes(
                &symbolic_struct_definition,
                resolve_primitive_size_in_bytes,
                resolve_struct_layout_definition,
                visited_struct_layout_ids,
            )?;

            visited_struct_layout_ids.remove(struct_layout_id);
            struct_size_in_bytes
        };

        Some(
            symbolic_field_definition
                .get_container_type()
                .get_total_size_in_bytes(unit_size_in_bytes),
        )
    }

    fn resolve_symbolic_struct_size_in_bytes<ResolvePrimitiveSize, ResolveStructLayout>(
        symbolic_struct_definition: &SymbolicStructDefinition,
        resolve_primitive_size_in_bytes: ResolvePrimitiveSize,
        resolve_struct_layout_definition: ResolveStructLayout,
        visited_struct_layout_ids: &mut HashSet<String>,
    ) -> Option<u64>
    where
        ResolvePrimitiveSize: Fn(&DataTypeRef) -> Option<u64> + Copy,
        ResolveStructLayout: Fn(&str) -> Option<SymbolicStructDefinition> + Copy,
    {
        symbolic_struct_definition
            .get_fields()
            .iter()
            .try_fold(0_u64, |accumulated_size_in_bytes, symbolic_field_definition| {
                let field_size_in_bytes = Self::resolve_symbolic_field_size_in_bytes(
                    symbolic_field_definition,
                    resolve_primitive_size_in_bytes,
                    resolve_struct_layout_definition,
                    visited_struct_layout_ids,
                )?;

                accumulated_size_in_bytes.checked_add(field_size_in_bytes)
            })
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

    fn merge_adjacent_u8_array_fields(module_fields: &mut Vec<ProjectSymbolModuleField>) {
        Self::sort_module_fields(module_fields);

        let mut merged_fields = Vec::new();

        for module_field in module_fields.drain(..) {
            let Some(module_field_length) = Self::resolve_u8_array_length(module_field.get_struct_layout_id()) else {
                merged_fields.push(module_field);
                continue;
            };

            let Some(previous_field) = merged_fields.last_mut() else {
                merged_fields.push(module_field);
                continue;
            };
            let Some(previous_field_length) = Self::resolve_u8_array_length(previous_field.get_struct_layout_id()) else {
                merged_fields.push(module_field);
                continue;
            };
            let Some(previous_field_end_offset) = previous_field.get_offset().checked_add(previous_field_length) else {
                merged_fields.push(module_field);
                continue;
            };

            if previous_field_end_offset != module_field.get_offset() {
                merged_fields.push(module_field);
                continue;
            }

            let merged_length = previous_field_length.saturating_add(module_field_length);
            previous_field.set_struct_layout_id(Self::u8_array_type_id(merged_length));
            previous_field.set_display_name(format!("u8_{:08X}", previous_field.get_offset()));
        }

        *module_fields = merged_fields;
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

    fn u8_array_type_id(length: u64) -> String {
        format!("u8[{}]", length)
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectSymbolLayoutMutation;
    use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
    use squalr_engine_api::structures::data_values::container_type::ContainerType;
    use squalr_engine_api::structures::projects::{
        project_symbol_catalog::ProjectSymbolCatalog, project_symbol_module::ProjectSymbolModule, project_symbol_module_field::ProjectSymbolModuleField,
    };
    use squalr_engine_api::structures::structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition};

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
                (resolved_struct_layout_id == "player.stats").then(|| {
                    SymbolicStructDefinition::new(
                        String::from("player.stats"),
                        vec![
                            SymbolicFieldDefinition::new_named(String::from("health"), DataTypeRef::new("u32"), ContainerType::None),
                            SymbolicFieldDefinition::new_named(String::from("team"), DataTypeRef::new("u16"), ContainerType::None),
                        ],
                    )
                })
            },
        )
    }

    #[test]
    fn upsert_module_field_splits_existing_u8_array_field() {
        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x20);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("buffer"), 0x00, String::from("u8[32]")));
        let mut project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![symbol_module], Vec::new(), Vec::new());

        let mutation_result = ProjectSymbolLayoutMutation::upsert_module_field(
            &mut project_symbol_catalog,
            "game.exe",
            String::from("Health"),
            0x08,
            String::from("u32"),
            resolve_test_field_size,
        )
        .expect("Expected u8[] carve mutation to succeed.");

        assert!(mutation_result.get_changed());
        let fields = project_symbol_catalog.get_symbol_modules()[0].get_fields();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].get_offset(), 0x00);
        assert_eq!(fields[0].get_struct_layout_id(), "u8[8]");
        assert_eq!(fields[1].get_display_name(), "Health");
        assert_eq!(fields[1].get_offset(), 0x08);
        assert_eq!(fields[1].get_struct_layout_id(), "u32");
        assert_eq!(fields[2].get_offset(), 0x0C);
        assert_eq!(fields[2].get_struct_layout_id(), "u8[20]");
    }

    #[test]
    fn repeated_split_of_u8_array_field_keeps_fields_ordered() {
        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x10);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("u8_00000000"), 0x00, String::from("u8[16]")));
        let mut project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![symbol_module], Vec::new(), Vec::new());

        ProjectSymbolLayoutMutation::upsert_module_field(
            &mut project_symbol_catalog,
            "game.exe",
            String::from("u8_00000000"),
            0x00,
            String::from("u8[8]"),
            resolve_test_field_size,
        )
        .expect("Expected first split mutation to succeed.");
        ProjectSymbolLayoutMutation::upsert_module_field(
            &mut project_symbol_catalog,
            "game.exe",
            String::from("u8_00000008"),
            0x08,
            String::from("u8[4]"),
            resolve_test_field_size,
        )
        .expect("Expected repeated split mutation to succeed.");

        let fields = project_symbol_catalog.get_symbol_modules()[0].get_fields();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].get_offset(), 0x00);
        assert_eq!(fields[0].get_struct_layout_id(), "u8[8]");
        assert_eq!(fields[1].get_offset(), 0x08);
        assert_eq!(fields[1].get_struct_layout_id(), "u8[4]");
        assert_eq!(fields[2].get_offset(), 0x0C);
        assert_eq!(fields[2].get_struct_layout_id(), "u8[4]");
    }

    #[test]
    fn split_tail_u8_array_field_keeps_module_fields_ordered() {
        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x10);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("u8_00000000"), 0x00, String::from("u8[8]")));
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("u8_00000008"), 0x08, String::from("u8[8]")));
        let mut project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![symbol_module], Vec::new(), Vec::new());

        ProjectSymbolLayoutMutation::upsert_module_field(
            &mut project_symbol_catalog,
            "game.exe",
            String::from("u8_00000008"),
            0x08,
            String::from("u8[4]"),
            resolve_test_field_size,
        )
        .expect("Expected tail split first half mutation to succeed.");
        ProjectSymbolLayoutMutation::upsert_module_field(
            &mut project_symbol_catalog,
            "game.exe",
            String::from("u8_0000000C"),
            0x0C,
            String::from("u8[4]"),
            resolve_test_field_size,
        )
        .expect("Expected tail split second half mutation to succeed.");

        let fields = project_symbol_catalog.get_symbol_modules()[0].get_fields();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].get_offset(), 0x00);
        assert_eq!(fields[0].get_struct_layout_id(), "u8[8]");
        assert_eq!(fields[1].get_offset(), 0x08);
        assert_eq!(fields[1].get_struct_layout_id(), "u8[4]");
        assert_eq!(fields[2].get_offset(), 0x0C);
        assert_eq!(fields[2].get_struct_layout_id(), "u8[4]");
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
    fn resolve_struct_layout_id_size_uses_pointer_slot_size_for_pointer_to_struct() {
        assert_eq!(resolve_test_field_size_with_structs("player.stats*(u64)"), Some(8));
        assert_eq!(resolve_test_field_size_with_structs("player.stats*(u32)"), Some(4));
    }

    #[test]
    fn upsert_module_field_carves_struct_layout_field_by_struct_size() {
        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x20);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("buffer"), 0x00, String::from("u8[32]")));
        let mut project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![symbol_module], Vec::new(), Vec::new());

        ProjectSymbolLayoutMutation::upsert_module_field(
            &mut project_symbol_catalog,
            "game.exe",
            String::from("PlayerStats"),
            0x08,
            String::from("player.stats"),
            resolve_test_field_size_with_structs,
        )
        .expect("Expected struct layout carve mutation to succeed.");

        let fields = project_symbol_catalog.get_symbol_modules()[0].get_fields();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].get_struct_layout_id(), "u8[8]");
        assert_eq!(fields[1].get_offset(), 0x08);
        assert_eq!(fields[1].get_struct_layout_id(), "player.stats");
        assert_eq!(fields[2].get_offset(), 0x0E);
        assert_eq!(fields[2].get_struct_layout_id(), "u8[18]");
    }
}
