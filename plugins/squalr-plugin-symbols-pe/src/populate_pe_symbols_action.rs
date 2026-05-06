use squalr_engine_api::{
    plugins::{
        PluginPermission,
        symbol_tree::symbol_tree_action::{ProcessMemoryStore, SymbolTreeAction, SymbolTreeActionContext, SymbolTreeActionSelection, SymbolTreeActionServices},
    },
    registries::symbols::{struct_layout_descriptor::StructLayoutDescriptor, symbolic_resolver_descriptor::SymbolicResolverDescriptor},
    structures::{
        data_types::data_type_ref::DataTypeRef,
        data_values::container_type::ContainerType,
        projects::{
            project_symbol_catalog::ProjectSymbolCatalog, project_symbol_module::ProjectSymbolModule, project_symbol_module_field::ProjectSymbolModuleField,
        },
        structs::{
            symbolic_field_definition::SymbolicFieldDefinition,
            symbolic_resolver_definition::{SymbolicResolverBinaryOperator, SymbolicResolverDefinition, SymbolicResolverNode},
            symbolic_struct_definition::SymbolicStructDefinition,
        },
    },
};
use std::str::FromStr;

const PE_HEADERS32_ID: &str = "win.pe.PE_HEADERS32";
const PE_HEADERS64_ID: &str = "win.pe.PE_HEADERS64";
const IMAGE_DOS_HEADER_ID: &str = "win.pe.IMAGE_DOS_HEADER";
const IMAGE_FILE_HEADER_ID: &str = "win.pe.IMAGE_FILE_HEADER";
const IMAGE_NT_HEADERS32_ID: &str = "win.pe.IMAGE_NT_HEADERS32";
const IMAGE_NT_HEADERS64_ID: &str = "win.pe.IMAGE_NT_HEADERS64";
const IMAGE_DATA_DIRECTORY_ID: &str = "win.pe.IMAGE_DATA_DIRECTORY";
const IMAGE_OPTIONAL_HEADER32_ID: &str = "win.pe.IMAGE_OPTIONAL_HEADER32";
const IMAGE_OPTIONAL_HEADER64_ID: &str = "win.pe.IMAGE_OPTIONAL_HEADER64";
const IMAGE_SECTION_HEADER_ID: &str = "win.pe.IMAGE_SECTION_HEADER";
const PE_RESOLVER_DOS_HEADER_OFFSET_ID: &str = "win.pe.resolver.dos_header_offset";
const PE_RESOLVER_DOS_STUB_COUNT_ID: &str = "win.pe.resolver.dos_stub_count";
const PE_RESOLVER_DOS_STUB_OFFSET_ID: &str = "win.pe.resolver.dos_stub_offset";
const PE_RESOLVER_E_LFANEW_OFFSET_ID: &str = "win.pe.resolver.e_lfanew_offset";
const PE_RESOLVER_NT_HEADERS_OFFSET_ID: &str = "win.pe.resolver.nt_headers_offset";
const PE_RESOLVER_NUMBER_OF_SECTIONS_ID: &str = "win.pe.resolver.number_of_sections";
const PE_RESOLVER_NUMBER_OF_SECTIONS_OFFSET_ID: &str = "win.pe.resolver.number_of_sections_offset";
const PE_RESOLVER_SECTION_HEADERS_OFFSET_ID: &str = "win.pe.resolver.section_headers_offset";
const PE_RESOLVER_SIZE_OF_OPTIONAL_HEADER_OFFSET_ID: &str = "win.pe.resolver.size_of_optional_header_offset";
const DOS_HEADER_SIZE_IN_BYTES: u64 = 64;
const IMAGE_NT_SIGNATURE_SIZE_IN_BYTES: u64 = 4;
const IMAGE_FILE_HEADER_SIZE_IN_BYTES: u64 = 20;
const IMAGE_SECTION_HEADER_SIZE_IN_BYTES: u64 = 40;
const INITIAL_PE_HEADER_READ_SIZE: u64 = 0x1000;
const MAX_PE_HEADER_READ_SIZE: u64 = 0x10000;
const DOS_STUB_OFFSET: u64 = DOS_HEADER_SIZE_IN_BYTES;
const PE_FILE_HEADER_NUMBER_OF_SECTIONS_OFFSET: u64 = IMAGE_NT_SIGNATURE_SIZE_IN_BYTES + 2;
const PE_FILE_HEADER_SIZE_OF_OPTIONAL_HEADER_OFFSET: u64 = IMAGE_NT_SIGNATURE_SIZE_IN_BYTES + 16;
const PE_SECTION_HEADERS_OFFSET_FROM_NT_HEADERS: u64 = IMAGE_NT_SIGNATURE_SIZE_IN_BYTES + IMAGE_FILE_HEADER_SIZE_IN_BYTES;
const PE32_OPTIONAL_HEADER_MAGIC: u16 = 0x10B;
const PE64_OPTIONAL_HEADER_MAGIC: u16 = 0x20B;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PeOptionalHeaderKind {
    Pe32,
    Pe64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PeHeaderLayout {
    pe_header_offset: u64,
    size_of_optional_header: u64,
    number_of_sections: u64,
    optional_header_kind: PeOptionalHeaderKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DesiredModuleField {
    display_name: String,
    offset: u64,
    struct_layout_id: String,
    size_in_bytes: u64,
}

pub struct PopulatePeSymbolsAction;

impl SymbolTreeAction for PopulatePeSymbolsAction {
    fn action_id(&self) -> &'static str {
        "builtin.symbols.pe.populate-pe-symbols"
    }

    fn label(
        &self,
        _context: &SymbolTreeActionContext,
    ) -> String {
        String::from("Populate PE Symbols")
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
            return Err(String::from("PE symbol population requires a module root selection."));
        };
        let module_name = module_name.clone();
        let module_name_for_update = module_name.clone();
        let pe_header_layout = analyze_pe_header_layout(services.process_memory(), &module_name)?;

        services.symbol_store().write_catalog(
            "populate PE symbols",
            Box::new(move |project_symbol_catalog| populate_pe_symbols(project_symbol_catalog, &module_name_for_update, &pe_header_layout)),
        )?;
        services.symbol_tree_window().request_refresh();
        services
            .symbol_tree_window()
            .focus_tree_node(&format!("module:{module_name}"));

        Ok(())
    }
}

fn populate_pe_symbols(
    project_symbol_catalog: &mut ProjectSymbolCatalog,
    module_name: &str,
    pe_header_layout: &PeHeaderLayout,
) -> Result<(), String> {
    upsert_pe_symbolic_resolver_descriptors(project_symbol_catalog);
    upsert_pe_struct_layout_descriptors(project_symbol_catalog)?;
    upsert_pe_module_fields(project_symbol_catalog, module_name, pe_header_layout)
}

fn upsert_pe_symbolic_resolver_descriptors(project_symbol_catalog: &mut ProjectSymbolCatalog) {
    let mut symbolic_resolver_descriptors = project_symbol_catalog
        .get_symbolic_resolver_descriptors()
        .to_vec();

    upsert_symbolic_resolver_descriptor(
        &mut symbolic_resolver_descriptors,
        literal_resolver_descriptor(PE_RESOLVER_DOS_HEADER_OFFSET_ID, 0),
    );
    upsert_symbolic_resolver_descriptor(
        &mut symbolic_resolver_descriptors,
        literal_resolver_descriptor(PE_RESOLVER_DOS_STUB_OFFSET_ID, DOS_STUB_OFFSET as i128),
    );
    upsert_symbolic_resolver_descriptor(
        &mut symbolic_resolver_descriptors,
        literal_resolver_descriptor(PE_RESOLVER_E_LFANEW_OFFSET_ID, 0x3C),
    );
    upsert_symbolic_resolver_descriptor(
        &mut symbolic_resolver_descriptors,
        local_field_resolver_descriptor(PE_RESOLVER_NT_HEADERS_OFFSET_ID, "e_lfanew"),
    );
    upsert_symbolic_resolver_descriptor(
        &mut symbolic_resolver_descriptors,
        local_field_resolver_descriptor(PE_RESOLVER_NUMBER_OF_SECTIONS_ID, "NumberOfSections"),
    );
    upsert_symbolic_resolver_descriptor(
        &mut symbolic_resolver_descriptors,
        e_lfanew_plus_resolver_descriptor(PE_RESOLVER_NUMBER_OF_SECTIONS_OFFSET_ID, PE_FILE_HEADER_NUMBER_OF_SECTIONS_OFFSET),
    );
    upsert_symbolic_resolver_descriptor(
        &mut symbolic_resolver_descriptors,
        e_lfanew_plus_resolver_descriptor(PE_RESOLVER_SIZE_OF_OPTIONAL_HEADER_OFFSET_ID, PE_FILE_HEADER_SIZE_OF_OPTIONAL_HEADER_OFFSET),
    );
    upsert_symbolic_resolver_descriptor(
        &mut symbolic_resolver_descriptors,
        SymbolicResolverDescriptor::new(
            PE_RESOLVER_DOS_STUB_COUNT_ID.to_string(),
            SymbolicResolverDefinition::new(SymbolicResolverNode::new_binary(
                SymbolicResolverBinaryOperator::Subtract,
                SymbolicResolverNode::new_local_field(String::from("e_lfanew")),
                SymbolicResolverNode::new_literal(DOS_HEADER_SIZE_IN_BYTES as i128),
            )),
        ),
    );
    upsert_symbolic_resolver_descriptor(
        &mut symbolic_resolver_descriptors,
        SymbolicResolverDescriptor::new(
            PE_RESOLVER_SECTION_HEADERS_OFFSET_ID.to_string(),
            SymbolicResolverDefinition::new(SymbolicResolverNode::new_binary(
                SymbolicResolverBinaryOperator::Add,
                e_lfanew_plus_node(PE_SECTION_HEADERS_OFFSET_FROM_NT_HEADERS),
                SymbolicResolverNode::new_local_field(String::from("SizeOfOptionalHeader")),
            )),
        ),
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

fn literal_resolver_descriptor(
    resolver_id: &str,
    value: i128,
) -> SymbolicResolverDescriptor {
    SymbolicResolverDescriptor::new(
        resolver_id.to_string(),
        SymbolicResolverDefinition::new(SymbolicResolverNode::new_literal(value)),
    )
}

fn local_field_resolver_descriptor(
    resolver_id: &str,
    field_name: &str,
) -> SymbolicResolverDescriptor {
    SymbolicResolverDescriptor::new(
        resolver_id.to_string(),
        SymbolicResolverDefinition::new(SymbolicResolverNode::new_local_field(field_name.to_string())),
    )
}

fn e_lfanew_plus_resolver_descriptor(
    resolver_id: &str,
    offset: u64,
) -> SymbolicResolverDescriptor {
    SymbolicResolverDescriptor::new(resolver_id.to_string(), SymbolicResolverDefinition::new(e_lfanew_plus_node(offset)))
}

fn e_lfanew_plus_node(offset: u64) -> SymbolicResolverNode {
    SymbolicResolverNode::new_binary(
        SymbolicResolverBinaryOperator::Add,
        SymbolicResolverNode::new_local_field(String::from("e_lfanew")),
        SymbolicResolverNode::new_literal(offset as i128),
    )
}

fn upsert_pe_struct_layout_descriptors(project_symbol_catalog: &mut ProjectSymbolCatalog) -> Result<(), String> {
    let mut struct_layout_descriptors = project_symbol_catalog.get_struct_layout_descriptors().to_vec();

    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, pe_headers32_descriptor()?);
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, pe_headers64_descriptor()?);
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, image_dos_header_descriptor());
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, image_file_header_descriptor());
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, image_nt_headers32_descriptor());
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, image_nt_headers64_descriptor());
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, image_data_directory_descriptor());
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, image_optional_header32_descriptor());
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, image_optional_header64_descriptor());
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, image_section_header_descriptor());
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

fn upsert_pe_module_fields(
    project_symbol_catalog: &mut ProjectSymbolCatalog,
    module_name: &str,
    pe_header_layout: &PeHeaderLayout,
) -> Result<(), String> {
    let desired_module_fields = build_desired_pe_module_fields(pe_header_layout)?;
    let minimum_size = desired_module_fields
        .iter()
        .filter_map(|desired_module_field| {
            desired_module_field
                .offset
                .checked_add(desired_module_field.size_in_bytes)
        })
        .max()
        .unwrap_or(DOS_HEADER_SIZE_IN_BYTES);

    project_symbol_catalog.ensure_symbol_module(module_name, minimum_size);
    let Some(symbol_module) = project_symbol_catalog.find_symbol_module_mut(module_name) else {
        return Err(format!("Could not resolve module `{module_name}` after creating it."));
    };

    upsert_pe_module_fields_in_module(symbol_module, &desired_module_fields)
}

fn build_desired_pe_module_fields(pe_header_layout: &PeHeaderLayout) -> Result<Vec<DesiredModuleField>, String> {
    let nt_headers_size = IMAGE_NT_SIGNATURE_SIZE_IN_BYTES
        .checked_add(IMAGE_FILE_HEADER_SIZE_IN_BYTES)
        .and_then(|header_prefix_size| header_prefix_size.checked_add(pe_header_layout.size_of_optional_header))
        .ok_or_else(|| String::from("PE NT headers size is too large."))?;
    let section_headers_offset = pe_header_layout
        .pe_header_offset
        .checked_add(nt_headers_size)
        .ok_or_else(|| String::from("PE section headers offset is too large."))?;
    let section_headers_size = pe_header_layout
        .number_of_sections
        .checked_mul(IMAGE_SECTION_HEADER_SIZE_IN_BYTES)
        .ok_or_else(|| String::from("PE section headers size is too large."))?;
    let pe_headers_size = section_headers_offset
        .checked_add(section_headers_size)
        .ok_or_else(|| String::from("PE headers size is too large."))?;

    Ok(vec![DesiredModuleField {
        display_name: String::from("PE Headers"),
        offset: 0,
        struct_layout_id: pe_header_layout.pe_headers_struct_layout_id().to_string(),
        size_in_bytes: pe_headers_size,
    }])
}

fn upsert_pe_module_fields_in_module(
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
                .ok_or_else(|| format!("PE field `{}` range is too large.", desired_module_field.display_name))
        })
        .collect::<Result<Vec<_>, _>>()?;

    module_fields.retain(|module_field| {
        !desired_field_ranges
            .iter()
            .any(|desired_field_range| module_field_overlaps_desired_pe_range(module_field, *desired_field_range))
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

fn module_field_overlaps_desired_pe_range(
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

fn analyze_pe_header_layout(
    process_memory_store: &dyn ProcessMemoryStore,
    module_name: &str,
) -> Result<PeHeaderLayout, String> {
    let initial_header_bytes = process_memory_store.read_module_bytes(module_name, 0, INITIAL_PE_HEADER_READ_SIZE)?;
    let pe_header_offset = read_pe_header_offset(&initial_header_bytes)?;
    let minimum_required_size = pe_header_offset
        .checked_add(IMAGE_NT_SIGNATURE_SIZE_IN_BYTES)
        .and_then(|offset| offset.checked_add(IMAGE_FILE_HEADER_SIZE_IN_BYTES))
        .ok_or_else(|| String::from("PE header offset is too large."))?;
    let header_bytes = if minimum_required_size <= initial_header_bytes.len() as u64 {
        initial_header_bytes
    } else {
        let read_size = minimum_required_size
            .max(INITIAL_PE_HEADER_READ_SIZE)
            .min(MAX_PE_HEADER_READ_SIZE);

        process_memory_store.read_module_bytes(module_name, 0, read_size)?
    };

    read_pe_layout_from_header_bytes(&header_bytes, pe_header_offset)
}

fn read_pe_header_offset(header_bytes: &[u8]) -> Result<u64, String> {
    if header_bytes.len() < DOS_HEADER_SIZE_IN_BYTES as usize {
        return Err(String::from("Module header read is too small for an IMAGE_DOS_HEADER."));
    }

    if header_bytes.get(0..2) != Some(b"MZ") {
        return Err(String::from("Selected module does not start with the MZ DOS signature."));
    }

    let e_lfanew_bytes = header_bytes
        .get(0x3C..0x40)
        .ok_or_else(|| String::from("IMAGE_DOS_HEADER.e_lfanew is not readable."))?;
    let pe_header_offset = u32::from_le_bytes([
        e_lfanew_bytes[0],
        e_lfanew_bytes[1],
        e_lfanew_bytes[2],
        e_lfanew_bytes[3],
    ]) as u64;

    if pe_header_offset < DOS_HEADER_SIZE_IN_BYTES {
        return Err(format!("IMAGE_DOS_HEADER.e_lfanew points inside the DOS header: 0x{pe_header_offset:X}."));
    }

    if pe_header_offset > MAX_PE_HEADER_READ_SIZE {
        return Err(format!("IMAGE_DOS_HEADER.e_lfanew is unexpectedly large: 0x{pe_header_offset:X}."));
    }

    Ok(pe_header_offset)
}

fn read_pe_layout_from_header_bytes(
    header_bytes: &[u8],
    pe_header_offset: u64,
) -> Result<PeHeaderLayout, String> {
    let pe_header_position = pe_header_offset as usize;
    let pe_signature = header_bytes
        .get(pe_header_position..pe_header_position.saturating_add(4))
        .ok_or_else(|| String::from("PE signature is not readable."))?;

    if pe_signature != b"PE\0\0" {
        return Err(format!("IMAGE_DOS_HEADER.e_lfanew does not point at a PE signature: 0x{pe_header_offset:X}."));
    }

    let file_header_position = pe_header_position.saturating_add(IMAGE_NT_SIGNATURE_SIZE_IN_BYTES as usize);
    let file_header_bytes = header_bytes
        .get(file_header_position..file_header_position.saturating_add(IMAGE_FILE_HEADER_SIZE_IN_BYTES as usize))
        .ok_or_else(|| String::from("IMAGE_FILE_HEADER is not readable."))?;
    let number_of_sections = u16::from_le_bytes([file_header_bytes[2], file_header_bytes[3]]) as u64;
    let size_of_optional_header = u16::from_le_bytes([file_header_bytes[16], file_header_bytes[17]]) as u64;
    let optional_header_position = file_header_position.saturating_add(IMAGE_FILE_HEADER_SIZE_IN_BYTES as usize);
    let optional_header_magic_bytes = header_bytes
        .get(optional_header_position..optional_header_position.saturating_add(2))
        .ok_or_else(|| String::from("IMAGE_OPTIONAL_HEADER magic is not readable."))?;
    let optional_header_magic = u16::from_le_bytes([optional_header_magic_bytes[0], optional_header_magic_bytes[1]]);
    let optional_header_kind = match optional_header_magic {
        PE32_OPTIONAL_HEADER_MAGIC => PeOptionalHeaderKind::Pe32,
        PE64_OPTIONAL_HEADER_MAGIC => PeOptionalHeaderKind::Pe64,
        _ => return Err(format!("Unsupported PE optional header magic: 0x{optional_header_magic:X}.")),
    };

    Ok(PeHeaderLayout {
        pe_header_offset,
        size_of_optional_header,
        number_of_sections,
        optional_header_kind,
    })
}

impl PeHeaderLayout {
    fn pe_headers_struct_layout_id(&self) -> &'static str {
        match self.optional_header_kind {
            PeOptionalHeaderKind::Pe32 => PE_HEADERS32_ID,
            PeOptionalHeaderKind::Pe64 => PE_HEADERS64_ID,
        }
    }
}

fn image_dos_header_descriptor() -> StructLayoutDescriptor {
    struct_layout_descriptor(
        IMAGE_DOS_HEADER_ID,
        vec![
            field("e_magic", "u16"),
            field("e_cblp", "u16"),
            field("e_cp", "u16"),
            field("e_crlc", "u16"),
            field("e_cparhdr", "u16"),
            field("e_minalloc", "u16"),
            field("e_maxalloc", "u16"),
            field("e_ss", "u16"),
            field("e_sp", "u16"),
            field("e_csum", "u16"),
            field("e_ip", "u16"),
            field("e_cs", "u16"),
            field("e_lfarlc", "u16"),
            field("e_ovno", "u16"),
            array_field("e_res", "u16", 4),
            field("e_oemid", "u16"),
            field("e_oeminfo", "u16"),
            array_field("e_res2", "u16", 10),
            field("e_lfanew", "u32"),
        ],
    )
}

fn pe_headers32_descriptor() -> Result<StructLayoutDescriptor, String> {
    pe_headers_descriptor(PE_HEADERS32_ID, IMAGE_NT_HEADERS32_ID)
}

fn pe_headers64_descriptor() -> Result<StructLayoutDescriptor, String> {
    pe_headers_descriptor(PE_HEADERS64_ID, IMAGE_NT_HEADERS64_ID)
}

fn pe_headers_descriptor(
    struct_layout_id: &str,
    nt_headers_struct_layout_id: &str,
) -> Result<StructLayoutDescriptor, String> {
    Ok(struct_layout_descriptor(
        struct_layout_id,
        vec![
            expression_field(&format!("e_lfanew:u32 @ resolver({})", PE_RESOLVER_E_LFANEW_OFFSET_ID))?,
            expression_field(&format!("NumberOfSections:u16 @ resolver({})", PE_RESOLVER_NUMBER_OF_SECTIONS_OFFSET_ID))?,
            expression_field(&format!(
                "SizeOfOptionalHeader:u16 @ resolver({})",
                PE_RESOLVER_SIZE_OF_OPTIONAL_HEADER_OFFSET_ID
            ))?,
            expression_field(&format!("DOSHeader:{} @ resolver({})", IMAGE_DOS_HEADER_ID, PE_RESOLVER_DOS_HEADER_OFFSET_ID))?,
            expression_field(&format!(
                "DOSStub:u8[resolver({})] @ resolver({})",
                PE_RESOLVER_DOS_STUB_COUNT_ID, PE_RESOLVER_DOS_STUB_OFFSET_ID
            ))?,
            expression_field(&format!(
                "NTHeaders:{} @ resolver({})",
                nt_headers_struct_layout_id, PE_RESOLVER_NT_HEADERS_OFFSET_ID
            ))?,
            expression_field(&format!(
                "SectionHeaders:{}[resolver({})] @ resolver({})",
                IMAGE_SECTION_HEADER_ID, PE_RESOLVER_NUMBER_OF_SECTIONS_ID, PE_RESOLVER_SECTION_HEADERS_OFFSET_ID
            ))?,
        ],
    ))
}

fn image_file_header_descriptor() -> StructLayoutDescriptor {
    struct_layout_descriptor(
        IMAGE_FILE_HEADER_ID,
        vec![
            field("Machine", "u16"),
            field("NumberOfSections", "u16"),
            field("TimeDateStamp", "u32"),
            field("PointerToSymbolTable", "u32"),
            field("NumberOfSymbols", "u32"),
            field("SizeOfOptionalHeader", "u16"),
            field("Characteristics", "u16"),
        ],
    )
}

fn image_nt_headers32_descriptor() -> StructLayoutDescriptor {
    struct_layout_descriptor(
        IMAGE_NT_HEADERS32_ID,
        vec![
            field("Signature", "u32"),
            field("FileHeader", IMAGE_FILE_HEADER_ID),
            field("OptionalHeader", IMAGE_OPTIONAL_HEADER32_ID),
        ],
    )
}

fn image_nt_headers64_descriptor() -> StructLayoutDescriptor {
    struct_layout_descriptor(
        IMAGE_NT_HEADERS64_ID,
        vec![
            field("Signature", "u32"),
            field("FileHeader", IMAGE_FILE_HEADER_ID),
            field("OptionalHeader", IMAGE_OPTIONAL_HEADER64_ID),
        ],
    )
}

fn image_data_directory_descriptor() -> StructLayoutDescriptor {
    struct_layout_descriptor(IMAGE_DATA_DIRECTORY_ID, vec![field("VirtualAddress", "u32"), field("Size", "u32")])
}

fn image_optional_header32_descriptor() -> StructLayoutDescriptor {
    struct_layout_descriptor(
        IMAGE_OPTIONAL_HEADER32_ID,
        vec![
            field("Magic", "u16"),
            field("MajorLinkerVersion", "u8"),
            field("MinorLinkerVersion", "u8"),
            field("SizeOfCode", "u32"),
            field("SizeOfInitializedData", "u32"),
            field("SizeOfUninitializedData", "u32"),
            field("AddressOfEntryPoint", "u32"),
            field("BaseOfCode", "u32"),
            field("BaseOfData", "u32"),
            field("ImageBase", "u32"),
            field("SectionAlignment", "u32"),
            field("FileAlignment", "u32"),
            field("MajorOperatingSystemVersion", "u16"),
            field("MinorOperatingSystemVersion", "u16"),
            field("MajorImageVersion", "u16"),
            field("MinorImageVersion", "u16"),
            field("MajorSubsystemVersion", "u16"),
            field("MinorSubsystemVersion", "u16"),
            field("Win32VersionValue", "u32"),
            field("SizeOfImage", "u32"),
            field("SizeOfHeaders", "u32"),
            field("CheckSum", "u32"),
            field("Subsystem", "u16"),
            field("DllCharacteristics", "u16"),
            field("SizeOfStackReserve", "u32"),
            field("SizeOfStackCommit", "u32"),
            field("SizeOfHeapReserve", "u32"),
            field("SizeOfHeapCommit", "u32"),
            field("LoaderFlags", "u32"),
            field("NumberOfRvaAndSizes", "u32"),
            array_field("DataDirectory", IMAGE_DATA_DIRECTORY_ID, 16),
        ],
    )
}

fn image_optional_header64_descriptor() -> StructLayoutDescriptor {
    struct_layout_descriptor(
        IMAGE_OPTIONAL_HEADER64_ID,
        vec![
            field("Magic", "u16"),
            field("MajorLinkerVersion", "u8"),
            field("MinorLinkerVersion", "u8"),
            field("SizeOfCode", "u32"),
            field("SizeOfInitializedData", "u32"),
            field("SizeOfUninitializedData", "u32"),
            field("AddressOfEntryPoint", "u32"),
            field("BaseOfCode", "u32"),
            field("ImageBase", "u64"),
            field("SectionAlignment", "u32"),
            field("FileAlignment", "u32"),
            field("MajorOperatingSystemVersion", "u16"),
            field("MinorOperatingSystemVersion", "u16"),
            field("MajorImageVersion", "u16"),
            field("MinorImageVersion", "u16"),
            field("MajorSubsystemVersion", "u16"),
            field("MinorSubsystemVersion", "u16"),
            field("Win32VersionValue", "u32"),
            field("SizeOfImage", "u32"),
            field("SizeOfHeaders", "u32"),
            field("CheckSum", "u32"),
            field("Subsystem", "u16"),
            field("DllCharacteristics", "u16"),
            field("SizeOfStackReserve", "u64"),
            field("SizeOfStackCommit", "u64"),
            field("SizeOfHeapReserve", "u64"),
            field("SizeOfHeapCommit", "u64"),
            field("LoaderFlags", "u32"),
            field("NumberOfRvaAndSizes", "u32"),
            array_field("DataDirectory", IMAGE_DATA_DIRECTORY_ID, 16),
        ],
    )
}

fn image_section_header_descriptor() -> StructLayoutDescriptor {
    struct_layout_descriptor(
        IMAGE_SECTION_HEADER_ID,
        vec![
            array_field("Name", "u8", 8),
            field("VirtualSize", "u32"),
            field("VirtualAddress", "u32"),
            field("SizeOfRawData", "u32"),
            field("PointerToRawData", "u32"),
            field("PointerToRelocations", "u32"),
            field("PointerToLinenumbers", "u32"),
            field("NumberOfRelocations", "u16"),
            field("NumberOfLinenumbers", "u16"),
            field("Characteristics", "u32"),
        ],
    )
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

fn array_field(
    field_name: &str,
    data_type_id: &str,
    length: u64,
) -> SymbolicFieldDefinition {
    SymbolicFieldDefinition::new_named(field_name.to_string(), DataTypeRef::new(data_type_id), ContainerType::ArrayFixed(length))
}

fn expression_field(field_definition: &str) -> Result<SymbolicFieldDefinition, String> {
    SymbolicFieldDefinition::from_str(field_definition).map_err(|parse_error| format!("Invalid built-in PE symbolic field `{field_definition}`: {parse_error}"))
}

fn resolve_known_module_field_size(struct_layout_id: &str) -> Option<u64> {
    if let Some(u8_array_length) = resolve_u8_array_length(struct_layout_id) {
        return Some(u8_array_length);
    }

    if let Some(section_header_count) = resolve_fixed_array_length(struct_layout_id, IMAGE_SECTION_HEADER_ID) {
        return section_header_count.checked_mul(IMAGE_SECTION_HEADER_SIZE_IN_BYTES);
    }

    match struct_layout_id {
        IMAGE_DOS_HEADER_ID => Some(DOS_HEADER_SIZE_IN_BYTES),
        IMAGE_NT_HEADERS32_ID => Some(IMAGE_NT_SIGNATURE_SIZE_IN_BYTES + IMAGE_FILE_HEADER_SIZE_IN_BYTES + 224),
        IMAGE_NT_HEADERS64_ID => Some(IMAGE_NT_SIGNATURE_SIZE_IN_BYTES + IMAGE_FILE_HEADER_SIZE_IN_BYTES + 240),
        IMAGE_FILE_HEADER_ID => Some(IMAGE_FILE_HEADER_SIZE_IN_BYTES),
        IMAGE_OPTIONAL_HEADER32_ID => Some(224),
        IMAGE_OPTIONAL_HEADER64_ID => Some(240),
        IMAGE_SECTION_HEADER_ID => Some(IMAGE_SECTION_HEADER_SIZE_IN_BYTES),
        _ => None,
    }
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

#[cfg(test)]
mod tests {
    use super::{
        IMAGE_SECTION_HEADER_ID, PE_HEADERS32_ID, PE_RESOLVER_SECTION_HEADERS_OFFSET_ID, PeHeaderLayout, PeOptionalHeaderKind, PopulatePeSymbolsAction,
        populate_pe_symbols,
    };
    use squalr_engine_api::{
        plugins::symbol_tree::symbol_tree_action::{
            ProcessMemoryStore, ProjectSymbolStore, SymbolTreeAction, SymbolTreeActionContext, SymbolTreeActionSelection, SymbolTreeActionServices,
            SymbolTreeWindowStore,
        },
        structures::projects::{
            project_symbol_catalog::ProjectSymbolCatalog, project_symbol_module::ProjectSymbolModule, project_symbol_module_field::ProjectSymbolModuleField,
        },
    };
    use std::sync::{Arc, Mutex};

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
                header_bytes: build_test_pe_header_bytes(),
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
        symbol_tree_window_store: TestSymbolTreeWindowStore,
    }

    impl TestSymbolTreeActionServices {
        fn new(project_symbol_catalog: ProjectSymbolCatalog) -> Self {
            Self {
                project_symbol_store: TestProjectSymbolStore::new(project_symbol_catalog),
                process_memory_store: TestProcessMemoryStore::new(),
                symbol_tree_window_store: TestSymbolTreeWindowStore,
            }
        }
    }

    impl SymbolTreeActionServices for TestSymbolTreeActionServices {
        fn symbol_store(&self) -> &dyn ProjectSymbolStore {
            &self.project_symbol_store
        }

        fn process_memory(&self) -> &dyn ProcessMemoryStore {
            &self.process_memory_store
        }

        fn symbol_tree_window(&self) -> &dyn SymbolTreeWindowStore {
            &self.symbol_tree_window_store
        }
    }

    #[test]
    fn action_is_visible_only_for_module_roots() {
        let action = PopulatePeSymbolsAction;
        let module_context = SymbolTreeActionContext::new(SymbolTreeActionSelection::ModuleRoot {
            module_name: String::from("game.exe"),
        });
        let derived_context = SymbolTreeActionContext::new(SymbolTreeActionSelection::DerivedNode {
            tree_node_key: String::from("u8:game.exe:0:64"),
        });

        assert!(action.is_visible(&module_context));
        assert!(!action.is_visible(&derived_context));
    }

    #[test]
    fn populate_pe_symbols_adds_formulaic_pe_headers_root() {
        let project_symbol_catalog =
            ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![ProjectSymbolModule::new(String::from("game.exe"), 0x2000)], Vec::new(), Vec::new());
        let services = TestSymbolTreeActionServices::new(project_symbol_catalog);
        let action = PopulatePeSymbolsAction;
        let context = SymbolTreeActionContext::new(SymbolTreeActionSelection::ModuleRoot {
            module_name: String::from("game.exe"),
        });

        action
            .execute(&context, &services)
            .expect("Expected PE symbol population to succeed.");

        let project_symbol_catalog = services.project_symbol_store.read_current_catalog();
        let symbol_module = project_symbol_catalog
            .find_symbol_module("game.exe")
            .expect("Expected module to exist.");

        assert_eq!(symbol_module.get_fields().len(), 1);
        assert_eq!(symbol_module.get_fields()[0].get_display_name(), "PE Headers");
        assert_eq!(symbol_module.get_fields()[0].get_offset(), 0);
        assert_eq!(symbol_module.get_fields()[0].get_struct_layout_id(), PE_HEADERS32_ID);
        assert!(
            project_symbol_catalog
                .find_symbolic_resolver_descriptor(PE_RESOLVER_SECTION_HEADERS_OFFSET_ID)
                .is_some()
        );
        let pe_headers_descriptor = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == PE_HEADERS32_ID)
            .expect("Expected PE headers descriptor.");
        assert!(
            pe_headers_descriptor
                .get_struct_layout_definition()
                .get_fields()
                .iter()
                .any(|field_definition| field_definition.get_field_name() == "DOSStub")
        );
        assert!(
            pe_headers_descriptor
                .get_struct_layout_definition()
                .get_fields()
                .iter()
                .any(|field_definition| {
                    field_definition.get_field_name() == "SectionHeaders"
                        && field_definition
                            .to_string()
                            .contains(PE_RESOLVER_SECTION_HEADERS_OFFSET_ID)
                })
        );
        assert!(
            project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .any(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == IMAGE_SECTION_HEADER_ID)
        );
    }

    #[test]
    fn populate_pe_symbols_replaces_existing_root_u8_array() {
        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x2000);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("u8_00000000"), 0, String::from("u8[512]")));
        let mut project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![symbol_module], Vec::new(), Vec::new());
        let pe_header_layout = PeHeaderLayout {
            pe_header_offset: 0x80,
            size_of_optional_header: 0xE0,
            number_of_sections: 3,
            optional_header_kind: PeOptionalHeaderKind::Pe32,
        };

        populate_pe_symbols(&mut project_symbol_catalog, "game.exe", &pe_header_layout).expect("Expected PE symbol population to replace u8[] root field.");

        let fields = project_symbol_catalog
            .find_symbol_module("game.exe")
            .expect("Expected module to exist.")
            .get_fields();

        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].get_display_name(), "PE Headers");
        assert_eq!(fields[0].get_struct_layout_id(), PE_HEADERS32_ID);
    }

    #[test]
    fn populate_pe_symbols_stomps_existing_fields_in_pe_header_span() {
        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x2000);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("Bogus DOS"), 0, String::from("bogus_dos_header")));
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("Bogus NT"), 0x90, String::from("u32")));
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("Outside"), 0x400, String::from("u32")));
        let mut project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![symbol_module], Vec::new(), Vec::new());
        let pe_header_layout = PeHeaderLayout {
            pe_header_offset: 0x80,
            size_of_optional_header: 0xE0,
            number_of_sections: 3,
            optional_header_kind: PeOptionalHeaderKind::Pe32,
        };

        populate_pe_symbols(&mut project_symbol_catalog, "game.exe", &pe_header_layout).expect("Expected PE symbol population to stomp conflicting fields.");

        let fields = project_symbol_catalog
            .find_symbol_module("game.exe")
            .expect("Expected module to exist.")
            .get_fields();

        assert_eq!(fields.len(), 2);
        assert!(
            fields
                .iter()
                .all(|field| field.get_display_name() != "Bogus DOS")
        );
        assert!(
            fields
                .iter()
                .all(|field| field.get_display_name() != "Bogus NT")
        );
        assert!(fields.iter().any(|field| field.get_display_name() == "Outside"));
        assert_eq!(fields[0].get_struct_layout_id(), PE_HEADERS32_ID);
        assert_eq!(fields[1].get_display_name(), "Outside");
    }

    fn build_test_pe_header_bytes() -> Vec<u8> {
        let mut header_bytes = vec![0_u8; 0x1000];
        header_bytes[0..2].copy_from_slice(b"MZ");
        header_bytes[0x3C..0x40].copy_from_slice(&0x80_u32.to_le_bytes());
        header_bytes[0x80..0x84].copy_from_slice(b"PE\0\0");
        header_bytes[0x86..0x88].copy_from_slice(&3_u16.to_le_bytes());
        header_bytes[0x94..0x96].copy_from_slice(&0xE0_u16.to_le_bytes());
        header_bytes[0x98..0x9A].copy_from_slice(&0x10B_u16.to_le_bytes());

        header_bytes
    }
}
