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
const ELF_IDENT_SIZE_IN_BYTES: u64 = 16;
const ELF_HEADER32_SIZE_IN_BYTES: u64 = 52;
const ELF_HEADER64_SIZE_IN_BYTES: u64 = 64;
const ELF_PROGRAM_HEADER32_SIZE_IN_BYTES: u64 = 32;
const ELF_PROGRAM_HEADER64_SIZE_IN_BYTES: u64 = 56;
const ELF_SECTION_HEADER32_SIZE_IN_BYTES: u64 = 40;
const ELF_SECTION_HEADER64_SIZE_IN_BYTES: u64 = 64;
const INITIAL_ELF_HEADER_READ_SIZE: u64 = 0x1000;
const MAX_ELF_HEADER_READ_SIZE: u64 = 0x200000;
const ELF_CLASS32: u8 = 1;
const ELF_CLASS64: u8 = 2;
const ELF_DATA_LITTLE_ENDIAN: u8 = 1;
const ELF_DATA_BIG_ENDIAN: u8 = 2;

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
    root_layout_id: String,
    program_headers_layout_id: String,
    section_headers_layout_id: String,
    include_section_headers: bool,
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
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, elf_headers_descriptor(elf_header_layout)?);
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, program_headers_descriptor(elf_header_layout)?);

    if elf_header_layout.include_section_headers {
        upsert_struct_layout_descriptor(&mut struct_layout_descriptors, section_headers_descriptor(elf_header_layout)?);
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

    if required_program_header_size > initial_header_bytes.len() as u64 {
        let read_size = required_program_header_size.min(MAX_ELF_HEADER_READ_SIZE);
        let header_bytes = process_memory_store.read_module_bytes(module_name, 0, read_size)?;

        if header_bytes.len() < required_program_header_size as usize {
            return Err(String::from("ELF program headers are not fully readable."));
        }
    }

    let section_headers_end_offset = elf_header_layout.section_headers_end_offset()?;
    if section_headers_end_offset > 0 && section_headers_end_offset <= MAX_ELF_HEADER_READ_SIZE {
        let section_header_bytes = if section_headers_end_offset <= initial_header_bytes.len() as u64 {
            initial_header_bytes
        } else {
            process_memory_store.read_module_bytes(module_name, 0, section_headers_end_offset)?
        };

        elf_header_layout.include_section_headers = section_header_bytes.len() >= section_headers_end_offset as usize;
    }

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
            root_layout_id: format!("{module_layout_id_prefix}.headers"),
            program_headers_layout_id: format!("{module_layout_id_prefix}.program_headers"),
            section_headers_layout_id: format!("{module_layout_id_prefix}.section_headers"),
            include_section_headers: false,
        },
        ElfHeaderKind::Elf64(byte_order) => ElfHeaderLayout {
            header_kind,
            program_header_offset: read_u64_at(header_bytes, 32, byte_order)?,
            section_header_offset: read_u64_at(header_bytes, 40, byte_order)?,
            program_header_entry_size: u64::from(read_u16_at(header_bytes, 54, byte_order)?),
            program_header_count: u64::from(read_u16_at(header_bytes, 56, byte_order)?),
            section_header_entry_size: u64::from(read_u16_at(header_bytes, 58, byte_order)?),
            section_header_count: u64::from(read_u16_at(header_bytes, 60, byte_order)?),
            root_layout_id: format!("{module_layout_id_prefix}.headers"),
            program_headers_layout_id: format!("{module_layout_id_prefix}.program_headers"),
            section_headers_layout_id: format!("{module_layout_id_prefix}.section_headers"),
            include_section_headers: false,
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

fn elf_headers_descriptor(elf_header_layout: &ElfHeaderLayout) -> Result<StructLayoutDescriptor, String> {
    let mut fields = vec![field_at(
        "ELFHeader",
        elf_header_layout.header_kind.elf_header_layout_id(),
        0,
    )];

    if elf_header_layout.program_header_count > 0 {
        fields.push(array_field_at(
            "ProgramHeaders",
            &elf_header_layout.program_headers_layout_id,
            elf_header_layout.program_header_count,
            elf_header_layout.program_header_offset,
        ));
    }

    if elf_header_layout.include_section_headers && elf_header_layout.section_header_count > 0 {
        fields.push(array_field_at(
            "SectionHeaders",
            &elf_header_layout.section_headers_layout_id,
            elf_header_layout.section_header_count,
            elf_header_layout.section_header_offset,
        ));
    }

    Ok(struct_layout_descriptor(
        &elf_header_layout.root_layout_id,
        fields,
        Some(elf_header_layout.headers_size_in_bytes()?),
    ))
}

fn program_headers_descriptor(elf_header_layout: &ElfHeaderLayout) -> Result<StructLayoutDescriptor, String> {
    let program_header_size = elf_header_layout
        .program_header_entry_size
        .max(elf_header_layout.header_kind.program_header_size_in_bytes());

    Ok(struct_layout_descriptor(
        &elf_header_layout.program_headers_layout_id,
        vec![field_at(
            "Header",
            elf_header_layout.header_kind.program_header_layout_id(),
            0,
        )],
        Some(program_header_size),
    ))
}

fn section_headers_descriptor(elf_header_layout: &ElfHeaderLayout) -> Result<StructLayoutDescriptor, String> {
    let section_header_size = elf_header_layout
        .section_header_entry_size
        .max(elf_header_layout.header_kind.section_header_size_in_bytes());

    Ok(struct_layout_descriptor(
        &elf_header_layout.section_headers_layout_id,
        vec![field_at(
            "Header",
            elf_header_layout.header_kind.section_header_layout_id(),
            0,
        )],
        Some(section_header_size),
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

    match struct_layout_id {
        ELF_HEADER32_ID => Some(ELF_HEADER32_SIZE_IN_BYTES),
        ELF_HEADER64_ID => Some(ELF_HEADER64_SIZE_IN_BYTES),
        ELF_PROGRAM_HEADER32_ID => Some(ELF_PROGRAM_HEADER32_SIZE_IN_BYTES),
        ELF_PROGRAM_HEADER64_ID => Some(ELF_PROGRAM_HEADER64_SIZE_IN_BYTES),
        ELF_SECTION_HEADER32_ID => Some(ELF_SECTION_HEADER32_SIZE_IN_BYTES),
        ELF_SECTION_HEADER64_ID => Some(ELF_SECTION_HEADER64_SIZE_IN_BYTES),
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

        Ok(self
            .header_size_in_bytes()
            .max(program_headers_end_offset)
            .max(section_headers_end_offset))
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
        ELF_HEADER64_ID, ELF_PROGRAM_HEADER64_ID, ElfByteOrder, ElfHeaderKind, PopulateElfSymbolsAction, analyze_elf_header_layout, populate_elf_symbols,
        sanitize_identifier,
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
        assert_eq!(elf_header_layout.program_header_count, 2);
        assert_eq!(elf_header_layout.section_header_offset, 0x200);
        assert_eq!(elf_header_layout.section_header_count, 3);
        assert!(elf_header_layout.include_section_headers);
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

        assert_eq!(symbol_module.get_fields().len(), 1);
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

        assert_eq!(fields.len(), 1);
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
            0xB0
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

    fn build_test_elf_header_bytes(include_section_headers: bool) -> Vec<u8> {
        let header_length = if include_section_headers { 0x300 } else { 0x180 };
        let mut header_bytes = vec![0_u8; header_length];

        header_bytes[0..4].copy_from_slice(b"\x7FELF");
        header_bytes[4] = 2;
        header_bytes[5] = 1;
        header_bytes[6] = 1;
        header_bytes[16..18].copy_from_slice(&3_u16.to_le_bytes());
        header_bytes[18..20].copy_from_slice(&0x3E_u16.to_le_bytes());
        header_bytes[20..24].copy_from_slice(&1_u32.to_le_bytes());
        header_bytes[24..32].copy_from_slice(&0x401000_u64.to_le_bytes());
        header_bytes[32..40].copy_from_slice(&0x40_u64.to_le_bytes());
        header_bytes[40..48].copy_from_slice(&0x200_u64.to_le_bytes());
        header_bytes[52..54].copy_from_slice(&64_u16.to_le_bytes());
        header_bytes[54..56].copy_from_slice(&56_u16.to_le_bytes());
        header_bytes[56..58].copy_from_slice(&2_u16.to_le_bytes());
        header_bytes[58..60].copy_from_slice(&64_u16.to_le_bytes());
        header_bytes[60..62].copy_from_slice(&3_u16.to_le_bytes());
        header_bytes[62..64].copy_from_slice(&1_u16.to_le_bytes());

        header_bytes
    }
}
