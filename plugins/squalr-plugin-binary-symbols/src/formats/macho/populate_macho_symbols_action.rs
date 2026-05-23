use squalr_engine_api::{
    plugins::{
        PluginPermission,
        symbol_tree::symbol_tree_action::{
            DataTypeRegistryStore, ProcessMemoryStore, SymbolTreeAction, SymbolTreeActionContext, SymbolTreeActionSelection, SymbolTreeActionServices,
        },
    },
    registries::symbols::{struct_layout_descriptor::StructLayoutDescriptor, symbolic_resolver_descriptor::SymbolicResolverDescriptor},
    structures::{
        data_types::data_type_ref::DataTypeRef,
        data_values::container_type::ContainerType,
        projects::{
            project_symbol_catalog::ProjectSymbolCatalog,
            project_symbol_module::ProjectSymbolModule,
            project_symbol_module_field::ProjectSymbolModuleField,
            symbol_layouts::symbol_layout_field_materializer::{SymbolLayoutFieldMaterializer, SymbolLayoutPositionedField},
        },
        structs::{
            symbolic_field_definition::{SymbolicFieldDefinition, SymbolicFieldOffsetResolution},
            symbolic_resolver_definition::{SymbolicResolverDefinition, SymbolicResolverNode, SymbolicResolverRelativeSymbolPath},
            symbolic_struct_definition::SymbolicStructDefinition,
        },
    },
};
use std::collections::{BTreeMap, HashSet};
use std::str::FromStr;

const MACHO_HEADERS32_ID: &str = "mac.macho.MACHO_HEADERS32";
const MACHO_HEADERS64_ID: &str = "mac.macho.MACHO_HEADERS64";
const MACH_HEADER32_ID: &str = "mac.macho.mach_header";
const MACH_HEADER64_ID: &str = "mac.macho.mach_header_64";
const MACHO_RESOLVER_LOAD_COMMANDS_SIZE_ID: &str = "mac.macho.resolver.load_commands_size";
const MACHO_HEADER32_SIZE_IN_BYTES: u64 = 28;
const MACHO_HEADER64_SIZE_IN_BYTES: u64 = 32;
const INITIAL_MACHO_HEADER_READ_SIZE: u64 = 0x1000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MachOByteOrder {
    LittleEndian,
    BigEndian,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MachOHeaderKind {
    Mach32(MachOByteOrder),
    Mach64(MachOByteOrder),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MachOHeaderLayout {
    header_kind: MachOHeaderKind,
    load_command_count: u64,
    load_commands_size: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DesiredModuleField {
    display_name: String,
    offset: u64,
    struct_layout_id: String,
    size_in_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PositionedLayoutField {
    offset: u64,
    size_in_bytes: u64,
    field_definition: SymbolicFieldDefinition,
}

pub struct PopulateMachOSymbolsAction;

impl SymbolTreeAction for PopulateMachOSymbolsAction {
    fn action_id(&self) -> &'static str {
        "builtin.symbols.binary.populate-macho-symbols"
    }

    fn label(
        &self,
        _context: &SymbolTreeActionContext,
    ) -> String {
        String::from("Populate Mach-O Symbols")
    }

    fn is_visible(
        &self,
        context: &SymbolTreeActionContext,
    ) -> bool {
        matches!(context.get_selection(), SymbolTreeActionSelection::ModuleRoot { .. })
    }

    fn required_permissions(&self) -> &'static [PluginPermission] {
        &[
            PluginPermission::ReadSymbolStore,
            PluginPermission::WriteSymbolStore,
            PluginPermission::ReadSymbolTreeWindow,
            PluginPermission::WriteSymbolTreeWindow,
            PluginPermission::ReadProcessMemory,
        ]
    }

    fn execute(
        &self,
        context: &SymbolTreeActionContext,
        services: &dyn SymbolTreeActionServices,
    ) -> Result<(), String> {
        let SymbolTreeActionSelection::ModuleRoot { module_name } = context.get_selection() else {
            return Err(String::from("Mach-O symbol population requires a module root selection."));
        };
        let module_name = module_name.clone();
        let module_name_for_update = module_name.clone();
        let macho_header_layout = analyze_macho_header_layout(services.process_memory(), &module_name)?;
        let data_type_size_by_id = collect_data_type_size_by_id(services.data_type_registry());

        services.symbol_store().write_catalog(
            "populate Mach-O symbols",
            Box::new(move |project_symbol_catalog| {
                populate_macho_symbols(project_symbol_catalog, &module_name_for_update, &macho_header_layout, &data_type_size_by_id)
            }),
        )?;
        services.symbol_tree_window().request_refresh();
        services
            .symbol_tree_window()
            .focus_tree_node(&format!("module:{module_name}"));

        Ok(())
    }
}

fn populate_macho_symbols(
    project_symbol_catalog: &mut ProjectSymbolCatalog,
    module_name: &str,
    macho_header_layout: &MachOHeaderLayout,
    data_type_size_by_id: &BTreeMap<String, u64>,
) -> Result<(), String> {
    upsert_macho_symbolic_resolver_descriptors(project_symbol_catalog);
    upsert_macho_struct_layout_descriptors(project_symbol_catalog)?;
    upsert_macho_module_fields(project_symbol_catalog, module_name, macho_header_layout, data_type_size_by_id)
}

fn upsert_macho_symbolic_resolver_descriptors(project_symbol_catalog: &mut ProjectSymbolCatalog) {
    let mut symbolic_resolver_descriptors = project_symbol_catalog
        .get_symbolic_resolver_descriptors()
        .to_vec();

    upsert_symbolic_resolver_descriptor(
        &mut symbolic_resolver_descriptors,
        relative_symbol_field_resolver_descriptor(MACHO_RESOLVER_LOAD_COMMANDS_SIZE_ID, &["Header", "sizeofcmds"]),
    );

    project_symbol_catalog.set_symbolic_resolver_descriptors(symbolic_resolver_descriptors);
}

fn upsert_symbolic_resolver_descriptor(
    symbolic_resolver_descriptors: &mut Vec<SymbolicResolverDescriptor>,
    new_symbolic_resolver_descriptor: SymbolicResolverDescriptor,
) {
    if let Some(existing_symbolic_resolver_descriptor) = symbolic_resolver_descriptors
        .iter_mut()
        .find(|resolver_descriptor| resolver_descriptor.get_resolver_id() == new_symbolic_resolver_descriptor.get_resolver_id())
    {
        *existing_symbolic_resolver_descriptor = new_symbolic_resolver_descriptor;
        return;
    }

    symbolic_resolver_descriptors.push(new_symbolic_resolver_descriptor);
}

fn relative_symbol_field_resolver_descriptor(
    resolver_id: &str,
    symbol_path_segments: &[&str],
) -> SymbolicResolverDescriptor {
    SymbolicResolverDescriptor::new(
        resolver_id.to_string(),
        SymbolicResolverDefinition::new(SymbolicResolverNode::new_relative_symbol_field(SymbolicResolverRelativeSymbolPath::new(
            symbol_path_segments
                .iter()
                .map(|symbol_path_segment| symbol_path_segment.to_string())
                .collect(),
        ))),
    )
}

fn upsert_macho_struct_layout_descriptors(project_symbol_catalog: &mut ProjectSymbolCatalog) -> Result<(), String> {
    let mut struct_layout_descriptors = project_symbol_catalog.get_struct_layout_descriptors().to_vec();

    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, macho_headers32_descriptor()?);
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, macho_headers64_descriptor()?);
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, mach_header32_descriptor());
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, mach_header64_descriptor());
    project_symbol_catalog.set_struct_layout_descriptors(struct_layout_descriptors);

    Ok(())
}

fn upsert_struct_layout_descriptor(
    struct_layout_descriptors: &mut Vec<StructLayoutDescriptor>,
    new_struct_layout_descriptor: StructLayoutDescriptor,
) {
    if let Some(existing_struct_layout_descriptor) = struct_layout_descriptors
        .iter_mut()
        .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == new_struct_layout_descriptor.get_struct_layout_id())
    {
        *existing_struct_layout_descriptor = new_struct_layout_descriptor;
        return;
    }

    struct_layout_descriptors.push(new_struct_layout_descriptor);
}

fn upsert_macho_module_fields(
    project_symbol_catalog: &mut ProjectSymbolCatalog,
    module_name: &str,
    macho_header_layout: &MachOHeaderLayout,
    data_type_size_by_id: &BTreeMap<String, u64>,
) -> Result<(), String> {
    let desired_module_fields = build_desired_macho_module_fields(macho_header_layout)?;
    let minimum_size = desired_module_fields
        .iter()
        .filter_map(|desired_module_field| {
            desired_module_field
                .offset
                .checked_add(desired_module_field.size_in_bytes)
        })
        .max()
        .unwrap_or(macho_header_layout.header_size_in_bytes());

    project_symbol_catalog.ensure_symbol_module(module_name, minimum_size);
    let Some(symbol_module) = project_symbol_catalog.find_symbol_module_mut(module_name) else {
        return Err(format!("Could not resolve module `{module_name}` after creating it."));
    };

    upsert_module_fields_in_module(symbol_module, &desired_module_fields, "Mach-O")?;
    let module_size = symbol_module.get_size();
    project_symbol_catalog.ensure_module_root_struct_layout(module_name, module_size, |data_type_ref| {
        data_type_size_by_id
            .get(data_type_ref.get_data_type_id())
            .copied()
    });
    upsert_module_root_layout_fields(project_symbol_catalog, module_name, &desired_module_fields, module_size, data_type_size_by_id)
}

fn build_desired_macho_module_fields(macho_header_layout: &MachOHeaderLayout) -> Result<Vec<DesiredModuleField>, String> {
    let macho_headers_size = macho_header_layout
        .header_size_in_bytes()
        .checked_add(macho_header_layout.load_commands_size)
        .ok_or_else(|| String::from("Mach-O headers size is too large."))?;

    Ok(vec![DesiredModuleField {
        display_name: String::from("Mach-O Headers"),
        offset: 0,
        struct_layout_id: macho_header_layout.headers_struct_layout_id().to_string(),
        size_in_bytes: macho_headers_size,
    }])
}

fn upsert_module_fields_in_module(
    symbol_module: &mut ProjectSymbolModule,
    desired_module_fields: &[DesiredModuleField],
    field_family_name: &str,
) -> Result<(), String> {
    let module_fields = symbol_module.get_fields_mut();
    let desired_field_ranges = desired_module_fields
        .iter()
        .map(|desired_module_field| {
            desired_module_field
                .offset
                .checked_add(desired_module_field.size_in_bytes)
                .map(|desired_field_end| (desired_module_field.offset, desired_field_end))
                .ok_or_else(|| format!("{field_family_name} field `{}` range is too large.", desired_module_field.display_name))
        })
        .collect::<Result<Vec<_>, _>>()?;

    module_fields.retain(|module_field| {
        !desired_field_ranges
            .iter()
            .any(|desired_field_range| module_field_overlaps_desired_range(module_field, *desired_field_range))
    });

    for desired_module_field in desired_module_fields {
        module_fields.push(ProjectSymbolModuleField::new(
            desired_module_field.display_name.clone(),
            desired_module_field.offset,
            desired_module_field.struct_layout_id.clone(),
        ));
    }

    sort_module_fields(module_fields);

    Ok(())
}

fn upsert_module_root_layout_fields(
    project_symbol_catalog: &mut ProjectSymbolCatalog,
    module_name: &str,
    desired_module_fields: &[DesiredModuleField],
    module_size: u64,
    data_type_size_by_id: &BTreeMap<String, u64>,
) -> Result<(), String> {
    let mut struct_layout_descriptors = project_symbol_catalog.get_struct_layout_descriptors().to_vec();
    let Some(module_root_layout_descriptor) = struct_layout_descriptors
        .iter_mut()
        .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == module_name)
    else {
        return Ok(());
    };
    let module_root_layout_definition = module_root_layout_descriptor
        .get_struct_layout_definition()
        .clone();
    let desired_field_ranges = desired_module_fields
        .iter()
        .map(|desired_module_field| {
            desired_module_field
                .offset
                .checked_add(desired_module_field.size_in_bytes)
                .map(|desired_field_end| (desired_module_field.offset, desired_field_end))
                .ok_or_else(|| format!("Module field `{}` range is too large.", desired_module_field.display_name))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut positioned_fields = collect_positioned_layout_fields(project_symbol_catalog, &module_root_layout_definition, data_type_size_by_id)
        .into_iter()
        .filter(|positioned_field| {
            !desired_field_ranges
                .iter()
                .any(|desired_field_range| positioned_layout_field_overlaps_range(positioned_field, *desired_field_range))
        })
        .collect::<Vec<_>>();

    positioned_fields.extend(
        desired_module_fields
            .iter()
            .map(|desired_module_field| PositionedLayoutField {
                offset: desired_module_field.offset,
                size_in_bytes: desired_module_field.size_in_bytes,
                field_definition: SymbolicFieldDefinition::new_named(
                    desired_module_field.display_name.clone(),
                    DataTypeRef::new(&desired_module_field.struct_layout_id),
                    ContainerType::None,
                ),
            }),
    );
    positioned_fields.sort_by(|left_field, right_field| {
        left_field.offset.cmp(&right_field.offset).then_with(|| {
            left_field
                .field_definition
                .get_field_name()
                .cmp(right_field.field_definition.get_field_name())
        })
    });

    let maximum_field_end = positioned_fields
        .iter()
        .filter_map(|positioned_field| {
            positioned_field
                .offset
                .checked_add(positioned_field.size_in_bytes)
        })
        .max()
        .unwrap_or(0);
    let declared_size_in_bytes = module_root_layout_definition
        .get_declared_size_in_bytes()
        .unwrap_or(0)
        .max(module_size)
        .max(maximum_field_end);
    let rebuilt_fields = SymbolLayoutFieldMaterializer::materialize_positioned_fields(
        module_root_layout_definition.get_layout_kind(),
        Some(declared_size_in_bytes),
        positioned_fields
            .into_iter()
            .map(|positioned_field| {
                SymbolLayoutPositionedField::new(positioned_field.offset, positioned_field.size_in_bytes, positioned_field.field_definition)
            })
            .collect(),
    )?;
    let rebuilt_module_root_layout_definition = SymbolicStructDefinition::new_with_layout_kind(
        module_root_layout_definition.get_symbol_namespace().to_string(),
        module_root_layout_definition.get_layout_kind(),
        rebuilt_fields,
    )
    .with_declared_size_in_bytes(Some(declared_size_in_bytes));

    *module_root_layout_descriptor = StructLayoutDescriptor::new(module_name.to_string(), rebuilt_module_root_layout_definition);
    project_symbol_catalog.set_struct_layout_descriptors(struct_layout_descriptors);

    Ok(())
}

fn collect_positioned_layout_fields(
    project_symbol_catalog: &ProjectSymbolCatalog,
    symbolic_struct_definition: &SymbolicStructDefinition,
    data_type_size_by_id: &BTreeMap<String, u64>,
) -> Vec<PositionedLayoutField> {
    let mut positioned_fields = Vec::new();
    let mut next_sequential_offset = 0_u64;

    for field_definition in symbolic_struct_definition.get_fields() {
        if field_definition.is_unassigned() {
            next_sequential_offset = next_sequential_offset.saturating_add(field_definition.get_unassigned_size_in_bytes().unwrap_or(0));
            continue;
        }

        let field_offset = match field_definition.get_offset_resolution() {
            SymbolicFieldOffsetResolution::Static(offset_in_bytes) => *offset_in_bytes,
            SymbolicFieldOffsetResolution::Sequential | SymbolicFieldOffsetResolution::Resolver(_)
                if symbolic_struct_definition.get_layout_kind().is_union() =>
            {
                0
            }
            SymbolicFieldOffsetResolution::Sequential | SymbolicFieldOffsetResolution::Resolver(_) => next_sequential_offset,
        };
        let field_size_in_bytes =
            estimate_symbolic_field_size_in_bytes(project_symbol_catalog, field_definition, data_type_size_by_id, &mut HashSet::new()).max(1);

        next_sequential_offset = next_sequential_offset.max(field_offset.saturating_add(field_size_in_bytes));
        positioned_fields.push(PositionedLayoutField {
            offset: field_offset,
            size_in_bytes: field_size_in_bytes,
            field_definition: field_definition.clone(),
        });
    }

    positioned_fields
}

fn positioned_layout_field_overlaps_range(
    positioned_field: &PositionedLayoutField,
    desired_field_range: (u64, u64),
) -> bool {
    let Some(field_end_offset) = positioned_field
        .offset
        .checked_add(positioned_field.size_in_bytes)
    else {
        return false;
    };

    positioned_field.offset < desired_field_range.1 && desired_field_range.0 < field_end_offset
}

fn module_field_overlaps_desired_range(
    module_field: &ProjectSymbolModuleField,
    desired_field_range: (u64, u64),
) -> bool {
    let (desired_field_start, desired_field_end) = desired_field_range;

    if module_field.get_offset() >= desired_field_start && module_field.get_offset() < desired_field_end {
        return true;
    }

    let Some(module_field_size) = resolve_known_module_field_size(module_field.get_struct_layout_id()) else {
        return false;
    };
    let Some(module_field_end) = module_field.get_offset().checked_add(module_field_size) else {
        return false;
    };

    module_field.get_offset() < desired_field_end && desired_field_start < module_field_end
}

fn analyze_macho_header_layout(
    process_memory_store: &dyn ProcessMemoryStore,
    module_name: &str,
) -> Result<MachOHeaderLayout, String> {
    let header_bytes = process_memory_store.read_module_bytes(module_name, 0, INITIAL_MACHO_HEADER_READ_SIZE)?;

    read_macho_layout_from_header_bytes(&header_bytes)
}

fn read_macho_layout_from_header_bytes(header_bytes: &[u8]) -> Result<MachOHeaderLayout, String> {
    if header_bytes.len() < 4 {
        return Err(String::from("Module header read is too small for a Mach-O magic."));
    }

    let header_kind = match header_bytes.get(0..4) {
        Some([0xCE, 0xFA, 0xED, 0xFE]) => MachOHeaderKind::Mach32(MachOByteOrder::LittleEndian),
        Some([0xFE, 0xED, 0xFA, 0xCE]) => MachOHeaderKind::Mach32(MachOByteOrder::BigEndian),
        Some([0xCF, 0xFA, 0xED, 0xFE]) => MachOHeaderKind::Mach64(MachOByteOrder::LittleEndian),
        Some([0xFE, 0xED, 0xFA, 0xCF]) => MachOHeaderKind::Mach64(MachOByteOrder::BigEndian),
        Some([0xCA, 0xFE, 0xBA, 0xBE]) | Some([0xBE, 0xBA, 0xFE, 0xCA]) => {
            return Err(String::from("Universal Mach-O binaries are not yet supported for header population."));
        }
        _ => return Err(String::from("Selected module does not start with a supported Mach-O signature.")),
    };

    let header_size_in_bytes = header_kind.header_size_in_bytes() as usize;
    if header_bytes.len() < header_size_in_bytes {
        return Err(format!("Module header read is too small for a {} header.", header_kind.display_name()));
    }

    let byte_order = header_kind.byte_order();
    let load_command_count = read_u32(&header_bytes[16..20], byte_order) as u64;
    let load_commands_size = read_u32(&header_bytes[20..24], byte_order) as u64;

    Ok(MachOHeaderLayout {
        header_kind,
        load_command_count,
        load_commands_size,
    })
}

impl MachOHeaderLayout {
    fn header_size_in_bytes(&self) -> u64 {
        self.header_kind.header_size_in_bytes()
    }

    fn headers_struct_layout_id(&self) -> &'static str {
        self.header_kind.headers_struct_layout_id()
    }
}

impl MachOHeaderKind {
    fn header_size_in_bytes(&self) -> u64 {
        match self {
            Self::Mach32(_) => MACHO_HEADER32_SIZE_IN_BYTES,
            Self::Mach64(_) => MACHO_HEADER64_SIZE_IN_BYTES,
        }
    }

    fn headers_struct_layout_id(&self) -> &'static str {
        match self {
            Self::Mach32(_) => MACHO_HEADERS32_ID,
            Self::Mach64(_) => MACHO_HEADERS64_ID,
        }
    }

    fn byte_order(&self) -> MachOByteOrder {
        match self {
            Self::Mach32(byte_order) | Self::Mach64(byte_order) => *byte_order,
        }
    }

    fn display_name(&self) -> &'static str {
        match self {
            Self::Mach32(_) => "Mach-O 32-bit",
            Self::Mach64(_) => "Mach-O 64-bit",
        }
    }
}

fn mach_header32_descriptor() -> StructLayoutDescriptor {
    struct_layout_descriptor(
        MACH_HEADER32_ID,
        vec![
            field("magic", "u32"),
            field("cputype", "i32"),
            field("cpusubtype", "i32"),
            field("filetype", "u32"),
            field("ncmds", "u32"),
            field("sizeofcmds", "u32"),
            field("flags", "u32"),
        ],
    )
}

fn mach_header64_descriptor() -> StructLayoutDescriptor {
    struct_layout_descriptor(
        MACH_HEADER64_ID,
        vec![
            field("magic", "u32"),
            field("cputype", "i32"),
            field("cpusubtype", "i32"),
            field("filetype", "u32"),
            field("ncmds", "u32"),
            field("sizeofcmds", "u32"),
            field("flags", "u32"),
            field("reserved", "u32"),
        ],
    )
}

fn macho_headers32_descriptor() -> Result<StructLayoutDescriptor, String> {
    macho_headers_descriptor(MACHO_HEADERS32_ID, MACH_HEADER32_ID, MACHO_HEADER32_SIZE_IN_BYTES)
}

fn macho_headers64_descriptor() -> Result<StructLayoutDescriptor, String> {
    macho_headers_descriptor(MACHO_HEADERS64_ID, MACH_HEADER64_ID, MACHO_HEADER64_SIZE_IN_BYTES)
}

fn macho_headers_descriptor(
    struct_layout_id: &str,
    mach_header_struct_layout_id: &str,
    load_commands_offset: u64,
) -> Result<StructLayoutDescriptor, String> {
    Ok(struct_layout_descriptor(
        struct_layout_id,
        vec![
            expression_field(&format!("Header:{mach_header_struct_layout_id}"))?,
            expression_field(&format!(
                "LoadCommands:u8[resolver({})] @ +0x{:X}",
                MACHO_RESOLVER_LOAD_COMMANDS_SIZE_ID, load_commands_offset
            ))?,
        ],
    ))
}

fn struct_layout_descriptor(
    struct_layout_id: &str,
    fields: Vec<SymbolicFieldDefinition>,
) -> StructLayoutDescriptor {
    StructLayoutDescriptor::new(
        struct_layout_id.to_string(),
        SymbolicStructDefinition::new(struct_layout_id.to_string(), fields),
    )
}

fn field(
    field_name: &str,
    data_type_id: &str,
) -> SymbolicFieldDefinition {
    SymbolicFieldDefinition::new_named(field_name.to_string(), DataTypeRef::new(data_type_id), ContainerType::None)
}

fn expression_field(field_definition: &str) -> Result<SymbolicFieldDefinition, String> {
    SymbolicFieldDefinition::from_str(field_definition)
        .map_err(|parse_error| format!("Invalid built-in Mach-O symbolic field `{field_definition}`: {parse_error}"))
}

fn collect_data_type_size_by_id(data_type_registry: &dyn DataTypeRegistryStore) -> BTreeMap<String, u64> {
    data_type_registry
        .get_registered_data_type_refs()
        .into_iter()
        .filter_map(|data_type_ref| {
            let unit_size_in_bytes = data_type_registry.get_unit_size_in_bytes(&data_type_ref);

            (unit_size_in_bytes > 0).then(|| (data_type_ref.get_data_type_id().to_string(), unit_size_in_bytes))
        })
        .collect()
}

fn resolve_known_module_field_size(struct_layout_id: &str) -> Option<u64> {
    if let Some(u8_array_length) = resolve_u8_array_length(struct_layout_id) {
        return Some(u8_array_length);
    }

    match struct_layout_id {
        MACH_HEADER32_ID => Some(MACHO_HEADER32_SIZE_IN_BYTES),
        MACH_HEADER64_ID => Some(MACHO_HEADER64_SIZE_IN_BYTES),
        _ => None,
    }
}

fn estimate_symbolic_field_size_in_bytes(
    project_symbol_catalog: &ProjectSymbolCatalog,
    field_definition: &SymbolicFieldDefinition,
    data_type_size_by_id: &BTreeMap<String, u64>,
    visited_struct_layout_ids: &mut HashSet<String>,
) -> u64 {
    if let Some(unassigned_size_in_bytes) = field_definition.get_unassigned_size_in_bytes() {
        return unassigned_size_in_bytes;
    }

    let unit_size_in_bytes = estimate_data_type_size_in_bytes(
        project_symbol_catalog,
        field_definition.get_data_type_ref().get_data_type_id(),
        data_type_size_by_id,
        visited_struct_layout_ids,
    );

    field_definition
        .get_container_type()
        .get_total_size_in_bytes(unit_size_in_bytes)
}

fn estimate_data_type_size_in_bytes(
    project_symbol_catalog: &ProjectSymbolCatalog,
    data_type_id: &str,
    data_type_size_by_id: &BTreeMap<String, u64>,
    visited_struct_layout_ids: &mut HashSet<String>,
) -> u64 {
    if let Some(known_size_in_bytes) = data_type_size_by_id
        .get(data_type_id)
        .copied()
        .or_else(|| resolve_known_module_field_size(data_type_id))
    {
        return known_size_in_bytes;
    }

    if !visited_struct_layout_ids.insert(data_type_id.to_string()) {
        return 0;
    }

    let size_in_bytes = project_symbol_catalog
        .get_struct_layout_descriptors()
        .iter()
        .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == data_type_id)
        .map(|struct_layout_descriptor| {
            estimate_symbolic_struct_size_in_bytes(
                project_symbol_catalog,
                struct_layout_descriptor.get_struct_layout_definition(),
                data_type_size_by_id,
                visited_struct_layout_ids,
            )
        })
        .or_else(|| {
            if data_type_id.contains(';') {
                SymbolicStructDefinition::from_str(data_type_id)
                    .ok()
                    .map(|symbolic_struct_definition| {
                        estimate_symbolic_struct_size_in_bytes(
                            project_symbol_catalog,
                            &symbolic_struct_definition,
                            data_type_size_by_id,
                            visited_struct_layout_ids,
                        )
                    })
            } else {
                None
            }
        })
        .unwrap_or(1);

    visited_struct_layout_ids.remove(data_type_id);

    size_in_bytes
}

fn estimate_symbolic_struct_size_in_bytes(
    project_symbol_catalog: &ProjectSymbolCatalog,
    symbolic_struct_definition: &SymbolicStructDefinition,
    data_type_size_by_id: &BTreeMap<String, u64>,
    visited_struct_layout_ids: &mut HashSet<String>,
) -> u64 {
    let mut next_sequential_offset = 0_u64;

    for field_definition in symbolic_struct_definition.get_fields() {
        if field_definition.is_unassigned() {
            next_sequential_offset = next_sequential_offset.saturating_add(field_definition.get_unassigned_size_in_bytes().unwrap_or(0));
            continue;
        }

        let field_offset = match field_definition.get_offset_resolution() {
            SymbolicFieldOffsetResolution::Static(offset_in_bytes) => *offset_in_bytes,
            SymbolicFieldOffsetResolution::Sequential | SymbolicFieldOffsetResolution::Resolver(_)
                if symbolic_struct_definition.get_layout_kind().is_union() =>
            {
                0
            }
            SymbolicFieldOffsetResolution::Sequential | SymbolicFieldOffsetResolution::Resolver(_) => next_sequential_offset,
        };
        let field_size_in_bytes =
            estimate_symbolic_field_size_in_bytes(project_symbol_catalog, field_definition, data_type_size_by_id, visited_struct_layout_ids);

        next_sequential_offset = next_sequential_offset.max(field_offset.saturating_add(field_size_in_bytes));
    }

    next_sequential_offset.max(
        symbolic_struct_definition
            .get_declared_size_in_bytes()
            .unwrap_or(0),
    )
}

fn resolve_u8_array_length(struct_layout_id: &str) -> Option<u64> {
    resolve_fixed_array_length(struct_layout_id, "u8")
}

fn resolve_fixed_array_length(
    struct_layout_id: &str,
    data_type_id: &str,
) -> Option<u64> {
    let length_text = struct_layout_id
        .strip_prefix(data_type_id)?
        .strip_prefix('[')?
        .strip_suffix(']')?;

    length_text.parse::<u64>().ok().filter(|length| *length > 0)
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

fn read_u32(
    bytes: &[u8],
    byte_order: MachOByteOrder,
) -> u32 {
    let bytes = [bytes[0], bytes[1], bytes[2], bytes[3]];

    match byte_order {
        MachOByteOrder::LittleEndian => u32::from_le_bytes(bytes),
        MachOByteOrder::BigEndian => u32::from_be_bytes(bytes),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        MACHO_HEADERS64_ID, MACHO_RESOLVER_LOAD_COMMANDS_SIZE_ID, MachOByteOrder, MachOHeaderKind, MachOHeaderLayout, PopulateMachOSymbolsAction,
        populate_macho_symbols, relative_symbol_field_resolver_descriptor,
    };
    use squalr_engine_api::{
        plugins::symbol_tree::symbol_tree_action::{
            DataTypeRegistryStore, ProcessMemoryStore, ProjectSymbolStore, SymbolTreeAction, SymbolTreeActionContext, SymbolTreeActionSelection,
            SymbolTreeActionServices, SymbolTreeWindowStore,
        },
        registries::symbols::{struct_layout_descriptor::StructLayoutDescriptor, symbol_registry::SymbolRegistry},
        structures::{
            data_types::data_type_ref::DataTypeRef,
            projects::{
                project_symbol_catalog::ProjectSymbolCatalog, project_symbol_module::ProjectSymbolModule, project_symbol_module_field::ProjectSymbolModuleField,
            },
            structs::{symbolic_resolver_definition::SymbolicResolverNode, symbolic_struct_definition::SymbolicStructDefinition},
        },
    };
    use std::{
        collections::BTreeMap,
        sync::{Arc, Mutex},
    };

    struct TestProjectSymbolStore {
        project_symbol_catalog: Arc<Mutex<ProjectSymbolCatalog>>,
    }

    impl TestProjectSymbolStore {
        fn new(project_symbol_catalog: ProjectSymbolCatalog) -> Self {
            Self {
                project_symbol_catalog: Arc::new(Mutex::new(project_symbol_catalog)),
            }
        }

        fn read_current_catalog(&self) -> ProjectSymbolCatalog {
            self.project_symbol_catalog
                .lock()
                .expect("Expected test catalog lock.")
                .clone()
        }
    }

    impl ProjectSymbolStore for TestProjectSymbolStore {
        fn read_catalog(&self) -> Result<ProjectSymbolCatalog, String> {
            Ok(self.read_current_catalog())
        }

        fn write_catalog(
            &self,
            _reason: &str,
            update_catalog: Box<dyn FnOnce(&mut ProjectSymbolCatalog) -> Result<(), String> + Send>,
        ) -> Result<(), String> {
            let mut project_symbol_catalog = self
                .project_symbol_catalog
                .lock()
                .map_err(|error| error.to_string())?;

            update_catalog(&mut project_symbol_catalog)
        }
    }

    struct TestProcessMemoryStore {
        header_bytes: Vec<u8>,
    }

    impl TestProcessMemoryStore {
        fn new() -> Self {
            Self {
                header_bytes: build_test_macho_header_bytes(),
            }
        }
    }

    impl ProcessMemoryStore for TestProcessMemoryStore {
        fn read_module_bytes(
            &self,
            _module_name: &str,
            offset: u64,
            length: u64,
        ) -> Result<Vec<u8>, String> {
            let read_start = offset as usize;
            let read_end = read_start.saturating_add(length as usize);

            Ok(self.header_bytes[read_start..read_end.min(self.header_bytes.len())].to_vec())
        }
    }

    struct TestSymbolTreeWindowStore;

    impl SymbolTreeWindowStore for TestSymbolTreeWindowStore {
        fn request_refresh(&self) {}

        fn focus_tree_node(
            &self,
            _tree_node_key: &str,
        ) {
        }
    }

    struct TestSymbolTreeActionServices {
        project_symbol_store: TestProjectSymbolStore,
        process_memory_store: TestProcessMemoryStore,
        data_type_registry: SymbolRegistry,
        symbol_tree_window_store: TestSymbolTreeWindowStore,
    }

    impl TestSymbolTreeActionServices {
        fn new(project_symbol_catalog: ProjectSymbolCatalog) -> Self {
            Self {
                project_symbol_store: TestProjectSymbolStore::new(project_symbol_catalog),
                process_memory_store: TestProcessMemoryStore::new(),
                data_type_registry: SymbolRegistry::new(),
                symbol_tree_window_store: TestSymbolTreeWindowStore,
            }
        }
    }

    impl DataTypeRegistryStore for TestSymbolTreeActionServices {
        fn get_registered_data_type_refs(&self) -> Vec<DataTypeRef> {
            self.data_type_registry.get_registered_data_type_refs()
        }

        fn get_unit_size_in_bytes(
            &self,
            data_type_ref: &DataTypeRef,
        ) -> u64 {
            self.data_type_registry.get_unit_size_in_bytes(data_type_ref)
        }
    }

    impl SymbolTreeActionServices for TestSymbolTreeActionServices {
        fn symbol_store(&self) -> &dyn ProjectSymbolStore {
            &self.project_symbol_store
        }

        fn process_memory(&self) -> &dyn ProcessMemoryStore {
            &self.process_memory_store
        }

        fn data_type_registry(&self) -> &dyn DataTypeRegistryStore {
            self
        }

        fn symbol_tree_window(&self) -> &dyn SymbolTreeWindowStore {
            &self.symbol_tree_window_store
        }
    }

    #[test]
    fn action_is_visible_only_for_module_roots() {
        let action = PopulateMachOSymbolsAction;
        let module_context = SymbolTreeActionContext::new(SymbolTreeActionSelection::ModuleRoot {
            module_name: String::from("Finder"),
        });
        let derived_context = SymbolTreeActionContext::new(SymbolTreeActionSelection::DerivedNode {
            tree_node_key: String::from("u8:Finder:0:64"),
        });

        assert!(action.is_visible(&module_context));
        assert!(!action.is_visible(&derived_context));
    }

    #[test]
    fn populate_macho_symbols_adds_macho_headers_root() {
        let project_symbol_catalog =
            ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![ProjectSymbolModule::new(String::from("Finder"), 0x2000)], Vec::new(), Vec::new());
        let services = TestSymbolTreeActionServices::new(project_symbol_catalog);
        let action = PopulateMachOSymbolsAction;
        let context = SymbolTreeActionContext::new(SymbolTreeActionSelection::ModuleRoot {
            module_name: String::from("Finder"),
        });

        action
            .execute(&context, &services)
            .expect("Expected Mach-O symbol population to succeed.");

        let project_symbol_catalog = services.project_symbol_store.read_current_catalog();
        let symbol_module = project_symbol_catalog
            .find_symbol_module("Finder")
            .expect("Expected module to exist.");

        assert_eq!(symbol_module.get_fields().len(), 1);
        assert_eq!(symbol_module.get_fields()[0].get_display_name(), "Mach-O Headers");
        assert_eq!(symbol_module.get_fields()[0].get_offset(), 0);
        assert_eq!(symbol_module.get_fields()[0].get_struct_layout_id(), MACHO_HEADERS64_ID);
        let macho_headers_descriptor = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == MACHO_HEADERS64_ID)
            .expect("Expected Mach-O headers descriptor.");
        let macho_header_field_names = macho_headers_descriptor
            .get_struct_layout_definition()
            .get_fields()
            .iter()
            .map(|field_definition| field_definition.get_field_name())
            .collect::<Vec<_>>();

        assert_eq!(macho_header_field_names, vec!["Header", "LoadCommands"]);
        assert!(
            project_symbol_catalog
                .find_symbolic_resolver_descriptor(MACHO_RESOLVER_LOAD_COMMANDS_SIZE_ID)
                .is_some()
        );
        assert_resolver_contains_relative_path(&project_symbol_catalog, MACHO_RESOLVER_LOAD_COMMANDS_SIZE_ID, "Header.sizeofcmds");
    }

    #[test]
    fn populate_macho_symbols_replaces_existing_root_u8_array() {
        let mut symbol_module = ProjectSymbolModule::new(String::from("Finder"), 0x2000);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("u8_00000000"), 0, String::from("u8[512]")));
        let mut project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![symbol_module], Vec::new(), Vec::new());
        let macho_header_layout = MachOHeaderLayout {
            header_kind: MachOHeaderKind::Mach64(MachOByteOrder::LittleEndian),
            load_command_count: 2,
            load_commands_size: 0x48,
        };

        populate_macho_symbols(&mut project_symbol_catalog, "Finder", &macho_header_layout, &default_data_type_size_by_id())
            .expect("Expected Mach-O symbol population to replace u8[] root field.");

        let fields = project_symbol_catalog
            .find_symbol_module("Finder")
            .expect("Expected module to exist.")
            .get_fields();

        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].get_display_name(), "Mach-O Headers");
        assert_eq!(fields[0].get_struct_layout_id(), MACHO_HEADERS64_ID);
    }

    fn assert_resolver_contains_relative_path(
        project_symbol_catalog: &ProjectSymbolCatalog,
        resolver_id: &str,
        expected_dot_path: &str,
    ) {
        let resolver_descriptor = project_symbol_catalog
            .find_symbolic_resolver_descriptor(resolver_id)
            .expect("Expected resolver descriptor.");

        assert!(
            symbolic_resolver_node_contains_relative_dot_path(resolver_descriptor.get_resolver_definition().get_root_node(), expected_dot_path),
            "Expected resolver `{resolver_id}` to contain relative path `{expected_dot_path}`."
        );
    }

    fn symbolic_resolver_node_contains_relative_dot_path(
        symbolic_resolver_node: &SymbolicResolverNode,
        expected_dot_path: &str,
    ) -> bool {
        match symbolic_resolver_node {
            SymbolicResolverNode::RelativeSymbolField { symbol_path } => symbol_path.to_string() == expected_dot_path,
            SymbolicResolverNode::Binary { left_node, right_node, .. } => {
                symbolic_resolver_node_contains_relative_dot_path(left_node, expected_dot_path)
                    || symbolic_resolver_node_contains_relative_dot_path(right_node, expected_dot_path)
            }
            SymbolicResolverNode::Conditional {
                condition_node,
                true_node,
                false_node,
            } => {
                symbolic_resolver_node_contains_relative_dot_path(condition_node, expected_dot_path)
                    || symbolic_resolver_node_contains_relative_dot_path(true_node, expected_dot_path)
                    || symbolic_resolver_node_contains_relative_dot_path(false_node, expected_dot_path)
            }
            SymbolicResolverNode::Literal(_)
            | SymbolicResolverNode::LocalField { .. }
            | SymbolicResolverNode::GlobalSymbolField { .. }
            | SymbolicResolverNode::RelativePointerChain { .. }
            | SymbolicResolverNode::GlobalPointerChain { .. }
            | SymbolicResolverNode::TypeSize { .. } => false,
        }
    }

    fn default_data_type_size_by_id() -> BTreeMap<String, u64> {
        let data_type_registry = SymbolRegistry::new();

        data_type_registry
            .get_registered_data_type_refs()
            .into_iter()
            .filter_map(|data_type_ref| {
                let unit_size_in_bytes = data_type_registry.get_unit_size_in_bytes(&data_type_ref);

                (unit_size_in_bytes > 0).then(|| (data_type_ref.get_data_type_id().to_string(), unit_size_in_bytes))
            })
            .collect()
    }

    fn build_test_macho_header_bytes() -> Vec<u8> {
        let mut header_bytes = vec![0_u8; 0x1000];
        header_bytes[0..4].copy_from_slice(&[0xCF, 0xFA, 0xED, 0xFE]);
        header_bytes[16..20].copy_from_slice(&2_u32.to_le_bytes());
        header_bytes[20..24].copy_from_slice(&0x48_u32.to_le_bytes());

        header_bytes
    }

    #[test]
    fn relative_symbol_field_resolver_keeps_relative_path() {
        let resolver_descriptor = relative_symbol_field_resolver_descriptor(MACHO_RESOLVER_LOAD_COMMANDS_SIZE_ID, &["Header", "sizeofcmds"]);

        assert_eq!(
            match resolver_descriptor.get_resolver_definition().get_root_node() {
                SymbolicResolverNode::RelativeSymbolField { symbol_path } => symbol_path.to_string(),
                _ => panic!("Expected relative path."),
            },
            "Header.sizeofcmds"
        );
    }

    #[test]
    fn populate_macho_symbols_updates_module_root_layout() {
        let module_root_layout_descriptor = StructLayoutDescriptor::new(
            String::from("Finder"),
            SymbolicStructDefinition::new(String::from("Finder"), Vec::new()).with_declared_size_in_bytes(Some(0x2000)),
        );
        let mut project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(
            vec![ProjectSymbolModule::new(String::from("Finder"), 0x2000)],
            vec![module_root_layout_descriptor],
            Vec::new(),
        );
        let macho_header_layout = MachOHeaderLayout {
            header_kind: MachOHeaderKind::Mach64(MachOByteOrder::LittleEndian),
            load_command_count: 2,
            load_commands_size: 0x48,
        };

        populate_macho_symbols(&mut project_symbol_catalog, "Finder", &macho_header_layout, &default_data_type_size_by_id())
            .expect("Expected Mach-O symbol population to update module root layout.");

        let module_root_layout_definition = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == "Finder")
            .expect("Expected Finder root layout.")
            .get_struct_layout_definition();

        assert!(
            module_root_layout_definition
                .get_fields()
                .iter()
                .any(|field_definition| field_definition.get_field_name() == "Mach-O Headers")
        );
    }
}
