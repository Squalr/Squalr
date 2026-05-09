use crate::structures::{
    data_types::data_type_ref::DataTypeRef,
    data_values::container_type::ContainerType,
    structs::{
        symbolic_field_definition::SymbolicFieldDefinition,
        symbolic_resolver_definition::{SymbolicResolverDefinition, SymbolicResolverEvaluationError, SymbolicResolverRelativeSymbolPath},
        symbolic_struct_definition::SymbolicStructDefinition,
        symbolic_struct_resolver::{
            ResolvedSymbolicField, ResolvedSymbolicFieldStatus, SymbolicStructResolverOptions,
            resolve_symbolic_struct_definition_with_resolvers_and_symbol_fields,
        },
    },
};
use std::cell::RefCell;

#[derive(Default)]
pub struct SymbolicGlobalSymbolResolverSession {
    global_symbol_stack: RefCell<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolicGlobalSymbolRoot<SymbolLocator> {
    locator: SymbolLocator,
    symbol_type: SymbolicGlobalSymbolRootType,
}

impl<SymbolLocator> SymbolicGlobalSymbolRoot<SymbolLocator> {
    pub fn new(
        locator: SymbolLocator,
        symbol_type: SymbolicGlobalSymbolRootType,
    ) -> Self {
        Self { locator, symbol_type }
    }

    pub fn get_locator(&self) -> &SymbolLocator {
        &self.locator
    }

    pub fn get_symbol_type(&self) -> &SymbolicGlobalSymbolRootType {
        &self.symbol_type
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolicGlobalSymbolRootType {
    Struct {
        struct_layout_definition: SymbolicStructDefinition,
    },
    Field {
        data_type_ref: DataTypeRef,
        container_type: ContainerType,
    },
}

/// Resolves a module-rooted global symbol path into a scalar value.
pub fn resolve_global_symbol_field_value<
    SymbolLocator,
    ResolveGlobalSymbolRoots,
    ResolveTypeSize,
    ReadScalarField,
    ResolveResolverDefinition,
    ResolveStructDefinition,
    OffsetLocator,
>(
    session: &SymbolicGlobalSymbolResolverSession,
    module_name: &str,
    symbol_path: &SymbolicResolverRelativeSymbolPath,
    resolve_global_symbol_roots: &ResolveGlobalSymbolRoots,
    resolve_type_size_in_bytes: &ResolveTypeSize,
    read_scalar_field: &ReadScalarField,
    resolve_resolver_definition: &ResolveResolverDefinition,
    resolve_struct_definition: &ResolveStructDefinition,
    offset_locator: &OffsetLocator,
) -> Result<i128, SymbolicResolverEvaluationError>
where
    SymbolLocator: Clone,
    ResolveGlobalSymbolRoots: Fn(&str, &str) -> Vec<SymbolicGlobalSymbolRoot<SymbolLocator>>,
    ResolveTypeSize: Fn(&DataTypeRef) -> Option<u64>,
    ReadScalarField: Fn(&SymbolLocator, &SymbolicFieldDefinition, u64) -> Result<Option<i128>, String>,
    ResolveResolverDefinition: Fn(&str) -> Option<SymbolicResolverDefinition>,
    ResolveStructDefinition: Fn(&DataTypeRef) -> Option<SymbolicStructDefinition>,
    OffsetLocator: Fn(&SymbolLocator, u64) -> SymbolLocator,
{
    let Some(root_symbol_path_segment) = symbol_path.get_segments().first() else {
        return Err(SymbolicResolverEvaluationError::UnknownGlobalSymbolPath(format!(
            "{}.{}",
            module_name, symbol_path
        )));
    };
    let root_symbol_path_segment = SymbolicResolverRelativeSymbolPath::parse_segment(root_symbol_path_segment)
        .map_err(|error| SymbolicResolverEvaluationError::UnknownGlobalSymbolPath(error.to_string()))?;
    let global_symbol_key = format!("{}.{}", module_name, symbol_path);
    {
        let mut global_symbol_stack = session.global_symbol_stack.borrow_mut();
        if global_symbol_stack
            .iter()
            .any(|stacked_global_symbol_key| stacked_global_symbol_key == &global_symbol_key)
        {
            return Err(SymbolicResolverEvaluationError::ResolverCycle(global_symbol_key));
        }

        global_symbol_stack.push(global_symbol_key);
    }

    let result = resolve_global_symbol_field_value_inner(
        session,
        module_name,
        root_symbol_path_segment.get_field_name(),
        root_symbol_path_segment.get_offset_in_bytes(),
        SymbolicResolverRelativeSymbolPath::new(symbol_path.get_segments().iter().skip(1).cloned().collect()),
        resolve_global_symbol_roots,
        resolve_type_size_in_bytes,
        read_scalar_field,
        resolve_resolver_definition,
        resolve_struct_definition,
        offset_locator,
    );
    session.global_symbol_stack.borrow_mut().pop();

    result
}

fn resolve_global_symbol_field_value_inner<
    SymbolLocator,
    ResolveGlobalSymbolRoots,
    ResolveTypeSize,
    ReadScalarField,
    ResolveResolverDefinition,
    ResolveStructDefinition,
    OffsetLocator,
>(
    session: &SymbolicGlobalSymbolResolverSession,
    module_name: &str,
    root_symbol_name: &str,
    root_symbol_offset_in_bytes: u64,
    relative_symbol_path: SymbolicResolverRelativeSymbolPath,
    resolve_global_symbol_roots: &ResolveGlobalSymbolRoots,
    resolve_type_size_in_bytes: &ResolveTypeSize,
    read_scalar_field: &ReadScalarField,
    resolve_resolver_definition: &ResolveResolverDefinition,
    resolve_struct_definition: &ResolveStructDefinition,
    offset_locator: &OffsetLocator,
) -> Result<i128, SymbolicResolverEvaluationError>
where
    SymbolLocator: Clone,
    ResolveGlobalSymbolRoots: Fn(&str, &str) -> Vec<SymbolicGlobalSymbolRoot<SymbolLocator>>,
    ResolveTypeSize: Fn(&DataTypeRef) -> Option<u64>,
    ReadScalarField: Fn(&SymbolLocator, &SymbolicFieldDefinition, u64) -> Result<Option<i128>, String>,
    ResolveResolverDefinition: Fn(&str) -> Option<SymbolicResolverDefinition>,
    ResolveStructDefinition: Fn(&DataTypeRef) -> Option<SymbolicStructDefinition>,
    OffsetLocator: Fn(&SymbolLocator, u64) -> SymbolLocator,
{
    let global_symbol_roots = resolve_global_symbol_roots(module_name, root_symbol_name);

    if global_symbol_roots.is_empty() {
        return Err(SymbolicResolverEvaluationError::UnknownGlobalSymbolPath(format!(
            "{}.{}",
            module_name, root_symbol_name
        )));
    }

    if global_symbol_roots.len() > 1 {
        return Err(SymbolicResolverEvaluationError::AmbiguousGlobalSymbolPath(format!(
            "{}.{}",
            module_name, root_symbol_name
        )));
    }

    let Some(global_symbol_root) = global_symbol_roots.into_iter().next() else {
        return Err(SymbolicResolverEvaluationError::UnknownGlobalSymbolPath(format!(
            "{}.{}",
            module_name, root_symbol_name
        )));
    };
    let root_locator = offset_locator(&global_symbol_root.locator, root_symbol_offset_in_bytes);

    match global_symbol_root.symbol_type {
        SymbolicGlobalSymbolRootType::Struct { struct_layout_definition } => resolve_struct_symbol_path_value(
            session,
            &root_locator,
            &struct_layout_definition,
            &relative_symbol_path,
            resolve_global_symbol_roots,
            resolve_type_size_in_bytes,
            read_scalar_field,
            resolve_resolver_definition,
            resolve_struct_definition,
            offset_locator,
        ),
        SymbolicGlobalSymbolRootType::Field { data_type_ref, container_type } => {
            if relative_symbol_path.is_empty() {
                if !matches!(container_type, ContainerType::None) {
                    return Err(SymbolicResolverEvaluationError::UnknownGlobalSymbolPath(format!(
                        "{}.{} is not a scalar field.",
                        module_name, root_symbol_name
                    )));
                }

                let field_definition = SymbolicFieldDefinition::new_named(root_symbol_name.to_string(), data_type_ref, container_type);
                let field_size_in_bytes = resolve_type_size_in_bytes(field_definition.get_data_type_ref())
                    .ok_or_else(|| SymbolicResolverEvaluationError::UnknownTypeSize(field_definition.get_data_type_ref().to_string()))?;

                return read_scalar_field(&root_locator, &field_definition, field_size_in_bytes)
                    .map_err(SymbolicResolverEvaluationError::UnknownGlobalSymbolPath)?
                    .ok_or_else(|| SymbolicResolverEvaluationError::UnknownGlobalSymbolPath(format!("{}.{}", module_name, root_symbol_name)));
            }

            let Some(struct_layout_definition) = resolve_struct_definition(&data_type_ref) else {
                return Err(SymbolicResolverEvaluationError::UnknownGlobalSymbolPath(format!(
                    "{}.{}.{}",
                    module_name, root_symbol_name, relative_symbol_path
                )));
            };

            resolve_struct_symbol_path_value(
                session,
                &root_locator,
                &struct_layout_definition,
                &relative_symbol_path,
                resolve_global_symbol_roots,
                resolve_type_size_in_bytes,
                read_scalar_field,
                resolve_resolver_definition,
                resolve_struct_definition,
                offset_locator,
            )
        }
    }
}

fn resolve_struct_symbol_path_value<
    SymbolLocator,
    ResolveGlobalSymbolRoots,
    ResolveTypeSize,
    ReadScalarField,
    ResolveResolverDefinition,
    ResolveStructDefinition,
    OffsetLocator,
>(
    session: &SymbolicGlobalSymbolResolverSession,
    root_locator: &SymbolLocator,
    struct_layout_definition: &SymbolicStructDefinition,
    symbol_path: &SymbolicResolverRelativeSymbolPath,
    resolve_global_symbol_roots: &ResolveGlobalSymbolRoots,
    resolve_type_size_in_bytes: &ResolveTypeSize,
    read_scalar_field: &ReadScalarField,
    resolve_resolver_definition: &ResolveResolverDefinition,
    resolve_struct_definition: &ResolveStructDefinition,
    offset_locator: &OffsetLocator,
) -> Result<i128, SymbolicResolverEvaluationError>
where
    SymbolLocator: Clone,
    ResolveGlobalSymbolRoots: Fn(&str, &str) -> Vec<SymbolicGlobalSymbolRoot<SymbolLocator>>,
    ResolveTypeSize: Fn(&DataTypeRef) -> Option<u64>,
    ReadScalarField: Fn(&SymbolLocator, &SymbolicFieldDefinition, u64) -> Result<Option<i128>, String>,
    ResolveResolverDefinition: Fn(&str) -> Option<SymbolicResolverDefinition>,
    ResolveStructDefinition: Fn(&DataTypeRef) -> Option<SymbolicStructDefinition>,
    OffsetLocator: Fn(&SymbolLocator, u64) -> SymbolLocator,
{
    let Some(symbol_path_segment) = symbol_path.get_segments().first() else {
        return Err(SymbolicResolverEvaluationError::UnknownRelativeSymbolPath(symbol_path.to_string()));
    };
    let symbol_path_segment = SymbolicResolverRelativeSymbolPath::parse_segment(symbol_path_segment)?;
    let resolved_symbolic_struct = resolve_symbolic_struct_definition_with_resolvers_and_symbol_fields(
        struct_layout_definition,
        resolve_type_size_in_bytes,
        |field_definition, field_offset, field_size_in_bytes| {
            let field_locator = offset_locator(root_locator, field_offset);

            read_scalar_field(&field_locator, field_definition, field_size_in_bytes)
        },
        resolve_resolver_definition,
        resolve_struct_definition,
        |module_name, nested_symbol_path| {
            resolve_global_symbol_field_value(
                session,
                module_name,
                nested_symbol_path,
                resolve_global_symbol_roots,
                resolve_type_size_in_bytes,
                read_scalar_field,
                resolve_resolver_definition,
                resolve_struct_definition,
                offset_locator,
            )
        },
        &SymbolicStructResolverOptions::default(),
    );

    for (field_definition, resolved_symbolic_field) in struct_layout_definition
        .get_fields()
        .iter()
        .zip(resolved_symbolic_struct.get_fields())
    {
        if field_definition.get_field_name() != symbol_path_segment.get_field_name() {
            continue;
        }

        let Some(field_offset) = resolved_symbolic_field.get_offset_in_bytes() else {
            return Err(SymbolicResolverEvaluationError::UnknownRelativeSymbolPath(format_unresolved_symbol_path(
                symbol_path,
                resolved_symbolic_field,
            )));
        };
        let field_locator = offset_locator(root_locator, field_offset.saturating_add(symbol_path_segment.get_offset_in_bytes()));
        let remaining_symbol_path = SymbolicResolverRelativeSymbolPath::new(symbol_path.get_segments().iter().skip(1).cloned().collect());

        if remaining_symbol_path.is_empty() {
            if !matches!(resolved_symbolic_field.get_container_type(), ContainerType::None) {
                return Err(SymbolicResolverEvaluationError::UnknownRelativeSymbolPath(format!(
                    "{} is not a scalar field.",
                    symbol_path
                )));
            }

            let Some(field_size_in_bytes) = resolved_symbolic_field.get_size_in_bytes() else {
                return Err(SymbolicResolverEvaluationError::UnknownRelativeSymbolPath(format_unresolved_symbol_path(
                    symbol_path,
                    resolved_symbolic_field,
                )));
            };

            return read_scalar_field(&field_locator, field_definition, field_size_in_bytes)
                .map_err(SymbolicResolverEvaluationError::UnknownRelativeSymbolPath)?
                .ok_or_else(|| SymbolicResolverEvaluationError::UnknownRelativeSymbolPath(symbol_path.to_string()));
        }

        let Some(nested_struct_layout_definition) = resolve_struct_definition(field_definition.get_data_type_ref()) else {
            return Err(SymbolicResolverEvaluationError::UnknownRelativeSymbolPath(symbol_path.to_string()));
        };

        return resolve_struct_symbol_path_value(
            session,
            &field_locator,
            &nested_struct_layout_definition,
            &remaining_symbol_path,
            resolve_global_symbol_roots,
            resolve_type_size_in_bytes,
            read_scalar_field,
            resolve_resolver_definition,
            resolve_struct_definition,
            offset_locator,
        );
    }

    Err(SymbolicResolverEvaluationError::UnknownRelativeSymbolPath(symbol_path.to_string()))
}

fn format_unresolved_symbol_path(
    symbol_path: &SymbolicResolverRelativeSymbolPath,
    resolved_symbolic_field: &ResolvedSymbolicField,
) -> String {
    match resolved_symbolic_field.get_status() {
        ResolvedSymbolicFieldStatus::Unresolved { reason } => format!("{}: {}", symbol_path, reason),
        ResolvedSymbolicFieldStatus::Clamped { reason, .. } => format!("{}: {}", symbol_path, reason),
        ResolvedSymbolicFieldStatus::Ready => symbol_path.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{SymbolicGlobalSymbolResolverSession, SymbolicGlobalSymbolRoot, SymbolicGlobalSymbolRootType, resolve_global_symbol_field_value};
    use crate::structures::{
        data_types::data_type_ref::DataTypeRef,
        data_values::container_type::ContainerType,
        structs::{
            symbolic_field_definition::SymbolicFieldDefinition,
            symbolic_resolver_definition::{
                SymbolicResolverBinaryOperator, SymbolicResolverDefinition, SymbolicResolverEvaluationError, SymbolicResolverNode,
                SymbolicResolverRelativeSymbolPath,
            },
            symbolic_struct_definition::SymbolicStructDefinition,
        },
    };
    use std::str::FromStr;

    #[test]
    fn global_symbol_resolver_reads_nested_scalar_path() {
        let session = SymbolicGlobalSymbolResolverSession::default();
        let globals_definition = SymbolicStructDefinition::new(
            String::from("Globals"),
            vec![
                SymbolicFieldDefinition::from_str("padding:u8[4]").expect("Expected padding field to parse."),
                SymbolicFieldDefinition::from_str("item_count:u32").expect("Expected item_count field to parse."),
            ],
        );

        let value = resolve_global_symbol_field_value(
            &session,
            "game.exe",
            &SymbolicResolverRelativeSymbolPath::from_dot_path("Globals.item_count"),
            &|module_name, root_symbol_name| {
                if module_name == "game.exe" && root_symbol_name == "Globals" {
                    vec![SymbolicGlobalSymbolRoot::new(
                        0x1000_u64,
                        SymbolicGlobalSymbolRootType::Struct {
                            struct_layout_definition: globals_definition.clone(),
                        },
                    )]
                } else {
                    Vec::new()
                }
            },
            &resolve_test_type_size,
            &|field_address, field_definition, _| {
                if *field_address == 0x1004 && field_definition.get_field_name() == "item_count" {
                    Ok(Some(7))
                } else {
                    Ok(None)
                }
            },
            &|_| None,
            &|_| None,
            &|base_address, offset| base_address.saturating_add(offset),
        )
        .expect("Expected global symbol path to resolve.");

        assert_eq!(value, 7);
    }

    #[test]
    fn global_symbol_resolver_applies_byte_offsets_to_path_segments() {
        let session = SymbolicGlobalSymbolResolverSession::default();
        let item_definition = SymbolicStructDefinition::new(
            String::from("Item"),
            vec![SymbolicFieldDefinition::from_str("value:u32").expect("Expected value field to parse.")],
        );
        let globals_definition = SymbolicStructDefinition::new(
            String::from("Globals"),
            vec![
                SymbolicFieldDefinition::from_str("padding:u8[4]").expect("Expected padding field to parse."),
                SymbolicFieldDefinition::from_str("items:Item[2]").expect("Expected items field to parse."),
            ],
        );

        let value = resolve_global_symbol_field_value(
            &session,
            "game.exe",
            &SymbolicResolverRelativeSymbolPath::from_dot_path("Globals.items+4.value"),
            &|module_name, root_symbol_name| {
                if module_name == "game.exe" && root_symbol_name == "Globals" {
                    vec![SymbolicGlobalSymbolRoot::new(
                        0x1000_u64,
                        SymbolicGlobalSymbolRootType::Struct {
                            struct_layout_definition: globals_definition.clone(),
                        },
                    )]
                } else {
                    Vec::new()
                }
            },
            &resolve_test_type_size,
            &|field_address, field_definition, _| {
                if *field_address == 0x1008 && field_definition.get_field_name() == "value" {
                    Ok(Some(11))
                } else {
                    Ok(None)
                }
            },
            &|_| None,
            &|data_type_ref| (data_type_ref.get_data_type_id() == "Item").then_some(item_definition.clone()),
            &|base_address, offset| base_address.saturating_add(offset),
        )
        .expect("Expected global symbol path to resolve.");

        assert_eq!(value, 11);
    }

    #[test]
    fn global_symbol_resolver_reports_ambiguous_roots() {
        let session = SymbolicGlobalSymbolResolverSession::default();
        let value = resolve_global_symbol_field_value(
            &session,
            "game.exe",
            &SymbolicResolverRelativeSymbolPath::from_dot_path("Globals.item_count"),
            &|_, _| {
                vec![
                    SymbolicGlobalSymbolRoot::new(
                        0x1000_u64,
                        SymbolicGlobalSymbolRootType::Field {
                            data_type_ref: DataTypeRef::new("u32"),
                            container_type: ContainerType::None,
                        },
                    ),
                    SymbolicGlobalSymbolRoot::new(
                        0x2000_u64,
                        SymbolicGlobalSymbolRootType::Field {
                            data_type_ref: DataTypeRef::new("u32"),
                            container_type: ContainerType::None,
                        },
                    ),
                ]
            },
            &resolve_test_type_size,
            &|_, _, _| Ok(None),
            &|_| None,
            &|_| None,
            &|base_address, offset| base_address.saturating_add(offset),
        );

        assert!(matches!(
            value,
            Err(SymbolicResolverEvaluationError::AmbiguousGlobalSymbolPath(symbol_path)) if symbol_path == "game.exe.Globals"
        ));
    }

    #[test]
    fn global_symbol_resolver_reports_recursive_global_cycles() {
        let session = SymbolicGlobalSymbolResolverSession::default();
        let root_definition = SymbolicStructDefinition::new(
            String::from("Globals"),
            vec![SymbolicFieldDefinition::from_str("value:u32 @ resolver(loop)").expect("Expected value field to parse.")],
        );
        let cyclic_resolver = SymbolicResolverDefinition::new(SymbolicResolverNode::new_binary(
            SymbolicResolverBinaryOperator::Add,
            SymbolicResolverNode::new_global_symbol_field(String::from("game.exe"), SymbolicResolverRelativeSymbolPath::from_dot_path("Globals.value")),
            SymbolicResolverNode::new_literal(1),
        ));

        let value = resolve_global_symbol_field_value(
            &session,
            "game.exe",
            &SymbolicResolverRelativeSymbolPath::from_dot_path("Globals.value"),
            &|module_name, root_symbol_name| {
                if module_name == "game.exe" && root_symbol_name == "Globals" {
                    vec![SymbolicGlobalSymbolRoot::new(
                        0x1000_u64,
                        SymbolicGlobalSymbolRootType::Struct {
                            struct_layout_definition: root_definition.clone(),
                        },
                    )]
                } else {
                    Vec::new()
                }
            },
            &resolve_test_type_size,
            &|_, _, _| Ok(Some(1)),
            &|resolver_id| (resolver_id == "loop").then_some(cyclic_resolver.clone()),
            &|_| None,
            &|base_address, offset| base_address.saturating_add(offset),
        );

        assert!(matches!(
            value,
            Err(SymbolicResolverEvaluationError::UnknownRelativeSymbolPath(symbol_path))
                if symbol_path.contains("Resolver cycle detected at `game.exe.Globals.value`")
        ));
    }

    fn resolve_test_type_size(data_type_ref: &DataTypeRef) -> Option<u64> {
        match data_type_ref.get_data_type_id() {
            "u8" => Some(1),
            "u32" => Some(4),
            "Item" => Some(4),
            _ => None,
        }
    }
}
