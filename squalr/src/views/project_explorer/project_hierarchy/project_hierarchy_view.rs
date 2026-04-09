use crate::{
    app_context::AppContext,
    ui::{
        converters::data_type_to_icon_converter::DataTypeToIconConverter,
        widgets::controls::{context_menu::context_menu::ContextMenu, toolbar_menu::toolbar_menu_item_view::ToolbarMenuItemView},
    },
    views::pointer_scanner::{pointer_scanner_view::PointerScannerView, view_data::pointer_scanner_view_data::PointerScannerViewData},
    views::project_explorer::project_hierarchy::{
        project_hierarchy_toolbar_view::ProjectHierarchyToolbarView,
        project_item_entry_view::ProjectItemEntryView,
        view_data::{
            project_hierarchy_create_item_kind::ProjectHierarchyCreateItemKind, project_hierarchy_drop_target::ProjectHierarchyDropTarget,
            project_hierarchy_frame_action::ProjectHierarchyFrameAction, project_hierarchy_menu_target::ProjectHierarchyMenuTarget,
            project_hierarchy_pending_operation::ProjectHierarchyPendingOperation, project_hierarchy_take_over_state::ProjectHierarchyTakeOverState,
            project_hierarchy_tree_entry::ProjectHierarchyTreeEntry, project_hierarchy_view_data::ProjectHierarchyViewData,
        },
    },
    views::struct_viewer::view_data::struct_viewer_view_data::StructViewerViewData,
};
use eframe::egui::{vec2, Align, CursorIcon, Layout, Pos2, Rect, Response, RichText, ScrollArea, TextureHandle, Ui, Widget};
use epaint::{CornerRadius, Stroke, StrokeKind};
use squalr_engine_api::commands::memory::read::memory_read_request::MemoryReadRequest;
use squalr_engine_api::commands::memory::read::memory_read_response::MemoryReadResponse;
use squalr_engine_api::commands::memory::write::memory_write_request::MemoryWriteRequest;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use squalr_engine_api::commands::project::save::project_save_request::ProjectSaveRequest;
use squalr_engine_api::commands::project_items::rename::project_items_rename_request::ProjectItemsRenameRequest;
use squalr_engine_api::commands::settings::scan::list::scan_settings_list_request::ScanSettingsListRequest;
use squalr_engine_api::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::data_values::container_type::ContainerType;
use squalr_engine_api::structures::memory::pointer::Pointer;
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_api::structures::projects::project_items::built_in_types::{
    project_item_type_address::ProjectItemTypeAddress, project_item_type_directory::ProjectItemTypeDirectory, project_item_type_pointer::ProjectItemTypePointer,
};
use squalr_engine_api::structures::projects::project_items::{project_item::ProjectItem, project_item_ref::ProjectItemRef};
use squalr_engine_api::structures::structs::valued_struct::ValuedStruct;
use squalr_engine_api::structures::structs::valued_struct_field::ValuedStructField;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct ProjectHierarchyView {
    app_context: Arc<AppContext>,
    project_hierarchy_toolbar_view: ProjectHierarchyToolbarView,
    project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
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

impl ProjectHierarchyView {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let project_hierarchy_view_data = app_context
            .dependency_container
            .get_dependency::<ProjectHierarchyViewData>();
        let project_hierarchy_toolbar_view = ProjectHierarchyToolbarView::new(app_context.clone());
        let struct_viewer_view_data = app_context
            .dependency_container
            .get_dependency::<StructViewerViewData>();
        let pointer_scanner_view_data = app_context
            .dependency_container
            .get_dependency::<PointerScannerViewData>();
        ProjectHierarchyViewData::refresh_project_items(project_hierarchy_view_data.clone(), app_context.clone());

        Self {
            app_context,
            project_hierarchy_toolbar_view,
            project_hierarchy_view_data,
            pointer_scanner_view_data,
            struct_viewer_view_data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectHierarchyView;
    use crossbeam_channel::{unbounded, Receiver};
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
    use squalr_engine_api::structures::memory::pointer::Pointer;
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

    fn create_execution_context(mock_memory_read_bindings: MockMemoryReadBindings) -> Arc<dyn EngineExecutionContext> {
        let engine_bindings: Arc<RwLock<dyn EngineApiUnprivilegedBindings>> = Arc::new(RwLock::new(mock_memory_read_bindings));

        EngineUnprivilegedState::new_with_options(engine_bindings, EngineUnprivilegedStateOptions { enable_console_logging: false })
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
    fn build_pointer_scanner_context_actions_returns_address_item_values() {
        let project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU64::get_value_from_primitive(0));

        let pointer_scanner_context_actions = ProjectHierarchyView::build_pointer_scanner_context_actions(&project_item);

        assert_eq!(
            pointer_scanner_context_actions,
            vec![super::PointerScannerContextAction::Address {
                label: "Pointer Scan",
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

        let pointer_scanner_context_actions = ProjectHierarchyView::build_pointer_scanner_context_actions(&project_item);

        assert_eq!(
            pointer_scanner_context_actions,
            vec![
                super::PointerScannerContextAction::Address {
                    label: "Pointer Scan for Base Address",
                    address: 0x1000,
                    module_name: "game.exe".to_string(),
                    data_type_id: "u16".to_string(),
                },
                super::PointerScannerContextAction::ResolvedPointer {
                    label: "Pointer Scan for Resolved Address",
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

        let pointer_scanner_context_actions = ProjectHierarchyView::build_pointer_scanner_context_actions(&project_item);

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
            label: "Pointer Scan for Resolved Address",
            pointer,
            data_type_id: "u16".to_string(),
        };

        let resolved_pointer_target = ProjectHierarchyView::resolve_pointer_scanner_context_action(&engine_execution_context, &resolved_pointer_action);

        assert_eq!(resolved_pointer_target, Some((0x2FF0, String::new(), "u16".to_string())));
    }

    #[test]
    fn build_struct_view_properties_exposes_runtime_value_field_as_editable() {
        let project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU64::get_value_from_primitive(0));

        let struct_view_properties = ProjectHierarchyView::build_struct_view_properties(&project_item);
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
            ProjectHierarchyView::build_memory_write_request_for_runtime_value_edit(&engine_execution_context, &address_project_item, &edited_field);

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
            ProjectHierarchyView::build_memory_write_request_for_runtime_value_edit(&engine_execution_context, &pointer_project_item, &edited_field);

        assert!(memory_write_request.is_some());
        let memory_write_request = memory_write_request.unwrap_or_else(|| panic!("Expected runtime value memory write request for pointer project item."));
        assert_eq!(memory_write_request.address, 0x2FF0);
        assert_eq!(memory_write_request.module_name, "");
        assert_eq!(memory_write_request.value, 0x1234u16.to_le_bytes().to_vec());
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
        let mut keyboard_activation_toggle_target: Option<(Vec<PathBuf>, bool)> = None;
        let mut is_delete_confirmation_active = false;
        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
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
                    ProjectHierarchyPendingOperation::Deleting => {
                        user_interface.label("Deleting project item(s)...");
                    }
                    ProjectHierarchyPendingOperation::Reordering => {
                        user_interface.label("Reordering project item(s)...");
                    }
                    _ => {}
                }

                match take_over_state {
                    ProjectHierarchyTakeOverState::None => {
                        ScrollArea::vertical()
                            .id_salt("project_hierarchy")
                            .auto_shrink([false, false])
                            .show(user_interface, |user_interface| {
                                for tree_entry in &tree_entries {
                                    let is_selected = selected_project_item_paths.contains(&tree_entry.project_item_path);
                                    let icon = Self::resolve_tree_entry_icon(self.app_context.clone(), &tree_entry.project_item);

                                    let row_response = user_interface.add(ProjectItemEntryView::new(
                                        self.app_context.clone(),
                                        &tree_entry.project_item_path,
                                        &tree_entry.display_name,
                                        &tree_entry.preview_path,
                                        &tree_entry.preview_value,
                                        tree_entry.is_activated,
                                        tree_entry.depth,
                                        icon,
                                        is_selected,
                                        tree_entry.is_directory,
                                        tree_entry.has_children,
                                        tree_entry.is_expanded,
                                        &mut project_hierarchy_frame_action,
                                    ));

                                    if row_response.drag_started() {
                                        drag_started_project_item_path = Some(tree_entry.project_item_path.clone());
                                    }

                                    let tree_entry_project_item_path = tree_entry.project_item_path.clone();
                                    let pointer_scanner_context_actions = Self::build_pointer_scanner_context_actions(&tree_entry.project_item);
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
                                                                Self::PROJECT_ITEM_MENU_WIDTH,
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

                                                Self::show_create_project_item_menu_items(
                                                    self.app_context.clone(),
                                                    user_interface,
                                                    &tree_entry_project_item_path,
                                                    &mut project_hierarchy_frame_action,
                                                    should_close,
                                                );

                                                if user_interface
                                                    .add_enabled(
                                                        can_delete_project_item_paths,
                                                        ToolbarMenuItemView::new(
                                                            self.app_context.clone(),
                                                            "Delete",
                                                            "project_hierarchy_ctx_delete",
                                                            &None,
                                                            Self::PROJECT_ITEM_MENU_WIDTH,
                                                        ),
                                                    )
                                                    .clicked()
                                                {
                                                    project_hierarchy_frame_action =
                                                        ProjectHierarchyFrameAction::RequestDeleteConfirmation(project_item_paths_for_delete.clone());
                                                    *should_close = true;
                                                }
                                            },
                                        )
                                        .width(Self::PROJECT_ITEM_MENU_WIDTH)
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
                        user_interface.horizontal_centered(|user_interface| {
                            user_interface.with_layout(Layout::left_to_right(Align::Center), |user_interface| {
                                user_interface.spacing_mut().item_spacing.x = 12.0;
                                let button_size = vec2(120.0, 28.0);
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
                }
            })
            .response;

        if is_delete_confirmation_active {
            if user_interface.input(|input_state| input_state.key_pressed(eframe::egui::Key::Escape))
                || user_interface.input(|input_state| input_state.key_pressed(eframe::egui::Key::Backspace))
            {
                should_cancel_take_over = true;
            }

            if user_interface.input(|input_state| input_state.key_pressed(eframe::egui::Key::Enter)) {
                delete_confirmation_project_item_paths = self
                    .project_hierarchy_view_data
                    .read("Project hierarchy confirm delete by keyboard")
                    .and_then(|project_hierarchy_view_data| match project_hierarchy_view_data.take_over_state.clone() {
                        ProjectHierarchyTakeOverState::DeleteConfirmation { project_item_paths } => Some(project_item_paths),
                        _ => None,
                    });
            }
        }

        if !is_delete_confirmation_active && user_interface.input(|input_state| input_state.key_pressed(eframe::egui::Key::Delete)) {
            ProjectHierarchyViewData::request_delete_confirmation_for_selected_project_item(self.project_hierarchy_view_data.clone());
        }

        if !is_delete_confirmation_active && user_interface.input(|input_state| input_state.key_pressed(eframe::egui::Key::Space)) {
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

        if should_cancel_take_over {
            ProjectHierarchyViewData::cancel_take_over(self.project_hierarchy_view_data.clone());
        }

        if let Some(project_item_paths) = delete_confirmation_project_item_paths {
            ProjectHierarchyViewData::delete_project_items(self.project_hierarchy_view_data.clone(), self.app_context.clone(), project_item_paths);
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
                ProjectHierarchyViewData::select_project_item(self.project_hierarchy_view_data.clone(), project_item_path, additive_selection, range_selection);
                self.focus_selected_project_items_in_struct_viewer();
            }
            ProjectHierarchyFrameAction::ToggleDirectoryExpansion(project_item_path) => {
                ProjectHierarchyViewData::toggle_directory_expansion(self.project_hierarchy_view_data.clone(), project_item_path);
            }
            ProjectHierarchyFrameAction::SetProjectItemActivation(project_item_path, is_activated) => {
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
                ProjectHierarchyViewData::create_project_item(
                    self.project_hierarchy_view_data.clone(),
                    self.app_context.clone(),
                    target_project_item_path,
                    create_item_kind,
                );
            }
            ProjectHierarchyFrameAction::OpenPointerScannerForAddress {
                address,
                module_name,
                data_type_id,
            } => {
                self.focus_pointer_scanner_for_address(address, &module_name, &data_type_id);
            }
            ProjectHierarchyFrameAction::RequestDeleteConfirmation(project_item_paths) => {
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
    const PROJECT_ITEM_MENU_WIDTH: f32 = 220.0;
    const DROP_INSERTION_BAND_HEIGHT: f32 = 7.0;

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
                    should_close,
                );
            },
        )
        .width(Self::PROJECT_ITEM_MENU_WIDTH)
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
        should_close: &mut bool,
    ) {
        for (label, item_id, create_item_kind) in [
            ("New Folder", "project_hierarchy_ctx_new_folder", ProjectHierarchyCreateItemKind::Directory),
            ("New Address", "project_hierarchy_ctx_new_address", ProjectHierarchyCreateItemKind::Address),
            ("New Pointer", "project_hierarchy_ctx_new_pointer", ProjectHierarchyCreateItemKind::Pointer),
        ] {
            if user_interface
                .add(ToolbarMenuItemView::new(
                    app_context.clone(),
                    label,
                    item_id,
                    &None,
                    Self::PROJECT_ITEM_MENU_WIDTH,
                ))
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
            ProjectHierarchyViewData::refresh_project_items(self.project_hierarchy_view_data.clone(), self.app_context.clone());
        }
    }

    fn focus_selected_project_items_in_struct_viewer(&self) {
        let selected_project_item_paths = self
            .project_hierarchy_view_data
            .read("Project hierarchy selected project items for struct viewer focus")
            .map(|project_hierarchy_view_data| project_hierarchy_view_data.collect_selected_project_item_paths_in_tree_order())
            .unwrap_or_default();
        let selected_project_items = self
            .project_hierarchy_view_data
            .read("Project hierarchy selected project item data for struct viewer focus")
            .map(|project_hierarchy_view_data| {
                project_hierarchy_view_data
                    .project_items
                    .iter()
                    .filter(|(project_item_ref, _)| selected_project_item_paths.contains(project_item_ref.get_project_item_path()))
                    .map(|(_, project_item)| project_item.clone())
                    .collect::<Vec<ProjectItem>>()
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
                StructViewerViewData::focus_valued_struct(
                    self.struct_viewer_view_data.clone(),
                    self.app_context.engine_unprivileged_state.clone(),
                    Self::build_struct_view_properties(&selected_project_item),
                    callback,
                );
            }
        } else {
            let selected_project_item_properties = selected_project_items
                .into_iter()
                .map(|selected_project_item| Self::build_struct_view_properties(&selected_project_item))
                .collect::<Vec<_>>();
            StructViewerViewData::focus_valued_structs(
                self.struct_viewer_view_data.clone(),
                self.app_context.engine_unprivileged_state.clone(),
                selected_project_item_properties,
                callback,
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
                Self::build_memory_write_request_for_runtime_value_edit(&engine_execution_context, project_item, &edited_field)
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

        None
    }

    fn build_pointer_scanner_context_actions(project_item: &ProjectItem) -> Vec<PointerScannerContextAction> {
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            let mut project_item = project_item.clone();

            return vec![PointerScannerContextAction::Address {
                label: "Pointer Scan",
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
                    label: "Pointer Scan for Base Address",
                    address: pointer.get_address(),
                    module_name: pointer.get_module_name().to_string(),
                    data_type_id: data_type_id.clone(),
                },
                PointerScannerContextAction::ResolvedPointer {
                    label: "Pointer Scan for Resolved Address",
                    pointer,
                    data_type_id,
                },
            ];
        }

        Vec::new()
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

    fn build_struct_view_properties(project_item: &ProjectItem) -> ValuedStruct {
        let properties = project_item.get_properties();

        ValuedStruct::new_anonymous(
            properties
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
                .collect(),
        )
    }

    fn is_runtime_value_field(field_name: &str) -> bool {
        field_name == ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE || field_name == ProjectItemTypePointer::PROPERTY_FREEZE_DISPLAY_VALUE
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

    fn focus_pointer_scanner_for_address(
        &self,
        address: u64,
        module_name: &str,
        data_type_id: &str,
    ) {
        PointerScannerViewData::set_scan_target_from_project_address(self.pointer_scanner_view_data.clone(), address, module_name, data_type_id);

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

    fn resolve_tree_entry_icon(
        app_context: Arc<AppContext>,
        project_item: &ProjectItem,
    ) -> Option<TextureHandle> {
        let icon_library = &app_context.theme.icon_library;
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id == ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID {
            Some(icon_library.icon_handle_file_system_open_folder.clone())
        } else if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            Some(icon_library.icon_handle_data_type_blue_blocks_8.clone())
        } else if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            let data_type_id = ProjectItemTypePointer::get_field_symbolic_struct_definition_reference(project_item)
                .map(|symbolic_struct_reference| {
                    symbolic_struct_reference
                        .get_symbolic_struct_namespace()
                        .to_string()
                })
                .unwrap_or_default();

            Some(DataTypeToIconConverter::convert_data_type_to_icon(&data_type_id, icon_library))
        } else {
            Some(icon_library.icon_handle_data_type_unknown.clone())
        }
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

        if edited_name.is_empty() {
            None
        } else {
            Some(edited_name.to_string())
        }
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
