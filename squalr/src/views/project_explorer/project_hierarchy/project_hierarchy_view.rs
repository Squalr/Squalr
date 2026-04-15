use crate::{
    app_context::AppContext,
    ui::{
        converters::data_type_to_icon_converter::DataTypeToIconConverter,
        widgets::controls::{context_menu::context_menu::ContextMenu, toolbar_menu::toolbar_menu_item_view::ToolbarMenuItemView},
    },
    views::code_viewer::{code_viewer_view::CodeViewerView, view_data::code_viewer_view_data::CodeViewerViewData},
    views::memory_viewer::{memory_viewer_view::MemoryViewerView, view_data::memory_viewer_view_data::MemoryViewerViewData},
    views::pointer_scanner::{pointer_scanner_view::PointerScannerView, view_data::pointer_scanner_view_data::PointerScannerViewData},
    views::project_explorer::project_hierarchy::{
        project_hierarchy_toolbar_view::ProjectHierarchyToolbarView,
        project_item_entry_view::ProjectItemEntryView,
        project_item_inline_rename_view::ProjectItemInlineRenameView,
        project_item_value_edit_take_over_view::ProjectItemValueEditTakeOverView,
        view_data::{
            project_hierarchy_create_item_kind::ProjectHierarchyCreateItemKind, project_hierarchy_drop_target::ProjectHierarchyDropTarget,
            project_hierarchy_frame_action::ProjectHierarchyFrameAction, project_hierarchy_menu_target::ProjectHierarchyMenuTarget,
            project_hierarchy_pending_operation::ProjectHierarchyPendingOperation, project_hierarchy_take_over_state::ProjectHierarchyTakeOverState,
            project_hierarchy_tree_entry::ProjectHierarchyTreeEntry, project_hierarchy_view_data::ProjectHierarchyViewData,
        },
    },
    views::struct_viewer::view_data::{struct_viewer_focus_target::StructViewerFocusTarget, struct_viewer_view_data::StructViewerViewData},
};
use eframe::egui::{Align, CursorIcon, Id, Key, Layout, Pos2, Rect, Response, RichText, ScrollArea, TextureHandle, Ui, Widget, vec2};
use epaint::{CornerRadius, Stroke, StrokeKind};
use squalr_engine_api::commands::memory::query::memory_query_request::MemoryQueryRequest;
use squalr_engine_api::commands::memory::query::memory_query_response::MemoryQueryResponse;
use squalr_engine_api::commands::memory::read::memory_read_request::MemoryReadRequest;
use squalr_engine_api::commands::memory::read::memory_read_response::MemoryReadResponse;
use squalr_engine_api::commands::memory::write::memory_write_request::MemoryWriteRequest;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use squalr_engine_api::commands::project::save::project_save_request::ProjectSaveRequest;
use squalr_engine_api::commands::project_items::convert_symbol_ref::project_items_convert_symbol_ref_request::ProjectItemSymbolRefConversionTarget;
use squalr_engine_api::commands::project_items::rename::project_items_rename_request::ProjectItemsRenameRequest;
use squalr_engine_api::commands::settings::scan::list::scan_settings_list_request::ScanSettingsListRequest;
use squalr_engine_api::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::plugins::instruction_set::normalize_instruction_data_type_id;
use squalr_engine_api::structures::data_types::built_in_types::{string::utf8::data_type_string_utf8::DataTypeStringUtf8, u64::data_type_u64::DataTypeU64};
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_values::{anonymous_value_string::AnonymousValueString, container_type::ContainerType};
use squalr_engine_api::structures::memory::address_display::try_resolve_virtual_module_address;
use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
use squalr_engine_api::structures::memory::pointer::Pointer;
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_api::structures::projects::project_info::ProjectInfo;
use squalr_engine_api::structures::projects::project_items::built_in_types::{
    project_item_type_address::ProjectItemTypeAddress, project_item_type_directory::ProjectItemTypeDirectory,
    project_item_type_pointer::ProjectItemTypePointer, project_item_type_symbol_ref::ProjectItemTypeSymbolRef,
};
use squalr_engine_api::structures::projects::project_items::{project_item::ProjectItem, project_item_ref::ProjectItemRef};
use squalr_engine_api::structures::projects::project_root_symbol::ProjectRootSymbol;
use squalr_engine_api::structures::projects::project_root_symbol_locator::ProjectRootSymbolLocator;
use squalr_engine_api::structures::structs::valued_struct_field::{ValuedStructField, ValuedStructFieldData};
use squalr_engine_api::structures::structs::{
    symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition, valued_struct::ValuedStruct,
};
use squalr_engine_session::{
    engine_unprivileged_state::EngineUnprivilegedState,
    virtual_snapshots::{virtual_snapshot_query::VirtualSnapshotQuery, virtual_snapshot_query_result::VirtualSnapshotQueryResult},
};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct ProjectHierarchyView {
    app_context: Arc<AppContext>,
    project_hierarchy_toolbar_view: ProjectHierarchyToolbarView,
    project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
    code_viewer_view_data: Dependency<CodeViewerViewData>,
    memory_viewer_view_data: Dependency<MemoryViewerViewData>,
    pointer_scanner_view_data: Dependency<PointerScannerViewData>,
    struct_viewer_view_data: Dependency<StructViewerViewData>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PointerScannerContextAction {
    Address {
        label: &'static str,
        address: u64,
        module_name: String,
        data_type_id: String,
    },
    ResolvedPointer {
        label: &'static str,
        pointer: Pointer,
        data_type_id: String,
    },
}

impl PointerScannerContextAction {
    fn label(&self) -> &'static str {
        match self {
            Self::Address { label, .. } | Self::ResolvedPointer { label, .. } => label,
        }
    }
}

#[derive(Clone)]
struct ProjectItemValueEditContext {
    project_item_name: String,
    value_field_name: String,
    validation_data_type_ref: DataTypeRef,
    initial_value_edit: AnonymousValueString,
}

impl ProjectHierarchyView {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let project_hierarchy_view_data = app_context
            .dependency_container
            .get_dependency::<ProjectHierarchyViewData>();
        let project_hierarchy_toolbar_view = ProjectHierarchyToolbarView::new(app_context.clone());
        let struct_viewer_view_data = app_context
            .dependency_container
            .get_dependency::<StructViewerViewData>();
        let memory_viewer_view_data = app_context
            .dependency_container
            .get_dependency::<MemoryViewerViewData>();
        let code_viewer_view_data = app_context
            .dependency_container
            .get_dependency::<CodeViewerViewData>();
        let pointer_scanner_view_data = app_context
            .dependency_container
            .get_dependency::<PointerScannerViewData>();
        ProjectHierarchyViewData::refresh_project_items(project_hierarchy_view_data.clone(), app_context.clone());

        Self {
            app_context,
            project_hierarchy_toolbar_view,
            project_hierarchy_view_data,
            code_viewer_view_data,
            memory_viewer_view_data,
            pointer_scanner_view_data,
            struct_viewer_view_data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectHierarchyView;
    use crate::{
        app_context::AppContext,
        models::docking::{docking_manager::DockingManager, hierarchy::dock_node::DockNode},
        ui::theme::Theme,
    };
    use crossbeam_channel::{Receiver, unbounded};
    use eframe::egui::Context;
    use squalr_engine_api::commands::memory::memory_command::MemoryCommand;
    use squalr_engine_api::commands::memory::read::memory_read_request::MemoryReadRequest;
    use squalr_engine_api::commands::memory::read::memory_read_response::MemoryReadResponse;
    use squalr_engine_api::commands::privileged_command::PrivilegedCommand;
    use squalr_engine_api::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
    use squalr_engine_api::commands::unprivileged_command::UnprivilegedCommand;
    use squalr_engine_api::commands::unprivileged_command_response::UnprivilegedCommandResponse;
    use squalr_engine_api::engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings;
    use squalr_engine_api::engine::engine_binding_error::EngineBindingError;
    use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
    use squalr_engine_api::structures::data_types::built_in_types::{string::utf8::data_type_string_utf8::DataTypeStringUtf8, u64::data_type_u64::DataTypeU64};
    use squalr_engine_api::structures::data_types::built_in_types::{
        u16::data_type_u16::DataTypeU16, u32::data_type_u32::DataTypeU32, u64::data_type_u64::DataTypeU64 as DataTypeU64Pointer,
    };
    use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
    use squalr_engine_api::structures::data_values::{container_type::ContainerType, data_value::DataValue};
    use squalr_engine_api::structures::memory::{normalized_module::NormalizedModule, pointer::Pointer};
    use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
    use squalr_engine_api::structures::projects::project_items::built_in_types::{
        project_item_type_address::ProjectItemTypeAddress, project_item_type_directory::ProjectItemTypeDirectory,
        project_item_type_pointer::ProjectItemTypePointer,
    };
    use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
    use squalr_engine_api::structures::structs::valued_struct::ValuedStruct;
    use squalr_engine_api::structures::structs::valued_struct_field::{ValuedStructField, ValuedStructFieldData};
    use squalr_engine_session::engine_unprivileged_state::{EngineUnprivilegedState, EngineUnprivilegedStateOptions};
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex, RwLock};

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct CapturedMemoryReadRequest {
        address: u64,
        module_name: String,
    }

    struct MockMemoryReadBindings {
        captured_memory_read_requests: Arc<Mutex<Vec<CapturedMemoryReadRequest>>>,
        memory_read_response_factory: Arc<dyn Fn(&MemoryReadRequest) -> MemoryReadResponse + Send + Sync>,
    }

    impl MockMemoryReadBindings {
        fn new(memory_read_response_factory: impl Fn(&MemoryReadRequest) -> MemoryReadResponse + Send + Sync + 'static) -> Self {
            Self {
                captured_memory_read_requests: Arc::new(Mutex::new(Vec::new())),
                memory_read_response_factory: Arc::new(memory_read_response_factory),
            }
        }
    }

    impl EngineApiUnprivilegedBindings for MockMemoryReadBindings {
        fn dispatch_privileged_command(
            &self,
            engine_command: PrivilegedCommand,
            callback: Box<dyn FnOnce(PrivilegedCommandResponse) + Send + Sync + 'static>,
        ) -> Result<(), EngineBindingError> {
            let PrivilegedCommand::Memory(MemoryCommand::Read { memory_read_request }) = engine_command else {
                return Err(EngineBindingError::unavailable("dispatching project hierarchy pointer memory reads in tests"));
            };
            let mut captured_memory_read_requests = self
                .captured_memory_read_requests
                .lock()
                .map_err(|error| EngineBindingError::lock_failure("capturing project hierarchy pointer memory reads in tests", error.to_string()))?;

            captured_memory_read_requests.push(CapturedMemoryReadRequest {
                address: memory_read_request.address,
                module_name: memory_read_request.module_name.clone(),
            });
            drop(captured_memory_read_requests);

            callback((self.memory_read_response_factory)(&memory_read_request).to_engine_response());

            Ok(())
        }

        fn dispatch_unprivileged_command(
            &self,
            _engine_command: UnprivilegedCommand,
            _engine_execution_context: &Arc<dyn EngineExecutionContext>,
            _callback: Box<dyn FnOnce(UnprivilegedCommandResponse) + Send + Sync + 'static>,
        ) -> Result<(), EngineBindingError> {
            Err(EngineBindingError::unavailable(
                "dispatching unprivileged commands in project hierarchy pointer tests",
            ))
        }

        fn subscribe_to_engine_events(&self) -> Result<Receiver<squalr_engine_api::engine::engine_event_envelope::EngineEventEnvelope>, EngineBindingError> {
            let (_event_sender, event_receiver) = unbounded();

            Ok(event_receiver)
        }
    }

    fn create_pointer_memory_read_response(
        pointer_value: u64,
        pointer_size: PointerScanPointerSize,
        success: bool,
    ) -> MemoryReadResponse {
        fn create_three_byte_pointer_value(
            pointer_value: u32,
            data_type_id: &str,
            is_big_endian: bool,
        ) -> squalr_engine_api::structures::data_values::data_value::DataValue {
            let value_bytes = if is_big_endian {
                vec![
                    (pointer_value >> 16) as u8,
                    (pointer_value >> 8) as u8,
                    pointer_value as u8,
                ]
            } else {
                vec![
                    pointer_value as u8,
                    (pointer_value >> 8) as u8,
                    (pointer_value >> 16) as u8,
                ]
            };

            squalr_engine_api::structures::data_values::data_value::DataValue::new(
                squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef::new(data_type_id),
                value_bytes,
            )
        }

        let valued_struct = if success {
            let value_field = match pointer_size {
                PointerScanPointerSize::Pointer24 => {
                    create_three_byte_pointer_value(pointer_value as u32, "u24", false).to_named_valued_struct_field("value".to_string(), true)
                }
                PointerScanPointerSize::Pointer24be => {
                    create_three_byte_pointer_value(pointer_value as u32, "u24be", true).to_named_valued_struct_field("value".to_string(), true)
                }
                PointerScanPointerSize::Pointer32 => {
                    DataTypeU32::get_value_from_primitive(pointer_value as u32).to_named_valued_struct_field("value".to_string(), true)
                }
                PointerScanPointerSize::Pointer32be => {
                    squalr_engine_api::structures::data_types::built_in_types::u32be::data_type_u32be::DataTypeU32be::get_value_from_primitive(
                        pointer_value as u32,
                    )
                    .to_named_valued_struct_field("value".to_string(), true)
                }
                PointerScanPointerSize::Pointer64 => {
                    DataTypeU64Pointer::get_value_from_primitive(pointer_value).to_named_valued_struct_field("value".to_string(), true)
                }
                PointerScanPointerSize::Pointer64be => {
                    squalr_engine_api::structures::data_types::built_in_types::u64be::data_type_u64be::DataTypeU64be::get_value_from_primitive(pointer_value)
                        .to_named_valued_struct_field("value".to_string(), true)
                }
            };

            ValuedStruct::new_anonymous(vec![value_field])
        } else {
            ValuedStruct::default()
        };

        MemoryReadResponse {
            valued_struct,
            address: pointer_value,
            success,
        }
    }

    fn create_value_memory_read_response(
        data_value: squalr_engine_api::structures::data_values::data_value::DataValue,
        success: bool,
    ) -> MemoryReadResponse {
        let valued_struct = if success {
            ValuedStruct::new_anonymous(vec![data_value.to_named_valued_struct_field("value".to_string(), true)])
        } else {
            ValuedStruct::default()
        };

        MemoryReadResponse {
            valued_struct,
            address: 0,
            success,
        }
    }

    fn create_execution_context(mock_memory_read_bindings: MockMemoryReadBindings) -> Arc<dyn EngineExecutionContext> {
        let engine_bindings: Arc<RwLock<dyn EngineApiUnprivilegedBindings>> = Arc::new(RwLock::new(mock_memory_read_bindings));

        EngineUnprivilegedState::new_with_options(engine_bindings, EngineUnprivilegedStateOptions { enable_console_logging: false })
    }

    fn create_engine_unprivileged_state(mock_memory_read_bindings: MockMemoryReadBindings) -> Arc<EngineUnprivilegedState> {
        let engine_bindings: Arc<RwLock<dyn EngineApiUnprivilegedBindings>> = Arc::new(RwLock::new(mock_memory_read_bindings));

        EngineUnprivilegedState::new_with_options(engine_bindings, EngineUnprivilegedStateOptions { enable_console_logging: false })
    }

    fn create_test_engine_unprivileged_state() -> Arc<EngineUnprivilegedState> {
        create_engine_unprivileged_state(MockMemoryReadBindings::new(|_memory_read_request| {
            create_pointer_memory_read_response(0, PointerScanPointerSize::Pointer64, false)
        }))
    }

    fn create_test_app_context() -> Arc<AppContext> {
        let egui_context = Context::default();
        let theme = Arc::new(Theme::new(&egui_context));
        let docking_manager = Arc::new(RwLock::new(DockingManager::new(DockNode::Window {
            window_identifier: "test_window".to_string(),
            is_visible: true,
        })));
        let engine_unprivileged_state = create_test_engine_unprivileged_state();

        Arc::new(AppContext::new(egui_context, theme, docking_manager, engine_unprivileged_state))
    }

    #[test]
    fn resolve_module_relative_address_adds_matching_module_base_address() {
        let modules = vec![
            NormalizedModule::new("Other.exe", 0x1000, 0x2000),
            NormalizedModule::new("Torchlight2.exe", 0x400000, 0x100000),
        ];

        let resolved_absolute_address = ProjectHierarchyView::resolve_module_relative_address(&modules, 0x30, "Torchlight2.exe");

        assert_eq!(resolved_absolute_address, Some(0x400030));
    }

    #[test]
    fn resolve_module_relative_address_matches_module_name_case_insensitively() {
        let modules = vec![NormalizedModule::new("Torchlight2.exe", 0x400000, 0x100000)];

        let resolved_absolute_address = ProjectHierarchyView::resolve_module_relative_address(&modules, 0x30, "torchlight2.exe");

        assert_eq!(resolved_absolute_address, Some(0x400030));
    }

    #[test]
    fn build_memory_write_request_for_address_item_address_edit_returns_request() {
        let mut project_item = ProjectItemTypeAddress::new_project_item("player_health", 0x1234, "game.exe", "", DataTypeU64::get_value_from_primitive(0));
        let expected_module_name = ProjectItemTypeAddress::get_field_module(&mut project_item);
        let edited_field = ValuedStructField::new(
            ProjectItemTypeAddress::PROPERTY_ADDRESS.to_string(),
            ValuedStructFieldData::Value(DataTypeU64::get_value_from_primitive(0xABCD)),
            false,
        );

        let memory_write_request = ProjectHierarchyView::build_memory_write_request_for_project_item_edit(&mut project_item, &edited_field);

        assert!(memory_write_request.is_some());
        let memory_write_request = memory_write_request.unwrap_or_else(|| panic!("Expected memory write request for address edit."));
        assert_eq!(memory_write_request.address, 0x1234);
        assert_eq!(memory_write_request.module_name, expected_module_name);
        assert_eq!(memory_write_request.value, 0xABCDu64.to_le_bytes().to_vec());
    }

    #[test]
    fn build_memory_write_request_for_address_item_non_address_edit_returns_none() {
        let mut project_item = ProjectItemTypeAddress::new_project_item("player_health", 0x1234, "game.exe", "", DataTypeU64::get_value_from_primitive(0));
        let edited_field = ValuedStructField::new(
            ProjectItemTypeAddress::PROPERTY_MODULE.to_string(),
            ValuedStructFieldData::Value(DataTypeStringUtf8::get_value_from_primitive_string("new_module.exe")),
            false,
        );

        let memory_write_request = ProjectHierarchyView::build_memory_write_request_for_project_item_edit(&mut project_item, &edited_field);

        assert!(memory_write_request.is_none());
    }

    #[test]
    fn build_memory_write_request_for_non_address_item_address_edit_returns_none() {
        let project_item_ref = ProjectItemRef::new(PathBuf::from("project/folder"));
        let mut project_item = ProjectItemTypeDirectory::new_project_item(&project_item_ref);
        let edited_field = ValuedStructField::new(
            ProjectItemTypeAddress::PROPERTY_ADDRESS.to_string(),
            ValuedStructFieldData::Value(DataTypeU64::get_value_from_primitive(0xABCD)),
            false,
        );

        let memory_write_request = ProjectHierarchyView::build_memory_write_request_for_project_item_edit(&mut project_item, &edited_field);

        assert!(memory_write_request.is_none());
    }

    #[test]
    fn build_project_item_rename_request_for_directory_uses_edited_name_without_extension() {
        let project_item_path = Path::new("C:/Projects/TestProject/project_items/Folder");
        let rename_request =
            ProjectHierarchyView::build_project_item_rename_request(project_item_path, ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID, "Renamed Folder");

        assert!(rename_request.is_some());
        let rename_request = rename_request.unwrap_or_else(|| panic!("Expected rename request for directory item."));
        assert_eq!(rename_request.project_item_name, "Renamed Folder".to_string());
    }

    #[test]
    fn build_project_item_rename_request_for_file_appends_json_extension() {
        let project_item_path = Path::new("C:/Projects/TestProject/project_items/health.json");
        let rename_request =
            ProjectHierarchyView::build_project_item_rename_request(project_item_path, ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID, "player_health");

        assert!(rename_request.is_some());
        let rename_request = rename_request.unwrap_or_else(|| panic!("Expected rename request for file item."));
        assert_eq!(rename_request.project_item_name, "player_health.json".to_string());
    }

    #[test]
    fn build_project_item_rename_request_returns_none_when_name_is_unchanged() {
        let project_item_path = Path::new("C:/Projects/TestProject/project_items/health.json");
        let rename_request =
            ProjectHierarchyView::build_project_item_rename_request(project_item_path, ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID, "health.json");

        assert!(rename_request.is_none());
    }

    #[test]
    fn should_apply_struct_field_edit_to_project_item_returns_false_for_directory_name_edits() {
        let should_apply_struct_field_edit = ProjectHierarchyView::should_apply_struct_field_edit_to_project_item(
            ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID,
            squalr_engine_api::structures::projects::project_items::project_item::ProjectItem::PROPERTY_NAME,
        );

        assert!(!should_apply_struct_field_edit);
    }

    #[test]
    fn should_apply_struct_field_edit_to_project_item_returns_true_for_file_name_edits() {
        let should_apply_struct_field_edit = ProjectHierarchyView::should_apply_struct_field_edit_to_project_item(
            ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID,
            squalr_engine_api::structures::projects::project_items::project_item::ProjectItem::PROPERTY_NAME,
        );

        assert!(should_apply_struct_field_edit);
    }

    #[test]
    fn should_apply_struct_field_edit_to_project_item_returns_true_for_non_name_edits() {
        let should_apply_struct_field_edit = ProjectHierarchyView::should_apply_struct_field_edit_to_project_item(
            ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID,
            ProjectItemTypeAddress::PROPERTY_MODULE,
        );

        assert!(should_apply_struct_field_edit);
    }

    #[test]
    fn project_item_menu_width_respects_minimum_width_for_short_labels() {
        assert_eq!(
            ProjectHierarchyView::project_item_menu_width_from_longest_label_width(0.0),
            ProjectHierarchyView::MIN_PROJECT_ITEM_MENU_WIDTH,
        );
    }

    #[test]
    fn project_item_menu_width_grows_for_wider_labels() {
        let longest_label_width = 200.0;

        assert_eq!(
            ProjectHierarchyView::project_item_menu_width_from_longest_label_width(longest_label_width),
            236.0,
        );
    }

    #[test]
    fn build_pointer_scanner_context_actions_returns_address_item_values() {
        let project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU64::get_value_from_primitive(0));

        let pointer_scanner_context_actions = ProjectHierarchyView::build_pointer_scanner_context_actions(None, &project_item);

        assert_eq!(
            pointer_scanner_context_actions,
            vec![super::PointerScannerContextAction::Address {
                label: "Open in Pointer Scan",
                address: 0x1234,
                module_name: "game.exe".to_string(),
                data_type_id: "u64".to_string(),
            }]
        );
    }

    #[test]
    fn build_pointer_scanner_context_actions_returns_pointer_item_entries() {
        let pointer = Pointer::new_with_size(0x1000, vec![0x20, -0x10], "game.exe".to_string(), PointerScanPointerSize::Pointer64);
        let project_item = ProjectItemTypePointer::new_project_item("Ammo Pointer", &pointer, "", "u16");

        let pointer_scanner_context_actions = ProjectHierarchyView::build_pointer_scanner_context_actions(None, &project_item);

        assert_eq!(
            pointer_scanner_context_actions,
            vec![
                super::PointerScannerContextAction::Address {
                    label: "Open Base Address in Pointer Scan",
                    address: 0x1000,
                    module_name: "game.exe".to_string(),
                    data_type_id: "u16".to_string(),
                },
                super::PointerScannerContextAction::ResolvedPointer {
                    label: "Open Resolved Address in Pointer Scan",
                    pointer,
                    data_type_id: "u16".to_string(),
                }
            ]
        );
    }

    #[test]
    fn build_pointer_scanner_context_actions_ignores_non_address_items() {
        let project_item_ref = ProjectItemRef::new(PathBuf::from("project/folder"));
        let project_item = ProjectItemTypeDirectory::new_project_item(&project_item_ref);

        let pointer_scanner_context_actions = ProjectHierarchyView::build_pointer_scanner_context_actions(None, &project_item);

        assert!(pointer_scanner_context_actions.is_empty());
    }

    #[test]
    fn resolve_pointer_scanner_context_action_resolves_pointer_target() {
        let engine_execution_context = create_execution_context(MockMemoryReadBindings::new(|memory_read_request| {
            match (memory_read_request.address, memory_read_request.module_name.as_str()) {
                (0x1000, "game.exe") => create_pointer_memory_read_response(0x2000, PointerScanPointerSize::Pointer64, true),
                (0x2020, "") => create_pointer_memory_read_response(0x3000, PointerScanPointerSize::Pointer64, true),
                unexpected_request => panic!("Unexpected pointer dereference request: {unexpected_request:?}"),
            }
        }));
        let pointer = Pointer::new_with_size(0x1000, vec![0x20, -0x10], "game.exe".to_string(), PointerScanPointerSize::Pointer64);

        let resolved_pointer_action = super::PointerScannerContextAction::ResolvedPointer {
            label: "Open Resolved Address in Pointer Scan",
            pointer,
            data_type_id: "u16".to_string(),
        };

        let resolved_pointer_target = ProjectHierarchyView::resolve_pointer_scanner_context_action(&engine_execution_context, &resolved_pointer_action);

        assert_eq!(resolved_pointer_target, Some((0x2FF0, String::new(), "u16".to_string())));
    }

    #[test]
    fn should_open_project_item_in_code_viewer_returns_true_for_instruction_data_type() {
        let project_item =
            ProjectItemTypeAddress::new_project_item("Patch", 0x1234, "game.exe", "", DataValue::new(DataTypeRef::new("i_x86[2]"), vec![0x90, 0x90]));

        assert!(ProjectHierarchyView::should_open_project_item_in_code_viewer(None, &project_item));
    }

    #[test]
    fn should_open_project_item_in_code_viewer_returns_false_for_plain_numeric_data_type() {
        let project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU64::get_value_from_primitive(0));

        assert!(!ProjectHierarchyView::should_open_project_item_in_code_viewer(None, &project_item));
    }

    #[test]
    fn resolve_tree_entry_icon_uses_address_item_symbolic_data_type_for_arrays() {
        let app_context = create_test_app_context();
        let project_item = ProjectItemTypeAddress::new_project_item("Ammo", 0x1234, "game.exe", "", DataTypeU64::get_value_from_primitive(0));
        let mut project_item = project_item;
        ProjectItemTypeAddress::set_field_symbolic_struct_definition_reference(&mut project_item, "u8[190]");

        let icon = ProjectHierarchyView::resolve_tree_entry_icon(app_context.clone(), None, &project_item)
            .unwrap_or_else(|| panic!("Expected address project item icon."));

        assert_eq!(
            icon.id(),
            app_context
                .theme
                .icon_library
                .icon_handle_data_type_purple_blocks_1
                .id()
        );
    }

    #[test]
    fn resolve_tree_entry_icon_uses_pointer_item_symbolic_data_type_for_arrays() {
        let app_context = create_test_app_context();
        let pointer = Pointer::new(0x1000, vec![0x20], "game.exe".to_string());
        let project_item = ProjectItemTypePointer::new_project_item("Ammo Pointer", &pointer, "", "u8[190]");

        let icon = ProjectHierarchyView::resolve_tree_entry_icon(app_context.clone(), None, &project_item)
            .unwrap_or_else(|| panic!("Expected pointer project item icon."));

        assert_eq!(
            icon.id(),
            app_context
                .theme
                .icon_library
                .icon_handle_data_type_purple_blocks_1
                .id()
        );
    }

    #[test]
    fn build_struct_view_properties_exposes_runtime_value_field_as_editable() {
        let project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU64::get_value_from_primitive(0));

        let struct_view_properties = ProjectHierarchyView::build_struct_view_properties(None, &project_item);
        let runtime_value_field = struct_view_properties
            .get_field(ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE)
            .expect("Expected runtime value field in struct view properties.");

        assert!(!runtime_value_field.get_is_read_only());
    }

    #[test]
    fn build_memory_write_request_for_runtime_value_edit_uses_address_target() {
        let engine_execution_context = create_execution_context(MockMemoryReadBindings::new(|_memory_read_request| {
            panic!("Did not expect pointer dereference for address project item runtime value edit.")
        }));
        let address_project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU16::get_value_from_primitive(0));
        let edited_field = ValuedStructField::new(
            ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE.to_string(),
            ValuedStructFieldData::Value(DataTypeU16::get_value_from_primitive(0xBEEF)),
            false,
        );

        let memory_write_request =
            ProjectHierarchyView::build_memory_write_request_for_runtime_value_edit(&engine_execution_context, None, &address_project_item, &edited_field);

        assert!(memory_write_request.is_some());
        let memory_write_request = memory_write_request.unwrap_or_else(|| panic!("Expected runtime value memory write request for address project item."));
        assert_eq!(memory_write_request.address, 0x1234);
        assert_eq!(memory_write_request.module_name, "game.exe");
        assert_eq!(memory_write_request.value, 0xBEEFu16.to_le_bytes().to_vec());
    }

    #[test]
    fn build_memory_write_request_for_runtime_value_edit_resolves_pointer_target() {
        let engine_execution_context = create_execution_context(MockMemoryReadBindings::new(|memory_read_request| {
            match (memory_read_request.address, memory_read_request.module_name.as_str()) {
                (0x1000, "game.exe") => create_pointer_memory_read_response(0x2000, PointerScanPointerSize::Pointer64, true),
                (0x2020, "") => create_pointer_memory_read_response(0x3000, PointerScanPointerSize::Pointer64, true),
                unexpected_request => panic!("Unexpected pointer dereference request: {unexpected_request:?}"),
            }
        }));
        let pointer = Pointer::new_with_size(0x1000, vec![0x20, -0x10], "game.exe".to_string(), PointerScanPointerSize::Pointer64);
        let pointer_project_item = ProjectItemTypePointer::new_project_item("Ammo Pointer", &pointer, "", "u16");
        let edited_field = ValuedStructField::new(
            ProjectItemTypePointer::PROPERTY_FREEZE_DISPLAY_VALUE.to_string(),
            ValuedStructFieldData::Value(DataTypeU16::get_value_from_primitive(0x1234)),
            false,
        );

        let memory_write_request =
            ProjectHierarchyView::build_memory_write_request_for_runtime_value_edit(&engine_execution_context, None, &pointer_project_item, &edited_field);

        assert!(memory_write_request.is_some());
        let memory_write_request = memory_write_request.unwrap_or_else(|| panic!("Expected runtime value memory write request for pointer project item."));
        assert_eq!(memory_write_request.address, 0x2FF0);
        assert_eq!(memory_write_request.module_name, "");
        assert_eq!(memory_write_request.value, 0x1234u16.to_le_bytes().to_vec());
    }

    #[test]
    fn build_project_item_value_edit_context_for_address_reads_live_value_for_editing() {
        let engine_unprivileged_state = create_engine_unprivileged_state(MockMemoryReadBindings::new(|memory_read_request| {
            assert_eq!(memory_read_request.address, 0x1234);
            assert_eq!(memory_read_request.module_name, "game.exe");

            create_value_memory_read_response(DataTypeU16::get_value_from_primitive(0xBEEF), true)
        }));
        let mut project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU16::get_value_from_primitive(0));
        ProjectItemTypeAddress::set_field_freeze_data_value_interpreter(&mut project_item, "4660");

        let value_edit_context = ProjectHierarchyView::build_project_item_value_edit_context(&engine_unprivileged_state, None, &project_item)
            .expect("Expected value edit context for address project item.");

        assert_eq!(value_edit_context.project_item_name, "Health");
        assert_eq!(value_edit_context.value_field_name, ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE);
        assert_eq!(
            value_edit_context.validation_data_type_ref,
            squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef::new(DataTypeU16::DATA_TYPE_ID)
        );
        assert_eq!(
            value_edit_context
                .initial_value_edit
                .get_anonymous_value_string(),
            "48879"
        );
    }

    #[test]
    fn build_project_item_value_edit_context_falls_back_to_preview_when_live_read_fails() {
        let engine_unprivileged_state = create_test_engine_unprivileged_state();
        let mut project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU16::get_value_from_primitive(0));
        ProjectItemTypeAddress::set_field_freeze_data_value_interpreter(&mut project_item, "4660");

        let value_edit_context = ProjectHierarchyView::build_project_item_value_edit_context(&engine_unprivileged_state, None, &project_item)
            .expect("Expected value edit context fallback for address project item.");

        assert_eq!(
            value_edit_context
                .initial_value_edit
                .get_anonymous_value_string(),
            "4660"
        );
    }

    #[test]
    fn build_project_item_value_edit_context_preserves_array_container_type() {
        let engine_unprivileged_state = create_test_engine_unprivileged_state();
        let pointer = Pointer::new(0x1000, vec![0x20], "game.exe".to_string());
        let mut project_item = ProjectItemTypePointer::new_project_item("Ammo", &pointer, "", "u16[2]");
        ProjectItemTypePointer::set_field_freeze_data_value_interpreter(&mut project_item, "1, 2");

        let value_edit_context = ProjectHierarchyView::build_project_item_value_edit_context(&engine_unprivileged_state, None, &project_item)
            .expect("Expected value edit context for pointer project item.");

        assert_eq!(value_edit_context.initial_value_edit.get_container_type(), ContainerType::ArrayFixed(2));
    }

    #[test]
    fn build_project_item_value_edit_context_returns_none_for_directory() {
        let engine_unprivileged_state = create_test_engine_unprivileged_state();
        let project_item_ref = ProjectItemRef::new(PathBuf::from("project/folder"));
        let project_item = ProjectItemTypeDirectory::new_project_item(&project_item_ref);

        let value_edit_context = ProjectHierarchyView::build_project_item_value_edit_context(&engine_unprivileged_state, None, &project_item);

        assert!(value_edit_context.is_none());
    }

    #[test]
    fn should_apply_struct_field_edit_to_project_item_ignores_runtime_value_field() {
        assert!(!ProjectHierarchyView::should_apply_struct_field_edit_to_project_item(
            ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID,
            ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE,
        ));
    }
}

impl Widget for ProjectHierarchyView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        self.sync_scan_settings_if_needed();
        let project_read_interval = self.get_project_read_interval();
        user_interface
            .ctx()
            .request_repaint_after(project_read_interval);

        self.refresh_if_project_changed();
        self.refresh_if_project_preview_values_stale(project_read_interval);

        let project_hierarchy_toolbar_view = self.project_hierarchy_toolbar_view.clone();
        let mut project_hierarchy_frame_action = ProjectHierarchyFrameAction::None;
        let mut drag_started_project_item_path: Option<PathBuf> = None;
        let mut hovered_drop_target_project_item_path: Option<ProjectHierarchyDropTarget> = None;
        let mut should_cancel_take_over = false;
        let mut delete_confirmation_project_item_paths: Option<Vec<std::path::PathBuf>> = None;
        let mut promote_symbol_overwrite_project_item_paths: Option<Vec<std::path::PathBuf>> = None;
        let mut rename_project_item_submission: Option<(PathBuf, String, String)> = None;
        let mut value_edit_project_item_submission: Option<(PathBuf, String, DataTypeRef, AnonymousValueString)> = None;
        let mut keyboard_activation_toggle_target: Option<(Vec<PathBuf>, bool)> = None;
        let mut is_delete_confirmation_active = false;
        let mut is_promote_symbol_conflict_active = false;
        let mut is_rename_take_over_active = false;
        let mut is_value_edit_take_over_active = false;
        let mut visible_preview_project_item_paths = Vec::new();
        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                let shared_struct_viewer_focus_target = self
                    .struct_viewer_view_data
                    .read("Project hierarchy shared struct viewer focus target")
                    .and_then(|struct_viewer_view_data| struct_viewer_view_data.get_focus_target().cloned());
                let active_struct_viewer_project_item_paths: HashSet<PathBuf> = match shared_struct_viewer_focus_target.as_ref() {
                    Some(StructViewerFocusTarget::ProjectHierarchy { project_item_paths }) => {
                        project_item_paths.iter().cloned().collect()
                    }
                    _ => HashSet::new(),
                };

                let project_hierarchy_view_data = match self.project_hierarchy_view_data.read("Project hierarchy view") {
                    Some(project_hierarchy_view_data) => project_hierarchy_view_data,
                    None => return,
                };
                let take_over_state = project_hierarchy_view_data.take_over_state.clone();
                let tree_entries = project_hierarchy_view_data.tree_entries.clone();
                let selected_project_item_paths = project_hierarchy_view_data.selected_project_item_paths.clone();
                let dragged_project_item_paths = project_hierarchy_view_data.dragged_project_item_paths.clone();
                let menu_target = project_hierarchy_view_data.menu_target.clone();
                let menu_position = project_hierarchy_view_data.menu_position;
                let selected_project_item_paths_in_tree_order = project_hierarchy_view_data.collect_selected_project_item_paths_in_tree_order();
                let pending_operation = project_hierarchy_view_data.pending_operation.clone();

                user_interface.add(project_hierarchy_toolbar_view);
                self.show_toolbar_add_menu(
                    &mut project_hierarchy_frame_action,
                    user_interface,
                    menu_target.as_ref(),
                    menu_position,
                );

                match pending_operation {
                    ProjectHierarchyPendingOperation::ConvertingSymbolRefs => {
                        user_interface.label("Converting symbol refs...");
                    }
                    ProjectHierarchyPendingOperation::Deleting => {
                        user_interface.label("Deleting project item(s)...");
                    }
                    ProjectHierarchyPendingOperation::Promoting => {
                        user_interface.label("Promoting project item(s) to symbols...");
                    }
                    ProjectHierarchyPendingOperation::Reordering => {
                        user_interface.label("Reordering project item(s)...");
                    }
                    _ => {}
                }

                let active_inline_rename = match &take_over_state {
                    ProjectHierarchyTakeOverState::RenameProjectItem {
                        project_item_path,
                        project_item_type_id,
                    } => Some((project_item_path.clone(), project_item_type_id.clone())),
                    _ => None,
                };
                is_rename_take_over_active = active_inline_rename.is_some();
                let active_value_edit_project_item_path = match &take_over_state {
                    ProjectHierarchyTakeOverState::EditProjectItemValue { project_item_path } => Some(project_item_path.clone()),
                    _ => None,
                };
                is_value_edit_take_over_active = active_value_edit_project_item_path.is_some();

                match take_over_state {
                    ProjectHierarchyTakeOverState::None | ProjectHierarchyTakeOverState::RenameProjectItem { .. } => {
                        ScrollArea::vertical()
                            .id_salt("project_hierarchy")
                            .auto_shrink([false, false])
                            .show_rows(user_interface, Self::PROJECT_ITEM_ROW_HEIGHT, tree_entries.len(), |user_interface, visible_row_range| {
                                visible_preview_project_item_paths.extend(
                                    tree_entries[visible_row_range.clone()]
                                        .iter()
                                        .map(|tree_entry| tree_entry.project_item_path.clone()),
                                );

                                for tree_entry in &tree_entries[visible_row_range] {
                                    let is_selected = selected_project_item_paths.contains(&tree_entry.project_item_path)
                                        && active_struct_viewer_project_item_paths.contains(&tree_entry.project_item_path);
                                    let icon = Self::resolve_tree_entry_icon(
                                        self.app_context.clone(),
                                        project_hierarchy_view_data.opened_project_info.as_ref(),
                                        &tree_entry.project_item,
                                    );

                                    let is_inline_rename_row = active_inline_rename
                                        .as_ref()
                                        .map(|(project_item_path, _)| project_item_path == &tree_entry.project_item_path)
                                        .unwrap_or(false);
                                    let (row_response, should_request_rename, should_request_value_edit) = if is_inline_rename_row {
                                        let (_, project_item_type_id) = active_inline_rename
                                            .as_ref()
                                            .unwrap_or_else(|| panic!("Expected inline rename state for rename row."));
                                        let rename_text_storage_id =
                                            Self::project_item_rename_text_storage_id(&tree_entry.project_item_path);
                                        let rename_highlight_storage_id =
                                            Self::project_item_rename_highlight_storage_id(&tree_entry.project_item_path);
                                        let mut rename_text = user_interface
                                            .ctx()
                                            .data_mut(|data| data.get_temp::<String>(rename_text_storage_id))
                                            .unwrap_or_else(|| tree_entry.display_name.clone());
                                        let mut should_highlight_text = user_interface
                                            .ctx()
                                            .data_mut(|data| data.get_temp::<bool>(rename_highlight_storage_id))
                                            .unwrap_or(true);
                                        let inline_rename_response = ProjectItemInlineRenameView::new(
                                            self.app_context.clone(),
                                            &tree_entry.project_item_path,
                                            &mut rename_text,
                                            &mut should_highlight_text,
                                            tree_entry.is_activated,
                                            tree_entry.depth,
                                            icon,
                                            is_selected,
                                            tree_entry.is_directory,
                                            tree_entry.has_children,
                                            tree_entry.is_expanded,
                                        )
                                        .show(user_interface);

                                        if inline_rename_response.should_commit {
                                            rename_project_item_submission = Some((
                                                tree_entry.project_item_path.clone(),
                                                project_item_type_id.clone(),
                                                rename_text.clone(),
                                            ));
                                        }

                                        if inline_rename_response.should_cancel {
                                            should_cancel_take_over = true;
                                        }

                                        user_interface.ctx().data_mut(|data| {
                                            data.insert_temp(rename_text_storage_id, rename_text);
                                            data.insert_temp(rename_highlight_storage_id, should_highlight_text);
                                        });

                                        (inline_rename_response.row_response, false, false)
                                    } else {
                                        let project_item_entry_view_response = ProjectItemEntryView::new(
                                            self.app_context.clone(),
                                            &tree_entry.project_item_path,
                                            &tree_entry.display_name,
                                            &tree_entry.preview_path,
                                            &tree_entry.preview_value,
                                            tree_entry.is_activated,
                                            tree_entry.depth,
                                            icon,
                                            is_selected,
                                            ProjectHierarchyViewData::is_cut_project_item_path(
                                                self.project_hierarchy_view_data.clone(),
                                                &tree_entry.project_item_path,
                                            ),
                                            tree_entry.is_directory,
                                            tree_entry.has_children,
                                            tree_entry.is_expanded,
                                            &mut project_hierarchy_frame_action,
                                        )
                                        .show(user_interface);

                                        (
                                            project_item_entry_view_response.row_response,
                                            project_item_entry_view_response.should_request_rename,
                                            project_item_entry_view_response.should_request_value_edit,
                                        )
                                    };

                                    if is_rename_take_over_active || is_value_edit_take_over_active {
                                        continue;
                                    }

                                    if should_request_rename {
                                        project_hierarchy_frame_action =
                                            ProjectHierarchyFrameAction::RequestRename(tree_entry.project_item_path.clone());
                                    } else if should_request_value_edit {
                                        project_hierarchy_frame_action =
                                            ProjectHierarchyFrameAction::RequestValueEdit(tree_entry.project_item_path.clone());
                                    }

                                    if row_response.drag_started() {
                                        drag_started_project_item_path = Some(tree_entry.project_item_path.clone());
                                    }

                                    let tree_entry_project_item_path = tree_entry.project_item_path.clone();
                                    let pointer_scanner_context_actions =
                                        Self::build_pointer_scanner_context_actions(project_hierarchy_view_data.opened_project_info.as_ref(), &tree_entry.project_item);
                                    let can_open_in_memory_viewer =
                                        Self::can_open_project_item_in_memory_viewer(project_hierarchy_view_data.opened_project_info.as_ref(), &tree_entry.project_item);
                                    let is_context_menu_visible =
                                        matches!(menu_target.as_ref(), Some(ProjectHierarchyMenuTarget::ProjectItem(menu_project_item_path)) if menu_project_item_path == &tree_entry.project_item_path);
                                    let default_context_menu_position = row_response.rect.left_bottom();

                                    if row_response.secondary_clicked() {
                                        ProjectHierarchyViewData::show_project_item_menu(
                                            self.project_hierarchy_view_data.clone(),
                                            tree_entry.project_item_path.clone(),
                                            row_response
                                                .hover_pos()
                                                .unwrap_or(default_context_menu_position),
                                        );
                                    }

                                    if is_context_menu_visible {
                                        let mut open = true;
                                        let project_item_paths_for_delete = if selected_project_item_paths_in_tree_order.contains(&tree_entry_project_item_path)
                                            && selected_project_item_paths_in_tree_order.len() > 1
                                        {
                                            selected_project_item_paths_in_tree_order.clone()
                                        } else {
                                            vec![tree_entry_project_item_path.clone()]
                                        };
                                        let can_delete_project_item_paths = ProjectHierarchyViewData::has_deletable_project_item_paths(
                                            self.project_hierarchy_view_data.clone(),
                                            &project_item_paths_for_delete,
                                        );
                                        let can_copy_project_item_paths = can_delete_project_item_paths;
                                        let can_cut_project_item_paths = can_delete_project_item_paths;
                                        let can_promote_project_item_paths = ProjectHierarchyViewData::has_promotable_project_item_paths(
                                            self.project_hierarchy_view_data.clone(),
                                            &project_item_paths_for_delete,
                                        );
                                        let can_paste_project_items = ProjectHierarchyViewData::can_paste_project_item_clipboard(
                                            self.project_hierarchy_view_data.clone(),
                                            &tree_entry_project_item_path,
                                        );
                                        let can_convert_project_item_paths =
                                            ProjectHierarchyViewData::has_convertible_symbol_ref_project_item_paths(
                                                self.project_hierarchy_view_data.clone(),
                                                &project_item_paths_for_delete,
                                            );
                                        let convert_project_item_menu_label =
                                            ProjectHierarchyViewData::get_convertible_symbol_ref_action_label(
                                                self.project_hierarchy_view_data.clone(),
                                                &project_item_paths_for_delete,
                                            )
                                            .unwrap_or_else(|| "Convert to Source Item Type".to_string());
                                        let runtime_viewer_label = if Self::should_open_project_item_in_code_viewer(
                                            project_hierarchy_view_data.opened_project_info.as_ref(),
                                            &tree_entry.project_item,
                                        ) {
                                            "Open in Code Viewer"
                                        } else {
                                            "Open in Memory Viewer"
                                        };
                                        let mut project_item_menu_labels =
                                            pointer_scanner_context_actions.iter().map(PointerScannerContextAction::label).collect::<Vec<_>>();
                                        let has_runtime_actions = !pointer_scanner_context_actions.is_empty()
                                            || can_open_in_memory_viewer
                                            || can_promote_project_item_paths
                                            || can_convert_project_item_paths;
                                        let has_create_actions = true;
                                        let has_clipboard_actions =
                                            can_cut_project_item_paths || can_copy_project_item_paths || can_paste_project_items;
                                        let has_delete_actions = can_delete_project_item_paths;
                                        if can_open_in_memory_viewer {
                                            project_item_menu_labels.push(runtime_viewer_label);
                                        }
                                        if can_promote_project_item_paths {
                                            project_item_menu_labels.push("Promote to Symbol");
                                        }
                                        if can_convert_project_item_paths {
                                            project_item_menu_labels.push(convert_project_item_menu_label.as_str());
                                        }
                                        if has_create_actions {
                                            project_item_menu_labels.extend(["New Folder", "New Address", "New Pointer", "New Symbol Ref"]);
                                        }
                                        if can_cut_project_item_paths {
                                            project_item_menu_labels.push("Cut");
                                        }
                                        if can_copy_project_item_paths {
                                            project_item_menu_labels.push("Copy");
                                        }
                                        if can_paste_project_items {
                                            project_item_menu_labels.push("Paste");
                                        }
                                        if has_delete_actions {
                                            project_item_menu_labels.push("Delete");
                                        }
                                        let project_item_menu_width = Self::calculate_project_item_menu_width(
                                            self.app_context.as_ref(),
                                            user_interface,
                                            &project_item_menu_labels,
                                        );
                                        ContextMenu::new(
                                            self.app_context.clone(),
                                            "project_hierarchy_context_menu",
                                            menu_position.unwrap_or(default_context_menu_position),
                                            |user_interface, should_close| {
                                                if !pointer_scanner_context_actions.is_empty() {
                                                    let engine_execution_context: Arc<dyn EngineExecutionContext> =
                                                        self.app_context.engine_unprivileged_state.clone();

                                                    for pointer_scanner_context_action in pointer_scanner_context_actions.clone() {
                                                        if user_interface
                                                            .add(ToolbarMenuItemView::new(
                                                                self.app_context.clone(),
                                                                pointer_scanner_context_action.label(),
                                                                pointer_scanner_context_action.label(),
                                                                &None,
                                                                project_item_menu_width,
                                                            ))
                                                            .clicked()
                                                        {
                                                            if let Some((address, module_name, data_type_id)) = Self::resolve_pointer_scanner_context_action(
                                                                &engine_execution_context,
                                                                &pointer_scanner_context_action,
                                                            ) {
                                                                project_hierarchy_frame_action = ProjectHierarchyFrameAction::OpenPointerScannerForAddress {
                                                                    address,
                                                                    module_name,
                                                                    data_type_id,
                                                                };
                                                                *should_close = true;
                                                            } else {
                                                                log::error!(
                                                                    "Failed to resolve pointer scan target for project item context action: {}.",
                                                                    pointer_scanner_context_action.label()
                                                                );
                                                            }
                                                        }
                                                    }
                                                }

                                                if can_open_in_memory_viewer {
                                                    if user_interface
                                                        .add(
                                                            ToolbarMenuItemView::new(
                                                                self.app_context.clone(),
                                                                runtime_viewer_label,
                                                                "project_hierarchy_ctx_open_runtime_viewer",
                                                                &None,
                                                                project_item_menu_width,
                                                            )
                                                            .icon(
                                                                if Self::should_open_project_item_in_code_viewer(
                                                                    project_hierarchy_view_data.opened_project_info.as_ref(),
                                                                    &tree_entry.project_item,
                                                                ) {
                                                                    self.app_context.theme.icon_library.icon_handle_project_cpu_instruction.clone()
                                                                } else {
                                                                    self.app_context.theme.icon_library.icon_handle_scan_collect_values.clone()
                                                                },
                                                            ),
                                                        )
                                                        .clicked()
                                                    {
                                                        let engine_execution_context: Arc<dyn EngineExecutionContext> =
                                                            self.app_context.engine_unprivileged_state.clone();

                                                        if let Some((address, module_name)) =
                                                            Self::resolve_project_item_runtime_value_target(
                                                                &engine_execution_context,
                                                                project_hierarchy_view_data.opened_project_info.as_ref(),
                                                                &tree_entry.project_item,
                                                            )
                                                        {
                                                            project_hierarchy_frame_action = if Self::should_open_project_item_in_code_viewer(
                                                                project_hierarchy_view_data.opened_project_info.as_ref(),
                                                                &tree_entry.project_item,
                                                            ) {
                                                                ProjectHierarchyFrameAction::OpenCodeViewerForAddress { address, module_name }
                                                            } else {
                                                                ProjectHierarchyFrameAction::OpenMemoryViewerForAddress {
                                                                    address,
                                                                    module_name,
                                                                    selection_byte_count: Self::resolve_project_item_runtime_value_byte_count(
                                                                        &self.app_context.engine_unprivileged_state,
                                                                        project_hierarchy_view_data.opened_project_info.as_ref(),
                                                                        &tree_entry.project_item,
                                                                    )
                                                                    .unwrap_or(1),
                                                                }
                                                            };
                                                            *should_close = true;
                                                        } else {
                                                            log::error!(
                                                                "Failed to resolve memory viewer target for project item: {:?}.",
                                                                tree_entry_project_item_path
                                                            );
                                                        }
                                                    }
                                                }

                                                if can_promote_project_item_paths {
                                                    if user_interface
                                                        .add(ToolbarMenuItemView::new(
                                                            self.app_context.clone(),
                                                            "Promote to Symbol",
                                                            "project_hierarchy_ctx_promote_to_symbol",
                                                            &None,
                                                            project_item_menu_width,
                                                        ))
                                                        .clicked()
                                                    {
                                                        project_hierarchy_frame_action = ProjectHierarchyFrameAction::PromoteToSymbol {
                                                            project_item_paths: project_item_paths_for_delete.clone(),
                                                            overwrite_conflicting_symbols: false,
                                                        };
                                                        *should_close = true;
                                                    }
                                                }

                                                if can_convert_project_item_paths {
                                                    if user_interface
                                                        .add(ToolbarMenuItemView::new(
                                                            self.app_context.clone(),
                                                            convert_project_item_menu_label.as_str(),
                                                            "project_hierarchy_ctx_convert_to_address_item",
                                                            &None,
                                                            project_item_menu_width,
                                                        ))
                                                        .clicked()
                                                    {
                                                        project_hierarchy_frame_action = ProjectHierarchyFrameAction::ConvertSymbolRef {
                                                            project_item_paths: project_item_paths_for_delete.clone(),
                                                            conversion_target: ProjectItemSymbolRefConversionTarget::Inferred,
                                                        };
                                                        *should_close = true;
                                                    }
                                                }

                                                if has_runtime_actions && has_create_actions {
                                                    user_interface.separator();
                                                }

                                                Self::show_create_project_item_menu_items(
                                                    self.app_context.clone(),
                                                    user_interface,
                                                    &tree_entry_project_item_path,
                                                    &mut project_hierarchy_frame_action,
                                                    project_item_menu_width,
                                                    should_close,
                                                );

                                                if (has_runtime_actions || has_create_actions) && has_clipboard_actions {
                                                    user_interface.separator();
                                                }

                                                if can_cut_project_item_paths {
                                                    if user_interface
                                                        .add(ToolbarMenuItemView::new(
                                                            self.app_context.clone(),
                                                            "Cut",
                                                            "project_hierarchy_ctx_cut",
                                                            &None,
                                                            project_item_menu_width,
                                                        )
                                                        .icon(self.app_context.theme.icon_library.icon_handle_data_type_unknown.clone()))
                                                        .clicked()
                                                    {
                                                        project_hierarchy_frame_action =
                                                            ProjectHierarchyFrameAction::CutProjectItems(project_item_paths_for_delete.clone());
                                                        *should_close = true;
                                                    }
                                                }

                                                if can_copy_project_item_paths {
                                                    if user_interface
                                                        .add(ToolbarMenuItemView::new(
                                                            self.app_context.clone(),
                                                            "Copy",
                                                            "project_hierarchy_ctx_copy",
                                                            &None,
                                                            project_item_menu_width,
                                                        )
                                                        .icon(self.app_context.theme.icon_library.icon_handle_data_type_unknown.clone()))
                                                        .clicked()
                                                    {
                                                        project_hierarchy_frame_action =
                                                            ProjectHierarchyFrameAction::CopyProjectItems(project_item_paths_for_delete.clone());
                                                        *should_close = true;
                                                    }
                                                }

                                                if can_paste_project_items {
                                                    if user_interface
                                                        .add(
                                                            ToolbarMenuItemView::new(
                                                                self.app_context.clone(),
                                                                "Paste",
                                                                "project_hierarchy_ctx_paste",
                                                                &None,
                                                                project_item_menu_width,
                                                            )
                                                            .icon(self.app_context.theme.icon_library.icon_handle_data_type_unknown.clone()),
                                                        )
                                                        .clicked()
                                                    {
                                                        project_hierarchy_frame_action = ProjectHierarchyFrameAction::PasteProjectItems {
                                                            target_project_item_path: tree_entry_project_item_path.clone(),
                                                        };
                                                        *should_close = true;
                                                    }
                                                }

                                                if (has_runtime_actions || has_create_actions || has_clipboard_actions) && has_delete_actions {
                                                    user_interface.separator();
                                                }

                                                if has_delete_actions {
                                                    if user_interface
                                                        .add(ToolbarMenuItemView::new(
                                                            self.app_context.clone(),
                                                            "Delete",
                                                            "project_hierarchy_ctx_delete",
                                                            &None,
                                                            project_item_menu_width,
                                                        )
                                                        .icon(self.app_context.theme.icon_library.icon_handle_common_delete.clone()))
                                                        .clicked()
                                                    {
                                                        project_hierarchy_frame_action =
                                                            ProjectHierarchyFrameAction::RequestDeleteConfirmation(project_item_paths_for_delete.clone());
                                                        *should_close = true;
                                                    }
                                                }
                                            },
                                        )
                                        .width(project_item_menu_width)
                                        .corner_radius(8)
                                        .show(user_interface, &mut open);

                                        if !open {
                                            ProjectHierarchyViewData::hide_menu(self.project_hierarchy_view_data.clone());
                                        }
                                    }

                                    let active_dragged_project_item_paths = drag_started_project_item_path
                                        .as_ref()
                                        .map(|drag_started_project_item_path| vec![drag_started_project_item_path.clone()])
                                        .or(dragged_project_item_paths.clone());

                                    if let Some(active_dragged_project_item_paths) = active_dragged_project_item_paths {
                                        if let Some(pointer_position) = user_interface.input(|input_state| input_state.pointer.hover_pos()) {
                                            if row_response.rect.contains(pointer_position) {
                                                if let Some(hovered_drop_target) = Self::resolve_drop_target(
                                                    &active_dragged_project_item_paths,
                                                    tree_entry,
                                                    row_response.rect,
                                                    pointer_position,
                                                ) {
                                                    hovered_drop_target_project_item_path = Some(hovered_drop_target.clone());
                                                    self.paint_drop_target_indicator(user_interface, row_response.rect, &hovered_drop_target);
                                                }
                                            }
                                        }
                                    }
                                }
                            });
                    }
                    ProjectHierarchyTakeOverState::EditProjectItemValue { project_item_path } => {
                        is_value_edit_take_over_active = true;

                        let project_item = tree_entries
                            .iter()
                            .find(|tree_entry| tree_entry.project_item_path == project_item_path)
                            .map(|tree_entry| tree_entry.project_item.clone());
                        let Some(project_item) = project_item else {
                            should_cancel_take_over = true;
                            return;
                        };
                        let Some(value_edit_context) = Self::build_project_item_value_edit_context(
                            &self.app_context.engine_unprivileged_state,
                            project_hierarchy_view_data.opened_project_info.as_ref(),
                            &project_item,
                        )
                        else {
                            should_cancel_take_over = true;
                            return;
                        };
                        let value_edit_storage_id = Self::project_item_value_edit_storage_id(&project_item_path);
                        let mut value_edit = user_interface
                            .ctx()
                            .data_mut(|data| data.get_temp::<AnonymousValueString>(value_edit_storage_id))
                            .unwrap_or_else(|| value_edit_context.initial_value_edit.clone());
                        let value_edit_display_values = Self::build_project_item_value_edit_display_values(
                            &self.app_context.engine_unprivileged_state,
                            &value_edit_context.validation_data_type_ref,
                            &value_edit,
                        );
                        let value_editor_id = format!("project_hierarchy_value_editor_{}", project_item_path.to_string_lossy());
                        let panel_width = user_interface.available_width().clamp(320.0, 560.0);

                        user_interface.add_space(12.0);
                        user_interface.horizontal(|user_interface| {
                            let side_spacing = ((user_interface.available_width() - panel_width) * 0.5).max(0.0);
                            user_interface.add_space(side_spacing);
                            user_interface.allocate_ui_with_layout(vec2(panel_width, 0.0), Layout::top_down(Align::Min), |user_interface| {
                                let value_edit_take_over_response = ProjectItemValueEditTakeOverView::new(
                                    self.app_context.clone(),
                                    &value_edit_context.project_item_name,
                                    &mut value_edit,
                                    &value_edit_context.validation_data_type_ref,
                                    &value_edit_display_values,
                                    &value_editor_id,
                                )
                                .show(user_interface);

                                if value_edit_take_over_response.should_commit {
                                    value_edit_project_item_submission = Some((
                                        project_item_path.clone(),
                                        value_edit_context.value_field_name.clone(),
                                        value_edit_context.validation_data_type_ref.clone(),
                                        value_edit.clone(),
                                    ));
                                }

                                if value_edit_take_over_response.should_cancel {
                                    should_cancel_take_over = true;
                                }
                            });
                        });

                        user_interface
                            .ctx()
                            .data_mut(|data| data.insert_temp(value_edit_storage_id, value_edit));
                    }
                    ProjectHierarchyTakeOverState::DeleteConfirmation { project_item_paths } => {
                        is_delete_confirmation_active = true;
                        let theme = &self.app_context.theme;

                        user_interface.add_space(12.0);
                        user_interface.vertical_centered(|user_interface| {
                            user_interface.label(
                                RichText::new("Confirm deletion of selected project item(s).")
                                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                                    .color(theme.foreground),
                            );
                        });
                        user_interface.add_space(8.0);

                        ScrollArea::vertical()
                            .id_salt("project_hierarchy_delete_confirmation")
                            .max_height(160.0)
                            .auto_shrink([false, false])
                            .show(user_interface, |user_interface| {
                                user_interface.vertical_centered(|user_interface| {
                                    for project_item_path in &project_item_paths {
                                        let project_item_name = project_item_path
                                            .file_name()
                                            .and_then(|value| value.to_str())
                                            .unwrap_or_default();
                                        user_interface.label(
                                            RichText::new(project_item_name)
                                                .font(theme.font_library.font_ubuntu_mono_bold.font_normal.clone())
                                                .color(theme.foreground),
                                        );
                                    }
                                });
                        });

                        user_interface.add_space(8.0);
                        user_interface.allocate_ui(vec2(user_interface.available_width(), 32.0), |user_interface| {
                            let button_size = vec2(120.0, 28.0);
                            let button_spacing = 12.0;
                            let total_button_row_width = button_size.x * 2.0 + button_spacing;
                            let side_spacing = ((user_interface.available_width() - total_button_row_width) * 0.5).max(0.0);

                            user_interface.horizontal(|user_interface| {
                                user_interface.add_space(side_spacing);
                                user_interface.spacing_mut().item_spacing.x = button_spacing;

                                let button_cancel = user_interface.add_sized(
                                    button_size,
                                    eframe::egui::Button::new(RichText::new("Cancel").color(theme.foreground))
                                        .fill(theme.background_control_secondary)
                                        .stroke(Stroke::new(1.0, theme.background_control_secondary_dark)),
                                );

                                if button_cancel.clicked() {
                                    should_cancel_take_over = true;
                                }

                                let button_confirm_delete = user_interface.add_sized(
                                    button_size,
                                    eframe::egui::Button::new(RichText::new("Delete").color(theme.foreground))
                                        .fill(theme.background_control_danger)
                                        .stroke(Stroke::new(1.0, theme.background_control_danger_dark)),
                                );

                                if button_confirm_delete.clicked() {
                                    delete_confirmation_project_item_paths = Some(project_item_paths);
                                }
                            });
                        });
                    }
                    ProjectHierarchyTakeOverState::PromoteSymbolConflict { project_item_paths, conflicts } => {
                        is_promote_symbol_conflict_active = true;
                        let theme = &self.app_context.theme;

                        user_interface.add_space(12.0);
                        user_interface.vertical_centered(|user_interface| {
                            user_interface.label(
                                RichText::new("Overwrite conflicting rooted symbol(s)?")
                                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                                    .color(theme.foreground),
                            );
                        });
                        user_interface.add_space(8.0);

                        ScrollArea::vertical()
                            .id_salt("project_hierarchy_promote_symbol_conflicts")
                            .max_height(180.0)
                            .auto_shrink([false, false])
                            .show(user_interface, |user_interface| {
                                user_interface.vertical(|user_interface| {
                                    for conflict in &conflicts {
                                        user_interface.label(
                                            RichText::new(format!(
                                                "{} -> {} ({})",
                                                conflict.requested_display_name, conflict.symbol_key, conflict.existing_locator_display
                                            ))
                                            .font(theme.font_library.font_ubuntu_mono_bold.font_normal.clone())
                                            .color(theme.foreground),
                                        );
                                    }
                                });
                            });

                        user_interface.add_space(8.0);
                        user_interface.allocate_ui(vec2(user_interface.available_width(), 32.0), |user_interface| {
                            let button_size = vec2(120.0, 28.0);
                            let button_spacing = 12.0;
                            let total_button_row_width = button_size.x * 2.0 + button_spacing;
                            let side_spacing = ((user_interface.available_width() - total_button_row_width) * 0.5).max(0.0);

                            user_interface.horizontal(|user_interface| {
                                user_interface.add_space(side_spacing);
                                user_interface.spacing_mut().item_spacing.x = button_spacing;

                                let button_cancel = user_interface.add_sized(
                                    button_size,
                                    eframe::egui::Button::new(RichText::new("Cancel").color(theme.foreground))
                                        .fill(theme.background_control_secondary)
                                        .stroke(Stroke::new(1.0, theme.background_control_secondary_dark)),
                                );

                                if button_cancel.clicked() {
                                    should_cancel_take_over = true;
                                }

                                let button_confirm_overwrite = user_interface.add_sized(
                                    button_size,
                                    eframe::egui::Button::new(RichText::new("Overwrite").color(theme.foreground))
                                        .fill(theme.background_control_secondary)
                                        .stroke(Stroke::new(1.0, theme.background_control_secondary_dark)),
                                );

                                if button_confirm_overwrite.clicked() {
                                    promote_symbol_overwrite_project_item_paths = Some(project_item_paths);
                                }
                            });
                        });
                    }
                }
            })
            .response;

        if is_delete_confirmation_active || is_promote_symbol_conflict_active {
            if user_interface.input(|input_state| input_state.key_pressed(Key::Escape))
                || user_interface.input(|input_state| input_state.key_pressed(Key::Backspace))
            {
                should_cancel_take_over = true;
            }
        }

        if is_delete_confirmation_active {
            if user_interface.input(|input_state| input_state.key_pressed(Key::Enter)) {
                delete_confirmation_project_item_paths = self
                    .project_hierarchy_view_data
                    .read("Project hierarchy confirm delete by keyboard")
                    .and_then(|project_hierarchy_view_data| match project_hierarchy_view_data.take_over_state.clone() {
                        ProjectHierarchyTakeOverState::DeleteConfirmation { project_item_paths } => Some(project_item_paths),
                        _ => None,
                    });
            }
        }

        if is_promote_symbol_conflict_active && user_interface.input(|input_state| input_state.key_pressed(Key::Enter)) {
            promote_symbol_overwrite_project_item_paths = self
                .project_hierarchy_view_data
                .read("Project hierarchy confirm promote overwrite by keyboard")
                .and_then(|project_hierarchy_view_data| match project_hierarchy_view_data.take_over_state.clone() {
                    ProjectHierarchyTakeOverState::PromoteSymbolConflict { project_item_paths, .. } => Some(project_item_paths),
                    _ => None,
                });
        }

        if !is_delete_confirmation_active
            && !is_promote_symbol_conflict_active
            && !is_rename_take_over_active
            && !is_value_edit_take_over_active
            && user_interface.input(|input_state| input_state.key_pressed(Key::Delete))
        {
            ProjectHierarchyViewData::request_delete_confirmation_for_selected_project_item(self.project_hierarchy_view_data.clone());
        }

        if !is_delete_confirmation_active
            && !is_promote_symbol_conflict_active
            && !is_rename_take_over_active
            && !is_value_edit_take_over_active
            && user_interface.input(|input_state| (input_state.modifiers.command || input_state.modifiers.ctrl) && input_state.key_pressed(Key::X))
        {
            if let Some(project_item_paths) = self
                .project_hierarchy_view_data
                .read("Project hierarchy keyboard cut selection")
                .map(|project_hierarchy_view_data| project_hierarchy_view_data.collect_selected_project_item_paths_in_tree_order())
                .filter(|project_item_paths| !project_item_paths.is_empty())
            {
                project_hierarchy_frame_action = ProjectHierarchyFrameAction::CutProjectItems(project_item_paths);
            }
        }

        if !is_delete_confirmation_active
            && !is_promote_symbol_conflict_active
            && !is_rename_take_over_active
            && !is_value_edit_take_over_active
            && user_interface.input(|input_state| (input_state.modifiers.command || input_state.modifiers.ctrl) && input_state.key_pressed(Key::C))
        {
            if let Some(project_item_paths) = self
                .project_hierarchy_view_data
                .read("Project hierarchy keyboard copy selection")
                .map(|project_hierarchy_view_data| project_hierarchy_view_data.collect_selected_project_item_paths_in_tree_order())
                .filter(|project_item_paths| !project_item_paths.is_empty())
            {
                project_hierarchy_frame_action = ProjectHierarchyFrameAction::CopyProjectItems(project_item_paths);
            }
        }

        if !is_delete_confirmation_active
            && !is_promote_symbol_conflict_active
            && !is_rename_take_over_active
            && !is_value_edit_take_over_active
            && user_interface.input(|input_state| (input_state.modifiers.command || input_state.modifiers.ctrl) && input_state.key_pressed(Key::V))
        {
            if let Some(target_project_item_path) = ProjectHierarchyViewData::get_selected_or_root_directory_path(self.project_hierarchy_view_data.clone()) {
                project_hierarchy_frame_action = ProjectHierarchyFrameAction::PasteProjectItems { target_project_item_path };
            }
        }

        if !is_delete_confirmation_active
            && !is_promote_symbol_conflict_active
            && !is_rename_take_over_active
            && !is_value_edit_take_over_active
            && user_interface.input(|input_state| input_state.key_pressed(Key::F2))
        {
            ProjectHierarchyViewData::request_rename_for_selected_project_item(self.project_hierarchy_view_data.clone());
        }

        if !is_delete_confirmation_active
            && !is_promote_symbol_conflict_active
            && !is_rename_take_over_active
            && !is_value_edit_take_over_active
            && user_interface.input(|input_state| input_state.key_pressed(Key::Space))
        {
            keyboard_activation_toggle_target = self
                .project_hierarchy_view_data
                .read("Project hierarchy keyboard activation toggle")
                .and_then(|project_hierarchy_view_data| {
                    let selected_project_item_paths = project_hierarchy_view_data.collect_selected_project_item_paths_in_tree_order();
                    if selected_project_item_paths.is_empty() {
                        return None;
                    }

                    let selected_project_items = project_hierarchy_view_data
                        .project_items
                        .iter()
                        .filter(|(project_item_ref, _)| selected_project_item_paths.contains(project_item_ref.get_project_item_path()))
                        .map(|(_, project_item)| project_item)
                        .collect::<Vec<&ProjectItem>>();
                    let should_activate = selected_project_items
                        .iter()
                        .any(|project_item| !project_item.get_is_activated());

                    Some((selected_project_item_paths, should_activate))
                });
        }

        if !is_delete_confirmation_active
            && !is_value_edit_take_over_active
            && ProjectHierarchyViewData::set_visible_preview_project_item_paths(self.project_hierarchy_view_data.clone(), visible_preview_project_item_paths)
        {
            self.sync_project_item_virtual_snapshot(project_read_interval);
        }

        if should_cancel_take_over {
            if let Some(project_item_path) = self
                .project_hierarchy_view_data
                .read("Project hierarchy clear inline rename state on cancel")
                .and_then(|project_hierarchy_view_data| match &project_hierarchy_view_data.take_over_state {
                    ProjectHierarchyTakeOverState::RenameProjectItem { project_item_path, .. } => Some(project_item_path.clone()),
                    _ => None,
                })
            {
                Self::clear_project_item_rename_state(user_interface, &project_item_path);
            }
            if let Some(project_item_path) = self
                .project_hierarchy_view_data
                .read("Project hierarchy clear value edit state on cancel")
                .and_then(|project_hierarchy_view_data| match &project_hierarchy_view_data.take_over_state {
                    ProjectHierarchyTakeOverState::EditProjectItemValue { project_item_path } => Some(project_item_path.clone()),
                    _ => None,
                })
            {
                Self::clear_project_item_value_edit_state(user_interface, &project_item_path);
            }
            ProjectHierarchyViewData::cancel_take_over(self.project_hierarchy_view_data.clone());
        }

        if let Some(project_item_paths) = delete_confirmation_project_item_paths {
            ProjectHierarchyViewData::delete_project_items(self.project_hierarchy_view_data.clone(), self.app_context.clone(), project_item_paths);
        }

        if let Some(project_item_paths) = promote_symbol_overwrite_project_item_paths {
            ProjectHierarchyViewData::promote_project_items_to_symbols(
                self.project_hierarchy_view_data.clone(),
                self.app_context.clone(),
                project_item_paths,
                true,
            );
        }

        if let Some((project_item_path, project_item_type_id, edited_name)) = rename_project_item_submission {
            Self::clear_project_item_rename_state(user_interface, &project_item_path);

            if let Some(project_item_rename_request) = Self::build_project_item_rename_request(&project_item_path, &project_item_type_id, edited_name.trim()) {
                let project_hierarchy_view_data = self.project_hierarchy_view_data.clone();
                let app_context = self.app_context.clone();
                let previous_project_item_path = project_item_path.clone();

                project_item_rename_request.send(&self.app_context.engine_unprivileged_state, move |project_items_rename_response| {
                    if !project_items_rename_response.success {
                        log::warn!("Project item rename command failed in hierarchy F2 rename flow.");
                        return;
                    }

                    ProjectHierarchyViewData::finish_project_item_rename(
                        project_hierarchy_view_data.clone(),
                        &previous_project_item_path,
                        &project_items_rename_response.renamed_project_item_path,
                    );
                    ProjectHierarchyViewData::refresh_project_items(project_hierarchy_view_data, app_context);
                });
            }

            ProjectHierarchyViewData::cancel_take_over(self.project_hierarchy_view_data.clone());
        }

        if let Some((project_item_path, value_field_name, validation_data_type_ref, value_edit)) = value_edit_project_item_submission {
            match self
                .app_context
                .engine_unprivileged_state
                .deanonymize_value_string(&validation_data_type_ref, &value_edit)
            {
                Ok(edited_data_value) => {
                    let edited_field = ValuedStructField::new(value_field_name, ValuedStructFieldData::Value(edited_data_value), false);

                    Self::apply_project_item_edits(self.app_context.clone(), vec![project_item_path.clone()], edited_field);
                    Self::clear_project_item_value_edit_state(user_interface, &project_item_path);
                    ProjectHierarchyViewData::cancel_take_over(self.project_hierarchy_view_data.clone());
                }
                Err(error) => {
                    log::warn!("Failed to commit project hierarchy runtime value edit: {}", error);
                }
            }
        }

        if let Some((project_item_paths, is_activated)) = keyboard_activation_toggle_target {
            ProjectHierarchyViewData::set_project_item_activation(
                self.project_hierarchy_view_data.clone(),
                self.app_context.clone(),
                project_item_paths,
                is_activated,
            );
        }

        if let Some(drag_started_project_item_path) = drag_started_project_item_path.clone() {
            ProjectHierarchyViewData::begin_reorder_drag(self.project_hierarchy_view_data.clone(), drag_started_project_item_path);
        }

        let persisted_dragged_project_item_paths = self
            .project_hierarchy_view_data
            .read("Project hierarchy check active drag")
            .and_then(|project_hierarchy_view_data| project_hierarchy_view_data.dragged_project_item_paths.clone());
        let active_dragged_project_item_paths = drag_started_project_item_path
            .map(|drag_started_project_item_path| vec![drag_started_project_item_path])
            .or(persisted_dragged_project_item_paths);

        if active_dragged_project_item_paths.is_some() {
            user_interface.output_mut(|platform_output| {
                platform_output.cursor_icon = CursorIcon::Move;
            });
        }

        if user_interface.input(|input_state| input_state.pointer.any_released()) {
            if active_dragged_project_item_paths.is_some() {
                if let Some(drop_target_project_item_path) = hovered_drop_target_project_item_path {
                    ProjectHierarchyViewData::commit_reorder_drop(
                        self.project_hierarchy_view_data.clone(),
                        self.app_context.clone(),
                        drop_target_project_item_path,
                    );
                } else {
                    ProjectHierarchyViewData::cancel_reorder_drag(self.project_hierarchy_view_data.clone());
                }
            }
        }

        match project_hierarchy_frame_action {
            ProjectHierarchyFrameAction::None => {}
            ProjectHierarchyFrameAction::SelectProjectItem {
                project_item_path,
                additive_selection,
                range_selection,
            } => {
                if is_rename_take_over_active {
                    Self::clear_active_project_item_rename_state(user_interface, self.project_hierarchy_view_data.clone());
                    ProjectHierarchyViewData::cancel_take_over(self.project_hierarchy_view_data.clone());
                }
                if is_value_edit_take_over_active {
                    ProjectHierarchyViewData::cancel_take_over(self.project_hierarchy_view_data.clone());
                }

                ProjectHierarchyViewData::select_project_item(self.project_hierarchy_view_data.clone(), project_item_path, additive_selection, range_selection);
                self.focus_selected_project_items_in_struct_viewer();
            }
            ProjectHierarchyFrameAction::ToggleDirectoryExpansion(project_item_path) => {
                if is_rename_take_over_active {
                    Self::clear_active_project_item_rename_state(user_interface, self.project_hierarchy_view_data.clone());
                    ProjectHierarchyViewData::cancel_take_over(self.project_hierarchy_view_data.clone());
                }
                if is_value_edit_take_over_active {
                    ProjectHierarchyViewData::cancel_take_over(self.project_hierarchy_view_data.clone());
                }

                ProjectHierarchyViewData::toggle_directory_expansion(self.project_hierarchy_view_data.clone(), project_item_path);
            }
            ProjectHierarchyFrameAction::SetProjectItemActivation(project_item_path, is_activated) => {
                if is_promote_symbol_conflict_active || is_rename_take_over_active || is_value_edit_take_over_active {
                    return response;
                }

                let project_item_paths = self
                    .project_hierarchy_view_data
                    .read("Project hierarchy checkbox activation selection")
                    .map(|project_hierarchy_view_data| {
                        if project_hierarchy_view_data
                            .selected_project_item_paths
                            .contains(&project_item_path)
                        {
                            project_hierarchy_view_data.collect_selected_project_item_paths_in_tree_order()
                        } else {
                            vec![project_item_path.clone()]
                        }
                    })
                    .unwrap_or_else(|| vec![project_item_path.clone()]);
                ProjectHierarchyViewData::set_project_item_activation(
                    self.project_hierarchy_view_data.clone(),
                    self.app_context.clone(),
                    project_item_paths,
                    is_activated,
                );
            }
            ProjectHierarchyFrameAction::CreateProjectItem {
                target_project_item_path,
                create_item_kind,
            } => {
                if is_promote_symbol_conflict_active || is_rename_take_over_active || is_value_edit_take_over_active {
                    return response;
                }

                ProjectHierarchyViewData::create_project_item(
                    self.project_hierarchy_view_data.clone(),
                    self.app_context.clone(),
                    target_project_item_path,
                    create_item_kind,
                );
            }
            ProjectHierarchyFrameAction::CopyProjectItems(project_item_paths) => {
                if is_promote_symbol_conflict_active || is_rename_take_over_active || is_value_edit_take_over_active {
                    return response;
                }

                ProjectHierarchyViewData::copy_project_items(self.project_hierarchy_view_data.clone(), project_item_paths);
            }
            ProjectHierarchyFrameAction::CutProjectItems(project_item_paths) => {
                if is_promote_symbol_conflict_active || is_rename_take_over_active || is_value_edit_take_over_active {
                    return response;
                }

                ProjectHierarchyViewData::cut_project_items(self.project_hierarchy_view_data.clone(), project_item_paths);
            }
            ProjectHierarchyFrameAction::PasteProjectItems { target_project_item_path } => {
                if is_promote_symbol_conflict_active || is_rename_take_over_active || is_value_edit_take_over_active {
                    return response;
                }

                ProjectHierarchyViewData::paste_project_item_clipboard(
                    self.project_hierarchy_view_data.clone(),
                    self.app_context.clone(),
                    target_project_item_path,
                );
            }
            ProjectHierarchyFrameAction::OpenPointerScannerForAddress {
                address,
                module_name,
                data_type_id,
            } => {
                if is_promote_symbol_conflict_active || is_rename_take_over_active || is_value_edit_take_over_active {
                    return response;
                }

                self.focus_pointer_scanner_for_address(address, &module_name, &data_type_id);
            }
            ProjectHierarchyFrameAction::OpenMemoryViewerForAddress {
                address,
                module_name,
                selection_byte_count,
            } => {
                if is_promote_symbol_conflict_active || is_rename_take_over_active || is_value_edit_take_over_active {
                    return response;
                }

                self.focus_memory_viewer_for_address(address, &module_name, selection_byte_count);
            }
            ProjectHierarchyFrameAction::OpenCodeViewerForAddress { address, module_name } => {
                if is_promote_symbol_conflict_active || is_rename_take_over_active || is_value_edit_take_over_active {
                    return response;
                }

                self.focus_code_viewer_for_address(address, &module_name);
            }
            ProjectHierarchyFrameAction::PromoteToSymbol {
                project_item_paths,
                overwrite_conflicting_symbols,
            } => {
                if is_rename_take_over_active || is_value_edit_take_over_active {
                    return response;
                }

                ProjectHierarchyViewData::promote_project_items_to_symbols(
                    self.project_hierarchy_view_data.clone(),
                    self.app_context.clone(),
                    project_item_paths,
                    overwrite_conflicting_symbols,
                );
            }
            ProjectHierarchyFrameAction::ConvertSymbolRef {
                project_item_paths,
                conversion_target: _conversion_target,
            } => {
                if is_promote_symbol_conflict_active || is_rename_take_over_active || is_value_edit_take_over_active {
                    return response;
                }

                ProjectHierarchyViewData::convert_symbol_refs_to_project_items(
                    self.project_hierarchy_view_data.clone(),
                    self.app_context.clone(),
                    project_item_paths,
                );
            }
            ProjectHierarchyFrameAction::RequestRename(project_item_path) => {
                if is_promote_symbol_conflict_active || is_value_edit_take_over_active {
                    return response;
                }

                if is_rename_take_over_active {
                    Self::clear_active_project_item_rename_state(user_interface, self.project_hierarchy_view_data.clone());
                }

                ProjectHierarchyViewData::request_rename_for_project_item(self.project_hierarchy_view_data.clone(), project_item_path);
            }
            ProjectHierarchyFrameAction::RequestValueEdit(project_item_path) => {
                if is_promote_symbol_conflict_active {
                    return response;
                }

                if is_rename_take_over_active {
                    Self::clear_active_project_item_rename_state(user_interface, self.project_hierarchy_view_data.clone());
                    ProjectHierarchyViewData::cancel_take_over(self.project_hierarchy_view_data.clone());
                }

                ProjectHierarchyViewData::request_value_edit_for_project_item(self.project_hierarchy_view_data.clone(), project_item_path);
            }
            ProjectHierarchyFrameAction::RequestDeleteConfirmation(project_item_paths) => {
                if is_promote_symbol_conflict_active || is_rename_take_over_active || is_value_edit_take_over_active {
                    return response;
                }

                ProjectHierarchyViewData::request_delete_confirmation(self.project_hierarchy_view_data.clone(), project_item_paths);
            }
        }

        response
    }
}

impl ProjectHierarchyView {
    const MIN_PROJECT_READ_INTERVAL_MS: u64 = 50;
    const MAX_PROJECT_READ_INTERVAL_MS: u64 = 5_000;
    const SCAN_SETTINGS_SYNC_INTERVAL_MS: u64 = 1_000;
    const MIN_PROJECT_ITEM_MENU_WIDTH: f32 = 160.0;
    const PROJECT_ITEM_MENU_ITEM_HORIZONTAL_PADDING: f32 = 36.0;
    const DROP_INSERTION_BAND_HEIGHT: f32 = 7.0;
    const PROJECT_ITEM_ROW_HEIGHT: f32 = 28.0;
    const PROJECT_ITEM_PREVIEW_VIRTUAL_SNAPSHOT_ID: &str = "project_hierarchy_preview";
    const MAX_PROJECT_ITEM_PREVIEW_ARRAY_CHARACTER_COUNT: usize = 96;
    const MAX_PROJECT_ITEM_PREVIEW_ELEMENT_COUNT: u64 = 4;
    const MAX_PROJECT_ITEM_PREVIEW_DISPLAY_ELEMENT_COUNT: usize = 4;

    fn show_toolbar_add_menu(
        &self,
        project_hierarchy_frame_action: &mut ProjectHierarchyFrameAction,
        user_interface: &mut Ui,
        menu_target: Option<&ProjectHierarchyMenuTarget>,
        menu_position: Option<Pos2>,
    ) {
        let Some(ProjectHierarchyMenuTarget::ToolbarAdd { target_project_item_path }) = menu_target else {
            return;
        };

        let Some(menu_position) = menu_position else {
            return;
        };
        let create_project_item_menu_labels = ["New Folder", "New Address", "New Pointer", "New Symbol Ref"];
        let project_item_menu_width = Self::calculate_project_item_menu_width(self.app_context.as_ref(), user_interface, &create_project_item_menu_labels);
        let mut open = true;
        ContextMenu::new(
            self.app_context.clone(),
            "project_hierarchy_toolbar_add_menu",
            menu_position,
            |user_interface, should_close| {
                Self::show_create_project_item_menu_items(
                    self.app_context.clone(),
                    user_interface,
                    target_project_item_path,
                    project_hierarchy_frame_action,
                    project_item_menu_width,
                    should_close,
                );
            },
        )
        .width(project_item_menu_width)
        .corner_radius(8)
        .show(user_interface, &mut open);

        if !open {
            ProjectHierarchyViewData::hide_menu(self.project_hierarchy_view_data.clone());
        }
    }

    fn show_create_project_item_menu_items(
        app_context: Arc<AppContext>,
        user_interface: &mut Ui,
        target_project_item_path: &Path,
        project_hierarchy_frame_action: &mut ProjectHierarchyFrameAction,
        project_item_menu_width: f32,
        should_close: &mut bool,
    ) {
        for (label, item_id, create_item_kind) in [
            ("New Folder", "project_hierarchy_ctx_new_folder", ProjectHierarchyCreateItemKind::Directory),
            ("New Address", "project_hierarchy_ctx_new_address", ProjectHierarchyCreateItemKind::Address),
            ("New Pointer", "project_hierarchy_ctx_new_pointer", ProjectHierarchyCreateItemKind::Pointer),
            (
                "New Symbol Ref",
                "project_hierarchy_ctx_new_symbol_ref",
                ProjectHierarchyCreateItemKind::SymbolRef,
            ),
        ] {
            if user_interface
                .add(
                    ToolbarMenuItemView::new(app_context.clone(), label, item_id, &None, project_item_menu_width).icon(match create_item_kind {
                        ProjectHierarchyCreateItemKind::Directory => app_context
                            .theme
                            .icon_library
                            .icon_handle_file_system_open_folder
                            .clone(),
                        ProjectHierarchyCreateItemKind::Address => app_context
                            .theme
                            .icon_library
                            .icon_handle_data_type_blue_blocks_4
                            .clone(),
                        ProjectHierarchyCreateItemKind::Pointer => app_context
                            .theme
                            .icon_library
                            .icon_handle_project_pointer_type
                            .clone(),
                        ProjectHierarchyCreateItemKind::SymbolRef => app_context
                            .theme
                            .icon_library
                            .icon_handle_data_type_unknown
                            .clone(),
                    }),
                )
                .clicked()
            {
                *project_hierarchy_frame_action = ProjectHierarchyFrameAction::CreateProjectItem {
                    target_project_item_path: target_project_item_path.to_path_buf(),
                    create_item_kind,
                };
                *should_close = true;
            }
        }
    }

    fn calculate_project_item_menu_width(
        app_context: &AppContext,
        user_interface: &mut Ui,
        item_labels: &[&str],
    ) -> f32 {
        let mut longest_label_width: f32 = 0.0;

        user_interface.ctx().fonts_mut(|fonts| {
            for item_label in item_labels {
                let galley = fonts.layout_no_wrap(
                    (*item_label).to_string(),
                    app_context
                        .theme
                        .font_library
                        .font_noto_sans
                        .font_normal
                        .clone(),
                    app_context.theme.foreground,
                );
                longest_label_width = longest_label_width.max(galley.size().x);
            }
        });

        Self::project_item_menu_width_from_longest_label_width(longest_label_width)
    }

    fn project_item_menu_width_from_longest_label_width(longest_label_width: f32) -> f32 {
        (longest_label_width + Self::PROJECT_ITEM_MENU_ITEM_HORIZONTAL_PADDING)
            .ceil()
            .max(Self::MIN_PROJECT_ITEM_MENU_WIDTH)
    }

    fn resolve_drop_target(
        active_dragged_project_item_paths: &[PathBuf],
        tree_entry: &ProjectHierarchyTreeEntry,
        row_rect: Rect,
        pointer_position: Pos2,
    ) -> Option<ProjectHierarchyDropTarget> {
        if active_dragged_project_item_paths.contains(&tree_entry.project_item_path) {
            return None;
        }

        let insertion_band_height = Self::DROP_INSERTION_BAND_HEIGHT.min(row_rect.height() / 2.0);

        if pointer_position.y <= row_rect.top() + insertion_band_height
            && Self::can_render_insertion_drop_target(active_dragged_project_item_paths, &tree_entry.project_item_path)
        {
            return Some(ProjectHierarchyDropTarget::Before(tree_entry.project_item_path.clone()));
        }

        if pointer_position.y >= row_rect.bottom() - insertion_band_height
            && Self::can_render_insertion_drop_target(active_dragged_project_item_paths, &tree_entry.project_item_path)
        {
            return Some(ProjectHierarchyDropTarget::After(tree_entry.project_item_path.clone()));
        }

        if tree_entry.is_directory && Self::can_render_into_directory_drop_target(active_dragged_project_item_paths, &tree_entry.project_item_path) {
            return Some(ProjectHierarchyDropTarget::Into(tree_entry.project_item_path.clone()));
        }

        None
    }

    fn can_render_insertion_drop_target(
        active_dragged_project_item_paths: &[PathBuf],
        target_project_item_path: &Path,
    ) -> bool {
        let Some(target_directory_path) = target_project_item_path.parent() else {
            return false;
        };

        !active_dragged_project_item_paths.contains(&target_project_item_path.to_path_buf())
            && active_dragged_project_item_paths
                .iter()
                .all(|dragged_project_item_path| !target_directory_path.starts_with(dragged_project_item_path))
    }

    fn can_render_into_directory_drop_target(
        active_dragged_project_item_paths: &[PathBuf],
        target_project_item_path: &Path,
    ) -> bool {
        !active_dragged_project_item_paths
            .iter()
            .any(|dragged_project_item_path| target_project_item_path.starts_with(dragged_project_item_path))
    }

    fn paint_drop_target_indicator(
        &self,
        user_interface: &mut Ui,
        row_rect: Rect,
        drop_target: &ProjectHierarchyDropTarget,
    ) {
        let theme = &self.app_context.theme;

        match drop_target {
            ProjectHierarchyDropTarget::Into(_) => {
                user_interface
                    .painter()
                    .rect_filled(row_rect, CornerRadius::ZERO, theme.selected_background);
                user_interface
                    .painter()
                    .rect_stroke(row_rect, CornerRadius::ZERO, Stroke::new(1.0, theme.selected_border), StrokeKind::Inside);
            }
            ProjectHierarchyDropTarget::Before(_) | ProjectHierarchyDropTarget::After(_) => {
                let indicator_y = match drop_target {
                    ProjectHierarchyDropTarget::Before(_) => row_rect.top() + 0.5,
                    ProjectHierarchyDropTarget::After(_) => row_rect.bottom() - 0.5,
                    ProjectHierarchyDropTarget::Into(_) => row_rect.center().y,
                };
                let indicator_left = row_rect.left() + 8.0;
                let indicator_right = row_rect.right() - 8.0;
                let indicator_cap_half_height = 5.0;

                user_interface.painter().line_segment(
                    [
                        Pos2::new(indicator_left, indicator_y),
                        Pos2::new(indicator_right, indicator_y),
                    ],
                    Stroke::new(3.0, theme.selected_border),
                );
                user_interface.painter().line_segment(
                    [
                        Pos2::new(indicator_left, indicator_y - indicator_cap_half_height),
                        Pos2::new(indicator_left, indicator_y + indicator_cap_half_height),
                    ],
                    Stroke::new(3.0, theme.selected_border),
                );
                user_interface.painter().line_segment(
                    [
                        Pos2::new(indicator_right, indicator_y - indicator_cap_half_height),
                        Pos2::new(indicator_right, indicator_y + indicator_cap_half_height),
                    ],
                    Stroke::new(3.0, theme.selected_border),
                );
            }
        }
    }

    fn sync_scan_settings_if_needed(&self) {
        let should_request_scan_settings = self
            .project_hierarchy_view_data
            .write("Project hierarchy scan settings sync check")
            .map(|mut project_hierarchy_view_data| {
                let now = Instant::now();
                let has_sync_interval_elapsed = project_hierarchy_view_data
                    .last_scan_settings_sync_timestamp
                    .map(|last_scan_settings_sync_timestamp| {
                        now.duration_since(last_scan_settings_sync_timestamp) >= Duration::from_millis(Self::SCAN_SETTINGS_SYNC_INTERVAL_MS)
                    })
                    .unwrap_or(true);

                if project_hierarchy_view_data.is_querying_scan_settings || !has_sync_interval_elapsed {
                    return false;
                }

                project_hierarchy_view_data.is_querying_scan_settings = true;
                project_hierarchy_view_data.last_scan_settings_sync_timestamp = Some(now);

                true
            })
            .unwrap_or(false);

        if !should_request_scan_settings {
            return;
        }

        let project_hierarchy_view_data = self.project_hierarchy_view_data.clone();
        let scan_settings_list_request = ScanSettingsListRequest {};
        scan_settings_list_request.send(&self.app_context.engine_unprivileged_state, move |scan_settings_list_response| {
            if let Some(mut project_hierarchy_view_data) = project_hierarchy_view_data.write("Project hierarchy scan settings sync response") {
                if let Ok(scan_settings) = scan_settings_list_response.scan_settings {
                    project_hierarchy_view_data.project_read_interval_ms = scan_settings.project_read_interval_ms;
                }

                project_hierarchy_view_data.is_querying_scan_settings = false;
            }
        });
    }

    fn get_project_read_interval(&self) -> Duration {
        let configured_project_read_interval_ms = self
            .project_hierarchy_view_data
            .read("Project hierarchy project read interval")
            .map(|project_hierarchy_view_data| project_hierarchy_view_data.project_read_interval_ms)
            .unwrap_or(200);
        let bounded_project_read_interval_ms =
            configured_project_read_interval_ms.clamp(Self::MIN_PROJECT_READ_INTERVAL_MS, Self::MAX_PROJECT_READ_INTERVAL_MS);

        Duration::from_millis(bounded_project_read_interval_ms)
    }

    fn refresh_if_project_preview_values_stale(
        &self,
        project_read_interval: Duration,
    ) {
        let should_refresh_project_items = self
            .project_hierarchy_view_data
            .write("Project hierarchy periodic project read check")
            .map(|mut project_hierarchy_view_data| {
                let has_open_project = project_hierarchy_view_data.opened_project_info.is_some();
                if !has_open_project || project_hierarchy_view_data.pending_operation != ProjectHierarchyPendingOperation::None {
                    return false;
                }

                let now = Instant::now();
                let has_refresh_interval_elapsed = project_hierarchy_view_data
                    .last_project_read_timestamp
                    .map(|last_project_read_timestamp| now.duration_since(last_project_read_timestamp) >= project_read_interval)
                    .unwrap_or(true);

                if !has_refresh_interval_elapsed {
                    return false;
                }

                project_hierarchy_view_data.last_project_read_timestamp = Some(now);

                true
            })
            .unwrap_or(false);

        if should_refresh_project_items {
            self.sync_project_item_virtual_snapshot(project_read_interval);
        }
    }

    fn sync_project_item_virtual_snapshot(
        &self,
        project_read_interval: Duration,
    ) {
        let virtual_snapshot_queries = self
            .project_hierarchy_view_data
            .read("Project hierarchy build virtual snapshot queries")
            .map(|project_hierarchy_view_data| {
                if project_hierarchy_view_data.opened_project_info.is_none() {
                    return Vec::new();
                }

                let requested_preview_project_item_paths = project_hierarchy_view_data.collect_requested_preview_project_item_paths();

                requested_preview_project_item_paths
                    .into_iter()
                    .filter_map(|project_item_path| {
                        project_hierarchy_view_data
                            .project_items
                            .iter()
                            .find(|(project_item_ref, _)| project_item_ref.get_project_item_path() == &project_item_path)
                            .and_then(|(_, project_item)| {
                                Self::build_project_item_virtual_snapshot_query(
                                    project_hierarchy_view_data.opened_project_info.as_ref(),
                                    &project_item_path,
                                    project_item,
                                    &self.app_context.engine_unprivileged_state,
                                )
                            })
                    })
                    .collect::<Vec<VirtualSnapshotQuery>>()
            })
            .unwrap_or_default();

        self.app_context
            .engine_unprivileged_state
            .set_virtual_snapshot_queries(Self::PROJECT_ITEM_PREVIEW_VIRTUAL_SNAPSHOT_ID, project_read_interval, virtual_snapshot_queries);
        self.app_context
            .engine_unprivileged_state
            .request_virtual_snapshot_refresh(Self::PROJECT_ITEM_PREVIEW_VIRTUAL_SNAPSHOT_ID);
        self.apply_project_item_virtual_snapshot_results();
    }

    fn apply_project_item_virtual_snapshot_results(&self) {
        let Some(virtual_snapshot) = self
            .app_context
            .engine_unprivileged_state
            .get_virtual_snapshot(Self::PROJECT_ITEM_PREVIEW_VIRTUAL_SNAPSHOT_ID)
        else {
            return;
        };
        let preview_fields_by_project_item_path = self
            .project_hierarchy_view_data
            .read("Project hierarchy apply virtual snapshot results")
            .map(|project_hierarchy_view_data| {
                project_hierarchy_view_data
                    .project_items
                    .iter()
                    .filter_map(|(project_item_ref, project_item)| {
                        let project_item_path = project_item_ref.get_project_item_path();
                        let query_id = project_item_path.to_string_lossy().to_string();
                        let query_result = virtual_snapshot.get_query_results().get(&query_id)?;
                        let preview_value = Self::build_project_item_preview_value_from_virtual_snapshot_result(
                            &self.app_context.engine_unprivileged_state,
                            project_hierarchy_view_data.opened_project_info.as_ref(),
                            project_item,
                            query_result,
                        );
                        let preview_path = if project_item.get_item_type().get_project_item_type_id() == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
                            query_result.evaluated_pointer_path.clone()
                        } else if project_item.get_item_type().get_project_item_type_id() == ProjectItemTypeSymbolRef::PROJECT_ITEM_TYPE_ID {
                            Self::resolve_project_rooted_symbol(project_hierarchy_view_data.opened_project_info.as_ref(), project_item)
                                .map(|rooted_symbol| rooted_symbol.get_root_locator().to_string())
                                .unwrap_or_default()
                        } else {
                            String::new()
                        };

                        Some((project_item_path.clone(), (preview_value, preview_path)))
                    })
                    .collect::<HashMap<PathBuf, (String, String)>>()
            })
            .unwrap_or_default();

        if !preview_fields_by_project_item_path.is_empty() {
            let _ = ProjectHierarchyViewData::set_project_item_preview_fields(self.project_hierarchy_view_data.clone(), &preview_fields_by_project_item_path);
        }
    }

    fn focus_selected_project_items_in_struct_viewer(&self) {
        let selected_project_item_paths = self
            .project_hierarchy_view_data
            .read("Project hierarchy selected project items for struct viewer focus")
            .map(|project_hierarchy_view_data| project_hierarchy_view_data.collect_selected_project_item_paths_in_tree_order())
            .unwrap_or_default();
        let (selected_project_items, opened_project_info) = self
            .project_hierarchy_view_data
            .read("Project hierarchy selected project item data for struct viewer focus")
            .map(|project_hierarchy_view_data| {
                (
                    project_hierarchy_view_data
                        .project_items
                        .iter()
                        .filter(|(project_item_ref, _)| selected_project_item_paths.contains(project_item_ref.get_project_item_path()))
                        .map(|(_, project_item)| project_item.clone())
                        .collect::<Vec<ProjectItem>>(),
                    project_hierarchy_view_data.opened_project_info.clone(),
                )
            })
            .unwrap_or_default();

        if selected_project_item_paths.is_empty() || selected_project_items.is_empty() {
            StructViewerViewData::clear_focus(self.struct_viewer_view_data.clone());
            return;
        }

        let app_context = self.app_context.clone();
        let selected_project_item_paths_for_edit = selected_project_item_paths.clone();
        let callback = Arc::new(move |edited_field: ValuedStructField| {
            Self::apply_project_item_edits(app_context.clone(), selected_project_item_paths_for_edit.clone(), edited_field);
        });

        if selected_project_items.len() == 1 {
            if let Some(selected_project_item) = selected_project_items.into_iter().next() {
                StructViewerViewData::focus_valued_struct_with_focus_target(
                    self.struct_viewer_view_data.clone(),
                    self.app_context.engine_unprivileged_state.clone(),
                    Self::build_struct_view_properties(opened_project_info.as_ref(), &selected_project_item),
                    callback,
                    Some(StructViewerFocusTarget::ProjectHierarchy {
                        project_item_paths: selected_project_item_paths,
                    }),
                );
            }
        } else {
            let selected_project_item_properties = selected_project_items
                .into_iter()
                .map(|selected_project_item| Self::build_struct_view_properties(opened_project_info.as_ref(), &selected_project_item))
                .collect::<Vec<_>>();
            StructViewerViewData::focus_valued_structs_with_focus_target(
                self.struct_viewer_view_data.clone(),
                self.app_context.engine_unprivileged_state.clone(),
                selected_project_item_properties,
                callback,
                Some(StructViewerFocusTarget::ProjectHierarchy {
                    project_item_paths: selected_project_item_paths,
                }),
            );
        }
    }

    fn apply_project_item_edits(
        app_context: Arc<AppContext>,
        project_item_paths: Vec<PathBuf>,
        edited_field: ValuedStructField,
    ) {
        if project_item_paths.is_empty() {
            return;
        }

        let project_manager = app_context.engine_unprivileged_state.get_project_manager();
        let opened_project_lock = project_manager.get_opened_project();
        let mut memory_write_requests = Vec::new();
        let mut rename_requests = Vec::new();
        let mut has_persisted_property_edits = false;
        let edited_field_name = edited_field.get_name().to_string();
        let engine_execution_context: Arc<dyn EngineExecutionContext> = app_context.engine_unprivileged_state.clone();
        let edited_name = if edited_field_name == ProjectItem::PROPERTY_NAME {
            Self::extract_string_value_from_edited_field(&edited_field)
        } else {
            None
        };

        let mut opened_project_guard = match opened_project_lock.write() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                log::error!("Failed to acquire opened project lock for struct viewer edit: {}", error);
                return;
            }
        };
        let opened_project = match opened_project_guard.as_mut() {
            Some(opened_project) => opened_project,
            None => {
                log::warn!("Cannot apply struct viewer edit without an opened project.");
                return;
            }
        };
        let opened_project_info = opened_project.get_project_info().clone();
        let root_project_item_path = opened_project
            .get_project_root_ref()
            .get_project_item_path()
            .clone();

        for project_item_path in &project_item_paths {
            if edited_field.get_name() == ProjectItem::PROPERTY_NAME && project_item_path == &root_project_item_path {
                log::debug!("Ignoring root project directory name edit in project hierarchy.");
                continue;
            }

            let project_item_ref = ProjectItemRef::new(project_item_path.clone());
            let project_item = match opened_project.get_project_item_mut(&project_item_ref) {
                Some(project_item) => project_item,
                None => {
                    log::warn!("Cannot apply struct viewer edit, project item was not found: {:?}", project_item_path);
                    continue;
                }
            };
            let project_item_type_id = project_item
                .get_item_type()
                .get_project_item_type_id()
                .to_string();
            let should_apply_field_edit = Self::should_apply_struct_field_edit_to_project_item(&project_item_type_id, &edited_field_name);

            if should_apply_field_edit {
                project_item.get_properties_mut().set_field_data(
                    edited_field.get_name(),
                    edited_field.get_field_data().clone(),
                    edited_field.get_is_read_only(),
                );
                project_item.set_has_unsaved_changes(true);
                has_persisted_property_edits = true;
            }

            if let Some(edited_name) = &edited_name {
                if let Some(project_items_rename_request) = Self::build_project_item_rename_request(project_item_path, &project_item_type_id, edited_name) {
                    rename_requests.push(project_items_rename_request);
                }
            }

            if let Some(memory_write_request) = Self::build_memory_write_request_for_project_item_edit(project_item, &edited_field) {
                memory_write_requests.push(memory_write_request);
            } else if let Some(memory_write_request) =
                Self::build_memory_write_request_for_runtime_value_edit(&engine_execution_context, Some(&opened_project_info), project_item, &edited_field)
            {
                memory_write_requests.push(memory_write_request);
            }
        }

        if !has_persisted_property_edits && rename_requests.is_empty() && memory_write_requests.is_empty() {
            return;
        }

        drop(opened_project_guard);

        if has_persisted_property_edits {
            if let Ok(mut opened_project_guard) = opened_project_lock.write() {
                if let Some(opened_project) = opened_project_guard.as_mut() {
                    opened_project
                        .get_project_info_mut()
                        .set_has_unsaved_changes(true);
                }
            }

            let project_save_request = ProjectSaveRequest {};

            project_save_request.send(&app_context.engine_unprivileged_state, |project_save_response| {
                if !project_save_response.success {
                    log::error!("Failed to persist project item edit through project save command.");
                }
            });
            project_manager.notify_project_items_changed();
        }

        for rename_request in rename_requests {
            rename_request.send(&app_context.engine_unprivileged_state, |project_items_rename_response| {
                if !project_items_rename_response.success {
                    log::warn!("Project item rename command failed while committing name edit.");
                }
            });
        }

        for memory_write_request in memory_write_requests {
            memory_write_request.send(&app_context.engine_unprivileged_state, |memory_write_response| {
                if !memory_write_response.success {
                    log::warn!("Project item address edit memory write command failed.");
                }
            });
        }
    }

    fn build_memory_write_request_for_project_item_edit(
        project_item: &mut ProjectItem,
        edited_field: &ValuedStructField,
    ) -> Option<MemoryWriteRequest> {
        if project_item.get_item_type().get_project_item_type_id() != ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            return None;
        }

        if edited_field.get_name() != ProjectItemTypeAddress::PROPERTY_ADDRESS {
            return None;
        }

        let edited_data_value = edited_field.get_data_value()?;
        let address = ProjectItemTypeAddress::get_field_address(project_item);
        let module_name = ProjectItemTypeAddress::get_field_module(project_item);

        Some(MemoryWriteRequest {
            address,
            module_name,
            value: edited_data_value.get_value_bytes().clone(),
        })
    }

    fn build_memory_write_request_for_runtime_value_edit(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        opened_project_info: Option<&ProjectInfo>,
        project_item: &ProjectItem,
        edited_field: &ValuedStructField,
    ) -> Option<MemoryWriteRequest> {
        if !Self::is_runtime_value_field(edited_field.get_name()) {
            return None;
        }

        let edited_data_value = edited_field.get_data_value()?;
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            let mut project_item = project_item.clone();
            let address = ProjectItemTypeAddress::get_field_address(&mut project_item);
            let module_name = ProjectItemTypeAddress::get_field_module(&mut project_item);

            return Some(MemoryWriteRequest {
                address,
                module_name,
                value: edited_data_value.get_value_bytes().clone(),
            });
        }

        if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            let pointer = ProjectItemTypePointer::get_field_pointer(project_item);
            let (address, module_name) = Self::resolve_pointer_write_target(engine_execution_context, &pointer)?;

            return Some(MemoryWriteRequest {
                address,
                module_name,
                value: edited_data_value.get_value_bytes().clone(),
            });
        }

        if project_item_type_id == ProjectItemTypeSymbolRef::PROJECT_ITEM_TYPE_ID {
            let rooted_symbol = Self::resolve_project_rooted_symbol(opened_project_info, project_item)?;

            return Some(MemoryWriteRequest {
                address: rooted_symbol.get_root_locator().get_focus_address(),
                module_name: rooted_symbol
                    .get_root_locator()
                    .get_focus_module_name()
                    .to_string(),
                value: edited_data_value.get_value_bytes().clone(),
            });
        }

        None
    }

    fn build_pointer_scanner_context_actions(
        opened_project_info: Option<&ProjectInfo>,
        project_item: &ProjectItem,
    ) -> Vec<PointerScannerContextAction> {
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            let mut project_item = project_item.clone();

            return vec![PointerScannerContextAction::Address {
                label: "Open in Pointer Scan",
                address: ProjectItemTypeAddress::get_field_address(&mut project_item),
                module_name: ProjectItemTypeAddress::get_field_module(&mut project_item),
                data_type_id: ProjectItemTypeAddress::get_field_symbolic_struct_definition_reference(&mut project_item)
                    .map(|symbolic_struct_reference| {
                        symbolic_struct_reference
                            .get_symbolic_struct_namespace()
                            .to_string()
                    })
                    .unwrap_or_default(),
            }];
        }

        if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            let pointer = ProjectItemTypePointer::get_field_pointer(project_item);
            let data_type_id = ProjectItemTypePointer::get_field_symbolic_struct_definition_reference(project_item)
                .map(|symbolic_struct_reference| {
                    symbolic_struct_reference
                        .get_symbolic_struct_namespace()
                        .to_string()
                })
                .unwrap_or_default();

            return vec![
                PointerScannerContextAction::Address {
                    label: "Open Base Address in Pointer Scan",
                    address: pointer.get_address(),
                    module_name: pointer.get_module_name().to_string(),
                    data_type_id: data_type_id.clone(),
                },
                PointerScannerContextAction::ResolvedPointer {
                    label: "Open Resolved Address in Pointer Scan",
                    pointer,
                    data_type_id,
                },
            ];
        }

        if project_item_type_id == ProjectItemTypeSymbolRef::PROJECT_ITEM_TYPE_ID {
            let Some(rooted_symbol) = Self::resolve_project_rooted_symbol(opened_project_info, project_item) else {
                return Vec::new();
            };

            return vec![PointerScannerContextAction::Address {
                label: "Open in Pointer Scan",
                address: rooted_symbol.get_root_locator().get_focus_address(),
                module_name: rooted_symbol
                    .get_root_locator()
                    .get_focus_module_name()
                    .to_string(),
                data_type_id: rooted_symbol.get_struct_layout_id().to_string(),
            }];
        }

        Vec::new()
    }

    fn can_open_project_item_in_memory_viewer(
        opened_project_info: Option<&ProjectInfo>,
        project_item: &ProjectItem,
    ) -> bool {
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID
            || project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID
            || (project_item_type_id == ProjectItemTypeSymbolRef::PROJECT_ITEM_TYPE_ID
                && Self::resolve_project_rooted_symbol(opened_project_info, project_item).is_some())
    }

    fn resolve_pointer_scanner_context_action(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        pointer_scanner_context_action: &PointerScannerContextAction,
    ) -> Option<(u64, String, String)> {
        match pointer_scanner_context_action {
            PointerScannerContextAction::Address {
                address,
                module_name,
                data_type_id,
                ..
            } => Some((*address, module_name.clone(), data_type_id.clone())),
            PointerScannerContextAction::ResolvedPointer { pointer, data_type_id, .. } => {
                let (address, module_name) = Self::resolve_pointer_write_target(engine_execution_context, pointer)?;

                Some((address, module_name, data_type_id.clone()))
            }
        }
    }

    fn build_struct_view_properties(
        opened_project_info: Option<&ProjectInfo>,
        project_item: &ProjectItem,
    ) -> ValuedStruct {
        let mut fields = project_item
            .get_properties()
            .get_fields()
            .iter()
            .map(|valued_struct_field| {
                let is_runtime_value_field = Self::is_runtime_value_field(valued_struct_field.get_name());

                ValuedStructField::new(
                    valued_struct_field.get_name().to_string(),
                    valued_struct_field.get_field_data().clone(),
                    if is_runtime_value_field {
                        false
                    } else {
                        valued_struct_field.get_is_read_only()
                    },
                )
            })
            .collect::<Vec<_>>();

        if project_item.get_item_type().get_project_item_type_id() == ProjectItemTypeSymbolRef::PROJECT_ITEM_TYPE_ID {
            if let Some(rooted_symbol) = Self::resolve_project_rooted_symbol(opened_project_info, project_item) {
                let (address, module_name) = match rooted_symbol.get_root_locator() {
                    ProjectRootSymbolLocator::AbsoluteAddress { address } => (*address, String::new()),
                    ProjectRootSymbolLocator::ModuleOffset { module_name, offset } => (*offset, module_name.clone()),
                };

                fields.push(
                    DataTypeU64::get_value_from_primitive(address).to_named_valued_struct_field(ProjectItemTypeAddress::PROPERTY_ADDRESS.to_string(), true),
                );
                fields.push(
                    DataTypeStringUtf8::get_value_from_primitive_string(&module_name)
                        .to_named_valued_struct_field(ProjectItemTypeAddress::PROPERTY_MODULE.to_string(), true),
                );
                fields.push(
                    DataTypeStringUtf8::get_value_from_primitive_string(rooted_symbol.get_struct_layout_id())
                        .to_named_valued_struct_field(ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE.to_string(), true),
                );
            }
        }

        ValuedStruct::new_anonymous(fields)
    }

    fn is_runtime_value_field(field_name: &str) -> bool {
        field_name == ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE
            || field_name == ProjectItemTypePointer::PROPERTY_FREEZE_DISPLAY_VALUE
            || field_name == ProjectItemTypeSymbolRef::PROPERTY_FREEZE_DISPLAY_VALUE
    }

    fn resolve_pointer_write_target(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        pointer: &Pointer,
    ) -> Option<(u64, String)> {
        let mut current_address = pointer.get_address();
        let mut current_module_name = pointer.get_module_name().to_string();

        for pointer_offset in pointer.get_offsets() {
            let pointer_value = Self::read_pointer_value(engine_execution_context, current_address, &current_module_name, pointer.get_pointer_size())?;
            current_address = Pointer::apply_pointer_offset(pointer_value, *pointer_offset)?;
            current_module_name.clear();
        }

        Some((current_address, current_module_name))
    }

    fn read_pointer_value(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        address: u64,
        module_name: &str,
        pointer_size: PointerScanPointerSize,
    ) -> Option<u64> {
        let symbolic_struct_definition = squalr_engine_api::structures::structs::symbolic_struct_definition::SymbolicStructDefinition::new_anonymous(vec![
            squalr_engine_api::structures::structs::symbolic_field_definition::SymbolicFieldDefinition::new(
                pointer_size.to_data_type_ref(),
                ContainerType::None,
            ),
        ]);
        let memory_read_response = Self::dispatch_memory_read_request(engine_execution_context, address, module_name, &symbolic_struct_definition)?;

        if !memory_read_response.success {
            return None;
        }

        let data_value = memory_read_response
            .valued_struct
            .get_fields()
            .first()
            .and_then(|valued_struct_field| valued_struct_field.get_data_value())?;

        pointer_size.read_address_value(data_value)
    }

    fn dispatch_memory_read_request(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        address: u64,
        module_name: &str,
        symbolic_struct_definition: &squalr_engine_api::structures::structs::symbolic_struct_definition::SymbolicStructDefinition,
    ) -> Option<MemoryReadResponse> {
        let memory_read_request = MemoryReadRequest {
            address,
            module_name: module_name.to_string(),
            symbolic_struct_definition: symbolic_struct_definition.clone(),
            suppress_logging: true,
        };
        let memory_read_command = memory_read_request.to_engine_command();
        let (memory_read_response_sender, memory_read_response_receiver) = mpsc::channel();

        let dispatch_result = match engine_execution_context.get_bindings().read() {
            Ok(engine_bindings) => engine_bindings.dispatch_privileged_command(
                memory_read_command,
                Box::new(move |engine_response| {
                    let conversion_result = match MemoryReadResponse::from_engine_response(engine_response) {
                        Ok(memory_read_response) => Ok(memory_read_response),
                        Err(unexpected_response) => Err(format!(
                            "Unexpected response variant for project hierarchy memory read request: {:?}",
                            unexpected_response
                        )),
                    };
                    let _ = memory_read_response_sender.send(conversion_result);
                }),
            ),
            Err(error) => {
                log::error!("Failed to acquire engine bindings lock for project hierarchy memory read request: {}", error);
                return None;
            }
        };

        if let Err(error) = dispatch_result {
            log::error!("Failed to dispatch project hierarchy memory read request: {}", error);
            return None;
        }

        match memory_read_response_receiver.recv_timeout(Duration::from_secs(2)) {
            Ok(Ok(memory_read_response)) => Some(memory_read_response),
            Ok(Err(error)) => {
                log::error!("Failed to convert project hierarchy memory read response: {}", error);
                None
            }
            Err(error) => {
                log::error!("Timed out waiting for project hierarchy memory read response: {}", error);
                None
            }
        }
    }

    fn dispatch_memory_query_request(engine_unprivileged_state: &Arc<EngineUnprivilegedState>) -> Option<MemoryQueryResponse> {
        let memory_query_request = MemoryQueryRequest::default();
        let memory_query_command = memory_query_request.to_engine_command();
        let (memory_query_response_sender, memory_query_response_receiver) = mpsc::channel();

        let dispatch_result = match engine_unprivileged_state.get_bindings().read() {
            Ok(engine_bindings) => engine_bindings.dispatch_privileged_command(
                memory_query_command,
                Box::new(move |engine_response| {
                    let conversion_result = match MemoryQueryResponse::from_engine_response(engine_response) {
                        Ok(memory_query_response) => Ok(memory_query_response),
                        Err(unexpected_response) => Err(format!(
                            "Unexpected response variant for project hierarchy memory query request: {:?}",
                            unexpected_response
                        )),
                    };
                    let _ = memory_query_response_sender.send(conversion_result);
                }),
            ),
            Err(error) => {
                log::error!("Failed to acquire engine bindings lock for project hierarchy memory query request: {}", error);
                return None;
            }
        };

        if let Err(error) = dispatch_result {
            log::error!("Failed to dispatch project hierarchy memory query request: {}", error);
            return None;
        }

        match memory_query_response_receiver.recv_timeout(Duration::from_secs(2)) {
            Ok(Ok(memory_query_response)) => Some(memory_query_response),
            Ok(Err(error)) => {
                log::error!("Failed to convert project hierarchy memory query response: {}", error);
                None
            }
            Err(error) => {
                log::error!("Timed out waiting for project hierarchy memory query response: {}", error);
                None
            }
        }
    }

    fn resolve_module_relative_address(
        modules: &[NormalizedModule],
        address: u64,
        module_name: &str,
    ) -> Option<u64> {
        modules
            .iter()
            .find(|normalized_module| {
                normalized_module
                    .get_module_name()
                    .eq_ignore_ascii_case(module_name)
            })
            .and_then(|normalized_module| normalized_module.get_base_address().checked_add(address))
    }

    fn focus_pointer_scanner_for_address(
        &self,
        address: u64,
        module_name: &str,
        data_type_id: &str,
    ) {
        let (resolved_target_address, resolved_target_module_name) = if module_name.trim().is_empty() {
            (address, String::new())
        } else if try_resolve_virtual_module_address(module_name, address).is_some() {
            (address, module_name.to_string())
        } else if let Some(resolved_absolute_address) = Self::dispatch_memory_query_request(&self.app_context.engine_unprivileged_state)
            .and_then(|memory_query_response| Self::resolve_module_relative_address(&memory_query_response.modules, address, module_name))
        {
            (resolved_absolute_address, String::new())
        } else {
            log::warn!(
                "Failed to resolve pointer scanner target for module-relative address {}+0x{:X}; falling back to unresolved offset.",
                module_name,
                address
            );
            (address, module_name.to_string())
        };

        PointerScannerViewData::set_scan_target_from_project_address(
            self.pointer_scanner_view_data.clone(),
            resolved_target_address,
            &resolved_target_module_name,
            data_type_id,
        );

        match self.app_context.docking_manager.write() {
            Ok(mut docking_manager) => {
                docking_manager.set_window_visibility(PointerScannerView::WINDOW_ID, true);
                docking_manager.select_tab_by_window_id(PointerScannerView::WINDOW_ID);
            }
            Err(error) => {
                log::error!("Failed to acquire docking manager while opening the pointer scanner: {}", error);
            }
        }
    }

    fn focus_memory_viewer_for_address(
        &self,
        address: u64,
        module_name: &str,
        selection_byte_count: u64,
    ) {
        MemoryViewerViewData::request_focus_address_range(
            self.memory_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            address,
            module_name.to_string(),
            selection_byte_count,
        );

        match self.app_context.docking_manager.write() {
            Ok(mut docking_manager) => {
                docking_manager.set_window_visibility(MemoryViewerView::WINDOW_ID, true);
                docking_manager.select_tab_by_window_id(MemoryViewerView::WINDOW_ID);
            }
            Err(error) => {
                log::error!("Failed to acquire docking manager while opening the memory viewer: {}", error);
            }
        }
    }

    fn focus_code_viewer_for_address(
        &self,
        address: u64,
        module_name: &str,
    ) {
        CodeViewerViewData::request_focus_address(
            self.code_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            address,
            module_name.to_string(),
        );

        match self.app_context.docking_manager.write() {
            Ok(mut docking_manager) => {
                docking_manager.set_window_visibility(CodeViewerView::WINDOW_ID, true);
                docking_manager.select_tab_by_window_id(CodeViewerView::WINDOW_ID);
            }
            Err(error) => {
                log::error!("Failed to acquire docking manager while opening the code viewer: {}", error);
            }
        }
    }

    fn should_open_project_item_in_code_viewer(
        opened_project_info: Option<&ProjectInfo>,
        project_item: &ProjectItem,
    ) -> bool {
        Self::resolve_project_item_symbolic_struct_namespace(opened_project_info, project_item)
            .and_then(|symbolic_struct_namespace| normalize_instruction_data_type_id(&symbolic_struct_namespace))
            .map(|data_type_id| matches!(data_type_id.as_str(), "i_x86" | "i_x64"))
            .unwrap_or(false)
    }

    fn resolve_tree_entry_icon(
        app_context: Arc<AppContext>,
        opened_project_info: Option<&ProjectInfo>,
        project_item: &ProjectItem,
    ) -> Option<TextureHandle> {
        let icon_library = &app_context.theme.icon_library;
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id == ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID {
            Some(icon_library.icon_handle_file_system_open_folder.clone())
        } else {
            let icon_data_type_id = Self::resolve_project_item_icon_data_type_id(opened_project_info, project_item).unwrap_or_default();

            Some(DataTypeToIconConverter::convert_data_type_to_icon(&icon_data_type_id, icon_library))
        }
    }

    fn resolve_project_item_icon_data_type_id(
        opened_project_info: Option<&ProjectInfo>,
        project_item: &ProjectItem,
    ) -> Option<String> {
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            let mut address_project_item = project_item.clone();

            return ProjectItemTypeAddress::get_field_symbolic_struct_definition_reference(&mut address_project_item).map(|symbolic_struct_reference| {
                symbolic_struct_reference
                    .get_symbolic_struct_namespace()
                    .to_string()
            });
        }

        if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            return ProjectItemTypePointer::get_field_symbolic_struct_definition_reference(project_item).map(|symbolic_struct_reference| {
                symbolic_struct_reference
                    .get_symbolic_struct_namespace()
                    .to_string()
            });
        }

        if project_item_type_id == ProjectItemTypeSymbolRef::PROJECT_ITEM_TYPE_ID {
            return Self::resolve_project_rooted_symbol(opened_project_info, project_item)
                .map(|rooted_symbol| rooted_symbol.get_struct_layout_id().to_string());
        }

        None
    }

    fn refresh_if_project_changed(&self) {
        let (opened_project_directory_path, opened_project_item_paths, opened_project_sort_order) = match self
            .app_context
            .engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .read()
        {
            Ok(opened_project_guard) => opened_project_guard
                .as_ref()
                .map(|opened_project| {
                    let opened_project_directory_path = opened_project.get_project_info().get_project_directory();
                    let opened_project_item_paths = opened_project
                        .get_project_items()
                        .keys()
                        .map(|project_item_ref| project_item_ref.get_project_item_path().clone())
                        .collect::<HashSet<PathBuf>>();
                    let opened_project_sort_order = opened_project
                        .get_project_info()
                        .get_project_manifest()
                        .get_project_item_sort_order()
                        .iter()
                        .cloned()
                        .collect::<Vec<PathBuf>>();

                    (opened_project_directory_path, opened_project_item_paths, opened_project_sort_order)
                })
                .unwrap_or((None, HashSet::new(), Vec::new())),
            Err(error) => {
                log::error!("Failed to acquire opened project lock for hierarchy refresh check: {}", error);
                (None, HashSet::new(), Vec::new())
            }
        };

        let (loaded_project_directory_path, loaded_project_item_paths, loaded_project_sort_order) = self
            .project_hierarchy_view_data
            .read("Project hierarchy refresh check")
            .map(|project_hierarchy_view_data| {
                let loaded_project_directory_path = project_hierarchy_view_data
                    .opened_project_info
                    .as_ref()
                    .and_then(|project_info| project_info.get_project_directory());
                let loaded_project_item_paths = project_hierarchy_view_data
                    .project_items
                    .iter()
                    .map(|(project_item_ref, _)| project_item_ref.get_project_item_path().clone())
                    .collect::<HashSet<PathBuf>>();
                let loaded_project_sort_order = project_hierarchy_view_data
                    .opened_project_info
                    .as_ref()
                    .map(|project_info| {
                        project_info
                            .get_project_manifest()
                            .get_project_item_sort_order()
                            .iter()
                            .cloned()
                            .collect::<Vec<PathBuf>>()
                    })
                    .unwrap_or_default();

                (loaded_project_directory_path, loaded_project_item_paths, loaded_project_sort_order)
            })
            .unwrap_or((None, HashSet::new(), Vec::new()));

        let project_directory_changed = opened_project_directory_path != loaded_project_directory_path;
        let project_items_changed = opened_project_item_paths != loaded_project_item_paths;
        let sort_order_changed = opened_project_sort_order != loaded_project_sort_order;

        if project_directory_changed || project_items_changed || sort_order_changed {
            ProjectHierarchyViewData::refresh_project_items(self.project_hierarchy_view_data.clone(), self.app_context.clone());
        }
    }

    fn extract_string_value_from_edited_field(edited_field: &ValuedStructField) -> Option<String> {
        let data_value = edited_field.get_data_value()?;
        let edited_name = String::from_utf8(data_value.get_value_bytes().clone()).ok()?;
        let edited_name = edited_name.trim();

        if edited_name.is_empty() { None } else { Some(edited_name.to_string()) }
    }

    fn project_item_rename_text_storage_id(project_item_path: &Path) -> Id {
        Id::new(("project_hierarchy_rename_text", project_item_path.to_path_buf()))
    }

    fn project_item_rename_highlight_storage_id(project_item_path: &Path) -> Id {
        Id::new(("project_hierarchy_rename_highlight", project_item_path.to_path_buf()))
    }

    fn clear_project_item_rename_state(
        user_interface: &Ui,
        project_item_path: &Path,
    ) {
        let rename_text_storage_id = Self::project_item_rename_text_storage_id(project_item_path);
        let rename_highlight_storage_id = Self::project_item_rename_highlight_storage_id(project_item_path);

        user_interface.ctx().data_mut(|data| {
            data.remove::<String>(rename_text_storage_id);
            data.remove::<bool>(rename_highlight_storage_id);
        });
    }

    fn clear_active_project_item_rename_state(
        user_interface: &Ui,
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
    ) {
        let active_project_item_path = project_hierarchy_view_data
            .read("Project hierarchy resolve active inline rename state")
            .and_then(|project_hierarchy_view_data| match &project_hierarchy_view_data.take_over_state {
                ProjectHierarchyTakeOverState::RenameProjectItem { project_item_path, .. } => Some(project_item_path.clone()),
                _ => None,
            });

        if let Some(active_project_item_path) = active_project_item_path {
            Self::clear_project_item_rename_state(user_interface, &active_project_item_path);
        }
    }

    fn project_item_value_edit_storage_id(project_item_path: &Path) -> Id {
        Id::new(("project_hierarchy_value_edit", project_item_path.to_path_buf()))
    }

    fn clear_project_item_value_edit_state(
        user_interface: &Ui,
        project_item_path: &Path,
    ) {
        let value_edit_storage_id = Self::project_item_value_edit_storage_id(project_item_path);

        user_interface.ctx().data_mut(|data| {
            data.remove::<AnonymousValueString>(value_edit_storage_id);
        });
    }

    fn build_project_item_value_edit_context(
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
        opened_project_info: Option<&ProjectInfo>,
        project_item: &ProjectItem,
    ) -> Option<ProjectItemValueEditContext> {
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();
        let value_field_name = if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE
        } else if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            ProjectItemTypePointer::PROPERTY_FREEZE_DISPLAY_VALUE
        } else if project_item_type_id == ProjectItemTypeSymbolRef::PROJECT_ITEM_TYPE_ID {
            ProjectItemTypeSymbolRef::PROPERTY_FREEZE_DISPLAY_VALUE
        } else {
            return None;
        };
        let value_field = project_item.get_properties().get_field(value_field_name)?;
        let value_data_value = value_field.get_data_value()?;
        let symbolic_struct_namespace = Self::resolve_project_item_symbolic_struct_namespace(opened_project_info, project_item);
        let symbolic_field_definition = symbolic_struct_namespace
            .as_deref()
            .and_then(|symbolic_struct_namespace| SymbolicFieldDefinition::from_str(symbolic_struct_namespace).ok());
        let validation_data_type_ref = symbolic_field_definition
            .as_ref()
            .map(|symbolic_field_definition| symbolic_field_definition.get_data_type_ref().clone())
            .unwrap_or_else(|| value_data_value.get_data_type_ref().clone());
        let container_type = symbolic_field_definition
            .map(|symbolic_field_definition| symbolic_field_definition.get_container_type())
            .unwrap_or(ContainerType::None);
        let default_format = engine_unprivileged_state.get_default_anonymous_value_string_format(&validation_data_type_ref);
        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let initial_value_edit = symbolic_struct_namespace
            .as_deref()
            .and_then(|symbolic_struct_namespace| {
                Self::read_project_item_runtime_value_from_memory(&engine_execution_context, opened_project_info, project_item, symbolic_struct_namespace)
            })
            .unwrap_or_else(|| {
                let raw_display_value = String::from_utf8(value_data_value.get_value_bytes().clone()).unwrap_or_default();

                AnonymousValueString::new(raw_display_value, default_format, container_type)
            });

        Some(ProjectItemValueEditContext {
            project_item_name: project_item.get_field_name(),
            value_field_name: value_field_name.to_string(),
            validation_data_type_ref,
            initial_value_edit,
        })
    }

    fn read_project_item_runtime_value_from_memory(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        opened_project_info: Option<&ProjectInfo>,
        project_item: &ProjectItem,
        symbolic_struct_namespace: &str,
    ) -> Option<AnonymousValueString> {
        let (address, module_name) = Self::resolve_project_item_runtime_value_target(engine_execution_context, opened_project_info, project_item)?;
        let symbolic_struct_definition = engine_execution_context.resolve_struct_layout_definition(symbolic_struct_namespace)?;
        let memory_read_response = Self::dispatch_memory_read_request(engine_execution_context, address, &module_name, &symbolic_struct_definition)?;

        if !memory_read_response.success {
            return None;
        }

        let read_data_value = memory_read_response
            .valued_struct
            .get_fields()
            .first()
            .and_then(|valued_struct_field| valued_struct_field.get_data_value())?;
        let default_format = engine_execution_context.get_default_anonymous_value_string_format(read_data_value.get_data_type_ref());

        engine_execution_context
            .anonymize_value(read_data_value, default_format)
            .ok()
    }

    fn resolve_project_item_runtime_value_target(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        opened_project_info: Option<&ProjectInfo>,
        project_item: &ProjectItem,
    ) -> Option<(u64, String)> {
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            let mut project_item = project_item.clone();

            return Some((
                ProjectItemTypeAddress::get_field_address(&mut project_item),
                ProjectItemTypeAddress::get_field_module(&mut project_item),
            ));
        }

        if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            let pointer = ProjectItemTypePointer::get_field_pointer(project_item);

            return Self::resolve_pointer_write_target(engine_execution_context, &pointer);
        }

        if project_item_type_id == ProjectItemTypeSymbolRef::PROJECT_ITEM_TYPE_ID {
            let rooted_symbol = Self::resolve_project_rooted_symbol(opened_project_info, project_item)?;

            return Some((
                rooted_symbol.get_root_locator().get_focus_address(),
                rooted_symbol
                    .get_root_locator()
                    .get_focus_module_name()
                    .to_string(),
            ));
        }

        None
    }

    fn build_project_item_value_edit_display_values(
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
        validation_data_type_ref: &DataTypeRef,
        value_edit: &AnonymousValueString,
    ) -> Vec<AnonymousValueString> {
        let Ok(data_value) = engine_unprivileged_state.deanonymize_value_string(validation_data_type_ref, value_edit) else {
            return Vec::new();
        };

        engine_unprivileged_state
            .anonymize_value_to_supported_formats(&data_value)
            .unwrap_or_else(|_| vec![value_edit.clone()])
    }

    fn build_project_item_virtual_snapshot_query(
        opened_project_info: Option<&ProjectInfo>,
        project_item_path: &Path,
        project_item: &ProjectItem,
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
    ) -> Option<VirtualSnapshotQuery> {
        let query_id = project_item_path.to_string_lossy().to_string();
        let symbolic_struct_namespace = Self::resolve_project_item_symbolic_struct_namespace(opened_project_info, project_item)?;
        let symbolic_struct_definition = Self::build_project_item_preview_symbolic_struct_definition(engine_unprivileged_state, &symbolic_struct_namespace)?;
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            let mut project_item = project_item.clone();

            return Some(VirtualSnapshotQuery::Address {
                query_id,
                address: ProjectItemTypeAddress::get_field_address(&mut project_item),
                module_name: ProjectItemTypeAddress::get_field_module(&mut project_item),
                symbolic_struct_definition,
            });
        }

        if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            return Some(VirtualSnapshotQuery::Pointer {
                query_id,
                pointer: ProjectItemTypePointer::get_field_pointer(project_item),
                symbolic_struct_definition,
            });
        }

        if project_item_type_id == ProjectItemTypeSymbolRef::PROJECT_ITEM_TYPE_ID {
            let rooted_symbol = Self::resolve_project_rooted_symbol(opened_project_info, project_item)?;

            return Some(VirtualSnapshotQuery::Address {
                query_id,
                address: rooted_symbol.get_root_locator().get_focus_address(),
                module_name: rooted_symbol
                    .get_root_locator()
                    .get_focus_module_name()
                    .to_string(),
                symbolic_struct_definition,
            });
        }

        None
    }

    fn build_project_item_preview_symbolic_struct_definition(
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
        symbolic_struct_namespace: &str,
    ) -> Option<SymbolicStructDefinition> {
        let symbolic_struct_definition = engine_unprivileged_state.resolve_struct_layout_definition(symbolic_struct_namespace)?;
        let preview_field_definition = SymbolicFieldDefinition::from_str(symbolic_struct_namespace).ok();

        let Some(preview_field_definition) = preview_field_definition else {
            return Some(symbolic_struct_definition);
        };

        let preview_container_type = match preview_field_definition.get_container_type() {
            ContainerType::ArrayFixed(length) if length > Self::MAX_PROJECT_ITEM_PREVIEW_ELEMENT_COUNT => {
                ContainerType::ArrayFixed(Self::MAX_PROJECT_ITEM_PREVIEW_ELEMENT_COUNT)
            }
            container_type => container_type,
        };

        if preview_container_type == preview_field_definition.get_container_type() {
            Some(symbolic_struct_definition)
        } else {
            Some(SymbolicStructDefinition::new_anonymous(vec![SymbolicFieldDefinition::new(
                preview_field_definition.get_data_type_ref().clone(),
                preview_container_type,
            )]))
        }
    }

    fn build_project_item_preview_value_from_virtual_snapshot_result(
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
        opened_project_info: Option<&ProjectInfo>,
        project_item: &ProjectItem,
        virtual_snapshot_query_result: &VirtualSnapshotQueryResult,
    ) -> String {
        let Some(memory_read_response) = virtual_snapshot_query_result.memory_read_response.as_ref() else {
            return String::new();
        };

        if !memory_read_response.success {
            return String::new();
        }

        let first_read_field_data_value = memory_read_response
            .valued_struct
            .get_fields()
            .first()
            .and_then(|valued_struct_field| valued_struct_field.get_data_value());
        let Some(first_read_field_data_value) = first_read_field_data_value else {
            return String::new();
        };

        let default_anonymous_value_string_format =
            engine_unprivileged_state.get_default_anonymous_value_string_format(first_read_field_data_value.get_data_type_ref());
        let symbolic_field_container_type = Self::resolve_project_item_symbolic_container_type(opened_project_info, project_item);
        let preview_was_truncated = Self::project_item_preview_was_truncated(opened_project_info, project_item);

        engine_unprivileged_state
            .anonymize_value(first_read_field_data_value, default_anonymous_value_string_format)
            .map(|anonymous_value_string| {
                Self::format_project_item_preview_value(&anonymous_value_string, symbolic_field_container_type, preview_was_truncated)
            })
            .unwrap_or_default()
    }

    fn resolve_project_rooted_symbol<'a>(
        opened_project_info: Option<&'a ProjectInfo>,
        project_item: &ProjectItem,
    ) -> Option<&'a ProjectRootSymbol> {
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id != ProjectItemTypeSymbolRef::PROJECT_ITEM_TYPE_ID {
            return None;
        }

        let symbol_key = ProjectItemTypeSymbolRef::get_field_symbol_key(project_item);
        let project_symbol_catalog = opened_project_info?.get_project_symbol_catalog();

        project_symbol_catalog.find_rooted_symbol(&symbol_key)
    }

    fn resolve_project_item_symbolic_struct_namespace(
        opened_project_info: Option<&ProjectInfo>,
        project_item: &ProjectItem,
    ) -> Option<String> {
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            let mut project_item = project_item.clone();

            return ProjectItemTypeAddress::get_field_symbolic_struct_definition_reference(&mut project_item).map(|symbolic_struct_reference| {
                symbolic_struct_reference
                    .get_symbolic_struct_namespace()
                    .to_string()
            });
        }

        if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            return ProjectItemTypePointer::get_field_symbolic_struct_definition_reference(project_item).map(|symbolic_struct_reference| {
                symbolic_struct_reference
                    .get_symbolic_struct_namespace()
                    .to_string()
            });
        }

        if project_item_type_id == ProjectItemTypeSymbolRef::PROJECT_ITEM_TYPE_ID {
            return Self::resolve_project_rooted_symbol(opened_project_info, project_item)
                .map(|rooted_symbol| rooted_symbol.get_struct_layout_id().to_string());
        }

        None
    }

    fn resolve_project_item_symbolic_container_type(
        opened_project_info: Option<&ProjectInfo>,
        project_item: &ProjectItem,
    ) -> ContainerType {
        Self::resolve_project_item_symbolic_struct_namespace(opened_project_info, project_item)
            .and_then(|symbolic_struct_namespace| SymbolicFieldDefinition::from_str(&symbolic_struct_namespace).ok())
            .map(|symbolic_field_definition| symbolic_field_definition.get_container_type())
            .unwrap_or(ContainerType::None)
    }

    fn project_item_preview_was_truncated(
        opened_project_info: Option<&ProjectInfo>,
        project_item: &ProjectItem,
    ) -> bool {
        let Some(symbolic_struct_namespace) = Self::resolve_project_item_symbolic_struct_namespace(opened_project_info, project_item) else {
            return false;
        };
        let Some(symbolic_field_definition) = SymbolicFieldDefinition::from_str(&symbolic_struct_namespace).ok() else {
            return false;
        };

        matches!(
            symbolic_field_definition.get_container_type(),
            ContainerType::ArrayFixed(length) if length > Self::MAX_PROJECT_ITEM_PREVIEW_ELEMENT_COUNT
        )
    }

    fn resolve_project_item_runtime_value_byte_count(
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
        opened_project_info: Option<&ProjectInfo>,
        project_item: &ProjectItem,
    ) -> Option<u64> {
        let symbolic_struct_namespace = Self::resolve_project_item_symbolic_struct_namespace(opened_project_info, project_item)?;
        let symbolic_field_definition = SymbolicFieldDefinition::from_str(&symbolic_struct_namespace).ok()?;
        let unit_size_in_bytes = engine_unprivileged_state
            .get_default_value(symbolic_field_definition.get_data_type_ref())
            .map(|default_value| default_value.get_size_in_bytes())
            .unwrap_or(1);

        Some(
            symbolic_field_definition
                .get_container_type()
                .get_total_size_in_bytes(unit_size_in_bytes),
        )
    }

    fn format_project_item_preview_value(
        anonymous_value_string: &AnonymousValueString,
        symbolic_field_container_type: ContainerType,
        preview_was_truncated: bool,
    ) -> String {
        let effective_container_type = if matches!(anonymous_value_string.get_container_type(), ContainerType::Array | ContainerType::ArrayFixed(_)) {
            anonymous_value_string.get_container_type()
        } else {
            symbolic_field_container_type
        };
        let display_value = anonymous_value_string.get_anonymous_value_string();

        if matches!(effective_container_type, ContainerType::Array | ContainerType::ArrayFixed(_)) && !display_value.is_empty() {
            let preview_value = if preview_was_truncated {
                Self::append_project_item_preview_ellipsis(display_value)
            } else {
                Self::truncate_project_item_preview_value(display_value)
            };

            format!("[{}]", preview_value)
        } else {
            display_value.to_string()
        }
    }

    fn append_project_item_preview_ellipsis(display_value: &str) -> String {
        if let Some(truncated_array_preview) = Self::format_project_item_preview_from_elements(display_value, true) {
            return truncated_array_preview;
        }

        let trimmed_display_value = display_value.trim_end_matches(|character: char| character.is_ascii_whitespace() || matches!(character, ',' | ';'));

        if trimmed_display_value.is_empty() {
            String::from("...")
        } else {
            format!("{}...", trimmed_display_value)
        }
    }

    fn truncate_project_item_preview_value(display_value: &str) -> String {
        if let Some(truncated_array_preview) = Self::format_project_item_preview_from_elements(display_value, false) {
            return truncated_array_preview;
        }

        let display_value_character_count = display_value.chars().count();

        if display_value_character_count <= Self::MAX_PROJECT_ITEM_PREVIEW_ARRAY_CHARACTER_COUNT {
            return display_value.to_string();
        }

        let truncated_prefix: String = display_value
            .chars()
            .take(Self::MAX_PROJECT_ITEM_PREVIEW_ARRAY_CHARACTER_COUNT)
            .collect::<String>()
            .trim_end_matches(|character: char| character.is_ascii_whitespace() || matches!(character, ',' | ';'))
            .to_string();

        format!("{}...", truncated_prefix)
    }

    fn format_project_item_preview_from_elements(
        display_value: &str,
        force_ellipsis: bool,
    ) -> Option<String> {
        let array_elements = Self::split_project_item_preview_elements(display_value);

        if array_elements.len() <= 1 {
            return None;
        }

        let visible_element_count = array_elements
            .len()
            .min(Self::MAX_PROJECT_ITEM_PREVIEW_DISPLAY_ELEMENT_COUNT);
        let mut preview_elements = array_elements
            .iter()
            .take(visible_element_count)
            .map(|array_element| (*array_element).to_string())
            .collect::<Vec<_>>();
        let has_hidden_elements = force_ellipsis || array_elements.len() > visible_element_count;

        if has_hidden_elements {
            preview_elements.push(String::from("..."));
        }

        Some(preview_elements.join(", "))
    }

    fn split_project_item_preview_elements(display_value: &str) -> Vec<&str> {
        display_value
            .split([',', ';'])
            .map(str::trim)
            .filter(|array_element| !array_element.is_empty())
            .collect::<Vec<_>>()
    }

    fn build_project_item_rename_request(
        project_item_path: &Path,
        project_item_type_id: &str,
        edited_name: &str,
    ) -> Option<ProjectItemsRenameRequest> {
        let sanitized_file_name = Path::new(edited_name)
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .map(str::trim)
            .filter(|file_name| !file_name.is_empty())?
            .to_string();
        let is_directory_project_item = project_item_type_id == ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID;
        let renamed_project_item_name = if is_directory_project_item {
            sanitized_file_name
        } else {
            let mut file_name_with_extension = sanitized_file_name.clone();
            let expected_extension = Project::PROJECT_ITEM_EXTENSION.trim_start_matches('.');
            let has_expected_extension = Path::new(&sanitized_file_name)
                .extension()
                .and_then(|extension| extension.to_str())
                .map(|extension| extension.eq_ignore_ascii_case(expected_extension))
                .unwrap_or(false);

            if !has_expected_extension {
                file_name_with_extension.push('.');
                file_name_with_extension.push_str(expected_extension);
            }

            file_name_with_extension
        };
        let current_file_name = project_item_path
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .unwrap_or_default();

        if current_file_name == renamed_project_item_name {
            return None;
        }

        Some(ProjectItemsRenameRequest {
            project_item_path: project_item_path.to_path_buf(),
            project_item_name: renamed_project_item_name,
        })
    }

    fn should_apply_struct_field_edit_to_project_item(
        project_item_type_id: &str,
        edited_field_name: &str,
    ) -> bool {
        if Self::is_runtime_value_field(edited_field_name) {
            return false;
        }

        !(edited_field_name == ProjectItem::PROPERTY_NAME && project_item_type_id == ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID)
    }
}
