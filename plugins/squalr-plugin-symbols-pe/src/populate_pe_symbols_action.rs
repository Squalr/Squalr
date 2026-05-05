use squalr_engine_api::{
    plugins::{
        PluginPermission,
        symbol_tree::symbol_tree_action::{SymbolTreeAction, SymbolTreeActionContext, SymbolTreeActionSelection, SymbolTreeActionServices},
    },
    registries::symbols::struct_layout_descriptor::StructLayoutDescriptor,
    structures::{
        data_types::data_type_ref::DataTypeRef,
        data_values::container_type::ContainerType,
        projects::{
            project_symbol_catalog::ProjectSymbolCatalog, project_symbol_module::ProjectSymbolModule, project_symbol_module_field::ProjectSymbolModuleField,
        },
        structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition},
    },
};

const IMAGE_DOS_HEADER_ID: &str = "win.pe.IMAGE_DOS_HEADER";
const IMAGE_FILE_HEADER_ID: &str = "win.pe.IMAGE_FILE_HEADER";
const IMAGE_DATA_DIRECTORY_ID: &str = "win.pe.IMAGE_DATA_DIRECTORY";
const IMAGE_OPTIONAL_HEADER64_ID: &str = "win.pe.IMAGE_OPTIONAL_HEADER64";
const DOS_HEADER_SIZE_IN_BYTES: u64 = 64;

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

        services.symbol_store().write_catalog(
            "populate PE symbols",
            Box::new(move |project_symbol_catalog| populate_pe_symbols(project_symbol_catalog, &module_name_for_update)),
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
) -> Result<(), String> {
    upsert_pe_struct_layout_descriptors(project_symbol_catalog);
    upsert_dos_header_module_field(project_symbol_catalog, module_name)
}

fn upsert_pe_struct_layout_descriptors(project_symbol_catalog: &mut ProjectSymbolCatalog) {
    let mut struct_layout_descriptors = project_symbol_catalog.get_struct_layout_descriptors().to_vec();

    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, image_dos_header_descriptor());
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, image_file_header_descriptor());
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, image_data_directory_descriptor());
    upsert_struct_layout_descriptor(&mut struct_layout_descriptors, image_optional_header64_descriptor());
    project_symbol_catalog.set_struct_layout_descriptors(struct_layout_descriptors);
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

fn upsert_dos_header_module_field(
    project_symbol_catalog: &mut ProjectSymbolCatalog,
    module_name: &str,
) -> Result<(), String> {
    project_symbol_catalog.ensure_symbol_module(module_name, DOS_HEADER_SIZE_IN_BYTES);
    let Some(symbol_module) = project_symbol_catalog.find_symbol_module_mut(module_name) else {
        return Err(format!("Could not resolve module `{module_name}` after creating it."));
    };

    upsert_dos_header_module_field_in_module(symbol_module)
}

fn upsert_dos_header_module_field_in_module(symbol_module: &mut ProjectSymbolModule) -> Result<(), String> {
    let module_fields = symbol_module.get_fields_mut();

    if let Some(existing_field_position) = module_fields
        .iter()
        .position(|module_field| module_field.get_offset() == 0)
    {
        let existing_struct_layout_id = module_fields[existing_field_position]
            .get_struct_layout_id()
            .to_string();

        if existing_struct_layout_id == IMAGE_DOS_HEADER_ID {
            module_fields[existing_field_position].set_display_name(String::from("DOS Header"));
            return Ok(());
        }

        if let Some(existing_u8_array_length) = resolve_u8_array_length(&existing_struct_layout_id) {
            if existing_u8_array_length < DOS_HEADER_SIZE_IN_BYTES {
                return Err(String::from("The module starts with a u8[] field smaller than an IMAGE_DOS_HEADER."));
            }

            module_fields.remove(existing_field_position);
            module_fields.push(ProjectSymbolModuleField::new(String::from("DOS Header"), 0, IMAGE_DOS_HEADER_ID.to_string()));

            if existing_u8_array_length > DOS_HEADER_SIZE_IN_BYTES {
                module_fields.push(ProjectSymbolModuleField::new(
                    format!("u8_{:08X}", DOS_HEADER_SIZE_IN_BYTES),
                    DOS_HEADER_SIZE_IN_BYTES,
                    format!("u8[{}]", existing_u8_array_length.saturating_sub(DOS_HEADER_SIZE_IN_BYTES)),
                ));
            }

            sort_module_fields(module_fields);
            return Ok(());
        }

        return Err(format!(
            "The module already has `{}` at offset 0.",
            module_fields[existing_field_position].get_display_name()
        ));
    }

    if module_fields
        .iter()
        .any(|module_field| module_field.get_offset() < DOS_HEADER_SIZE_IN_BYTES)
    {
        return Err(String::from("The first 64 bytes of this module already contain symbol fields."));
    }

    module_fields.push(ProjectSymbolModuleField::new(String::from("DOS Header"), 0, IMAGE_DOS_HEADER_ID.to_string()));
    sort_module_fields(module_fields);

    Ok(())
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

fn image_data_directory_descriptor() -> StructLayoutDescriptor {
    struct_layout_descriptor(IMAGE_DATA_DIRECTORY_ID, vec![field("VirtualAddress", "u32"), field("Size", "u32")])
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

fn resolve_u8_array_length(struct_layout_id: &str) -> Option<u64> {
    let length_text = struct_layout_id.strip_prefix("u8[")?.strip_suffix(']')?;

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
    use super::{IMAGE_DOS_HEADER_ID, PopulatePeSymbolsAction, populate_pe_symbols};
    use squalr_engine_api::{
        plugins::symbol_tree::symbol_tree_action::{
            ProjectSymbolStore, SymbolTreeAction, SymbolTreeActionContext, SymbolTreeActionSelection, SymbolTreeActionServices, SymbolTreeWindowStore,
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
        symbol_tree_window_store: TestSymbolTreeWindowStore,
    }

    impl TestSymbolTreeActionServices {
        fn new(project_symbol_catalog: ProjectSymbolCatalog) -> Self {
            Self {
                project_symbol_store: TestProjectSymbolStore::new(project_symbol_catalog),
                symbol_tree_window_store: TestSymbolTreeWindowStore,
            }
        }
    }

    impl SymbolTreeActionServices for TestSymbolTreeActionServices {
        fn symbol_store(&self) -> &dyn ProjectSymbolStore {
            &self.project_symbol_store
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
    fn populate_pe_symbols_adds_dos_header_and_struct_descriptors() {
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
        assert_eq!(symbol_module.get_fields()[0].get_display_name(), "DOS Header");
        assert_eq!(symbol_module.get_fields()[0].get_struct_layout_id(), IMAGE_DOS_HEADER_ID);
        assert!(
            project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .any(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == IMAGE_DOS_HEADER_ID)
        );
    }

    #[test]
    fn populate_pe_symbols_splits_existing_root_u8_array() {
        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x2000);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("u8_00000000"), 0, String::from("u8[128]")));
        let mut project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![symbol_module], Vec::new(), Vec::new());

        populate_pe_symbols(&mut project_symbol_catalog, "game.exe").expect("Expected PE symbol population to split u8[] root field.");

        let fields = project_symbol_catalog
            .find_symbol_module("game.exe")
            .expect("Expected module to exist.")
            .get_fields();

        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].get_struct_layout_id(), IMAGE_DOS_HEADER_ID);
        assert_eq!(fields[1].get_offset(), 64);
        assert_eq!(fields[1].get_struct_layout_id(), "u8[64]");
    }
}
