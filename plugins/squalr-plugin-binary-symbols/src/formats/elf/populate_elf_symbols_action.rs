use squalr_engine_api::{
    plugins::{
        PluginPermission,
        symbol_tree::symbol_tree_action::{
            DataTypeRegistryStore, ProcessMemoryStore, SymbolTreeAction, SymbolTreeActionContext, SymbolTreeActionSelection, SymbolTreeActionServices,
        },
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

const ELF_HEADER32_ID: &str = "linux.elf.Elf32_Ehdr";
const ELF_HEADER64_ID: &str = "linux.elf.Elf64_Ehdr";
const ELF_PROGRAM_HEADER32_ID: &str = "linux.elf.Elf32_Phdr";
const ELF_PROGRAM_HEADER64_ID: &str = "linux.elf.Elf64_Phdr";
const ELF_SECTION_HEADER32_ID: &str = "linux.elf.Elf32_Shdr";
const ELF_SECTION_HEADER64_ID: &str = "linux.elf.Elf64_Shdr";
const ELF_DYNAMIC32_ID: &str = "linux.elf.Elf32_Dyn";
const ELF_DYNAMIC64_ID: &str = "linux.elf.Elf64_Dyn";
const ELF_SYMBOL32_ID: &str = "linux.elf.Elf32_Sym";
const ELF_SYMBOL64_ID: &str = "linux.elf.Elf64_Sym";
const STRING_UTF8_NULL_TERMINATED_DATA_TYPE_ID: &str = "string_utf8{null_terminated}";
const ELF_IDENT_SIZE_IN_BYTES: u64 = 16;
const ELF_HEADER32_SIZE_IN_BYTES: u64 = 52;
const ELF_HEADER64_SIZE_IN_BYTES: u64 = 64;
const ELF_PROGRAM_HEADER32_SIZE_IN_BYTES: u64 = 32;
const ELF_PROGRAM_HEADER64_SIZE_IN_BYTES: u64 = 56;
const ELF_SECTION_HEADER32_SIZE_IN_BYTES: u64 = 40;
const ELF_SECTION_HEADER64_SIZE_IN_BYTES: u64 = 64;
const ELF_DYNAMIC32_SIZE_IN_BYTES: u64 = 8;
const ELF_DYNAMIC64_SIZE_IN_BYTES: u64 = 16;
const ELF_SYMBOL32_SIZE_IN_BYTES: u64 = 16;
const ELF_SYMBOL64_SIZE_IN_BYTES: u64 = 24;
const INITIAL_ELF_HEADER_READ_SIZE: u64 = 0x1000;
const MAX_ELF_HEADER_READ_SIZE: u64 = 0x200000;
const MAX_ELF_DYNAMIC_READ_SIZE: u64 = 0x200000;
const MAX_ELF_STRING_TABLE_READ_SIZE: u64 = 0x200000;
const MAX_ELF_DYNAMIC_ENTRY_COUNT: u64 = 4096;
const MAX_ELF_DYNAMIC_SYMBOL_COUNT: u64 = 16384;
const ELF_CLASS32: u8 = 1;
const ELF_CLASS64: u8 = 2;
const ELF_DATA_LITTLE_ENDIAN: u8 = 1;
const ELF_DATA_BIG_ENDIAN: u8 = 2;
const PT_LOAD: u32 = 1;
const PT_DYNAMIC: u32 = 2;
const PT_INTERP: u32 = 3;
const DT_NULL: i64 = 0;
const DT_NEEDED: i64 = 1;
const DT_HASH: i64 = 4;
const DT_STRTAB: i64 = 5;
const DT_SYMTAB: i64 = 6;
const DT_STRSZ: i64 = 10;
const DT_SONAME: i64 = 14;
const DT_RPATH: i64 = 15;
const DT_SYMENT: i64 = 11;
const DT_RUNPATH: i64 = 29;
const DT_GNU_HASH: i64 = 0x6FFFFEF5;
const DT_AUXILIARY: i64 = 0x7FFFFFFD;
const DT_FILTER: i64 = 0x7FFFFFFF;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ElfByteOrder {
    LittleEndian,
    BigEndian,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ElfHeaderKind {
    Elf32(ElfByteOrder),
    Elf64(ElfByteOrder),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ElfHeaderLayout {
    header_kind: ElfHeaderKind,
    program_header_offset: u64,
    program_header_entry_size: u64,
    program_header_count: u64,
    section_header_offset: u64,
    section_header_entry_size: u64,
    section_header_count: u64,
    section_header_string_table_index: u64,
    root_layout_id: String,
    program_headers_layout_id: String,
    section_headers_layout_id: String,
    dynamic_entries_layout_id: String,
    dynamic_symbols_layout_id: String,
    include_section_headers: bool,
    program_headers: Vec<ElfProgramHeader>,
    section_headers: Vec<ElfSectionHeader>,
    section_header_names: BTreeMap<u64, String>,
    interpreter: Option<ResolvedElfString>,
    dynamic_table: Option<ElfDynamicTable>,
    dynamic_strings: Vec<ResolvedElfString>,
    dynamic_symbols: Vec<ElfDynamicSymbol>,
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct ElfProgramHeader {
    table_index: u64,
    header_offset: u64,
    program_type: u32,
    file_offset: u64,
    virtual_address: u64,
    file_size: u64,
    memory_size: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ElfSectionHeader {
    table_index: u64,
    header_offset: u64,
    name_offset: u32,
    section_type: u32,
    virtual_address: u64,
    file_offset: u64,
    size_in_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ElfDynamicTable {
    offset: u64,
    size_in_bytes: u64,
    entries: Vec<ElfDynamicEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ElfDynamicEntry {
    table_index: u64,
    entry_offset: u64,
    tag: i64,
    value: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ResolvedElfString {
    display_name: String,
    offset: u64,
    length_in_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ElfDynamicSymbol {
    table_index: u64,
    symbol_offset: u64,
    name: Option<String>,
}

pub struct PopulateElfSymbolsAction;

impl SymbolTreeAction for PopulateElfSymbolsAction {
    fn action_id(&self) -> &'static str {
        "builtin.symbols.binary.populate-elf-symbols"
    }

    fn label(
        &self,
        _context: &SymbolTreeActionContext,
    ) -> String {
        String::from("Populate ELF Symbols")
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
            return Err(String::from("ELF symbol population requires a module root selection."));
        };
        let module_name = module_name.clone();
        let module_name_for_update = module_name.clone();
        let elf_header_layout = analyze_elf_header_layout(services.process_memory(), &module_name)?;
        let data_type_size_by_id = collect_data_type_size_by_id(services.data_type_registry());

        services.symbol_store().write_catalog(
            "populate ELF symbols",
            Box::new(move |project_symbol_catalog| {
                populate_elf_symbols(project_symbol_catalog, &module_name_for_update, &elf_header_layout, &data_type_size_by_id)
            }),
        )?;
        services.symbol_tree_window().request_refresh();
        services
            .symbol_tree_window()
            .focus_tree_node(&format!("module:{module_name}"));

        Ok(())
    }
}

fn populate_elf_symbols(
    project_symbol_catalog: &mut ProjectSymbolCatalog,
    module_name: &str,
    elf_header_layout: &ElfHeaderLayout,
    data_type_size_by_id: &BTreeMap<String, u64>,
) -> Result<(), String> {
    upsert_elf_struct_layout_descriptors(project_symbol_catalog, elf_header_layout)?;
    upsert_elf_module_fields(project_symbol_catalog, module_name, elf_header_layout, data_type_size_by_id)
}

fn upsert_elf_struct_layout_descriptors(
    project_symbol_catalog: &mut ProjectSymbolCatalog,
    elf_header_layout: &ElfHeaderLayout,
) -> Result<(), String> {
    let mut struct_layout_descriptors = project_symbol_catalog.get_struct_layout_descriptors().to_vec();
    let byte_order = elf_header_layout.header_kind.byte_order();

    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, elf_header32_descriptor(byte_order));
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, elf_header64_descriptor(byte_order));
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, program_header32_descriptor(byte_order));
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, program_header64_descriptor(byte_order));
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, section_header32_descriptor(byte_order));
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, section_header64_descriptor(byte_order));
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, dynamic32_descriptor(byte_order));
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, dynamic64_descriptor(byte_order));
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, symbol32_descriptor(byte_order));
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, symbol64_descriptor(byte_order));
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, elf_headers_descriptor(elf_header_layout)?);
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, program_headers_descriptor(elf_header_layout)?);

    if elf_header_layout.include_section_headers {
        upsert_struct_layout_descriptor(&mut struct_layout_descriptors, section_headers_descriptor(elf_header_layout)?);
    }
    if elf_header_layout.dynamic_table.is_some() {
        upsert_struct_layout_descriptor(&mut struct_layout_descriptors, dynamic_entries_descriptor(elf_header_layout)?);
    }
    if !elf_header_layout.dynamic_symbols.is_empty() {
        upsert_struct_layout_descriptor(&mut struct_layout_descriptors, dynamic_symbols_descriptor(elf_header_layout)?);
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

fn upsert_elf_module_fields(
    project_symbol_catalog: &mut ProjectSymbolCatalog,
    module_name: &str,
    elf_header_layout: &ElfHeaderLayout,
    data_type_size_by_id: &BTreeMap<String, u64>,
) -> Result<(), String> {
    let desired_module_fields = build_desired_elf_module_fields(elf_header_layout)?;
    let minimum_size = desired_module_fields
        .iter()
        .filter_map(|desired_module_field| {
            desired_module_field
                .offset
                .checked_add(desired_module_field.size_in_bytes)
        })
        .max()
        .unwrap_or(elf_header_layout.header_size_in_bytes());

    project_symbol_catalog.ensure_symbol_module(module_name, minimum_size);
    let Some(symbol_module) = project_symbol_catalog.find_symbol_module_mut(module_name) else {
        return Err(format!("Could not resolve module `{module_name}` after creating it."));
    };

    upsert_module_fields_in_module(symbol_module, &desired_module_fields)?;
    let module_size = symbol_module.get_size();
    project_symbol_catalog.ensure_module_root_struct_layout(module_name, module_size, |data_type_ref| {
        data_type_size_by_id
            .get(data_type_ref.get_data_type_id())
            .copied()
    });
    upsert_module_root_layout_fields(project_symbol_catalog, module_name, &desired_module_fields, module_size, data_type_size_by_id)
}

fn build_desired_elf_module_fields(elf_header_layout: &ElfHeaderLayout) -> Result<Vec<DesiredModuleField>, String> {
    Ok(vec![DesiredModuleField {
        display_name: String::from("ELF Headers"),
        offset: 0,
        struct_layout_id: elf_header_layout.root_layout_id.clone(),
        size_in_bytes: elf_header_layout.headers_size_in_bytes()?,
    }])
}

fn upsert_module_fields_in_module(
    symbol_module: &mut ProjectSymbolModule,
    desired_module_fields: &[DesiredModuleField],
) -> Result<(), String> {
    let module_fields = symbol_module.get_fields_mut();
    let desired_field_ranges = desired_module_fields
        .iter()
        .map(|desired_module_field| {
            desired_module_field
                .offset
                .checked_add(desired_module_field.size_in_bytes)
                .map(|desired_field_end| (desired_module_field.offset, desired_field_end))
                .ok_or_else(|| format!("ELF field `{}` range is too large.", desired_module_field.display_name))
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
                .ok_or_else(|| format!("ELF field `{}` range is too large.", desired_module_field.display_name))
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

fn analyze_elf_header_layout(
    process_memory_store: &dyn ProcessMemoryStore,
    module_name: &str,
) -> Result<ElfHeaderLayout, String> {
    let initial_header_bytes = process_memory_store.read_module_bytes(module_name, 0, INITIAL_ELF_HEADER_READ_SIZE)?;
    let mut elf_header_layout = read_elf_header_layout(module_name, &initial_header_bytes)?;
    let required_program_header_size = elf_header_layout.program_headers_end_offset()?;
    let header_bytes = if required_program_header_size > initial_header_bytes.len() as u64 {
        let read_size = required_program_header_size.min(MAX_ELF_HEADER_READ_SIZE);
        let header_bytes = process_memory_store.read_module_bytes(module_name, 0, read_size)?;

        if header_bytes.len() < required_program_header_size as usize {
            return Err(String::from("ELF program headers are not fully readable."));
        }

        header_bytes
    } else {
        initial_header_bytes.clone()
    };

    elf_header_layout.program_headers = read_program_headers(&elf_header_layout, &header_bytes)?;

    let section_headers_end_offset = elf_header_layout.section_headers_end_offset()?;
    if section_headers_end_offset > 0 && section_headers_end_offset <= MAX_ELF_HEADER_READ_SIZE {
        let section_header_bytes = if section_headers_end_offset <= header_bytes.len() as u64 {
            header_bytes.clone()
        } else {
            process_memory_store.read_module_bytes(module_name, 0, section_headers_end_offset)?
        };

        elf_header_layout.include_section_headers = section_header_bytes.len() >= section_headers_end_offset as usize;

        if elf_header_layout.include_section_headers {
            elf_header_layout.section_headers = read_section_headers(&elf_header_layout, &section_header_bytes)?;
            elf_header_layout.section_header_names = resolve_section_header_names(process_memory_store, module_name, &elf_header_layout);
        }
    }

    elf_header_layout.interpreter = resolve_interpreter(process_memory_store, module_name, &elf_header_layout);
    elf_header_layout.dynamic_table = resolve_dynamic_table(process_memory_store, module_name, &elf_header_layout);
    elf_header_layout.dynamic_strings = resolve_dynamic_strings(process_memory_store, module_name, &elf_header_layout);
    elf_header_layout.dynamic_symbols = resolve_dynamic_symbols(process_memory_store, module_name, &elf_header_layout);

    Ok(elf_header_layout)
}

fn read_elf_header_layout(
    module_name: &str,
    header_bytes: &[u8],
) -> Result<ElfHeaderLayout, String> {
    if header_bytes.len() < ELF_IDENT_SIZE_IN_BYTES as usize {
        return Err(String::from("Module header read is too small for an ELF ident."));
    }

    if !header_bytes.starts_with(b"\x7FELF") {
        return Err(String::from("Selected module does not start with an ELF signature."));
    }

    let byte_order = match header_bytes[5] {
        ELF_DATA_LITTLE_ENDIAN => ElfByteOrder::LittleEndian,
        ELF_DATA_BIG_ENDIAN => ElfByteOrder::BigEndian,
        data_encoding => return Err(format!("Unsupported ELF data encoding `{data_encoding}`.")),
    };
    let header_kind = match header_bytes[4] {
        ELF_CLASS32 => ElfHeaderKind::Elf32(byte_order),
        ELF_CLASS64 => ElfHeaderKind::Elf64(byte_order),
        elf_class => return Err(format!("Unsupported ELF class `{elf_class}`.")),
    };

    if header_bytes.len() < header_kind.header_size_in_bytes() as usize {
        return Err(format!("Module header read is too small for a {} header.", header_kind.display_name()));
    }

    let module_layout_id_prefix = format!("linux.elf.{}", sanitize_identifier(module_name));
    let mut elf_header_layout = match header_kind {
        ElfHeaderKind::Elf32(byte_order) => ElfHeaderLayout {
            header_kind,
            program_header_offset: u64::from(read_u32_at(header_bytes, 28, byte_order)?),
            section_header_offset: u64::from(read_u32_at(header_bytes, 32, byte_order)?),
            program_header_entry_size: u64::from(read_u16_at(header_bytes, 42, byte_order)?),
            program_header_count: u64::from(read_u16_at(header_bytes, 44, byte_order)?),
            section_header_entry_size: u64::from(read_u16_at(header_bytes, 46, byte_order)?),
            section_header_count: u64::from(read_u16_at(header_bytes, 48, byte_order)?),
            section_header_string_table_index: u64::from(read_u16_at(header_bytes, 50, byte_order)?),
            root_layout_id: format!("{module_layout_id_prefix}.headers"),
            program_headers_layout_id: format!("{module_layout_id_prefix}.program_headers"),
            section_headers_layout_id: format!("{module_layout_id_prefix}.section_headers"),
            dynamic_entries_layout_id: format!("{module_layout_id_prefix}.dynamic_entries"),
            dynamic_symbols_layout_id: format!("{module_layout_id_prefix}.dynamic_symbols"),
            include_section_headers: false,
            program_headers: Vec::new(),
            section_headers: Vec::new(),
            section_header_names: BTreeMap::new(),
            interpreter: None,
            dynamic_table: None,
            dynamic_strings: Vec::new(),
            dynamic_symbols: Vec::new(),
        },
        ElfHeaderKind::Elf64(byte_order) => ElfHeaderLayout {
            header_kind,
            program_header_offset: read_u64_at(header_bytes, 32, byte_order)?,
            section_header_offset: read_u64_at(header_bytes, 40, byte_order)?,
            program_header_entry_size: u64::from(read_u16_at(header_bytes, 54, byte_order)?),
            program_header_count: u64::from(read_u16_at(header_bytes, 56, byte_order)?),
            section_header_entry_size: u64::from(read_u16_at(header_bytes, 58, byte_order)?),
            section_header_count: u64::from(read_u16_at(header_bytes, 60, byte_order)?),
            section_header_string_table_index: u64::from(read_u16_at(header_bytes, 62, byte_order)?),
            root_layout_id: format!("{module_layout_id_prefix}.headers"),
            program_headers_layout_id: format!("{module_layout_id_prefix}.program_headers"),
            section_headers_layout_id: format!("{module_layout_id_prefix}.section_headers"),
            dynamic_entries_layout_id: format!("{module_layout_id_prefix}.dynamic_entries"),
            dynamic_symbols_layout_id: format!("{module_layout_id_prefix}.dynamic_symbols"),
            include_section_headers: false,
            program_headers: Vec::new(),
            section_headers: Vec::new(),
            section_header_names: BTreeMap::new(),
            interpreter: None,
            dynamic_table: None,
            dynamic_strings: Vec::new(),
            dynamic_symbols: Vec::new(),
        },
    };

    if elf_header_layout.program_header_entry_size == 0 {
        elf_header_layout.program_header_count = 0;
    }
    if elf_header_layout.section_header_entry_size == 0 {
        elf_header_layout.section_header_count = 0;
    }

    Ok(elf_header_layout)
}

fn read_program_headers(
    elf_header_layout: &ElfHeaderLayout,
    header_bytes: &[u8],
) -> Result<Vec<ElfProgramHeader>, String> {
    if elf_header_layout.program_header_count == 0 {
        return Ok(Vec::new());
    }
    if elf_header_layout.program_header_entry_size < elf_header_layout.header_kind.program_header_size_in_bytes() {
        return Err(format!(
            "ELF program header entry size {} is smaller than {}.",
            elf_header_layout.program_header_entry_size,
            elf_header_layout.header_kind.program_header_size_in_bytes()
        ));
    }

    (0..elf_header_layout.program_header_count)
        .map(|table_index| {
            let header_offset = elf_header_layout
                .program_header_offset
                .checked_add(
                    table_index
                        .checked_mul(elf_header_layout.program_header_entry_size)
                        .ok_or_else(|| String::from("ELF program header offset is too large."))?,
                )
                .ok_or_else(|| String::from("ELF program header offset is too large."))?;
            let header_start = usize::try_from(header_offset).map_err(|_| String::from("ELF program header offset does not fit in memory."))?;

            read_program_header_at(elf_header_layout, header_bytes, table_index, header_offset, header_start)
        })
        .collect()
}

fn read_program_header_at(
    elf_header_layout: &ElfHeaderLayout,
    header_bytes: &[u8],
    table_index: u64,
    header_offset: u64,
    header_start: usize,
) -> Result<ElfProgramHeader, String> {
    let byte_order = elf_header_layout.header_kind.byte_order();

    match elf_header_layout.header_kind {
        ElfHeaderKind::Elf32(_) => Ok(ElfProgramHeader {
            table_index,
            header_offset,
            program_type: read_u32_at(header_bytes, header_start, byte_order)?,
            file_offset: u64::from(read_u32_at(header_bytes, header_start + 4, byte_order)?),
            virtual_address: u64::from(read_u32_at(header_bytes, header_start + 8, byte_order)?),
            file_size: u64::from(read_u32_at(header_bytes, header_start + 16, byte_order)?),
            memory_size: u64::from(read_u32_at(header_bytes, header_start + 20, byte_order)?),
        }),
        ElfHeaderKind::Elf64(_) => Ok(ElfProgramHeader {
            table_index,
            header_offset,
            program_type: read_u32_at(header_bytes, header_start, byte_order)?,
            file_offset: read_u64_at(header_bytes, header_start + 8, byte_order)?,
            virtual_address: read_u64_at(header_bytes, header_start + 16, byte_order)?,
            file_size: read_u64_at(header_bytes, header_start + 32, byte_order)?,
            memory_size: read_u64_at(header_bytes, header_start + 40, byte_order)?,
        }),
    }
}

fn read_section_headers(
    elf_header_layout: &ElfHeaderLayout,
    header_bytes: &[u8],
) -> Result<Vec<ElfSectionHeader>, String> {
    if elf_header_layout.section_header_count == 0 {
        return Ok(Vec::new());
    }
    if elf_header_layout.section_header_entry_size < elf_header_layout.header_kind.section_header_size_in_bytes() {
        return Err(format!(
            "ELF section header entry size {} is smaller than {}.",
            elf_header_layout.section_header_entry_size,
            elf_header_layout.header_kind.section_header_size_in_bytes()
        ));
    }

    (0..elf_header_layout.section_header_count)
        .map(|table_index| {
            let header_offset = elf_header_layout
                .section_header_offset
                .checked_add(
                    table_index
                        .checked_mul(elf_header_layout.section_header_entry_size)
                        .ok_or_else(|| String::from("ELF section header offset is too large."))?,
                )
                .ok_or_else(|| String::from("ELF section header offset is too large."))?;
            let header_start = usize::try_from(header_offset).map_err(|_| String::from("ELF section header offset does not fit in memory."))?;

            read_section_header_at(elf_header_layout, header_bytes, table_index, header_offset, header_start)
        })
        .collect()
}

fn read_section_header_at(
    elf_header_layout: &ElfHeaderLayout,
    header_bytes: &[u8],
    table_index: u64,
    header_offset: u64,
    header_start: usize,
) -> Result<ElfSectionHeader, String> {
    let byte_order = elf_header_layout.header_kind.byte_order();

    match elf_header_layout.header_kind {
        ElfHeaderKind::Elf32(_) => Ok(ElfSectionHeader {
            table_index,
            header_offset,
            name_offset: read_u32_at(header_bytes, header_start, byte_order)?,
            section_type: read_u32_at(header_bytes, header_start + 4, byte_order)?,
            virtual_address: u64::from(read_u32_at(header_bytes, header_start + 12, byte_order)?),
            file_offset: u64::from(read_u32_at(header_bytes, header_start + 16, byte_order)?),
            size_in_bytes: u64::from(read_u32_at(header_bytes, header_start + 20, byte_order)?),
        }),
        ElfHeaderKind::Elf64(_) => Ok(ElfSectionHeader {
            table_index,
            header_offset,
            name_offset: read_u32_at(header_bytes, header_start, byte_order)?,
            section_type: read_u32_at(header_bytes, header_start + 4, byte_order)?,
            virtual_address: read_u64_at(header_bytes, header_start + 16, byte_order)?,
            file_offset: read_u64_at(header_bytes, header_start + 24, byte_order)?,
            size_in_bytes: read_u64_at(header_bytes, header_start + 32, byte_order)?,
        }),
    }
}

fn resolve_section_header_names(
    process_memory_store: &dyn ProcessMemoryStore,
    module_name: &str,
    elf_header_layout: &ElfHeaderLayout,
) -> BTreeMap<u64, String> {
    let Some(string_table_section) = elf_header_layout
        .section_headers
        .iter()
        .find(|section_header| section_header.table_index == elf_header_layout.section_header_string_table_index)
    else {
        return BTreeMap::new();
    };
    let Some(string_table_bytes) = read_section_bytes(process_memory_store, module_name, elf_header_layout, string_table_section) else {
        return BTreeMap::new();
    };

    elf_header_layout
        .section_headers
        .iter()
        .filter_map(|section_header| {
            read_c_string_at(&string_table_bytes, u64::from(section_header.name_offset)).map(|section_name| (section_header.table_index, section_name))
        })
        .collect()
}

fn read_section_bytes(
    process_memory_store: &dyn ProcessMemoryStore,
    module_name: &str,
    elf_header_layout: &ElfHeaderLayout,
    section_header: &ElfSectionHeader,
) -> Option<Vec<u8>> {
    if section_header.size_in_bytes == 0 || section_header.size_in_bytes > MAX_ELF_STRING_TABLE_READ_SIZE {
        return None;
    }

    if section_header.virtual_address != 0 {
        if let Some(section_offset) = virtual_address_to_module_offset(elf_header_layout, section_header.virtual_address) {
            if let Ok(section_bytes) = process_memory_store.read_module_bytes(module_name, section_offset, section_header.size_in_bytes) {
                if section_bytes.len() as u64 == section_header.size_in_bytes {
                    return Some(section_bytes);
                }
            }
        }
    }

    process_memory_store
        .read_module_bytes(module_name, section_header.file_offset, section_header.size_in_bytes)
        .ok()
        .filter(|section_bytes| section_bytes.len() as u64 == section_header.size_in_bytes)
}

fn resolve_interpreter(
    process_memory_store: &dyn ProcessMemoryStore,
    module_name: &str,
    elf_header_layout: &ElfHeaderLayout,
) -> Option<ResolvedElfString> {
    let interpreter_header = elf_header_layout
        .program_headers
        .iter()
        .find(|program_header| program_header.program_type == PT_INTERP)?;
    let interpreter_offset = program_header_module_offset(elf_header_layout, interpreter_header)?;
    let interpreter_length = interpreter_header
        .file_size
        .min(interpreter_header.memory_size)
        .max(interpreter_header.file_size);

    if interpreter_length == 0 || interpreter_length > MAX_ELF_STRING_TABLE_READ_SIZE {
        return None;
    }

    let interpreter_bytes = process_memory_store
        .read_module_bytes(module_name, interpreter_offset, interpreter_length)
        .ok()?;
    let string_length = c_string_length_in_bytes(&interpreter_bytes, 0)?;

    Some(ResolvedElfString {
        display_name: String::from("ELF Interpreter"),
        offset: interpreter_offset,
        length_in_bytes: string_length,
    })
}

fn resolve_dynamic_table(
    process_memory_store: &dyn ProcessMemoryStore,
    module_name: &str,
    elf_header_layout: &ElfHeaderLayout,
) -> Option<ElfDynamicTable> {
    let dynamic_header = elf_header_layout
        .program_headers
        .iter()
        .find(|program_header| program_header.program_type == PT_DYNAMIC)?;
    let dynamic_offset = program_header_module_offset(elf_header_layout, dynamic_header)?;
    let dynamic_size = dynamic_header
        .file_size
        .min(dynamic_header.memory_size)
        .max(dynamic_header.file_size);

    if dynamic_size == 0 || dynamic_size > MAX_ELF_DYNAMIC_READ_SIZE {
        return None;
    }

    let dynamic_bytes = process_memory_store
        .read_module_bytes(module_name, dynamic_offset, dynamic_size)
        .ok()?;
    let entry_size = elf_header_layout.header_kind.dynamic_size_in_bytes();
    if dynamic_bytes.len() < entry_size as usize {
        return None;
    }
    let entry_count = (dynamic_bytes.len() as u64 / entry_size).min(MAX_ELF_DYNAMIC_ENTRY_COUNT);
    let mut dynamic_entries = Vec::new();

    for table_index in 0..entry_count {
        let entry_start = usize::try_from(table_index.saturating_mul(entry_size)).ok()?;
        let (tag, value) = read_dynamic_entry_at(elf_header_layout, &dynamic_bytes, entry_start).ok()?;
        let entry_offset = dynamic_offset.checked_add(table_index.saturating_mul(entry_size))?;

        dynamic_entries.push(ElfDynamicEntry {
            table_index,
            entry_offset,
            tag,
            value,
        });

        if tag == DT_NULL {
            break;
        }
    }
    if dynamic_entries.is_empty() {
        return None;
    }

    let parsed_size_in_bytes = u64::try_from(dynamic_entries.len())
        .ok()?
        .saturating_mul(entry_size);

    Some(ElfDynamicTable {
        offset: dynamic_offset,
        size_in_bytes: parsed_size_in_bytes,
        entries: dynamic_entries,
    })
}

fn read_dynamic_entry_at(
    elf_header_layout: &ElfHeaderLayout,
    dynamic_bytes: &[u8],
    entry_start: usize,
) -> Result<(i64, u64), String> {
    let byte_order = elf_header_layout.header_kind.byte_order();

    match elf_header_layout.header_kind {
        ElfHeaderKind::Elf32(_) => Ok((
            i64::from(read_i32_at(dynamic_bytes, entry_start, byte_order)?),
            u64::from(read_u32_at(dynamic_bytes, entry_start + 4, byte_order)?),
        )),
        ElfHeaderKind::Elf64(_) => Ok((
            read_i64_at(dynamic_bytes, entry_start, byte_order)?,
            read_u64_at(dynamic_bytes, entry_start + 8, byte_order)?,
        )),
    }
}

fn resolve_dynamic_strings(
    process_memory_store: &dyn ProcessMemoryStore,
    module_name: &str,
    elf_header_layout: &ElfHeaderLayout,
) -> Vec<ResolvedElfString> {
    let Some(dynamic_table) = &elf_header_layout.dynamic_table else {
        return Vec::new();
    };
    let Some((dynamic_string_table_offset, dynamic_string_table_bytes)) =
        read_dynamic_string_table(process_memory_store, module_name, elf_header_layout, dynamic_table)
    else {
        return Vec::new();
    };

    dynamic_table
        .entries
        .iter()
        .filter(|dynamic_entry| matches!(dynamic_entry.tag, DT_NEEDED | DT_SONAME | DT_RPATH | DT_RUNPATH | DT_AUXILIARY | DT_FILTER))
        .filter_map(|dynamic_entry| {
            let string_length = c_string_length_in_bytes(&dynamic_string_table_bytes, dynamic_entry.value)?;
            let string_value = read_c_string_at(&dynamic_string_table_bytes, dynamic_entry.value)?;

            Some(ResolvedElfString {
                display_name: dynamic_string_field_name(dynamic_entry.tag, &string_value, dynamic_entry.table_index),
                offset: dynamic_string_table_offset.saturating_add(dynamic_entry.value),
                length_in_bytes: string_length,
            })
        })
        .collect()
}

fn resolve_dynamic_symbols(
    process_memory_store: &dyn ProcessMemoryStore,
    module_name: &str,
    elf_header_layout: &ElfHeaderLayout,
) -> Vec<ElfDynamicSymbol> {
    let Some(dynamic_table) = &elf_header_layout.dynamic_table else {
        return Vec::new();
    };
    let Some((_dynamic_string_table_offset, dynamic_string_table_bytes)) =
        read_dynamic_string_table(process_memory_store, module_name, elf_header_layout, dynamic_table)
    else {
        return Vec::new();
    };
    let Some(symbol_table_address) = dynamic_value(dynamic_table, DT_SYMTAB) else {
        return Vec::new();
    };
    let symbol_entry_size = dynamic_value(dynamic_table, DT_SYMENT).unwrap_or_else(|| elf_header_layout.header_kind.symbol_size_in_bytes());
    if symbol_entry_size < elf_header_layout.header_kind.symbol_size_in_bytes() {
        return Vec::new();
    }
    let Some(symbol_table_offset) = virtual_address_to_module_offset(elf_header_layout, symbol_table_address) else {
        return Vec::new();
    };
    let symbol_count = resolve_dynamic_symbol_count(process_memory_store, module_name, elf_header_layout, dynamic_table)
        .unwrap_or(0)
        .min(MAX_ELF_DYNAMIC_SYMBOL_COUNT);
    if symbol_count == 0 {
        return Vec::new();
    }
    let symbol_table_size = symbol_count.saturating_mul(symbol_entry_size);
    let Ok(symbol_table_bytes) = process_memory_store.read_module_bytes(module_name, symbol_table_offset, symbol_table_size) else {
        return Vec::new();
    };

    (0..symbol_count)
        .filter_map(|table_index| {
            let symbol_start = usize::try_from(table_index.saturating_mul(symbol_entry_size)).ok()?;
            let symbol_name_offset = read_symbol_name_offset_at(elf_header_layout, &symbol_table_bytes, symbol_start).ok()?;
            let symbol_offset = symbol_table_offset.checked_add(table_index.saturating_mul(symbol_entry_size))?;
            let name = if symbol_name_offset == 0 {
                None
            } else {
                read_c_string_at(&dynamic_string_table_bytes, u64::from(symbol_name_offset))
            };

            Some(ElfDynamicSymbol {
                table_index,
                symbol_offset,
                name,
            })
        })
        .collect()
}

fn read_dynamic_string_table(
    process_memory_store: &dyn ProcessMemoryStore,
    module_name: &str,
    elf_header_layout: &ElfHeaderLayout,
    dynamic_table: &ElfDynamicTable,
) -> Option<(u64, Vec<u8>)> {
    let dynamic_string_table_address = dynamic_value(dynamic_table, DT_STRTAB)?;
    let dynamic_string_table_size = dynamic_value(dynamic_table, DT_STRSZ)?;
    let dynamic_string_table_offset = virtual_address_to_module_offset(elf_header_layout, dynamic_string_table_address)?;

    if dynamic_string_table_size == 0 || dynamic_string_table_size > MAX_ELF_STRING_TABLE_READ_SIZE {
        return None;
    }

    let dynamic_string_table_bytes = process_memory_store
        .read_module_bytes(module_name, dynamic_string_table_offset, dynamic_string_table_size)
        .ok()?;

    Some((dynamic_string_table_offset, dynamic_string_table_bytes))
}

fn resolve_dynamic_symbol_count(
    process_memory_store: &dyn ProcessMemoryStore,
    module_name: &str,
    elf_header_layout: &ElfHeaderLayout,
    dynamic_table: &ElfDynamicTable,
) -> Option<u64> {
    if let Some(hash_address) = dynamic_value(dynamic_table, DT_HASH) {
        return read_sysv_hash_symbol_count(process_memory_store, module_name, elf_header_layout, hash_address);
    }

    dynamic_value(dynamic_table, DT_GNU_HASH)
        .and_then(|hash_address| read_gnu_hash_symbol_count(process_memory_store, module_name, elf_header_layout, hash_address))
}

fn read_sysv_hash_symbol_count(
    process_memory_store: &dyn ProcessMemoryStore,
    module_name: &str,
    elf_header_layout: &ElfHeaderLayout,
    hash_address: u64,
) -> Option<u64> {
    let hash_offset = virtual_address_to_module_offset(elf_header_layout, hash_address)?;
    let hash_bytes = process_memory_store
        .read_module_bytes(module_name, hash_offset, 8)
        .ok()?;

    Some(u64::from(read_u32_at(&hash_bytes, 4, elf_header_layout.header_kind.byte_order()).ok()?))
}

fn read_gnu_hash_symbol_count(
    process_memory_store: &dyn ProcessMemoryStore,
    module_name: &str,
    elf_header_layout: &ElfHeaderLayout,
    hash_address: u64,
) -> Option<u64> {
    let hash_offset = virtual_address_to_module_offset(elf_header_layout, hash_address)?;
    let header_bytes = process_memory_store
        .read_module_bytes(module_name, hash_offset, 16)
        .ok()?;
    let byte_order = elf_header_layout.header_kind.byte_order();
    let bucket_count = u64::from(read_u32_at(&header_bytes, 0, byte_order).ok()?);
    let symbol_offset = u64::from(read_u32_at(&header_bytes, 4, byte_order).ok()?);
    let bloom_word_count = u64::from(read_u32_at(&header_bytes, 8, byte_order).ok()?);
    let bloom_word_size = if elf_header_layout.header_kind.is_64() { 8 } else { 4 };
    let buckets_offset = 16_u64.checked_add(bloom_word_count.checked_mul(bloom_word_size)?)?;
    let bucket_bytes = process_memory_store
        .read_module_bytes(module_name, hash_offset.checked_add(buckets_offset)?, bucket_count.checked_mul(4)?)
        .ok()?;
    let maximum_bucket_symbol = (0..bucket_count)
        .filter_map(|bucket_number| {
            let bucket_start = usize::try_from(bucket_number.saturating_mul(4)).ok()?;
            read_u32_at(&bucket_bytes, bucket_start, byte_order).ok()
        })
        .max()
        .map(u64::from)?;

    if maximum_bucket_symbol < symbol_offset {
        return Some(symbol_offset);
    }

    let chain_start = maximum_bucket_symbol.saturating_sub(symbol_offset);
    let chain_offset = hash_offset
        .checked_add(buckets_offset)?
        .checked_add(bucket_count.checked_mul(4)?)?
        .checked_add(chain_start.checked_mul(4)?)?;
    let chain_bytes = process_memory_store
        .read_module_bytes(module_name, chain_offset, 4_u64.saturating_mul(MAX_ELF_DYNAMIC_SYMBOL_COUNT))
        .ok()?;
    let chain_length = (0..(chain_bytes.len() as u64 / 4))
        .find(|chain_entry_number| {
            let chain_start = usize::try_from(chain_entry_number.saturating_mul(4)).unwrap_or_default();
            read_u32_at(&chain_bytes, chain_start, byte_order)
                .map(|chain_value| chain_value & 1 != 0)
                .unwrap_or(false)
        })
        .unwrap_or(0);

    Some(
        maximum_bucket_symbol
            .saturating_add(chain_length)
            .saturating_add(1),
    )
}

fn read_symbol_name_offset_at(
    elf_header_layout: &ElfHeaderLayout,
    symbol_table_bytes: &[u8],
    symbol_start: usize,
) -> Result<u32, String> {
    read_u32_at(symbol_table_bytes, symbol_start, elf_header_layout.header_kind.byte_order())
}

fn dynamic_value(
    dynamic_table: &ElfDynamicTable,
    tag: i64,
) -> Option<u64> {
    dynamic_table
        .entries
        .iter()
        .find(|dynamic_entry| dynamic_entry.tag == tag)
        .map(|dynamic_entry| dynamic_entry.value)
}

fn program_header_module_offset(
    elf_header_layout: &ElfHeaderLayout,
    program_header: &ElfProgramHeader,
) -> Option<u64> {
    virtual_address_to_module_offset(elf_header_layout, program_header.virtual_address).or(Some(program_header.file_offset))
}

fn virtual_address_to_module_offset(
    elf_header_layout: &ElfHeaderLayout,
    virtual_address: u64,
) -> Option<u64> {
    let virtual_base = elf_header_layout
        .program_headers
        .iter()
        .filter(|program_header| program_header.program_type == PT_LOAD)
        .map(|program_header| {
            program_header
                .virtual_address
                .saturating_sub(program_header.file_offset)
        })
        .min()
        .unwrap_or(0);
    let normalized_address = virtual_address
        .checked_sub(virtual_base)
        .unwrap_or(virtual_address);

    if elf_header_layout.program_headers.iter().any(|program_header| {
        let load_start = program_header.virtual_address.saturating_sub(virtual_base);
        let load_end = load_start.saturating_add(program_header.memory_size.max(program_header.file_size));

        normalized_address >= load_start && normalized_address < load_end
    }) {
        return Some(normalized_address);
    }

    Some(normalized_address)
}

fn program_header_field_name(program_header: &ElfProgramHeader) -> String {
    format!("{}_{:02}", program_header_type_name(program_header.program_type), program_header.table_index)
}

fn section_header_field_name(
    section_header: &ElfSectionHeader,
    section_header_names: &BTreeMap<u64, String>,
) -> String {
    let section_name = section_header_names
        .get(&section_header.table_index)
        .map(|name| sanitize_field_name(name))
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| section_header_type_name(section_header.section_type).to_string());

    format!("Section_{}_{:02}", section_name, section_header.table_index)
}

fn dynamic_entry_field_name(dynamic_entry: &ElfDynamicEntry) -> String {
    format!("{}_{:02}", dynamic_tag_name(dynamic_entry.tag), dynamic_entry.table_index)
}

fn dynamic_string_field_name(
    tag: i64,
    string_value: &str,
    table_index: u64,
) -> String {
    let sanitized_value = sanitize_field_name(string_value);
    let tag_name = dynamic_tag_name(tag);

    if sanitized_value.is_empty() {
        format!("{tag_name}_{table_index:02}")
    } else {
        format!("{tag_name}_{sanitized_value}_{table_index:02}")
    }
}

fn dynamic_symbol_field_name(dynamic_symbol: &ElfDynamicSymbol) -> String {
    dynamic_symbol
        .name
        .as_ref()
        .map(|symbol_name| sanitize_field_name(symbol_name))
        .filter(|symbol_name| !symbol_name.is_empty())
        .map(|symbol_name| format!("Symbol_{}_{:04}", symbol_name, dynamic_symbol.table_index))
        .unwrap_or_else(|| format!("Symbol_{:04}", dynamic_symbol.table_index))
}

fn program_header_type_name(program_type: u32) -> &'static str {
    match program_type {
        0 => "PT_NULL",
        PT_LOAD => "PT_LOAD",
        PT_DYNAMIC => "PT_DYNAMIC",
        PT_INTERP => "PT_INTERP",
        4 => "PT_NOTE",
        5 => "PT_SHLIB",
        6 => "PT_PHDR",
        7 => "PT_TLS",
        0x6474_E550 => "PT_GNU_EH_FRAME",
        0x6474_E551 => "PT_GNU_STACK",
        0x6474_E552 => "PT_GNU_RELRO",
        _ => "PT_UNKNOWN",
    }
}

fn section_header_type_name(section_type: u32) -> &'static str {
    match section_type {
        0 => "SHT_NULL",
        1 => "SHT_PROGBITS",
        2 => "SHT_SYMTAB",
        3 => "SHT_STRTAB",
        4 => "SHT_RELA",
        5 => "SHT_HASH",
        6 => "SHT_DYNAMIC",
        7 => "SHT_NOTE",
        8 => "SHT_NOBITS",
        9 => "SHT_REL",
        11 => "SHT_DYNSYM",
        14 => "SHT_INIT_ARRAY",
        15 => "SHT_FINI_ARRAY",
        0x6FFF_FFF6 => "SHT_GNU_HASH",
        0x6FFF_FFFE => "SHT_GNU_VERNEED",
        0x6FFF_FFFF => "SHT_GNU_VERSYM",
        _ => "SHT_UNKNOWN",
    }
}

fn dynamic_tag_name(tag: i64) -> &'static str {
    match tag {
        DT_NULL => "DT_NULL",
        DT_NEEDED => "DT_NEEDED",
        DT_HASH => "DT_HASH",
        DT_STRTAB => "DT_STRTAB",
        DT_SYMTAB => "DT_SYMTAB",
        DT_STRSZ => "DT_STRSZ",
        DT_SYMENT => "DT_SYMENT",
        DT_SONAME => "DT_SONAME",
        DT_RPATH => "DT_RPATH",
        DT_RUNPATH => "DT_RUNPATH",
        DT_GNU_HASH => "DT_GNU_HASH",
        DT_AUXILIARY => "DT_AUXILIARY",
        DT_FILTER => "DT_FILTER",
        _ => "DT_UNKNOWN",
    }
}

fn elf_headers_descriptor(elf_header_layout: &ElfHeaderLayout) -> Result<StructLayoutDescriptor, String> {
    let mut fields = vec![field_at(
        "ELFHeader",
        elf_header_layout.header_kind.elf_header_layout_id(),
        0,
    )];

    if elf_header_layout.program_header_count > 0 {
        fields.push(field_at(
            "ProgramHeaders",
            &elf_header_layout.program_headers_layout_id,
            elf_header_layout.program_header_offset,
        ));
    }

    if elf_header_layout.include_section_headers && elf_header_layout.section_header_count > 0 {
        fields.push(field_at(
            "SectionHeaders",
            &elf_header_layout.section_headers_layout_id,
            elf_header_layout.section_header_offset,
        ));
    }

    if let Some(interpreter) = &elf_header_layout.interpreter {
        fields.push(array_field_at(
            &interpreter.display_name,
            STRING_UTF8_NULL_TERMINATED_DATA_TYPE_ID,
            interpreter.length_in_bytes,
            interpreter.offset,
        ));
    }

    if let Some(dynamic_table) = &elf_header_layout.dynamic_table {
        fields.push(field_at("DynamicEntries", &elf_header_layout.dynamic_entries_layout_id, dynamic_table.offset));
    }

    fields.extend(elf_header_layout.dynamic_strings.iter().map(|dynamic_string| {
        array_field_at(
            &dynamic_string.display_name,
            STRING_UTF8_NULL_TERMINATED_DATA_TYPE_ID,
            dynamic_string.length_in_bytes,
            dynamic_string.offset,
        )
    }));

    if let Some(first_dynamic_symbol) = elf_header_layout.dynamic_symbols.first() {
        fields.push(field_at(
            "DynamicSymbols",
            &elf_header_layout.dynamic_symbols_layout_id,
            first_dynamic_symbol.symbol_offset,
        ));
    }

    Ok(struct_layout_descriptor(
        &elf_header_layout.root_layout_id,
        fields,
        Some(elf_header_layout.headers_size_in_bytes()?),
    ))
}

fn program_headers_descriptor(elf_header_layout: &ElfHeaderLayout) -> Result<StructLayoutDescriptor, String> {
    let table_size = elf_header_layout
        .program_headers_end_offset()?
        .saturating_sub(elf_header_layout.program_header_offset);
    let fields = if elf_header_layout.program_headers.is_empty() {
        vec![field_at(
            "Header",
            elf_header_layout.header_kind.program_header_layout_id(),
            0,
        )]
    } else {
        elf_header_layout
            .program_headers
            .iter()
            .map(|program_header| {
                field_at(
                    &program_header_field_name(program_header),
                    elf_header_layout.header_kind.program_header_layout_id(),
                    program_header
                        .header_offset
                        .saturating_sub(elf_header_layout.program_header_offset),
                )
            })
            .collect()
    };

    Ok(struct_layout_descriptor(&elf_header_layout.program_headers_layout_id, fields, Some(table_size)))
}

fn section_headers_descriptor(elf_header_layout: &ElfHeaderLayout) -> Result<StructLayoutDescriptor, String> {
    let table_size = elf_header_layout
        .section_headers_end_offset()?
        .saturating_sub(elf_header_layout.section_header_offset);
    let fields = if elf_header_layout.section_headers.is_empty() {
        vec![field_at(
            "Header",
            elf_header_layout.header_kind.section_header_layout_id(),
            0,
        )]
    } else {
        elf_header_layout
            .section_headers
            .iter()
            .map(|section_header| {
                field_at(
                    &section_header_field_name(section_header, &elf_header_layout.section_header_names),
                    elf_header_layout.header_kind.section_header_layout_id(),
                    section_header
                        .header_offset
                        .saturating_sub(elf_header_layout.section_header_offset),
                )
            })
            .collect()
    };

    Ok(struct_layout_descriptor(&elf_header_layout.section_headers_layout_id, fields, Some(table_size)))
}

fn dynamic_entries_descriptor(elf_header_layout: &ElfHeaderLayout) -> Result<StructLayoutDescriptor, String> {
    let Some(dynamic_table) = &elf_header_layout.dynamic_table else {
        return Err(String::from("ELF dynamic entries descriptor requires a parsed dynamic table."));
    };
    let fields = dynamic_table
        .entries
        .iter()
        .map(|dynamic_entry| {
            field_at(
                &dynamic_entry_field_name(dynamic_entry),
                elf_header_layout.header_kind.dynamic_layout_id(),
                dynamic_entry.entry_offset.saturating_sub(dynamic_table.offset),
            )
        })
        .collect();

    Ok(struct_layout_descriptor(
        &elf_header_layout.dynamic_entries_layout_id,
        fields,
        Some(dynamic_table.size_in_bytes),
    ))
}

fn dynamic_symbols_descriptor(elf_header_layout: &ElfHeaderLayout) -> Result<StructLayoutDescriptor, String> {
    let Some(first_dynamic_symbol) = elf_header_layout.dynamic_symbols.first() else {
        return Err(String::from("ELF dynamic symbols descriptor requires parsed dynamic symbols."));
    };
    let fields = elf_header_layout
        .dynamic_symbols
        .iter()
        .map(|dynamic_symbol| {
            field_at(
                &dynamic_symbol_field_name(dynamic_symbol),
                elf_header_layout.header_kind.symbol_layout_id(),
                dynamic_symbol
                    .symbol_offset
                    .saturating_sub(first_dynamic_symbol.symbol_offset),
            )
        })
        .collect::<Vec<_>>();
    let declared_size_in_bytes = u64::try_from(fields.len())
        .unwrap_or_default()
        .saturating_mul(elf_header_layout.header_kind.symbol_size_in_bytes());

    Ok(struct_layout_descriptor(
        &elf_header_layout.dynamic_symbols_layout_id,
        fields,
        Some(declared_size_in_bytes),
    ))
}

fn elf_header32_descriptor(byte_order: ElfByteOrder) -> StructLayoutDescriptor {
    struct_layout_descriptor(
        ELF_HEADER32_ID,
        vec![
            array_field_at("e_ident", "u8", ELF_IDENT_SIZE_IN_BYTES, 0),
            field_at("e_type", u16_type_id(byte_order), 16),
            field_at("e_machine", u16_type_id(byte_order), 18),
            field_at("e_version", u32_type_id(byte_order), 20),
            field_at("e_entry", u32_type_id(byte_order), 24),
            field_at("e_phoff", u32_type_id(byte_order), 28),
            field_at("e_shoff", u32_type_id(byte_order), 32),
            field_at("e_flags", u32_type_id(byte_order), 36),
            field_at("e_ehsize", u16_type_id(byte_order), 40),
            field_at("e_phentsize", u16_type_id(byte_order), 42),
            field_at("e_phnum", u16_type_id(byte_order), 44),
            field_at("e_shentsize", u16_type_id(byte_order), 46),
            field_at("e_shnum", u16_type_id(byte_order), 48),
            field_at("e_shstrndx", u16_type_id(byte_order), 50),
        ],
        Some(ELF_HEADER32_SIZE_IN_BYTES),
    )
}

fn elf_header64_descriptor(byte_order: ElfByteOrder) -> StructLayoutDescriptor {
    struct_layout_descriptor(
        ELF_HEADER64_ID,
        vec![
            array_field_at("e_ident", "u8", ELF_IDENT_SIZE_IN_BYTES, 0),
            field_at("e_type", u16_type_id(byte_order), 16),
            field_at("e_machine", u16_type_id(byte_order), 18),
            field_at("e_version", u32_type_id(byte_order), 20),
            field_at("e_entry", u64_type_id(byte_order), 24),
            field_at("e_phoff", u64_type_id(byte_order), 32),
            field_at("e_shoff", u64_type_id(byte_order), 40),
            field_at("e_flags", u32_type_id(byte_order), 48),
            field_at("e_ehsize", u16_type_id(byte_order), 52),
            field_at("e_phentsize", u16_type_id(byte_order), 54),
            field_at("e_phnum", u16_type_id(byte_order), 56),
            field_at("e_shentsize", u16_type_id(byte_order), 58),
            field_at("e_shnum", u16_type_id(byte_order), 60),
            field_at("e_shstrndx", u16_type_id(byte_order), 62),
        ],
        Some(ELF_HEADER64_SIZE_IN_BYTES),
    )
}

fn program_header32_descriptor(byte_order: ElfByteOrder) -> StructLayoutDescriptor {
    struct_layout_descriptor(
        ELF_PROGRAM_HEADER32_ID,
        vec![
            field_at("p_type", u32_type_id(byte_order), 0),
            field_at("p_offset", u32_type_id(byte_order), 4),
            field_at("p_vaddr", u32_type_id(byte_order), 8),
            field_at("p_paddr", u32_type_id(byte_order), 12),
            field_at("p_filesz", u32_type_id(byte_order), 16),
            field_at("p_memsz", u32_type_id(byte_order), 20),
            field_at("p_flags", u32_type_id(byte_order), 24),
            field_at("p_align", u32_type_id(byte_order), 28),
        ],
        Some(ELF_PROGRAM_HEADER32_SIZE_IN_BYTES),
    )
}

fn program_header64_descriptor(byte_order: ElfByteOrder) -> StructLayoutDescriptor {
    struct_layout_descriptor(
        ELF_PROGRAM_HEADER64_ID,
        vec![
            field_at("p_type", u32_type_id(byte_order), 0),
            field_at("p_flags", u32_type_id(byte_order), 4),
            field_at("p_offset", u64_type_id(byte_order), 8),
            field_at("p_vaddr", u64_type_id(byte_order), 16),
            field_at("p_paddr", u64_type_id(byte_order), 24),
            field_at("p_filesz", u64_type_id(byte_order), 32),
            field_at("p_memsz", u64_type_id(byte_order), 40),
            field_at("p_align", u64_type_id(byte_order), 48),
        ],
        Some(ELF_PROGRAM_HEADER64_SIZE_IN_BYTES),
    )
}

fn section_header32_descriptor(byte_order: ElfByteOrder) -> StructLayoutDescriptor {
    struct_layout_descriptor(
        ELF_SECTION_HEADER32_ID,
        vec![
            field_at("sh_name", u32_type_id(byte_order), 0),
            field_at("sh_type", u32_type_id(byte_order), 4),
            field_at("sh_flags", u32_type_id(byte_order), 8),
            field_at("sh_addr", u32_type_id(byte_order), 12),
            field_at("sh_offset", u32_type_id(byte_order), 16),
            field_at("sh_size", u32_type_id(byte_order), 20),
            field_at("sh_link", u32_type_id(byte_order), 24),
            field_at("sh_info", u32_type_id(byte_order), 28),
            field_at("sh_addralign", u32_type_id(byte_order), 32),
            field_at("sh_entsize", u32_type_id(byte_order), 36),
        ],
        Some(ELF_SECTION_HEADER32_SIZE_IN_BYTES),
    )
}

fn section_header64_descriptor(byte_order: ElfByteOrder) -> StructLayoutDescriptor {
    struct_layout_descriptor(
        ELF_SECTION_HEADER64_ID,
        vec![
            field_at("sh_name", u32_type_id(byte_order), 0),
            field_at("sh_type", u32_type_id(byte_order), 4),
            field_at("sh_flags", u64_type_id(byte_order), 8),
            field_at("sh_addr", u64_type_id(byte_order), 16),
            field_at("sh_offset", u64_type_id(byte_order), 24),
            field_at("sh_size", u64_type_id(byte_order), 32),
            field_at("sh_link", u32_type_id(byte_order), 40),
            field_at("sh_info", u32_type_id(byte_order), 44),
            field_at("sh_addralign", u64_type_id(byte_order), 48),
            field_at("sh_entsize", u64_type_id(byte_order), 56),
        ],
        Some(ELF_SECTION_HEADER64_SIZE_IN_BYTES),
    )
}

fn dynamic32_descriptor(byte_order: ElfByteOrder) -> StructLayoutDescriptor {
    struct_layout_descriptor(
        ELF_DYNAMIC32_ID,
        vec![
            field_at("d_tag", i32_type_id(byte_order), 0),
            field_at("d_val", u32_type_id(byte_order), 4),
        ],
        Some(ELF_DYNAMIC32_SIZE_IN_BYTES),
    )
}

fn dynamic64_descriptor(byte_order: ElfByteOrder) -> StructLayoutDescriptor {
    struct_layout_descriptor(
        ELF_DYNAMIC64_ID,
        vec![
            field_at("d_tag", i64_type_id(byte_order), 0),
            field_at("d_val", u64_type_id(byte_order), 8),
        ],
        Some(ELF_DYNAMIC64_SIZE_IN_BYTES),
    )
}

fn symbol32_descriptor(byte_order: ElfByteOrder) -> StructLayoutDescriptor {
    struct_layout_descriptor(
        ELF_SYMBOL32_ID,
        vec![
            field_at("st_name", u32_type_id(byte_order), 0),
            field_at("st_value", u32_type_id(byte_order), 4),
            field_at("st_size", u32_type_id(byte_order), 8),
            field_at("st_info", "u8", 12),
            field_at("st_other", "u8", 13),
            field_at("st_shndx", u16_type_id(byte_order), 14),
        ],
        Some(ELF_SYMBOL32_SIZE_IN_BYTES),
    )
}

fn symbol64_descriptor(byte_order: ElfByteOrder) -> StructLayoutDescriptor {
    struct_layout_descriptor(
        ELF_SYMBOL64_ID,
        vec![
            field_at("st_name", u32_type_id(byte_order), 0),
            field_at("st_info", "u8", 4),
            field_at("st_other", "u8", 5),
            field_at("st_shndx", u16_type_id(byte_order), 6),
            field_at("st_value", u64_type_id(byte_order), 8),
            field_at("st_size", u64_type_id(byte_order), 16),
        ],
        Some(ELF_SYMBOL64_SIZE_IN_BYTES),
    )
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
    if let Some(string_array_length) = resolve_string_array_length(struct_layout_id) {
        return Some(string_array_length);
    }

    match struct_layout_id {
        ELF_HEADER32_ID => Some(ELF_HEADER32_SIZE_IN_BYTES),
        ELF_HEADER64_ID => Some(ELF_HEADER64_SIZE_IN_BYTES),
        ELF_PROGRAM_HEADER32_ID => Some(ELF_PROGRAM_HEADER32_SIZE_IN_BYTES),
        ELF_PROGRAM_HEADER64_ID => Some(ELF_PROGRAM_HEADER64_SIZE_IN_BYTES),
        ELF_SECTION_HEADER32_ID => Some(ELF_SECTION_HEADER32_SIZE_IN_BYTES),
        ELF_SECTION_HEADER64_ID => Some(ELF_SECTION_HEADER64_SIZE_IN_BYTES),
        ELF_DYNAMIC32_ID => Some(ELF_DYNAMIC32_SIZE_IN_BYTES),
        ELF_DYNAMIC64_ID => Some(ELF_DYNAMIC64_SIZE_IN_BYTES),
        ELF_SYMBOL32_ID => Some(ELF_SYMBOL32_SIZE_IN_BYTES),
        ELF_SYMBOL64_ID => Some(ELF_SYMBOL64_SIZE_IN_BYTES),
        _ => parse_sized_layout_id_suffix(struct_layout_id),
    }
}

fn estimate_symbolic_field_size_in_bytes(
    project_symbol_catalog: &ProjectSymbolCatalog,
    field_definition: &SymbolicFieldDefinition,
    data_type_size_by_id: &BTreeMap<String, u64>,
    visited_struct_layout_ids: &mut HashSet<String>,
) -> u64 {
    let unit_size = resolve_known_module_field_size(field_definition.get_data_type_ref().get_data_type_id())
        .or_else(|| {
            data_type_size_by_id
                .get(field_definition.get_data_type_ref().get_data_type_id())
                .copied()
        })
        .or_else(|| {
            estimate_struct_layout_size_in_bytes(
                project_symbol_catalog,
                field_definition.get_data_type_ref().get_data_type_id(),
                data_type_size_by_id,
                visited_struct_layout_ids,
            )
        })
        .unwrap_or(0);

    field_definition
        .get_container_type()
        .get_total_size_in_bytes(unit_size)
}

fn estimate_struct_layout_size_in_bytes(
    project_symbol_catalog: &ProjectSymbolCatalog,
    struct_layout_id: &str,
    data_type_size_by_id: &BTreeMap<String, u64>,
    visited_struct_layout_ids: &mut HashSet<String>,
) -> Option<u64> {
    if !visited_struct_layout_ids.insert(struct_layout_id.to_string()) {
        return None;
    }

    let struct_layout_definition = project_symbol_catalog
        .get_struct_layout_descriptors()
        .iter()
        .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == struct_layout_id)
        .map(StructLayoutDescriptor::get_struct_layout_definition);
    let estimated_size = struct_layout_definition
        .map(|symbolic_struct_definition| {
            estimate_symbolic_struct_definition_size_in_bytes(
                project_symbol_catalog,
                symbolic_struct_definition,
                data_type_size_by_id,
                visited_struct_layout_ids,
            )
        })
        .filter(|estimated_size| *estimated_size > 0);

    visited_struct_layout_ids.remove(struct_layout_id);

    estimated_size
}

fn estimate_symbolic_struct_definition_size_in_bytes(
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

fn resolve_string_array_length(struct_layout_id: &str) -> Option<u64> {
    resolve_fixed_array_length(struct_layout_id, STRING_UTF8_NULL_TERMINATED_DATA_TYPE_ID)
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

fn sanitize_field_name(field_name: &str) -> String {
    let sanitized_field_name = field_name
        .chars()
        .map(|field_name_character| {
            if field_name_character.is_ascii_alphanumeric() {
                field_name_character
            } else {
                '_'
            }
        })
        .collect::<String>();

    sanitized_field_name.trim_matches('_').to_string()
}

fn read_c_string_at(
    bytes: &[u8],
    offset: u64,
) -> Option<String> {
    let offset = usize::try_from(offset).ok()?;
    let string_bytes = bytes.get(offset..)?;
    let terminator_position = string_bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(string_bytes.len());

    if terminator_position == 0 {
        return None;
    }

    std::str::from_utf8(&string_bytes[..terminator_position])
        .ok()
        .map(ToString::to_string)
}

fn c_string_length_in_bytes(
    bytes: &[u8],
    offset: u64,
) -> Option<u64> {
    let offset = usize::try_from(offset).ok()?;
    let string_bytes = bytes.get(offset..)?;
    let terminator_position = string_bytes.iter().position(|byte| *byte == 0)?;

    u64::try_from(terminator_position.saturating_add(1)).ok()
}

fn i32_type_id(byte_order: ElfByteOrder) -> &'static str {
    match byte_order {
        ElfByteOrder::LittleEndian => "i32",
        ElfByteOrder::BigEndian => "i32be",
    }
}

fn i64_type_id(byte_order: ElfByteOrder) -> &'static str {
    match byte_order {
        ElfByteOrder::LittleEndian => "i64",
        ElfByteOrder::BigEndian => "i64be",
    }
}

fn u16_type_id(byte_order: ElfByteOrder) -> &'static str {
    match byte_order {
        ElfByteOrder::LittleEndian => "u16",
        ElfByteOrder::BigEndian => "u16be",
    }
}

fn u32_type_id(byte_order: ElfByteOrder) -> &'static str {
    match byte_order {
        ElfByteOrder::LittleEndian => "u32",
        ElfByteOrder::BigEndian => "u32be",
    }
}

fn u64_type_id(byte_order: ElfByteOrder) -> &'static str {
    match byte_order {
        ElfByteOrder::LittleEndian => "u64",
        ElfByteOrder::BigEndian => "u64be",
    }
}

fn read_i32_at(
    bytes: &[u8],
    offset_in_bytes: usize,
    byte_order: ElfByteOrder,
) -> Result<i32, String> {
    let bytes = read_array_at::<4>(bytes, offset_in_bytes)?;

    Ok(match byte_order {
        ElfByteOrder::LittleEndian => i32::from_le_bytes(bytes),
        ElfByteOrder::BigEndian => i32::from_be_bytes(bytes),
    })
}

fn read_i64_at(
    bytes: &[u8],
    offset_in_bytes: usize,
    byte_order: ElfByteOrder,
) -> Result<i64, String> {
    let bytes = read_array_at::<8>(bytes, offset_in_bytes)?;

    Ok(match byte_order {
        ElfByteOrder::LittleEndian => i64::from_le_bytes(bytes),
        ElfByteOrder::BigEndian => i64::from_be_bytes(bytes),
    })
}

fn read_u16_at(
    bytes: &[u8],
    offset_in_bytes: usize,
    byte_order: ElfByteOrder,
) -> Result<u16, String> {
    let bytes = read_array_at::<2>(bytes, offset_in_bytes)?;

    Ok(match byte_order {
        ElfByteOrder::LittleEndian => u16::from_le_bytes(bytes),
        ElfByteOrder::BigEndian => u16::from_be_bytes(bytes),
    })
}

fn read_u32_at(
    bytes: &[u8],
    offset_in_bytes: usize,
    byte_order: ElfByteOrder,
) -> Result<u32, String> {
    let bytes = read_array_at::<4>(bytes, offset_in_bytes)?;

    Ok(match byte_order {
        ElfByteOrder::LittleEndian => u32::from_le_bytes(bytes),
        ElfByteOrder::BigEndian => u32::from_be_bytes(bytes),
    })
}

fn read_u64_at(
    bytes: &[u8],
    offset_in_bytes: usize,
    byte_order: ElfByteOrder,
) -> Result<u64, String> {
    let bytes = read_array_at::<8>(bytes, offset_in_bytes)?;

    Ok(match byte_order {
        ElfByteOrder::LittleEndian => u64::from_le_bytes(bytes),
        ElfByteOrder::BigEndian => u64::from_be_bytes(bytes),
    })
}

fn read_array_at<const BYTE_COUNT: usize>(
    bytes: &[u8],
    offset_in_bytes: usize,
) -> Result<[u8; BYTE_COUNT], String> {
    let end_offset = offset_in_bytes
        .checked_add(BYTE_COUNT)
        .ok_or_else(|| String::from("ELF read offset is too large."))?;
    let readable_bytes = bytes
        .get(offset_in_bytes..end_offset)
        .ok_or_else(|| format!("Expected {BYTE_COUNT} readable ELF bytes at offset {offset_in_bytes}."))?;

    readable_bytes
        .try_into()
        .map_err(|_| format!("Expected {BYTE_COUNT} readable ELF bytes at offset {offset_in_bytes}."))
}

impl ElfHeaderLayout {
    fn header_size_in_bytes(&self) -> u64 {
        self.header_kind.header_size_in_bytes()
    }

    fn program_headers_end_offset(&self) -> Result<u64, String> {
        table_end_offset(
            self.program_header_offset,
            self.program_header_entry_size,
            self.program_header_count,
            "ELF program header table",
        )
    }

    fn section_headers_end_offset(&self) -> Result<u64, String> {
        table_end_offset(
            self.section_header_offset,
            self.section_header_entry_size,
            self.section_header_count,
            "ELF section header table",
        )
    }

    fn headers_size_in_bytes(&self) -> Result<u64, String> {
        let program_headers_end_offset = self.program_headers_end_offset()?;
        let section_headers_end_offset = if self.include_section_headers {
            self.section_headers_end_offset()?
        } else {
            0
        };
        let interpreter_end_offset = self
            .interpreter
            .as_ref()
            .and_then(|interpreter| interpreter.offset.checked_add(interpreter.length_in_bytes))
            .unwrap_or(0);
        let dynamic_table_end_offset = self
            .dynamic_table
            .as_ref()
            .and_then(|dynamic_table| dynamic_table.offset.checked_add(dynamic_table.size_in_bytes))
            .unwrap_or(0);
        let dynamic_string_end_offset = self
            .dynamic_strings
            .iter()
            .filter_map(|dynamic_string| {
                dynamic_string
                    .offset
                    .checked_add(dynamic_string.length_in_bytes)
            })
            .max()
            .unwrap_or(0);
        let dynamic_symbol_end_offset = self
            .dynamic_symbols
            .last()
            .and_then(|dynamic_symbol| {
                dynamic_symbol
                    .symbol_offset
                    .checked_add(self.header_kind.symbol_size_in_bytes())
            })
            .unwrap_or(0);

        Ok(self
            .header_size_in_bytes()
            .max(program_headers_end_offset)
            .max(section_headers_end_offset)
            .max(interpreter_end_offset)
            .max(dynamic_table_end_offset)
            .max(dynamic_string_end_offset)
            .max(dynamic_symbol_end_offset))
    }
}

impl ElfHeaderKind {
    fn header_size_in_bytes(&self) -> u64 {
        match self {
            Self::Elf32(_) => ELF_HEADER32_SIZE_IN_BYTES,
            Self::Elf64(_) => ELF_HEADER64_SIZE_IN_BYTES,
        }
    }

    fn program_header_size_in_bytes(&self) -> u64 {
        match self {
            Self::Elf32(_) => ELF_PROGRAM_HEADER32_SIZE_IN_BYTES,
            Self::Elf64(_) => ELF_PROGRAM_HEADER64_SIZE_IN_BYTES,
        }
    }

    fn section_header_size_in_bytes(&self) -> u64 {
        match self {
            Self::Elf32(_) => ELF_SECTION_HEADER32_SIZE_IN_BYTES,
            Self::Elf64(_) => ELF_SECTION_HEADER64_SIZE_IN_BYTES,
        }
    }

    fn dynamic_size_in_bytes(&self) -> u64 {
        match self {
            Self::Elf32(_) => ELF_DYNAMIC32_SIZE_IN_BYTES,
            Self::Elf64(_) => ELF_DYNAMIC64_SIZE_IN_BYTES,
        }
    }

    fn symbol_size_in_bytes(&self) -> u64 {
        match self {
            Self::Elf32(_) => ELF_SYMBOL32_SIZE_IN_BYTES,
            Self::Elf64(_) => ELF_SYMBOL64_SIZE_IN_BYTES,
        }
    }

    fn is_64(&self) -> bool {
        matches!(self, Self::Elf64(_))
    }

    fn elf_header_layout_id(&self) -> &'static str {
        match self {
            Self::Elf32(_) => ELF_HEADER32_ID,
            Self::Elf64(_) => ELF_HEADER64_ID,
        }
    }

    fn program_header_layout_id(&self) -> &'static str {
        match self {
            Self::Elf32(_) => ELF_PROGRAM_HEADER32_ID,
            Self::Elf64(_) => ELF_PROGRAM_HEADER64_ID,
        }
    }

    fn section_header_layout_id(&self) -> &'static str {
        match self {
            Self::Elf32(_) => ELF_SECTION_HEADER32_ID,
            Self::Elf64(_) => ELF_SECTION_HEADER64_ID,
        }
    }

    fn dynamic_layout_id(&self) -> &'static str {
        match self {
            Self::Elf32(_) => ELF_DYNAMIC32_ID,
            Self::Elf64(_) => ELF_DYNAMIC64_ID,
        }
    }

    fn symbol_layout_id(&self) -> &'static str {
        match self {
            Self::Elf32(_) => ELF_SYMBOL32_ID,
            Self::Elf64(_) => ELF_SYMBOL64_ID,
        }
    }

    fn byte_order(&self) -> ElfByteOrder {
        match self {
            Self::Elf32(byte_order) | Self::Elf64(byte_order) => *byte_order,
        }
    }

    fn display_name(&self) -> &'static str {
        match self {
            Self::Elf32(_) => "ELF 32-bit",
            Self::Elf64(_) => "ELF 64-bit",
        }
    }
}

fn table_end_offset(
    table_offset: u64,
    entry_size: u64,
    entry_count: u64,
    table_name: &str,
) -> Result<u64, String> {
    if table_offset == 0 || entry_size == 0 || entry_count == 0 {
        return Ok(0);
    }

    let table_size = entry_size
        .checked_mul(entry_count)
        .ok_or_else(|| format!("{table_name} size is too large."))?;

    table_offset
        .checked_add(table_size)
        .ok_or_else(|| format!("{table_name} end offset is too large."))
}

#[cfg(test)]
mod tests {
    use super::{
        DT_HASH, DT_NEEDED, DT_NULL, DT_SONAME, DT_STRSZ, DT_STRTAB, DT_SYMENT, DT_SYMTAB, ELF_HEADER64_ID, ELF_PROGRAM_HEADER64_ID, ElfByteOrder,
        ElfHeaderKind, PT_DYNAMIC, PT_INTERP, PT_LOAD, PopulateElfSymbolsAction, analyze_elf_header_layout, populate_elf_symbols, sanitize_identifier,
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
                header_bytes: build_test_elf_header_bytes(true),
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

            if read_start >= self.header_bytes.len() {
                return Ok(Vec::new());
            }

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
        let action = PopulateElfSymbolsAction;
        let module_context = SymbolTreeActionContext::new(SymbolTreeActionSelection::ModuleRoot {
            module_name: String::from("squalr"),
        });
        let derived_context = SymbolTreeActionContext::new(SymbolTreeActionSelection::DerivedNode {
            tree_node_key: String::from("u8:squalr:0:64"),
        });

        assert!(action.is_visible(&module_context));
        assert!(!action.is_visible(&derived_context));
    }

    #[test]
    fn analyze_elf_header_layout_parses_64_bit_little_endian_tables() {
        let process_memory_store = TestProcessMemoryStore::new();
        let elf_header_layout = analyze_elf_header_layout(&process_memory_store, "squalr").expect("Expected ELF layout.");

        assert_eq!(elf_header_layout.header_kind, ElfHeaderKind::Elf64(ElfByteOrder::LittleEndian));
        assert_eq!(elf_header_layout.program_header_offset, 0x40);
        assert_eq!(elf_header_layout.program_header_count, 3);
        assert_eq!(elf_header_layout.section_header_offset, 0x500);
        assert_eq!(elf_header_layout.section_header_count, 4);
        assert!(elf_header_layout.include_section_headers);
        assert_eq!(
            elf_header_layout
                .interpreter
                .as_ref()
                .map(|interpreter| interpreter.offset),
            Some(0x300)
        );
        assert_eq!(elf_header_layout.dynamic_strings.len(), 2);
        assert_eq!(elf_header_layout.dynamic_symbols.len(), 3);
    }

    #[test]
    fn populate_elf_symbols_adds_elf_headers_root_and_tables() {
        let project_symbol_catalog =
            ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![ProjectSymbolModule::new(String::from("squalr"), 0x2000)], Vec::new(), Vec::new());
        let services = TestSymbolTreeActionServices::new(project_symbol_catalog);
        let action = PopulateElfSymbolsAction;
        let context = SymbolTreeActionContext::new(SymbolTreeActionSelection::ModuleRoot {
            module_name: String::from("squalr"),
        });

        action
            .execute(&context, &services)
            .expect("Expected ELF symbol population to succeed.");

        let project_symbol_catalog = services.project_symbol_store.read_current_catalog();
        let symbol_module = project_symbol_catalog
            .find_symbol_module("squalr")
            .expect("Expected module to exist.");
        let elf_headers_layout_descriptor = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == "linux.elf.squalr.headers")
            .expect("Expected ELF headers descriptor.");
        let elf_headers_fields = elf_headers_layout_descriptor
            .get_struct_layout_definition()
            .get_fields();

        assert!(
            symbol_module
                .get_fields()
                .iter()
                .any(|module_field| module_field.get_display_name() == "ELF Headers")
        );
        assert_eq!(symbol_module.get_fields()[0].get_display_name(), "ELF Headers");
        assert_eq!(symbol_module.get_fields()[0].get_struct_layout_id(), "linux.elf.squalr.headers");
        assert_eq!(elf_headers_fields[0].get_data_type_ref().get_data_type_id(), ELF_HEADER64_ID);
        assert!(
            elf_headers_fields
                .iter()
                .any(|field_definition| field_definition.get_field_name() == "ProgramHeaders")
        );
        assert!(
            elf_headers_fields
                .iter()
                .any(|field_definition| field_definition.get_field_name() == "SectionHeaders")
        );
        assert!(
            project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .any(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == ELF_PROGRAM_HEADER64_ID)
        );
    }

    #[test]
    fn populate_elf_symbols_resolves_runtime_metadata_names() {
        let project_symbol_catalog =
            ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![ProjectSymbolModule::new(String::from("squalr"), 0x2000)], Vec::new(), Vec::new());
        let services = TestSymbolTreeActionServices::new(project_symbol_catalog);
        let action = PopulateElfSymbolsAction;
        let context = SymbolTreeActionContext::new(SymbolTreeActionSelection::ModuleRoot {
            module_name: String::from("squalr"),
        });

        action
            .execute(&context, &services)
            .expect("Expected ELF symbol population to succeed.");

        let project_symbol_catalog = services.project_symbol_store.read_current_catalog();
        let elf_header_field_names = layout_field_names(&project_symbol_catalog, "linux.elf.squalr.headers");
        let program_header_field_names = layout_field_names(&project_symbol_catalog, "linux.elf.squalr.program_headers");
        let section_header_field_names = layout_field_names(&project_symbol_catalog, "linux.elf.squalr.section_headers");
        let dynamic_entry_field_names = layout_field_names(&project_symbol_catalog, "linux.elf.squalr.dynamic_entries");
        let dynamic_symbol_field_names = layout_field_names(&project_symbol_catalog, "linux.elf.squalr.dynamic_symbols");

        assert!(elf_header_field_names.contains(&String::from("ELF Interpreter")));
        assert!(elf_header_field_names.contains(&String::from("DT_NEEDED_libc_so_6_00")));
        assert!(elf_header_field_names.contains(&String::from("DT_SONAME_squalr_01")));
        assert!(elf_header_field_names.contains(&String::from("DynamicEntries")));
        assert!(elf_header_field_names.contains(&String::from("DynamicSymbols")));
        assert!(program_header_field_names.contains(&String::from("PT_INTERP_01")));
        assert!(program_header_field_names.contains(&String::from("PT_DYNAMIC_02")));
        assert!(section_header_field_names.contains(&String::from("Section_interp_01")));
        assert!(dynamic_entry_field_names.contains(&String::from("DT_NEEDED_00")));
        assert!(dynamic_symbol_field_names.contains(&String::from("Symbol_symbol_one_0001")));
    }

    #[test]
    fn populate_elf_symbols_replaces_existing_root_u8_array() {
        let mut symbol_module = ProjectSymbolModule::new(String::from("squalr"), 0x2000);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("u8_00000000"), 0, String::from("u8[512]")));
        let mut project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![symbol_module], Vec::new(), Vec::new());
        let elf_header_layout = analyze_elf_header_layout(&TestProcessMemoryStore::new(), "squalr").expect("Expected ELF layout.");

        populate_elf_symbols(&mut project_symbol_catalog, "squalr", &elf_header_layout, &default_data_type_size_by_id())
            .expect("Expected ELF symbol population to replace u8[] root field.");

        let fields = project_symbol_catalog
            .find_symbol_module("squalr")
            .expect("Expected module to exist.")
            .get_fields();

        assert!(
            fields
                .iter()
                .all(|field| field.get_display_name() != "u8_00000000")
        );
        assert_eq!(fields[0].get_display_name(), "ELF Headers");
        assert_eq!(fields[0].get_struct_layout_id(), "linux.elf.squalr.headers");
    }

    #[test]
    fn populate_elf_symbols_updates_module_root_layout() {
        let module_root_layout_descriptor = StructLayoutDescriptor::new(
            String::from("squalr"),
            SymbolicStructDefinition::new(String::from("squalr"), Vec::new()).with_declared_size_in_bytes(Some(0x2000)),
        );
        let mut project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(
            vec![ProjectSymbolModule::new(String::from("squalr"), 0x2000)],
            vec![module_root_layout_descriptor],
            Vec::new(),
        );
        let elf_header_layout = analyze_elf_header_layout(&TestProcessMemoryStore::new(), "squalr").expect("Expected ELF layout.");

        populate_elf_symbols(&mut project_symbol_catalog, "squalr", &elf_header_layout, &default_data_type_size_by_id())
            .expect("Expected ELF symbol population to update module root layout.");

        let module_root_layout_definition = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == "squalr")
            .expect("Expected squalr root layout.")
            .get_struct_layout_definition();

        assert!(
            module_root_layout_definition
                .get_fields()
                .iter()
                .any(|field_definition| field_definition.get_field_name() == "ELF Headers")
        );
    }

    #[test]
    fn section_headers_are_skipped_when_not_readable() {
        let mut process_memory_store = TestProcessMemoryStore::new();
        process_memory_store.header_bytes = build_test_elf_header_bytes(false);

        let elf_header_layout = analyze_elf_header_layout(&process_memory_store, "squalr").expect("Expected ELF layout.");

        assert!(!elf_header_layout.include_section_headers);
        assert_eq!(
            elf_header_layout
                .headers_size_in_bytes()
                .expect("Expected ELF header size."),
            0xE8
        );
    }

    #[test]
    fn sanitize_identifier_normalizes_module_names() {
        assert_eq!(sanitize_identifier("ld-linux-x86_64.so.2"), "ld_linux_x86_64_so_2");
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

    fn layout_field_names(
        project_symbol_catalog: &ProjectSymbolCatalog,
        struct_layout_id: &str,
    ) -> Vec<String> {
        project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == struct_layout_id)
            .expect("Expected struct layout descriptor.")
            .get_struct_layout_definition()
            .get_fields()
            .iter()
            .map(|field_definition| field_definition.get_field_name().to_string())
            .collect()
    }

    fn build_test_elf_header_bytes(include_section_headers: bool) -> Vec<u8> {
        let header_length = if include_section_headers { 0x900 } else { 0x180 };
        let mut header_bytes = vec![0_u8; header_length];
        let interpreter_bytes = b"/lib64/ld-linux-x86-64.so.2\0";
        let dynamic_string_table_bytes = b"\0libc.so.6\0squalr\0symbol_one\0";
        let section_string_table_bytes = b"\0.interp\0.dynamic\0.shstrtab\0";
        let dynamic_table_offset = 0x340_u64;
        let dynamic_table_size = 0x80_u64;

        header_bytes[0..4].copy_from_slice(b"\x7FELF");
        header_bytes[4] = 2;
        header_bytes[5] = 1;
        header_bytes[6] = 1;
        header_bytes[16..18].copy_from_slice(&3_u16.to_le_bytes());
        header_bytes[18..20].copy_from_slice(&0x3E_u16.to_le_bytes());
        header_bytes[20..24].copy_from_slice(&1_u32.to_le_bytes());
        header_bytes[24..32].copy_from_slice(&0x401000_u64.to_le_bytes());
        header_bytes[32..40].copy_from_slice(&0x40_u64.to_le_bytes());
        header_bytes[40..48].copy_from_slice(&0x500_u64.to_le_bytes());
        header_bytes[52..54].copy_from_slice(&64_u16.to_le_bytes());
        header_bytes[54..56].copy_from_slice(&56_u16.to_le_bytes());
        header_bytes[56..58].copy_from_slice(&3_u16.to_le_bytes());
        header_bytes[58..60].copy_from_slice(&64_u16.to_le_bytes());
        header_bytes[60..62].copy_from_slice(&4_u16.to_le_bytes());
        header_bytes[62..64].copy_from_slice(&3_u16.to_le_bytes());

        write_program_header64(&mut header_bytes, 0x40, PT_LOAD, 0, 0, header_length as u64, header_length as u64);
        write_program_header64(
            &mut header_bytes,
            0x40 + 56,
            PT_INTERP,
            0x300,
            0x300,
            interpreter_bytes.len() as u64,
            interpreter_bytes.len() as u64,
        );
        write_program_header64(
            &mut header_bytes,
            0x40 + 112,
            PT_DYNAMIC,
            dynamic_table_offset,
            dynamic_table_offset,
            dynamic_table_size,
            dynamic_table_size,
        );

        if include_section_headers {
            header_bytes[0x300..0x300 + interpreter_bytes.len()].copy_from_slice(interpreter_bytes);
            write_dynamic64(&mut header_bytes, dynamic_table_offset as usize, DT_NEEDED, 1);
            write_dynamic64(&mut header_bytes, dynamic_table_offset as usize + 16, DT_SONAME, 11);
            write_dynamic64(&mut header_bytes, dynamic_table_offset as usize + 32, DT_STRTAB, 0x780);
            write_dynamic64(
                &mut header_bytes,
                dynamic_table_offset as usize + 48,
                DT_STRSZ,
                dynamic_string_table_bytes.len() as u64,
            );
            write_dynamic64(&mut header_bytes, dynamic_table_offset as usize + 64, DT_SYMTAB, 0x820);
            write_dynamic64(&mut header_bytes, dynamic_table_offset as usize + 80, DT_SYMENT, 24);
            write_dynamic64(&mut header_bytes, dynamic_table_offset as usize + 96, DT_HASH, 0x760);
            write_dynamic64(&mut header_bytes, dynamic_table_offset as usize + 112, DT_NULL, 0);

            write_section_header64(&mut header_bytes, 0x500, 0, 0, 0, 0, 0);
            write_section_header64(&mut header_bytes, 0x500 + 64, 1, 1, 0x300, 0x300, interpreter_bytes.len() as u64);
            write_section_header64(
                &mut header_bytes,
                0x500 + 128,
                9,
                6,
                dynamic_table_offset,
                dynamic_table_offset,
                dynamic_table_size,
            );
            write_section_header64(&mut header_bytes, 0x500 + 192, 18, 3, 0x700, 0x700, section_string_table_bytes.len() as u64);

            header_bytes[0x700..0x700 + section_string_table_bytes.len()].copy_from_slice(section_string_table_bytes);
            header_bytes[0x760..0x764].copy_from_slice(&1_u32.to_le_bytes());
            header_bytes[0x764..0x768].copy_from_slice(&3_u32.to_le_bytes());
            header_bytes[0x780..0x780 + dynamic_string_table_bytes.len()].copy_from_slice(dynamic_string_table_bytes);
            write_symbol64(&mut header_bytes, 0x820, 0);
            write_symbol64(&mut header_bytes, 0x820 + 24, 18);
            write_symbol64(&mut header_bytes, 0x820 + 48, 0);
        }

        header_bytes
    }

    fn write_program_header64(
        header_bytes: &mut [u8],
        offset: usize,
        program_type: u32,
        file_offset: u64,
        virtual_address: u64,
        file_size: u64,
        memory_size: u64,
    ) {
        header_bytes[offset..offset + 4].copy_from_slice(&program_type.to_le_bytes());
        header_bytes[offset + 8..offset + 16].copy_from_slice(&file_offset.to_le_bytes());
        header_bytes[offset + 16..offset + 24].copy_from_slice(&virtual_address.to_le_bytes());
        header_bytes[offset + 32..offset + 40].copy_from_slice(&file_size.to_le_bytes());
        header_bytes[offset + 40..offset + 48].copy_from_slice(&memory_size.to_le_bytes());
    }

    fn write_section_header64(
        header_bytes: &mut [u8],
        offset: usize,
        name_offset: u32,
        section_type: u32,
        virtual_address: u64,
        file_offset: u64,
        size_in_bytes: u64,
    ) {
        header_bytes[offset..offset + 4].copy_from_slice(&name_offset.to_le_bytes());
        header_bytes[offset + 4..offset + 8].copy_from_slice(&section_type.to_le_bytes());
        header_bytes[offset + 16..offset + 24].copy_from_slice(&virtual_address.to_le_bytes());
        header_bytes[offset + 24..offset + 32].copy_from_slice(&file_offset.to_le_bytes());
        header_bytes[offset + 32..offset + 40].copy_from_slice(&size_in_bytes.to_le_bytes());
    }

    fn write_dynamic64(
        header_bytes: &mut [u8],
        offset: usize,
        tag: i64,
        value: u64,
    ) {
        header_bytes[offset..offset + 8].copy_from_slice(&tag.to_le_bytes());
        header_bytes[offset + 8..offset + 16].copy_from_slice(&value.to_le_bytes());
    }

    fn write_symbol64(
        header_bytes: &mut [u8],
        offset: usize,
        name_offset: u32,
    ) {
        header_bytes[offset..offset + 4].copy_from_slice(&name_offset.to_le_bytes());
    }
}
