use squalr_engine_api::{
    plugins::{
        symbol_tree::symbol_tree_action::{
            DataTypeRegistryStore, ProcessMemoryStore, SymbolTreeAction, SymbolTreeActionContext, SymbolTreeActionSelection, SymbolTreeActionServices,
        },
        PluginPermission,
    },
    registries::symbols::struct_layout_descriptor::StructLayoutDescriptor,
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
            symbolic_struct_definition::SymbolicStructDefinition,
        },
    },
};
use std::collections::{BTreeMap, HashSet};

const MACH_HEADER32_ID: &str = "mac.macho.mach_header";
const MACH_HEADER64_ID: &str = "mac.macho.mach_header_64";
const SECTION32_ID: &str = "mac.macho.section";
const SECTION64_ID: &str = "mac.macho.section_64";
const LOAD_COMMAND32_ID: &str = "mac.macho.load_command";
const LOAD_COMMAND64_ID: &str = "mac.macho.load_command_64";
const SEGMENT_COMMAND_ID: &str = "mac.macho.segment_command";
const SEGMENT_COMMAND64_ID: &str = "mac.macho.segment_command_64";
const SYMTAB_COMMAND_ID: &str = "mac.macho.symtab_command";
const DYSYMTAB_COMMAND_ID: &str = "mac.macho.dysymtab_command";
const UUID_COMMAND_ID: &str = "mac.macho.uuid_command";
const ENTRY_POINT_COMMAND_ID: &str = "mac.macho.entry_point_command";
const BUILD_VERSION_COMMAND_ID: &str = "mac.macho.build_version_command";
const BUILD_TOOL_VERSION_ID: &str = "mac.macho.build_tool_version";
const SOURCE_VERSION_COMMAND_ID: &str = "mac.macho.source_version_command";
const VERSION_MIN_COMMAND_ID: &str = "mac.macho.version_min_command";
const LINKEDIT_DATA_COMMAND_ID: &str = "mac.macho.linkedit_data_command";
const ENCRYPTION_INFO_COMMAND_ID: &str = "mac.macho.encryption_info_command";
const ENCRYPTION_INFO64_COMMAND_ID: &str = "mac.macho.encryption_info_command_64";
const DYLD_INFO_COMMAND_ID: &str = "mac.macho.dyld_info_command";

const MACHO_HEADER32_SIZE_IN_BYTES: u64 = 28;
const MACHO_HEADER64_SIZE_IN_BYTES: u64 = 32;
const LOAD_COMMAND_HEADER_SIZE_IN_BYTES: u64 = 8;
const SEGMENT_COMMAND32_SIZE_IN_BYTES: u64 = 56;
const SEGMENT_COMMAND64_SIZE_IN_BYTES: u64 = 72;
const SECTION32_SIZE_IN_BYTES: u64 = 68;
const SECTION64_SIZE_IN_BYTES: u64 = 80;
const SYMTAB_COMMAND_SIZE_IN_BYTES: u64 = 24;
const DYSYMTAB_COMMAND_SIZE_IN_BYTES: u64 = 80;
const UUID_COMMAND_SIZE_IN_BYTES: u64 = 24;
const ENTRY_POINT_COMMAND_SIZE_IN_BYTES: u64 = 24;
const BUILD_VERSION_COMMAND_SIZE_IN_BYTES: u64 = 24;
const BUILD_TOOL_VERSION_SIZE_IN_BYTES: u64 = 8;
const SOURCE_VERSION_COMMAND_SIZE_IN_BYTES: u64 = 16;
const VERSION_MIN_COMMAND_SIZE_IN_BYTES: u64 = 16;
const LINKEDIT_DATA_COMMAND_SIZE_IN_BYTES: u64 = 16;
const ENCRYPTION_INFO_COMMAND_SIZE_IN_BYTES: u64 = 20;
const ENCRYPTION_INFO64_COMMAND_SIZE_IN_BYTES: u64 = 24;
const DYLD_INFO_COMMAND_SIZE_IN_BYTES: u64 = 48;
const INITIAL_MACHO_HEADER_READ_SIZE: u64 = 0x1000;
const MAX_MACHO_HEADER_READ_SIZE: u64 = 0x200000;
const LC_REQ_DYLD: u32 = 0x8000_0000;
const LC_SEGMENT: u32 = 0x1;
const LC_SYMTAB: u32 = 0x2;
const LC_THREAD: u32 = 0x4;
const LC_UNIXTHREAD: u32 = 0x5;
const LC_DYSYMTAB: u32 = 0xB;
const LC_LOAD_DYLIB: u32 = 0xC;
const LC_ID_DYLIB: u32 = 0xD;
const LC_ID_DYLINKER: u32 = 0xF;
const LC_LOAD_WEAK_DYLIB: u32 = LC_REQ_DYLD | 0x18;
const LC_SEGMENT_64: u32 = 0x19;
const LC_UUID: u32 = 0x1B;
const LC_RPATH: u32 = LC_REQ_DYLD | 0x1C;
const LC_REEXPORT_DYLIB: u32 = LC_REQ_DYLD | 0x1F;
const LC_LAZY_LOAD_DYLIB: u32 = 0x20;
const LC_ENCRYPTION_INFO: u32 = 0x21;
const LC_DYLD_INFO: u32 = 0x22;
const LC_DYLD_INFO_ONLY: u32 = LC_REQ_DYLD | 0x22;
const LC_LOAD_UPWARD_DYLIB: u32 = LC_REQ_DYLD | 0x23;
const LC_CODE_SIGNATURE: u32 = 0x1D;
const LC_VERSION_MIN_MACOSX: u32 = 0x24;
const LC_VERSION_MIN_IPHONEOS: u32 = 0x25;
const LC_FUNCTION_STARTS: u32 = 0x26;
const LC_MAIN: u32 = LC_REQ_DYLD | 0x28;
const LC_SOURCE_VERSION: u32 = 0x2A;
const LC_ENCRYPTION_INFO_64: u32 = 0x2C;
const LC_VERSION_MIN_TVOS: u32 = 0x2F;
const LC_VERSION_MIN_WATCHOS: u32 = 0x30;
const LC_BUILD_VERSION: u32 = 0x32;
const LC_DYLD_EXPORTS_TRIE: u32 = LC_REQ_DYLD | 0x33;
const LC_DYLD_CHAINED_FIXUPS: u32 = LC_REQ_DYLD | 0x34;

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

#[derive(Clone, Debug)]
struct MachOHeaderLayout {
    header_kind: MachOHeaderKind,
    load_commands_size: u32,
    root_layout_id: String,
    load_commands_layout_id: String,
    command_layout_descriptors: Vec<StructLayoutDescriptor>,
    command_fields: Vec<SymbolicFieldDefinition>,
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

#[derive(Clone, Debug)]
struct ParsedLoadCommand {
    command_kind: MachOLoadCommandKind,
    layout_descriptor: StructLayoutDescriptor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MachOLoadCommandKind {
    Segment32 { section_count: u32 },
    Segment64 { section_count: u32 },
    Symtab,
    Dysymtab,
    Uuid,
    Dylib,
    Rpath,
    EntryPoint,
    BuildVersion { tool_count: u32 },
    SourceVersion,
    VersionMin,
    LinkEditData,
    EncryptionInfo,
    EncryptionInfo64,
    DyldInfo,
    ThreadLike,
    Unknown,
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
    upsert_macho_struct_layout_descriptors(project_symbol_catalog, macho_header_layout)?;
    upsert_macho_module_fields(project_symbol_catalog, module_name, macho_header_layout, data_type_size_by_id)
}

fn upsert_macho_struct_layout_descriptors(
    project_symbol_catalog: &mut ProjectSymbolCatalog,
    macho_header_layout: &MachOHeaderLayout,
) -> Result<(), String> {
    let mut struct_layout_descriptors = project_symbol_catalog.get_struct_layout_descriptors().to_vec();

    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, mach_header32_descriptor());
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, mach_header64_descriptor());
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, section32_descriptor());
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, section64_descriptor());
    upsert_struct_layout_descriptor(
        &mut struct_layout_descriptors,
        generic_load_command_descriptor(MachOHeaderKind::Mach32(MachOByteOrder::LittleEndian)),
    );
    upsert_struct_layout_descriptor(
        &mut struct_layout_descriptors,
        generic_load_command_descriptor(MachOHeaderKind::Mach64(MachOByteOrder::LittleEndian)),
    );
    upsert_struct_layout_descriptor(
        &mut struct_layout_descriptors,
        segment_command_descriptor(MachOHeaderKind::Mach32(MachOByteOrder::LittleEndian), 0, 0)?,
    );
    upsert_struct_layout_descriptor(
        &mut struct_layout_descriptors,
        segment_command_descriptor(MachOHeaderKind::Mach64(MachOByteOrder::LittleEndian), 0, 0)?,
    );
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, symtab_command_descriptor(MachOByteOrder::LittleEndian));
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, dysymtab_command_descriptor(MachOByteOrder::LittleEndian));
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, uuid_command_descriptor(MachOByteOrder::LittleEndian));
    upsert_struct_layout_descriptor(
        &mut struct_layout_descriptors,
        build_version_command_descriptor(MachOByteOrder::LittleEndian, 0)?,
    );
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, build_tool_version_descriptor(MachOByteOrder::LittleEndian));
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, source_version_command_descriptor(MachOByteOrder::LittleEndian));
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, version_min_command_descriptor(MachOByteOrder::LittleEndian));
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, linkedit_data_command_descriptor(MachOByteOrder::LittleEndian));
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, entry_point_command_descriptor(MachOByteOrder::LittleEndian));
    upsert_struct_layout_descriptor(
        &mut struct_layout_descriptors,
        encryption_info_command_descriptor(MachOByteOrder::LittleEndian, false),
    );
    upsert_struct_layout_descriptor(
        &mut struct_layout_descriptors,
        encryption_info_command_descriptor(MachOByteOrder::LittleEndian, true),
    );
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, dyld_info_command_descriptor(MachOByteOrder::LittleEndian));

    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, macho_headers_descriptor(macho_header_layout)?);
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, load_commands_descriptor(macho_header_layout)?);

    for command_layout_descriptor in &macho_header_layout.command_layout_descriptors {
        upsert_struct_layout_descriptor(&mut struct_layout_descriptors, command_layout_descriptor.clone());
    }

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
        .checked_add(u64::from(macho_header_layout.load_commands_size))
        .ok_or_else(|| String::from("Mach-O headers size is too large."))?;

    Ok(vec![DesiredModuleField {
        display_name: String::from("Mach-O Headers"),
        offset: 0,
        struct_layout_id: macho_header_layout.root_layout_id.clone(),
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
    let initial_header_bytes = process_memory_store.read_module_bytes(module_name, 0, INITIAL_MACHO_HEADER_READ_SIZE)?;
    let (header_kind, load_command_count, load_commands_size) = read_macho_header_metadata(&initial_header_bytes)?;
    let required_size = header_kind
        .header_size_in_bytes()
        .checked_add(u64::from(load_commands_size))
        .ok_or_else(|| String::from("Mach-O header read size is too large."))?;
    let header_bytes = if required_size <= initial_header_bytes.len() as u64 {
        initial_header_bytes
    } else {
        let read_size = required_size.min(MAX_MACHO_HEADER_READ_SIZE);

        process_memory_store.read_module_bytes(module_name, 0, read_size)?
    };

    build_macho_header_layout(module_name, header_kind, load_command_count, load_commands_size, &header_bytes)
}

fn read_macho_header_metadata(header_bytes: &[u8]) -> Result<(MachOHeaderKind, u32, u32), String> {
    if header_bytes.len() < 4 {
        return Err(String::from("Module header read is too small for a Mach-O magic."));
    }

    let header_kind = match header_bytes.get(0..4) {
        Some([0xCE, 0xFA, 0xED, 0xFE]) => MachOHeaderKind::Mach32(MachOByteOrder::LittleEndian),
        Some([0xFE, 0xED, 0xFA, 0xCE]) => MachOHeaderKind::Mach32(MachOByteOrder::BigEndian),
        Some([0xCF, 0xFA, 0xED, 0xFE]) => MachOHeaderKind::Mach64(MachOByteOrder::LittleEndian),
        Some([0xFE, 0xED, 0xFA, 0xCF]) => MachOHeaderKind::Mach64(MachOByteOrder::BigEndian),
        Some([0xCA, 0xFE, 0xBA, 0xBE]) | Some([0xBE, 0xBA, 0xFE, 0xCA]) | Some([0xCA, 0xFE, 0xBA, 0xBF]) | Some([0xBF, 0xBA, 0xFE, 0xCA]) => {
            return Err(String::from("Universal Mach-O binaries are not yet supported for header population."));
        }
        _ => return Err(String::from("Selected module does not start with a supported Mach-O signature.")),
    };

    let header_size_in_bytes = header_kind.header_size_in_bytes() as usize;
    if header_bytes.len() < header_size_in_bytes {
        return Err(format!("Module header read is too small for a {} header.", header_kind.display_name()));
    }

    let byte_order = header_kind.byte_order();
    let load_command_count = read_u32(&header_bytes[16..20], byte_order)?;
    let load_commands_size = read_u32(&header_bytes[20..24], byte_order)?;

    Ok((header_kind, load_command_count, load_commands_size))
}

fn build_macho_header_layout(
    module_name: &str,
    header_kind: MachOHeaderKind,
    load_command_count: u32,
    load_commands_size: u32,
    header_bytes: &[u8],
) -> Result<MachOHeaderLayout, String> {
    let header_size_in_bytes = header_kind.header_size_in_bytes();
    let total_header_size = header_size_in_bytes
        .checked_add(u64::from(load_commands_size))
        .ok_or_else(|| String::from("Mach-O header span is too large."))?;

    if header_bytes.len() < total_header_size as usize {
        return Err(String::from("Mach-O load commands are not fully readable."));
    }

    let module_layout_id_prefix = format!("mac.macho.{}", sanitize_identifier(module_name));
    let root_layout_id = format!("{module_layout_id_prefix}.headers");
    let load_commands_layout_id = format!("{module_layout_id_prefix}.load_commands");
    let mut command_layout_descriptors = Vec::new();
    let mut command_fields = Vec::new();
    let mut next_command_offset = header_size_in_bytes;
    let byte_order = header_kind.byte_order();

    for command_index in 0..load_command_count {
        let command_start = usize::try_from(next_command_offset).map_err(|_| String::from("Mach-O command offset is too large."))?;
        let command_header_end = command_start
            .checked_add(LOAD_COMMAND_HEADER_SIZE_IN_BYTES as usize)
            .ok_or_else(|| String::from("Mach-O command header end offset is too large."))?;
        let command_header_bytes = header_bytes
            .get(command_start..command_header_end)
            .ok_or_else(|| format!("Mach-O load command {command_index} header is not readable."))?;
        let command_raw = read_u32(&command_header_bytes[0..4], byte_order)?;
        let command_size = u64::from(read_u32(&command_header_bytes[4..8], byte_order)?);
        if command_size < LOAD_COMMAND_HEADER_SIZE_IN_BYTES {
            return Err(format!("Mach-O load command {command_index} has an invalid cmdsize of {command_size}."));
        }
        let command_end_offset = next_command_offset
            .checked_add(command_size)
            .ok_or_else(|| String::from("Mach-O command end offset is too large."))?;
        let command_bytes = header_bytes
            .get(command_start..usize::try_from(command_end_offset).map_err(|_| String::from("Mach-O command span is too large."))?)
            .ok_or_else(|| format!("Mach-O load command {command_index} is not fully readable."))?;
        let parsed_load_command = parse_load_command(
            &module_layout_id_prefix,
            header_kind,
            command_index,
            next_command_offset,
            command_raw,
            command_size,
            command_bytes,
        )?;
        let field_name = parsed_load_command.command_kind.field_name(command_index);

        command_fields.push(field_at(
            &field_name,
            parsed_load_command.layout_descriptor.get_struct_layout_id(),
            next_command_offset - header_size_in_bytes,
        ));
        command_layout_descriptors.push(parsed_load_command.layout_descriptor);
        next_command_offset = command_end_offset;
    }

    if next_command_offset != total_header_size {
        return Err(format!(
            "Mach-O load command span mismatch: expected {} bytes of commands, parsed {} bytes.",
            load_commands_size,
            next_command_offset.saturating_sub(header_size_in_bytes)
        ));
    }

    Ok(MachOHeaderLayout {
        header_kind,
        load_commands_size,
        root_layout_id,
        load_commands_layout_id,
        command_layout_descriptors,
        command_fields,
    })
}

fn parse_load_command(
    module_layout_id_prefix: &str,
    header_kind: MachOHeaderKind,
    command_index: u32,
    _command_offset_in_bytes: u64,
    command_raw: u32,
    command_size_in_bytes: u64,
    command_bytes: &[u8],
) -> Result<ParsedLoadCommand, String> {
    let command_layout_id = format!("{module_layout_id_prefix}.load_command_{command_index:04}");
    let byte_order = header_kind.byte_order();
    let command_kind = match command_raw {
        LC_SEGMENT => {
            let section_count = read_u32(&command_bytes[48..52], byte_order)?;
            MachOLoadCommandKind::Segment32 { section_count }
        }
        LC_SEGMENT_64 => {
            let section_count = read_u32(&command_bytes[64..68], byte_order)?;
            MachOLoadCommandKind::Segment64 { section_count }
        }
        LC_SYMTAB => MachOLoadCommandKind::Symtab,
        LC_DYSYMTAB => MachOLoadCommandKind::Dysymtab,
        LC_UUID => MachOLoadCommandKind::Uuid,
        LC_LOAD_DYLIB | LC_ID_DYLIB | LC_LOAD_WEAK_DYLIB | LC_REEXPORT_DYLIB | LC_LAZY_LOAD_DYLIB | LC_LOAD_UPWARD_DYLIB => MachOLoadCommandKind::Dylib,
        LC_RPATH | LC_ID_DYLINKER => MachOLoadCommandKind::Rpath,
        LC_MAIN => MachOLoadCommandKind::EntryPoint,
        LC_BUILD_VERSION => {
            let tool_count = read_u32(&command_bytes[20..24], byte_order)?;
            MachOLoadCommandKind::BuildVersion { tool_count }
        }
        LC_SOURCE_VERSION => MachOLoadCommandKind::SourceVersion,
        LC_VERSION_MIN_MACOSX | LC_VERSION_MIN_IPHONEOS | LC_VERSION_MIN_TVOS | LC_VERSION_MIN_WATCHOS => MachOLoadCommandKind::VersionMin,
        LC_CODE_SIGNATURE | LC_FUNCTION_STARTS | LC_DYLD_EXPORTS_TRIE | LC_DYLD_CHAINED_FIXUPS => MachOLoadCommandKind::LinkEditData,
        LC_ENCRYPTION_INFO => MachOLoadCommandKind::EncryptionInfo,
        LC_ENCRYPTION_INFO_64 => MachOLoadCommandKind::EncryptionInfo64,
        LC_DYLD_INFO | LC_DYLD_INFO_ONLY => MachOLoadCommandKind::DyldInfo,
        LC_THREAD | LC_UNIXTHREAD => MachOLoadCommandKind::ThreadLike,
        _ => MachOLoadCommandKind::Unknown,
    };
    let layout_descriptor = command_kind.build_layout_descriptor(&command_layout_id, header_kind, command_size_in_bytes, command_bytes)?;

    Ok(ParsedLoadCommand {
        command_kind,
        layout_descriptor,
    })
}

impl MachOLoadCommandKind {
    fn field_name(
        &self,
        command_index: u32,
    ) -> String {
        match self {
            Self::Segment32 { .. } | Self::Segment64 { .. } => format!("SegmentCommand{command_index:02}"),
            Self::Symtab => format!("SymtabCommand{command_index:02}"),
            Self::Dysymtab => format!("DysymtabCommand{command_index:02}"),
            Self::Uuid => format!("UuidCommand{command_index:02}"),
            Self::Dylib => format!("DylibCommand{command_index:02}"),
            Self::Rpath => format!("PathCommand{command_index:02}"),
            Self::EntryPoint => format!("EntryPointCommand{command_index:02}"),
            Self::BuildVersion { .. } => format!("BuildVersionCommand{command_index:02}"),
            Self::SourceVersion => format!("SourceVersionCommand{command_index:02}"),
            Self::VersionMin => format!("VersionMinCommand{command_index:02}"),
            Self::LinkEditData => format!("LinkEditDataCommand{command_index:02}"),
            Self::EncryptionInfo => format!("EncryptionInfoCommand{command_index:02}"),
            Self::EncryptionInfo64 => format!("EncryptionInfo64Command{command_index:02}"),
            Self::DyldInfo => format!("DyldInfoCommand{command_index:02}"),
            Self::ThreadLike => format!("ThreadCommand{command_index:02}"),
            Self::Unknown => format!("LoadCommand{command_index:02}"),
        }
    }

    fn build_layout_descriptor(
        &self,
        layout_id: &str,
        header_kind: MachOHeaderKind,
        command_size_in_bytes: u64,
        command_bytes: &[u8],
    ) -> Result<StructLayoutDescriptor, String> {
        let byte_order = header_kind.byte_order();

        match self {
            Self::Segment32 { section_count } => segment_command_descriptor(header_kind, *section_count, command_size_in_bytes)
                .map(|descriptor| rename_struct_layout_descriptor(layout_id, descriptor, Some(command_size_in_bytes))),
            Self::Segment64 { section_count } => segment_command_descriptor(header_kind, *section_count, command_size_in_bytes)
                .map(|descriptor| rename_struct_layout_descriptor(layout_id, descriptor, Some(command_size_in_bytes))),
            Self::Symtab => Ok(rename_struct_layout_descriptor(
                layout_id,
                symtab_command_descriptor(byte_order),
                Some(command_size_in_bytes),
            )),
            Self::Dysymtab => Ok(rename_struct_layout_descriptor(
                layout_id,
                dysymtab_command_descriptor(byte_order),
                Some(command_size_in_bytes),
            )),
            Self::Uuid => Ok(rename_struct_layout_descriptor(
                layout_id,
                uuid_command_descriptor(byte_order),
                Some(command_size_in_bytes),
            )),
            Self::Dylib => dylib_command_descriptor(layout_id, byte_order, command_size_in_bytes, command_bytes),
            Self::Rpath => rpath_command_descriptor(layout_id, byte_order, command_size_in_bytes, command_bytes),
            Self::EntryPoint => Ok(rename_struct_layout_descriptor(
                layout_id,
                entry_point_command_descriptor(byte_order),
                Some(command_size_in_bytes),
            )),
            Self::BuildVersion { tool_count } => build_version_command_descriptor(byte_order, *tool_count)
                .map(|descriptor| rename_struct_layout_descriptor(layout_id, descriptor, Some(command_size_in_bytes))),
            Self::SourceVersion => Ok(rename_struct_layout_descriptor(
                layout_id,
                source_version_command_descriptor(byte_order),
                Some(command_size_in_bytes),
            )),
            Self::VersionMin => Ok(rename_struct_layout_descriptor(
                layout_id,
                version_min_command_descriptor(byte_order),
                Some(command_size_in_bytes),
            )),
            Self::LinkEditData => Ok(rename_struct_layout_descriptor(
                layout_id,
                linkedit_data_command_descriptor(byte_order),
                Some(command_size_in_bytes),
            )),
            Self::EncryptionInfo => Ok(rename_struct_layout_descriptor(
                layout_id,
                encryption_info_command_descriptor(byte_order, false),
                Some(command_size_in_bytes),
            )),
            Self::EncryptionInfo64 => Ok(rename_struct_layout_descriptor(
                layout_id,
                encryption_info_command_descriptor(byte_order, true),
                Some(command_size_in_bytes),
            )),
            Self::DyldInfo => Ok(rename_struct_layout_descriptor(
                layout_id,
                dyld_info_command_descriptor(byte_order),
                Some(command_size_in_bytes),
            )),
            Self::ThreadLike | Self::Unknown => generic_variable_command_descriptor(layout_id, header_kind, command_size_in_bytes),
        }
    }
}

fn rename_struct_layout_descriptor(
    layout_id: &str,
    struct_layout_descriptor: StructLayoutDescriptor,
    declared_size_in_bytes: Option<u64>,
) -> StructLayoutDescriptor {
    StructLayoutDescriptor::new(
        layout_id.to_string(),
        SymbolicStructDefinition::new_with_layout_kind(
            layout_id.to_string(),
            struct_layout_descriptor
                .get_struct_layout_definition()
                .get_layout_kind(),
            struct_layout_descriptor
                .get_struct_layout_definition()
                .get_fields()
                .to_vec(),
        )
        .with_declared_size_in_bytes(declared_size_in_bytes),
    )
}

fn macho_headers_descriptor(macho_header_layout: &MachOHeaderLayout) -> Result<StructLayoutDescriptor, String> {
    let mut fields = Vec::new();
    fields.push(field("Header", macho_header_layout.header_kind.mach_header_layout_id()));
    fields.push(field_at(
        "LoadCommands",
        &macho_header_layout.load_commands_layout_id,
        macho_header_layout.header_size_in_bytes(),
    ));

    Ok(StructLayoutDescriptor::new(
        macho_header_layout.root_layout_id.clone(),
        SymbolicStructDefinition::new(macho_header_layout.root_layout_id.clone(), fields).with_declared_size_in_bytes(Some(
            macho_header_layout
                .header_size_in_bytes()
                .checked_add(u64::from(macho_header_layout.load_commands_size))
                .ok_or_else(|| String::from("Mach-O headers size is too large."))?,
        )),
    ))
}

fn load_commands_descriptor(macho_header_layout: &MachOHeaderLayout) -> Result<StructLayoutDescriptor, String> {
    Ok(StructLayoutDescriptor::new(
        macho_header_layout.load_commands_layout_id.clone(),
        SymbolicStructDefinition::new(macho_header_layout.load_commands_layout_id.clone(), macho_header_layout.command_fields.clone())
            .with_declared_size_in_bytes(Some(u64::from(macho_header_layout.load_commands_size))),
    ))
}

fn mach_header32_descriptor() -> StructLayoutDescriptor {
    struct_layout_descriptor(
        MACH_HEADER32_ID,
        vec![
            field_at("magic", "u32", 0),
            field_at("cputype", "i32", 4),
            field_at("cpusubtype", "i32", 8),
            field_at("filetype", "u32", 12),
            field_at("ncmds", "u32", 16),
            field_at("sizeofcmds", "u32", 20),
            field_at("flags", "u32", 24),
        ],
        Some(MACHO_HEADER32_SIZE_IN_BYTES),
    )
}

fn mach_header64_descriptor() -> StructLayoutDescriptor {
    struct_layout_descriptor(
        MACH_HEADER64_ID,
        vec![
            field_at("magic", "u32", 0),
            field_at("cputype", "i32", 4),
            field_at("cpusubtype", "i32", 8),
            field_at("filetype", "u32", 12),
            field_at("ncmds", "u32", 16),
            field_at("sizeofcmds", "u32", 20),
            field_at("flags", "u32", 24),
            field_at("reserved", "u32", 28),
        ],
        Some(MACHO_HEADER64_SIZE_IN_BYTES),
    )
}

fn section32_descriptor() -> StructLayoutDescriptor {
    struct_layout_descriptor(
        SECTION32_ID,
        vec![
            array_field_at("sectname", "u8", 16, 0),
            array_field_at("segname", "u8", 16, 16),
            field_at("addr", "u32", 32),
            field_at("size", "u32", 36),
            field_at("offset", "u32", 40),
            field_at("align", "u32", 44),
            field_at("reloff", "u32", 48),
            field_at("nreloc", "u32", 52),
            field_at("flags", "u32", 56),
            field_at("reserved1", "u32", 60),
            field_at("reserved2", "u32", 64),
        ],
        Some(SECTION32_SIZE_IN_BYTES),
    )
}

fn section64_descriptor() -> StructLayoutDescriptor {
    struct_layout_descriptor(
        SECTION64_ID,
        vec![
            array_field_at("sectname", "u8", 16, 0),
            array_field_at("segname", "u8", 16, 16),
            field_at("addr", "u64", 32),
            field_at("size", "u64", 40),
            field_at("offset", "u32", 48),
            field_at("align", "u32", 52),
            field_at("reloff", "u32", 56),
            field_at("nreloc", "u32", 60),
            field_at("flags", "u32", 64),
            field_at("reserved1", "u32", 68),
            field_at("reserved2", "u32", 72),
            field_at("reserved3", "u32", 76),
        ],
        Some(SECTION64_SIZE_IN_BYTES),
    )
}

fn generic_load_command_descriptor(header_kind: MachOHeaderKind) -> StructLayoutDescriptor {
    let (layout_id, u32_type) = match header_kind.byte_order() {
        MachOByteOrder::LittleEndian => (
            match header_kind {
                MachOHeaderKind::Mach32(_) => LOAD_COMMAND32_ID,
                MachOHeaderKind::Mach64(_) => LOAD_COMMAND64_ID,
            },
            "u32",
        ),
        MachOByteOrder::BigEndian => (
            match header_kind {
                MachOHeaderKind::Mach32(_) => LOAD_COMMAND32_ID,
                MachOHeaderKind::Mach64(_) => LOAD_COMMAND64_ID,
            },
            "u32be",
        ),
    };

    struct_layout_descriptor(
        layout_id,
        vec![field_at("cmd", u32_type, 0), field_at("cmdsize", u32_type, 4)],
        Some(LOAD_COMMAND_HEADER_SIZE_IN_BYTES),
    )
}

fn segment_command_descriptor(
    header_kind: MachOHeaderKind,
    section_count: u32,
    command_size_in_bytes: u64,
) -> Result<StructLayoutDescriptor, String> {
    let is_64 = header_kind.is_64();
    let byte_order = header_kind.byte_order();
    let u32_type = u32_type_id(byte_order);
    let u64_type = u64_type_id(byte_order);
    let i32_type = i32_type_id(byte_order);
    let layout_id = match header_kind {
        MachOHeaderKind::Mach32(_) => {
            if section_count == 0 {
                SEGMENT_COMMAND_ID.to_string()
            } else {
                format!("{SEGMENT_COMMAND_ID}[{section_count}]")
            }
        }
        MachOHeaderKind::Mach64(_) => {
            if section_count == 0 {
                SEGMENT_COMMAND64_ID.to_string()
            } else {
                format!("{SEGMENT_COMMAND64_ID}[{section_count}]")
            }
        }
    };
    let header_size_in_bytes = if is_64 {
        SEGMENT_COMMAND64_SIZE_IN_BYTES
    } else {
        SEGMENT_COMMAND32_SIZE_IN_BYTES
    };
    let section_layout_id = if is_64 { SECTION64_ID } else { SECTION32_ID };
    let mut fields = vec![
        field_at("cmd", u32_type, 0),
        field_at("cmdsize", u32_type, 4),
        array_field_at("segname", "u8", 16, 8),
    ];

    if is_64 {
        fields.extend([
            field_at("vmaddr", u64_type, 24),
            field_at("vmsize", u64_type, 32),
            field_at("fileoff", u64_type, 40),
            field_at("filesize", u64_type, 48),
            field_at("maxprot", i32_type, 56),
            field_at("initprot", i32_type, 60),
            field_at("nsects", u32_type, 64),
            field_at("flags", u32_type, 68),
        ]);
    } else {
        fields.extend([
            field_at("vmaddr", u32_type, 24),
            field_at("vmsize", u32_type, 28),
            field_at("fileoff", u32_type, 32),
            field_at("filesize", u32_type, 36),
            field_at("maxprot", i32_type, 40),
            field_at("initprot", i32_type, 44),
            field_at("nsects", u32_type, 48),
            field_at("flags", u32_type, 52),
        ]);
    }

    if section_count > 0 {
        fields.push(array_field_at("Sections", section_layout_id, u64::from(section_count), header_size_in_bytes));
    }

    let section_block_size = if is_64 {
        u64::from(section_count).saturating_mul(SECTION64_SIZE_IN_BYTES)
    } else {
        u64::from(section_count).saturating_mul(SECTION32_SIZE_IN_BYTES)
    };
    let parsed_size_in_bytes = header_size_in_bytes
        .checked_add(section_block_size)
        .ok_or_else(|| String::from("Mach-O segment command size is too large."))?;

    if command_size_in_bytes > parsed_size_in_bytes {
        fields.push(array_field_at(
            "TrailingBytes",
            "u8",
            command_size_in_bytes - parsed_size_in_bytes,
            parsed_size_in_bytes,
        ));
    }

    Ok(struct_layout_descriptor(
        &layout_id,
        fields,
        Some(command_size_in_bytes.max(parsed_size_in_bytes)),
    ))
}

fn symtab_command_descriptor(byte_order: MachOByteOrder) -> StructLayoutDescriptor {
    let u32_type = u32_type_id(byte_order);

    struct_layout_descriptor(
        SYMTAB_COMMAND_ID,
        vec![
            field_at("cmd", u32_type, 0),
            field_at("cmdsize", u32_type, 4),
            field_at("symoff", u32_type, 8),
            field_at("nsyms", u32_type, 12),
            field_at("stroff", u32_type, 16),
            field_at("strsize", u32_type, 20),
        ],
        Some(SYMTAB_COMMAND_SIZE_IN_BYTES),
    )
}

fn dysymtab_command_descriptor(byte_order: MachOByteOrder) -> StructLayoutDescriptor {
    let u32_type = u32_type_id(byte_order);
    let field_names = [
        "cmd",
        "cmdsize",
        "ilocalsym",
        "nlocalsym",
        "iextdefsym",
        "nextdefsym",
        "iundefsym",
        "nundefsym",
        "tocoff",
        "ntoc",
        "modtaboff",
        "nmodtab",
        "extrefsymoff",
        "nextrefsyms",
        "indirectsymoff",
        "nindirectsyms",
        "extreloff",
        "nextrel",
        "locreloff",
        "nlocrel",
    ];

    struct_layout_descriptor(
        DYSYMTAB_COMMAND_ID,
        field_names
            .iter()
            .enumerate()
            .map(|(field_index, field_name)| field_at(field_name, u32_type, u64::try_from(field_index * 4).unwrap_or_default()))
            .collect(),
        Some(DYSYMTAB_COMMAND_SIZE_IN_BYTES),
    )
}

fn uuid_command_descriptor(byte_order: MachOByteOrder) -> StructLayoutDescriptor {
    let u32_type = u32_type_id(byte_order);

    struct_layout_descriptor(
        UUID_COMMAND_ID,
        vec![
            field_at("cmd", u32_type, 0),
            field_at("cmdsize", u32_type, 4),
            array_field_at("uuid", "u8", 16, 8),
        ],
        Some(UUID_COMMAND_SIZE_IN_BYTES),
    )
}

fn dylib_command_descriptor(
    layout_id: &str,
    byte_order: MachOByteOrder,
    command_size_in_bytes: u64,
    command_bytes: &[u8],
) -> Result<StructLayoutDescriptor, String> {
    let u32_type = u32_type_id(byte_order);
    let name_offset = u64::from(read_u32(&command_bytes[8..12], byte_order)?);
    let mut fields = vec![
        field_at("cmd", u32_type, 0),
        field_at("cmdsize", u32_type, 4),
        field_at("name_offset", u32_type, 8),
        field_at("timestamp", u32_type, 12),
        field_at("current_version", u32_type, 16),
        field_at("compatibility_version", u32_type, 20),
    ];

    if command_size_in_bytes > name_offset {
        fields.push(array_field_at("PathBytes", "u8", command_size_in_bytes - name_offset, name_offset));
    }

    Ok(struct_layout_descriptor(layout_id, fields, Some(command_size_in_bytes)))
}

fn rpath_command_descriptor(
    layout_id: &str,
    byte_order: MachOByteOrder,
    command_size_in_bytes: u64,
    command_bytes: &[u8],
) -> Result<StructLayoutDescriptor, String> {
    let u32_type = u32_type_id(byte_order);
    let path_offset = u64::from(read_u32(&command_bytes[8..12], byte_order)?);
    let mut fields = vec![
        field_at("cmd", u32_type, 0),
        field_at("cmdsize", u32_type, 4),
        field_at("path_offset", u32_type, 8),
    ];

    if command_size_in_bytes > path_offset {
        fields.push(array_field_at("PathBytes", "u8", command_size_in_bytes - path_offset, path_offset));
    }

    Ok(struct_layout_descriptor(layout_id, fields, Some(command_size_in_bytes)))
}

fn entry_point_command_descriptor(byte_order: MachOByteOrder) -> StructLayoutDescriptor {
    let u32_type = u32_type_id(byte_order);
    let u64_type = u64_type_id(byte_order);

    struct_layout_descriptor(
        ENTRY_POINT_COMMAND_ID,
        vec![
            field_at("cmd", u32_type, 0),
            field_at("cmdsize", u32_type, 4),
            field_at("entryoff", u64_type, 8),
            field_at("stacksize", u64_type, 16),
        ],
        Some(ENTRY_POINT_COMMAND_SIZE_IN_BYTES),
    )
}

fn build_version_command_descriptor(
    byte_order: MachOByteOrder,
    tool_count: u32,
) -> Result<StructLayoutDescriptor, String> {
    let u32_type = u32_type_id(byte_order);
    let tool_layout_id = if tool_count == 0 {
        BUILD_TOOL_VERSION_ID.to_string()
    } else {
        format!("{BUILD_TOOL_VERSION_ID}[{tool_count}]")
    };
    let mut fields = vec![
        field_at("cmd", u32_type, 0),
        field_at("cmdsize", u32_type, 4),
        field_at("platform", u32_type, 8),
        field_at("minos", u32_type, 12),
        field_at("sdk", u32_type, 16),
        field_at("ntools", u32_type, 20),
    ];

    if tool_count > 0 {
        fields.push(array_field_at(
            "Tools",
            &tool_layout_id,
            u64::from(tool_count),
            BUILD_VERSION_COMMAND_SIZE_IN_BYTES,
        ));
    }

    let declared_size_in_bytes = BUILD_VERSION_COMMAND_SIZE_IN_BYTES
        .checked_add(u64::from(tool_count).saturating_mul(BUILD_TOOL_VERSION_SIZE_IN_BYTES))
        .ok_or_else(|| String::from("Mach-O build version command size is too large."))?;

    let layout_id = if tool_count == 0 {
        BUILD_VERSION_COMMAND_ID.to_string()
    } else {
        format!("{BUILD_VERSION_COMMAND_ID}[{tool_count}]")
    };

    Ok(struct_layout_descriptor(&layout_id, fields, Some(declared_size_in_bytes)))
}

fn build_tool_version_descriptor(byte_order: MachOByteOrder) -> StructLayoutDescriptor {
    let u32_type = u32_type_id(byte_order);

    struct_layout_descriptor(
        BUILD_TOOL_VERSION_ID,
        vec![field_at("tool", u32_type, 0), field_at("version", u32_type, 4)],
        Some(BUILD_TOOL_VERSION_SIZE_IN_BYTES),
    )
}

fn source_version_command_descriptor(byte_order: MachOByteOrder) -> StructLayoutDescriptor {
    let u32_type = u32_type_id(byte_order);
    let u64_type = u64_type_id(byte_order);

    struct_layout_descriptor(
        SOURCE_VERSION_COMMAND_ID,
        vec![
            field_at("cmd", u32_type, 0),
            field_at("cmdsize", u32_type, 4),
            field_at("version", u64_type, 8),
        ],
        Some(SOURCE_VERSION_COMMAND_SIZE_IN_BYTES),
    )
}

fn version_min_command_descriptor(byte_order: MachOByteOrder) -> StructLayoutDescriptor {
    let u32_type = u32_type_id(byte_order);

    struct_layout_descriptor(
        VERSION_MIN_COMMAND_ID,
        vec![
            field_at("cmd", u32_type, 0),
            field_at("cmdsize", u32_type, 4),
            field_at("version", u32_type, 8),
            field_at("sdk", u32_type, 12),
        ],
        Some(VERSION_MIN_COMMAND_SIZE_IN_BYTES),
    )
}

fn linkedit_data_command_descriptor(byte_order: MachOByteOrder) -> StructLayoutDescriptor {
    let u32_type = u32_type_id(byte_order);

    struct_layout_descriptor(
        LINKEDIT_DATA_COMMAND_ID,
        vec![
            field_at("cmd", u32_type, 0),
            field_at("cmdsize", u32_type, 4),
            field_at("dataoff", u32_type, 8),
            field_at("datasize", u32_type, 12),
        ],
        Some(LINKEDIT_DATA_COMMAND_SIZE_IN_BYTES),
    )
}

fn encryption_info_command_descriptor(
    byte_order: MachOByteOrder,
    is_64: bool,
) -> StructLayoutDescriptor {
    let u32_type = u32_type_id(byte_order);
    let layout_id = if is_64 { ENCRYPTION_INFO64_COMMAND_ID } else { ENCRYPTION_INFO_COMMAND_ID };
    let mut fields = vec![
        field_at("cmd", u32_type, 0),
        field_at("cmdsize", u32_type, 4),
        field_at("cryptoff", u32_type, 8),
        field_at("cryptsize", u32_type, 12),
        field_at("cryptid", u32_type, 16),
    ];

    if is_64 {
        fields.push(field_at("pad", u32_type, 20));
    }

    struct_layout_descriptor(
        layout_id,
        fields,
        Some(if is_64 {
            ENCRYPTION_INFO64_COMMAND_SIZE_IN_BYTES
        } else {
            ENCRYPTION_INFO_COMMAND_SIZE_IN_BYTES
        }),
    )
}

fn dyld_info_command_descriptor(byte_order: MachOByteOrder) -> StructLayoutDescriptor {
    let u32_type = u32_type_id(byte_order);
    let field_names = [
        "cmd",
        "cmdsize",
        "rebase_off",
        "rebase_size",
        "bind_off",
        "bind_size",
        "weak_bind_off",
        "weak_bind_size",
        "lazy_bind_off",
        "lazy_bind_size",
        "export_off",
        "export_size",
    ];

    struct_layout_descriptor(
        DYLD_INFO_COMMAND_ID,
        field_names
            .iter()
            .enumerate()
            .map(|(field_index, field_name)| field_at(field_name, u32_type, u64::try_from(field_index * 4).unwrap_or_default()))
            .collect(),
        Some(DYLD_INFO_COMMAND_SIZE_IN_BYTES),
    )
}

fn generic_variable_command_descriptor(
    layout_id: &str,
    header_kind: MachOHeaderKind,
    command_size_in_bytes: u64,
) -> Result<StructLayoutDescriptor, String> {
    let u32_type = u32_type_id(header_kind.byte_order());
    let mut fields = vec![field_at("cmd", u32_type, 0), field_at("cmdsize", u32_type, 4)];

    if command_size_in_bytes > LOAD_COMMAND_HEADER_SIZE_IN_BYTES {
        fields.push(array_field_at(
            "PayloadBytes",
            "u8",
            command_size_in_bytes - LOAD_COMMAND_HEADER_SIZE_IN_BYTES,
            LOAD_COMMAND_HEADER_SIZE_IN_BYTES,
        ));
    }

    Ok(struct_layout_descriptor(layout_id, fields, Some(command_size_in_bytes)))
}

fn struct_layout_descriptor(
    struct_layout_id: &str,
    fields: Vec<SymbolicFieldDefinition>,
    declared_size_in_bytes: Option<u64>,
) -> StructLayoutDescriptor {
    StructLayoutDescriptor::new(
        struct_layout_id.to_string(),
        SymbolicStructDefinition::new(struct_layout_id.to_string(), fields).with_declared_size_in_bytes(declared_size_in_bytes),
    )
}

fn field(
    field_name: &str,
    data_type_id: &str,
) -> SymbolicFieldDefinition {
    SymbolicFieldDefinition::new_named(field_name.to_string(), DataTypeRef::new(data_type_id), ContainerType::None)
}

fn field_at(
    field_name: &str,
    data_type_id: &str,
    offset_in_bytes: u64,
) -> SymbolicFieldDefinition {
    SymbolicFieldDefinition::new_named(field_name.to_string(), DataTypeRef::new(data_type_id), ContainerType::None)
        .with_offset_resolution(SymbolicFieldOffsetResolution::new_static(offset_in_bytes))
}

fn array_field_at(
    field_name: &str,
    data_type_id: &str,
    length: u64,
    offset_in_bytes: u64,
) -> SymbolicFieldDefinition {
    SymbolicFieldDefinition::new_named(field_name.to_string(), DataTypeRef::new(data_type_id), ContainerType::ArrayFixed(length))
        .with_offset_resolution(SymbolicFieldOffsetResolution::new_static(offset_in_bytes))
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
        SECTION32_ID => Some(SECTION32_SIZE_IN_BYTES),
        SECTION64_ID => Some(SECTION64_SIZE_IN_BYTES),
        SYMTAB_COMMAND_ID => Some(SYMTAB_COMMAND_SIZE_IN_BYTES),
        DYSYMTAB_COMMAND_ID => Some(DYSYMTAB_COMMAND_SIZE_IN_BYTES),
        UUID_COMMAND_ID => Some(UUID_COMMAND_SIZE_IN_BYTES),
        ENTRY_POINT_COMMAND_ID => Some(ENTRY_POINT_COMMAND_SIZE_IN_BYTES),
        SOURCE_VERSION_COMMAND_ID => Some(SOURCE_VERSION_COMMAND_SIZE_IN_BYTES),
        VERSION_MIN_COMMAND_ID => Some(VERSION_MIN_COMMAND_SIZE_IN_BYTES),
        LINKEDIT_DATA_COMMAND_ID => Some(LINKEDIT_DATA_COMMAND_SIZE_IN_BYTES),
        ENCRYPTION_INFO_COMMAND_ID => Some(ENCRYPTION_INFO_COMMAND_SIZE_IN_BYTES),
        ENCRYPTION_INFO64_COMMAND_ID => Some(ENCRYPTION_INFO64_COMMAND_SIZE_IN_BYTES),
        DYLD_INFO_COMMAND_ID => Some(DYLD_INFO_COMMAND_SIZE_IN_BYTES),
        BUILD_TOOL_VERSION_ID => Some(BUILD_TOOL_VERSION_SIZE_IN_BYTES),
        _ => parse_sized_layout_id_suffix(struct_layout_id),
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

fn parse_sized_layout_id_suffix(struct_layout_id: &str) -> Option<u64> {
    let size_text = struct_layout_id
        .rsplit_once(".size_")
        .map(|(_, size_text)| size_text)?;

    size_text.parse::<u64>().ok()
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

fn sanitize_identifier(identifier: &str) -> String {
    let sanitized_identifier = identifier
        .chars()
        .map(|identifier_character| {
            if identifier_character.is_ascii_alphanumeric() {
                identifier_character.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>();

    sanitized_identifier.trim_matches('_').to_string()
}

fn u32_type_id(byte_order: MachOByteOrder) -> &'static str {
    match byte_order {
        MachOByteOrder::LittleEndian => "u32",
        MachOByteOrder::BigEndian => "u32be",
    }
}

fn u64_type_id(byte_order: MachOByteOrder) -> &'static str {
    match byte_order {
        MachOByteOrder::LittleEndian => "u64",
        MachOByteOrder::BigEndian => "u64be",
    }
}

fn i32_type_id(byte_order: MachOByteOrder) -> &'static str {
    match byte_order {
        MachOByteOrder::LittleEndian => "i32",
        MachOByteOrder::BigEndian => "i32be",
    }
}

fn read_u32(
    bytes: &[u8],
    byte_order: MachOByteOrder,
) -> Result<u32, String> {
    let bytes = <[u8; 4]>::try_from(bytes).map_err(|_| String::from("Expected 4 readable bytes."))?;

    Ok(match byte_order {
        MachOByteOrder::LittleEndian => u32::from_le_bytes(bytes),
        MachOByteOrder::BigEndian => u32::from_be_bytes(bytes),
    })
}

impl MachOHeaderLayout {
    fn header_size_in_bytes(&self) -> u64 {
        self.header_kind.header_size_in_bytes()
    }
}

impl MachOHeaderKind {
    fn header_size_in_bytes(&self) -> u64 {
        match self {
            Self::Mach32(_) => MACHO_HEADER32_SIZE_IN_BYTES,
            Self::Mach64(_) => MACHO_HEADER64_SIZE_IN_BYTES,
        }
    }

    fn mach_header_layout_id(&self) -> &'static str {
        match self {
            Self::Mach32(_) => MACH_HEADER32_ID,
            Self::Mach64(_) => MACH_HEADER64_ID,
        }
    }

    fn byte_order(&self) -> MachOByteOrder {
        match self {
            Self::Mach32(byte_order) | Self::Mach64(byte_order) => *byte_order,
        }
    }

    fn is_64(&self) -> bool {
        matches!(self, Self::Mach64(_))
    }

    fn display_name(&self) -> &'static str {
        match self {
            Self::Mach32(_) => "Mach-O 32-bit",
            Self::Mach64(_) => "Mach-O 64-bit",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        analyze_macho_header_layout, build_version_command_descriptor, load_commands_descriptor, populate_macho_symbols, sanitize_identifier, MachOByteOrder,
        MachOHeaderKind, PopulateMachOSymbolsAction, MACH_HEADER64_ID,
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
            structs::symbolic_struct_definition::SymbolicStructDefinition,
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
    fn analyze_macho_header_layout_parses_typed_commands_and_sections() {
        let process_memory_store = TestProcessMemoryStore::new();
        let macho_header_layout = analyze_macho_header_layout(&process_memory_store, "Finder").expect("Expected Mach-O layout.");

        assert_eq!(macho_header_layout.header_kind, MachOHeaderKind::Mach64(MachOByteOrder::LittleEndian));
        assert_eq!(macho_header_layout.command_layout_descriptors.len(), 2);
        assert!(macho_header_layout
            .command_layout_descriptors
            .iter()
            .any(|struct_layout_descriptor| {
                struct_layout_descriptor
                    .get_struct_layout_definition()
                    .get_fields()
                    .iter()
                    .any(|field_definition| field_definition.get_field_name() == "Sections")
            }));
    }

    #[test]
    fn populate_macho_symbols_adds_macho_headers_root_and_typed_load_commands() {
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
        let macho_headers_layout_descriptor = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == "mac.macho.finder.headers")
            .expect("Expected Mach-O headers descriptor.");
        let load_commands_layout_descriptor = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == "mac.macho.finder.load_commands")
            .expect("Expected load commands descriptor.");

        assert_eq!(symbol_module.get_fields().len(), 1);
        assert_eq!(symbol_module.get_fields()[0].get_display_name(), "Mach-O Headers");
        assert_eq!(
            macho_headers_layout_descriptor
                .get_struct_layout_definition()
                .get_fields()[0]
                .get_data_type_ref()
                .get_data_type_id(),
            MACH_HEADER64_ID
        );
        assert_eq!(
            load_commands_layout_descriptor
                .get_struct_layout_definition()
                .get_fields()
                .len(),
            2
        );
        assert_eq!(
            load_commands_layout_descriptor
                .get_struct_layout_definition()
                .get_fields()[0]
                .get_field_name(),
            "SegmentCommand00"
        );
    }

    #[test]
    fn populate_macho_symbols_replaces_existing_root_u8_array() {
        let mut symbol_module = ProjectSymbolModule::new(String::from("Finder"), 0x2000);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("u8_00000000"), 0, String::from("u8[512]")));
        let mut project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![symbol_module], Vec::new(), Vec::new());
        let macho_header_layout = analyze_macho_header_layout(&TestProcessMemoryStore::new(), "Finder").expect("Expected Mach-O layout.");

        populate_macho_symbols(&mut project_symbol_catalog, "Finder", &macho_header_layout, &default_data_type_size_by_id())
            .expect("Expected Mach-O symbol population to replace u8[] root field.");

        let fields = project_symbol_catalog
            .find_symbol_module("Finder")
            .expect("Expected module to exist.")
            .get_fields();

        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].get_display_name(), "Mach-O Headers");
        assert_eq!(fields[0].get_struct_layout_id(), "mac.macho.finder.headers");
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
        let macho_header_layout = analyze_macho_header_layout(&TestProcessMemoryStore::new(), "Finder").expect("Expected Mach-O layout.");

        populate_macho_symbols(&mut project_symbol_catalog, "Finder", &macho_header_layout, &default_data_type_size_by_id())
            .expect("Expected Mach-O symbol population to update module root layout.");

        let module_root_layout_definition = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == "Finder")
            .expect("Expected Finder root layout.")
            .get_struct_layout_definition();

        assert!(module_root_layout_definition
            .get_fields()
            .iter()
            .any(|field_definition| field_definition.get_field_name() == "Mach-O Headers"));
    }

    #[test]
    fn sanitize_identifier_normalizes_module_names() {
        assert_eq!(sanitize_identifier("Finder.app"), "finder_app");
    }

    #[test]
    fn build_version_descriptor_materializes_tool_array() {
        let build_version_layout_descriptor = build_version_command_descriptor(MachOByteOrder::LittleEndian, 2).expect("Expected build version layout.");

        assert!(build_version_layout_descriptor
            .get_struct_layout_definition()
            .get_fields()
            .iter()
            .any(|field_definition| field_definition.get_field_name() == "Tools"));
    }

    #[test]
    fn load_commands_descriptor_uses_static_command_offsets() {
        let macho_header_layout = analyze_macho_header_layout(&TestProcessMemoryStore::new(), "Finder").expect("Expected Mach-O layout.");
        let load_commands_layout_descriptor = load_commands_descriptor(&macho_header_layout).expect("Expected load commands layout.");

        assert!(load_commands_layout_descriptor
            .get_struct_layout_definition()
            .get_fields()
            .iter()
            .all(|field_definition| matches!(
                field_definition.get_offset_resolution(),
                squalr_engine_api::structures::structs::symbolic_field_definition::SymbolicFieldOffsetResolution::Static(_)
            )));
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
        header_bytes[20..24].copy_from_slice(&0x60_u32.to_le_bytes());

        let first_command_offset = super::MACHO_HEADER64_SIZE_IN_BYTES as usize;
        header_bytes[first_command_offset..first_command_offset + 4].copy_from_slice(&0x19_u32.to_le_bytes());
        header_bytes[first_command_offset + 4..first_command_offset + 8].copy_from_slice(&0x48_u32.to_le_bytes());
        header_bytes[first_command_offset + 64..first_command_offset + 68].copy_from_slice(&0x1_u32.to_le_bytes());
        header_bytes[first_command_offset + 72..first_command_offset + 88].copy_from_slice(b"__text\0\0\0\0\0\0\0\0\0\0");
        header_bytes[first_command_offset + 88..first_command_offset + 104].copy_from_slice(b"__TEXT\0\0\0\0\0\0\0\0\0\0");

        let second_command_offset = first_command_offset + 0x48;
        header_bytes[second_command_offset..second_command_offset + 4].copy_from_slice(&0x2_u32.to_le_bytes());
        header_bytes[second_command_offset + 4..second_command_offset + 8].copy_from_slice(&0x18_u32.to_le_bytes());
        header_bytes[second_command_offset + 8..second_command_offset + 12].copy_from_slice(&0x1000_u32.to_le_bytes());
        header_bytes[second_command_offset + 12..second_command_offset + 16].copy_from_slice(&0x10_u32.to_le_bytes());
        header_bytes[second_command_offset + 16..second_command_offset + 20].copy_from_slice(&0x2000_u32.to_le_bytes());
        header_bytes[second_command_offset + 20..second_command_offset + 24].copy_from_slice(&0x80_u32.to_le_bytes());

        header_bytes
    }
}
