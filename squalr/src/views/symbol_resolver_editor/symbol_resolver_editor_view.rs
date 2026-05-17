use crate::{
    app_context::AppContext,
    ui::{draw::icon_draw::IconDraw, list_navigation::ListNavigationDirection, widgets::controls::button::Button as ThemeButton},
    views::{
        struct_viewer::view_data::{struct_viewer_focus_target::StructViewerFocusTarget, struct_viewer_view_data::StructViewerViewData},
        symbol_resolver_editor::symbol_resolver_list_view::{SymbolResolverListAction, SymbolResolverListView},
        symbol_resolver_editor::symbol_resolver_takeover_host_view::{SymbolResolverTakeoverAction, SymbolResolverTakeoverHostView},
        symbol_resolver_editor::view_data::symbol_resolver_editor_view_data::{
            SymbolResolverEditDraft, SymbolResolverEditorTakeOverState, SymbolResolverEditorViewData, SymbolResolverNodeKind,
        },
    },
};
use eframe::egui::{Align, Direction, Key, Layout, Response, RichText, Sense, TextureHandle, Ui, UiBuilder, Widget, vec2};
use epaint::{Color32, CornerRadius};
use squalr_engine_api::commands::{
    project_symbols::{
        delete_resolver::project_symbols_delete_resolver_request::ProjectSymbolsDeleteResolverRequest,
        upsert_resolver::project_symbols_upsert_resolver_request::ProjectSymbolsUpsertResolverRequest,
    },
    unprivileged_command_request::UnprivilegedCommandRequest,
};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::registries::symbols::symbolic_resolver_descriptor::SymbolicResolverDescriptor;
use squalr_engine_api::structures::{
    data_types::{
        built_in_types::{i64::data_type_i64::DataTypeI64, string::utf8::data_type_string_utf8::DataTypeStringUtf8},
        data_type_ref::DataTypeRef,
    },
    memory::symbolic_pointer_chain::{SymbolicPointerChain, SymbolicPointerChainLink},
    projects::project_symbol_catalog::ProjectSymbolCatalog,
    structs::{
        symbolic_resolver_definition::{SymbolicResolverBinaryOperator, SymbolicResolverNode, SymbolicResolverRelativeSymbolPath},
        valued_struct::ValuedStruct,
        valued_struct_field::ValuedStructField,
    },
};
use std::sync::Arc;

#[derive(Clone)]
pub struct SymbolResolverEditorView {
    app_context: Arc<AppContext>,
    symbol_resolver_editor_view_data: Dependency<SymbolResolverEditorViewData>,
    struct_viewer_view_data: Dependency<StructViewerViewData>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ResolverFrameAction {
    None,
    BeginCreateResolver,
    BeginRenameResolver(String),
    BeginOpenResolver(String),
    SelectResolver(String),
    ShowResolverContextMenu(String, epaint::Pos2),
    RequestDeleteConfirmation(String),
    ConfirmDeleteResolver(String),
    SaveDraft,
    CancelDraft,
}

impl SymbolResolverEditorView {
    pub const WINDOW_ID: &'static str = "window_symbol_resolver_editor";
    const TOOLBAR_HEIGHT: f32 = 28.0;
    const ICON_BUTTON_WIDTH: f32 = 36.0;
    const DETAILS_FIELD_LOCAL_FIELD: &'static str = "local_field";

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let symbol_resolver_editor_view_data = app_context
            .dependency_container
            .register(SymbolResolverEditorViewData::new());
        let struct_viewer_view_data = app_context
            .dependency_container
            .get_dependency::<StructViewerViewData>();

        Self {
            app_context,
            symbol_resolver_editor_view_data,
            struct_viewer_view_data,
        }
    }

    fn get_opened_project_symbol_catalog(&self) -> Option<ProjectSymbolCatalog> {
        let opened_project = self
            .app_context
            .engine_unprivileged_state
            .get_project_manager()
            .get_opened_project();
        let opened_project = opened_project.read().ok()?;

        opened_project.as_ref().map(|opened_project| {
            opened_project
                .get_project_info()
                .get_project_symbol_catalog()
                .clone()
        })
    }

    fn persist_symbol_resolver_descriptor_with_context(
        app_context: &Arc<AppContext>,
        original_resolver_id: Option<String>,
        resolver_descriptor: &SymbolicResolverDescriptor,
    ) {
        let Ok(request) = ProjectSymbolsUpsertResolverRequest::from_resolver_descriptor(original_resolver_id, resolver_descriptor) else {
            log::error!(
                "Failed to serialize symbol resolver `{}` for persistence.",
                resolver_descriptor.get_resolver_id()
            );
            return;
        };

        request.send(&app_context.engine_unprivileged_state, |response| {
            if !response.success {
                log::error!(
                    "Failed to persist symbol resolver `{}` through project-symbols upsert-resolver command: {}.",
                    response.resolver_id,
                    response.error.as_deref().unwrap_or("unknown error")
                );
            }
        });
    }

    fn persist_symbol_resolver_descriptor(
        &self,
        original_resolver_id: Option<String>,
        resolver_descriptor: &SymbolicResolverDescriptor,
    ) {
        Self::persist_symbol_resolver_descriptor_with_context(&self.app_context, original_resolver_id, resolver_descriptor);
    }

    fn delete_symbol_resolver(
        &self,
        resolver_id: &str,
    ) {
        ProjectSymbolsDeleteResolverRequest::new(resolver_id).send(&self.app_context.engine_unprivileged_state, |response| {
            if !response.success {
                log::error!(
                    "Failed to delete symbol resolver `{}` through project-symbols delete-resolver command: {}.",
                    response.resolver_id,
                    response.error.as_deref().unwrap_or("unknown error")
                );
            }
        });
    }

    fn default_data_type_ref(&self) -> DataTypeRef {
        self.app_context
            .engine_unprivileged_state
            .get_registered_data_type_refs()
            .first()
            .cloned()
            .unwrap_or_else(|| DataTypeRef::new("u32"))
    }

    fn render_selection_toolbar(
        &self,
        user_interface: &mut Ui,
    ) -> ResolverFrameAction {
        let theme = &self.app_context.theme;
        let (allocated_size_rectangle, _response) =
            user_interface.allocate_exact_size(vec2(user_interface.available_width(), Self::TOOLBAR_HEIGHT), Sense::empty());
        user_interface
            .painter()
            .rect_filled(allocated_size_rectangle, CornerRadius::ZERO, theme.background_primary);

        let mut action = ResolverFrameAction::None;
        let mut toolbar_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(allocated_size_rectangle)
                .layout(Layout::left_to_right(Align::Center)),
        );

        let add_response = self.render_icon_button(&mut toolbar_user_interface, &theme.icon_library.icon_handle_common_add, "Add resolver.", false);
        if add_response.clicked() {
            action = ResolverFrameAction::BeginCreateResolver;
        }

        action
    }

    fn render_icon_button(
        &self,
        user_interface: &mut Ui,
        icon_handle: &TextureHandle,
        tooltip_text: &str,
        is_disabled: bool,
    ) -> Response {
        let theme = &self.app_context.theme;
        let button_response = user_interface.add_sized(
            vec2(Self::ICON_BUTTON_WIDTH, Self::TOOLBAR_HEIGHT),
            ThemeButton::new_from_theme(theme)
                .with_tooltip_text(tooltip_text)
                .background_color(Color32::TRANSPARENT)
                .disabled(is_disabled),
        );

        IconDraw::draw_tinted(
            user_interface,
            button_response.rect,
            icon_handle,
            if is_disabled { theme.foreground_preview } else { theme.foreground },
        );

        button_response
    }

    fn focus_current_selection_in_struct_viewer(&self) {
        let (selected_node_path, draft) = self
            .symbol_resolver_editor_view_data
            .read("SymbolResolverEditor focus details selection")
            .map(|view_data| (view_data.get_selected_node_path().map(<[usize]>::to_vec), view_data.get_draft().cloned()))
            .unwrap_or((None, None));
        let Some(draft) = draft else {
            return;
        };

        self.focus_draft_selection_in_struct_viewer(selected_node_path, &draft);
    }

    fn clear_struct_viewer_if_symbol_resolver_focused(&self) {
        let is_symbol_resolver_focused = self
            .struct_viewer_view_data
            .read("SymbolResolverEditor check details focus")
            .and_then(|struct_viewer_view_data| struct_viewer_view_data.get_focus_target().cloned())
            .is_some_and(|focus_target| matches!(focus_target, StructViewerFocusTarget::SymbolResolverEditor { .. }));

        if is_symbol_resolver_focused {
            StructViewerViewData::clear_focus(self.struct_viewer_view_data.clone());
        }
    }

    fn focus_draft_selection_in_struct_viewer(
        &self,
        selected_node_path: Option<Vec<usize>>,
        draft: &SymbolResolverEditDraft,
    ) {
        let Some(details_struct) = Self::build_details_struct(draft, selected_node_path.as_deref()) else {
            return;
        };
        let selection_key = Self::build_struct_viewer_focus_target_key(draft, selected_node_path.as_deref());
        let edit_callback = Self::build_struct_viewer_edit_callback(
            self.app_context.clone(),
            self.symbol_resolver_editor_view_data.clone(),
            self.struct_viewer_view_data.clone(),
            selected_node_path,
            self.default_data_type_ref(),
        );

        StructViewerViewData::focus_valued_struct_with_focus_target(
            self.struct_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            details_struct,
            edit_callback,
            Some(StructViewerFocusTarget::SymbolResolverEditor { selection_key }),
        );
    }

    fn focus_resolver_in_struct_viewer(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        resolver_id: &str,
    ) {
        if project_symbol_catalog
            .find_symbolic_resolver_descriptor(resolver_id)
            .is_none()
        {
            self.clear_struct_viewer_if_symbol_resolver_focused();
            return;
        }

        let details_struct = Self::build_resolver_details_struct(resolver_id);
        let selection_key = format!("resolver|{}", resolver_id);
        let edit_callback = Self::build_struct_viewer_resolver_name_edit_callback(
            self.app_context.clone(),
            self.symbol_resolver_editor_view_data.clone(),
            self.struct_viewer_view_data.clone(),
            resolver_id.to_string(),
        );

        StructViewerViewData::focus_valued_struct_with_focus_target(
            self.struct_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            details_struct,
            edit_callback,
            Some(StructViewerFocusTarget::SymbolResolverEditor { selection_key }),
        );
    }

    fn build_resolver_details_struct(resolver_id: &str) -> ValuedStruct {
        ValuedStruct::new_anonymous(vec![
            DataTypeStringUtf8::get_value_from_primitive_string(resolver_id)
                .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_ID.to_string(), false),
        ])
    }

    fn build_struct_viewer_resolver_name_edit_callback(
        app_context: Arc<AppContext>,
        symbol_resolver_editor_view_data: Dependency<SymbolResolverEditorViewData>,
        struct_viewer_view_data: Dependency<StructViewerViewData>,
        resolver_id: String,
    ) -> Arc<dyn Fn(ValuedStructField) + Send + Sync> {
        Arc::new(move |edited_field: ValuedStructField| {
            let Some(project_symbol_catalog) = Self::get_opened_project_symbol_catalog_from_context(&app_context) else {
                return;
            };
            let Ok((original_resolver_id, resolver_descriptor)) =
                Self::build_resolver_descriptor_after_name_details_edit(&project_symbol_catalog, &resolver_id, &edited_field)
            else {
                return;
            };
            let saved_resolver_id = resolver_descriptor.get_resolver_id().to_string();

            Self::persist_symbol_resolver_descriptor_with_context(&app_context, original_resolver_id, &resolver_descriptor);

            if let Some(mut view_data) = symbol_resolver_editor_view_data.write("SymbolResolverEditor apply resolver name details edit") {
                view_data.select_resolver(Some(saved_resolver_id.clone()));
            }

            let details_struct = Self::build_resolver_details_struct(&saved_resolver_id);
            let selection_key = format!("resolver|{}", saved_resolver_id);
            let edit_callback = Self::build_struct_viewer_resolver_name_edit_callback(
                app_context.clone(),
                symbol_resolver_editor_view_data.clone(),
                struct_viewer_view_data.clone(),
                saved_resolver_id.clone(),
            );

            StructViewerViewData::focus_valued_struct_with_focus_target(
                struct_viewer_view_data.clone(),
                app_context.engine_unprivileged_state.clone(),
                details_struct,
                edit_callback,
                Some(StructViewerFocusTarget::SymbolResolverEditor { selection_key }),
            );
        })
    }

    fn get_opened_project_symbol_catalog_from_context(app_context: &Arc<AppContext>) -> Option<ProjectSymbolCatalog> {
        let opened_project = app_context
            .engine_unprivileged_state
            .get_project_manager()
            .get_opened_project();
        let opened_project = opened_project.read().ok()?;

        opened_project.as_ref().map(|opened_project| {
            opened_project
                .get_project_info()
                .get_project_symbol_catalog()
                .clone()
        })
    }

    fn build_resolver_descriptor_after_name_details_edit(
        project_symbol_catalog: &ProjectSymbolCatalog,
        current_resolver_id: &str,
        edited_field: &ValuedStructField,
    ) -> Result<(Option<String>, SymbolicResolverDescriptor), String> {
        if edited_field.get_name() != StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_ID {
            return Err(String::from("Edited field is not the resolver name."));
        }

        let edited_resolver_id = StructViewerViewData::read_utf8_field_text(edited_field)
            .trim()
            .to_string();
        if edited_resolver_id == current_resolver_id {
            return Err(String::from("Resolver name is unchanged."));
        }

        let resolver_descriptor = project_symbol_catalog
            .find_symbolic_resolver_descriptor(current_resolver_id)
            .ok_or_else(|| String::from("Resolver no longer exists."))?;
        let draft = SymbolResolverEditDraft {
            original_resolver_id: Some(current_resolver_id.to_string()),
            resolver_id: edited_resolver_id,
            resolver_definition: resolver_descriptor.get_resolver_definition().clone(),
        };
        let resolver_descriptor = SymbolResolverEditorViewData::build_resolver_descriptor(project_symbol_catalog, &draft)?;

        Ok((draft.original_resolver_id.clone(), resolver_descriptor))
    }

    fn build_struct_viewer_edit_callback(
        app_context: Arc<AppContext>,
        symbol_resolver_editor_view_data: Dependency<SymbolResolverEditorViewData>,
        struct_viewer_view_data: Dependency<StructViewerViewData>,
        selected_node_path: Option<Vec<usize>>,
        default_data_type_ref: DataTypeRef,
    ) -> Arc<dyn Fn(ValuedStructField) + Send + Sync> {
        Arc::new(move |edited_field: ValuedStructField| {
            let mut should_refocus_details = false;
            let updated_draft = {
                let Some(mut view_data) = symbol_resolver_editor_view_data.write("SymbolResolverEditor apply details edit") else {
                    return;
                };
                let Some(mut draft) = view_data.get_draft().cloned() else {
                    return;
                };

                if let Some(selected_node_path) = selected_node_path.as_deref() {
                    should_refocus_details = Self::apply_node_details_edit(&mut draft, selected_node_path, &edited_field, default_data_type_ref.clone());
                }

                view_data.update_draft(draft.clone());
                draft
            };

            if should_refocus_details {
                let Some(details_struct) = Self::build_details_struct(&updated_draft, selected_node_path.as_deref()) else {
                    return;
                };
                let selection_key = Self::build_struct_viewer_focus_target_key(&updated_draft, selected_node_path.as_deref());
                let edit_callback = Self::build_struct_viewer_edit_callback(
                    app_context.clone(),
                    symbol_resolver_editor_view_data.clone(),
                    struct_viewer_view_data.clone(),
                    selected_node_path.clone(),
                    default_data_type_ref.clone(),
                );

                StructViewerViewData::focus_valued_struct_with_focus_target(
                    struct_viewer_view_data.clone(),
                    app_context.engine_unprivileged_state.clone(),
                    details_struct,
                    edit_callback,
                    Some(StructViewerFocusTarget::SymbolResolverEditor { selection_key }),
                );
            }
        })
    }

    fn apply_node_details_edit(
        draft: &mut SymbolResolverEditDraft,
        selected_node_path: &[usize],
        edited_field: &ValuedStructField,
        default_data_type_ref: DataTypeRef,
    ) -> bool {
        let edited_field_name = edited_field.get_name();
        let edited_text = StructViewerViewData::read_utf8_field_text(edited_field);
        let Some(selected_node) = Self::get_node_mut(draft.resolver_definition.get_root_node_mut(), selected_node_path) else {
            return false;
        };

        match edited_field_name {
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_NODE_KIND => {
                let Some(next_kind) = Self::resolver_node_kind_from_label(&edited_text) else {
                    return false;
                };

                if next_kind != Self::resolver_node_kind(selected_node) {
                    *selected_node = SymbolResolverEditorViewData::default_node_for_kind(next_kind, default_data_type_ref);
                    return true;
                }
            }
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_LITERAL_VALUE => {
                if let (SymbolicResolverNode::Literal(value), Some(parsed_value)) = (selected_node, Self::read_i64_field_value(edited_field)) {
                    *value = i128::from(parsed_value);
                }
            }
            Self::DETAILS_FIELD_LOCAL_FIELD => {
                if let SymbolicResolverNode::LocalField { field_name } = selected_node {
                    *field_name = edited_text;
                }
            }
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_RELATIVE_SYMBOL_PATH => {
                if let SymbolicResolverNode::RelativeSymbolField { symbol_path } = selected_node {
                    *symbol_path = SymbolicResolverRelativeSymbolPath::from_dot_path(&edited_text);
                }
                if let SymbolicResolverNode::RelativePointerChain { pointer_chain } = selected_node {
                    *pointer_chain =
                        SymbolicPointerChain::new_absolute(SymbolicPointerChainLink::parse_text_list(&edited_text), pointer_chain.get_pointer_size());
                }
            }
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_GLOBAL_MODULE => {
                if let SymbolicResolverNode::GlobalSymbolField { module_name, .. } = selected_node {
                    *module_name = edited_text.trim().to_string();
                }
                if let SymbolicResolverNode::GlobalPointerChain { pointer_chain } = selected_node {
                    pointer_chain.set_module_name(edited_text.trim().to_string());
                }
            }
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_GLOBAL_SYMBOL_PATH => {
                if let SymbolicResolverNode::GlobalSymbolField { symbol_path, .. } = selected_node {
                    *symbol_path = SymbolicResolverRelativeSymbolPath::from_dot_path(&edited_text);
                }
                if let SymbolicResolverNode::GlobalPointerChain { pointer_chain } = selected_node {
                    *pointer_chain = SymbolicPointerChain::new(
                        pointer_chain.get_module_name().to_string(),
                        SymbolicPointerChainLink::parse_text_list(&edited_text),
                        pointer_chain.get_pointer_size(),
                    );
                }
            }
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_DATA_TYPE => {
                if let SymbolicResolverNode::TypeSize { data_type_ref } = selected_node {
                    *data_type_ref = DataTypeRef::new(edited_text.trim());
                }
            }
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_OPERATOR => {
                if let (SymbolicResolverNode::Binary { operator, .. }, Some(next_operator)) = (selected_node, Self::resolver_operator_from_label(&edited_text))
                {
                    *operator = next_operator;
                }
            }
            _ => {}
        }

        false
    }

    fn build_details_struct(
        draft: &SymbolResolverEditDraft,
        selected_node_path: Option<&[usize]>,
    ) -> Option<ValuedStruct> {
        let selected_node_path = selected_node_path?;
        let selected_node = Self::get_node(draft.resolver_definition.get_root_node(), selected_node_path)?;
        let mut fields = vec![
            DataTypeStringUtf8::get_value_from_primitive_string(Self::resolver_node_kind_label(Self::resolver_node_kind(selected_node)))
                .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_NODE_KIND.to_string(), false),
        ];

        match selected_node {
            SymbolicResolverNode::Literal(value) => {
                fields.push(
                    DataTypeI64::get_value_from_primitive(Self::clamp_i128_to_i64(*value))
                        .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_LITERAL_VALUE.to_string(), false),
                );
            }
            SymbolicResolverNode::LocalField { field_name } => {
                fields.push(
                    DataTypeStringUtf8::get_value_from_primitive_string(field_name)
                        .to_named_valued_struct_field(Self::DETAILS_FIELD_LOCAL_FIELD.to_string(), false),
                );
            }
            SymbolicResolverNode::RelativeSymbolField { symbol_path } => {
                fields.push(
                    DataTypeStringUtf8::get_value_from_primitive_string(&symbol_path.to_string())
                        .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_RELATIVE_SYMBOL_PATH.to_string(), false),
                );
            }
            SymbolicResolverNode::GlobalSymbolField { module_name, symbol_path } => {
                fields.push(
                    DataTypeStringUtf8::get_value_from_primitive_string(module_name)
                        .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_GLOBAL_MODULE.to_string(), false),
                );
                fields.push(
                    DataTypeStringUtf8::get_value_from_primitive_string(&symbol_path.to_string())
                        .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_GLOBAL_SYMBOL_PATH.to_string(), false),
                );
            }
            SymbolicResolverNode::RelativePointerChain { pointer_chain } => {
                fields.push(
                    DataTypeStringUtf8::get_value_from_primitive_string(&SymbolicPointerChainLink::display_text_list(pointer_chain.get_links()))
                        .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_RELATIVE_SYMBOL_PATH.to_string(), false),
                );
            }
            SymbolicResolverNode::GlobalPointerChain { pointer_chain } => {
                fields.push(
                    DataTypeStringUtf8::get_value_from_primitive_string(pointer_chain.get_module_name())
                        .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_GLOBAL_MODULE.to_string(), false),
                );
                fields.push(
                    DataTypeStringUtf8::get_value_from_primitive_string(&SymbolicPointerChainLink::display_text_list(pointer_chain.get_links()))
                        .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_GLOBAL_SYMBOL_PATH.to_string(), false),
                );
            }
            SymbolicResolverNode::TypeSize { data_type_ref } => {
                fields.push(
                    DataTypeStringUtf8::get_value_from_primitive_string(data_type_ref.get_data_type_id())
                        .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_DATA_TYPE.to_string(), false),
                );
            }
            SymbolicResolverNode::Binary { operator, .. } => {
                fields.push(
                    DataTypeStringUtf8::get_value_from_primitive_string(operator.label())
                        .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_RESOLVER_OPERATOR.to_string(), false),
                );
            }
            SymbolicResolverNode::Conditional { .. } => {}
        }

        Some(ValuedStruct::new_anonymous(fields))
    }

    fn build_struct_viewer_focus_target_key(
        draft: &SymbolResolverEditDraft,
        selected_node_path: Option<&[usize]>,
    ) -> String {
        let resolver_key = draft
            .original_resolver_id
            .as_deref()
            .unwrap_or(draft.resolver_id.as_str());
        let node_path_key = selected_node_path
            .map(|node_path| {
                node_path
                    .iter()
                    .map(usize::to_string)
                    .collect::<Vec<_>>()
                    .join(".")
            })
            .unwrap_or_default();

        format!("{}|{}", resolver_key, node_path_key)
    }

    fn select_resolver(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        resolver_id: &str,
    ) {
        if let Some(mut view_data) = self
            .symbol_resolver_editor_view_data
            .write("SymbolResolverEditor select resolver")
        {
            view_data.select_resolver(Some(resolver_id.to_string()));
        }
        self.focus_resolver_in_struct_viewer(project_symbol_catalog, resolver_id);
    }

    fn apply_frame_action(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        action: ResolverFrameAction,
        draft: Option<&SymbolResolverEditDraft>,
    ) {
        match action {
            ResolverFrameAction::None => {}
            ResolverFrameAction::BeginCreateResolver => {
                if let Some(mut view_data) = self
                    .symbol_resolver_editor_view_data
                    .write("SymbolResolverEditor begin create resolver")
                {
                    view_data.begin_create_resolver(project_symbol_catalog);
                }
                self.clear_struct_viewer_if_symbol_resolver_focused();
            }
            ResolverFrameAction::BeginRenameResolver(resolver_id) => {
                if let Some(mut view_data) = self
                    .symbol_resolver_editor_view_data
                    .write("SymbolResolverEditor begin rename resolver")
                {
                    view_data.begin_rename_resolver(project_symbol_catalog, &resolver_id);
                }
                self.clear_struct_viewer_if_symbol_resolver_focused();
            }
            ResolverFrameAction::BeginOpenResolver(resolver_id) => {
                if let Some(mut view_data) = self
                    .symbol_resolver_editor_view_data
                    .write("SymbolResolverEditor begin open resolver")
                {
                    view_data.begin_open_resolver(project_symbol_catalog, &resolver_id);
                }
                self.focus_current_selection_in_struct_viewer();
            }
            ResolverFrameAction::SelectResolver(resolver_id) => {
                self.select_resolver(project_symbol_catalog, &resolver_id);
            }
            ResolverFrameAction::ShowResolverContextMenu(resolver_id, position) => {
                if let Some(mut view_data) = self
                    .symbol_resolver_editor_view_data
                    .write("SymbolResolverEditor show resolver context menu")
                {
                    view_data.show_resolver_context_menu(resolver_id.clone(), position);
                }
                self.focus_resolver_in_struct_viewer(project_symbol_catalog, &resolver_id);
            }
            ResolverFrameAction::RequestDeleteConfirmation(resolver_id) => {
                if let Some(mut view_data) = self
                    .symbol_resolver_editor_view_data
                    .write("SymbolResolverEditor begin delete confirmation")
                {
                    view_data.begin_delete_confirmation(&resolver_id);
                }
                self.clear_struct_viewer_if_symbol_resolver_focused();
            }
            ResolverFrameAction::ConfirmDeleteResolver(resolver_id) => {
                self.delete_symbol_resolver(&resolver_id);
                if let Some(mut view_data) = self
                    .symbol_resolver_editor_view_data
                    .write("SymbolResolverEditor delete resolver")
                {
                    view_data.cancel_take_over_state();
                }
                self.clear_struct_viewer_if_symbol_resolver_focused();
            }
            ResolverFrameAction::SaveDraft => {
                let current_draft = self
                    .symbol_resolver_editor_view_data
                    .read("SymbolResolverEditor read draft for save")
                    .and_then(|view_data| view_data.get_draft().cloned())
                    .or_else(|| draft.cloned());
                if let Some(current_draft) = current_draft.as_ref() {
                    self.save_draft(project_symbol_catalog, current_draft);
                }
            }
            ResolverFrameAction::CancelDraft => {
                if let Some(mut view_data) = self
                    .symbol_resolver_editor_view_data
                    .write("SymbolResolverEditor cancel resolver edit")
                {
                    view_data.cancel_take_over_state();
                }
                self.clear_struct_viewer_if_symbol_resolver_focused();
            }
        }
    }

    fn map_list_action(list_action: SymbolResolverListAction) -> ResolverFrameAction {
        match list_action {
            SymbolResolverListAction::None => ResolverFrameAction::None,
            SymbolResolverListAction::BeginRenameResolver(resolver_id) => ResolverFrameAction::BeginRenameResolver(resolver_id),
            SymbolResolverListAction::BeginOpenResolver(resolver_id) => ResolverFrameAction::BeginOpenResolver(resolver_id),
            SymbolResolverListAction::SelectResolver(resolver_id) => ResolverFrameAction::SelectResolver(resolver_id),
            SymbolResolverListAction::ShowResolverContextMenu(resolver_id, position) => ResolverFrameAction::ShowResolverContextMenu(resolver_id, position),
            SymbolResolverListAction::RequestDeleteConfirmation(resolver_id) => ResolverFrameAction::RequestDeleteConfirmation(resolver_id),
        }
    }

    fn apply_takeover_action(
        &self,
        takeover_action: SymbolResolverTakeoverAction,
    ) -> ResolverFrameAction {
        match takeover_action {
            SymbolResolverTakeoverAction::None => ResolverFrameAction::None,
            SymbolResolverTakeoverAction::SaveDraft => ResolverFrameAction::SaveDraft,
            SymbolResolverTakeoverAction::CancelDraft => ResolverFrameAction::CancelDraft,
            SymbolResolverTakeoverAction::ConfirmDeleteResolver(resolver_id) => ResolverFrameAction::ConfirmDeleteResolver(resolver_id),
            SymbolResolverTakeoverAction::SelectNode { resolver_id, node_path } => {
                if let Some(mut view_data) = self
                    .symbol_resolver_editor_view_data
                    .write("SymbolResolverEditor select node")
                {
                    view_data.select_node(resolver_id, node_path);
                }
                self.focus_current_selection_in_struct_viewer();

                ResolverFrameAction::None
            }
        }
    }

    fn save_draft(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        draft: &SymbolResolverEditDraft,
    ) {
        match SymbolResolverEditorViewData::build_resolver_descriptor(project_symbol_catalog, draft) {
            Ok(resolver_descriptor) => {
                let saved_resolver_id = draft.resolver_id.trim().to_string();
                self.persist_symbol_resolver_descriptor(draft.original_resolver_id.clone(), &resolver_descriptor);
                if let Some(mut view_data) = self
                    .symbol_resolver_editor_view_data
                    .write("SymbolResolverEditor save resolver")
                {
                    view_data.cancel_take_over_state();
                    view_data.select_resolver(Some(saved_resolver_id));
                }
                self.clear_struct_viewer_if_symbol_resolver_focused();
            }
            Err(error) => {
                log::error!("Failed to apply symbol resolver draft: {}.", error);
            }
        }
    }

    fn resolver_node_kind(resolver_node: &SymbolicResolverNode) -> SymbolResolverNodeKind {
        match resolver_node {
            SymbolicResolverNode::Literal(_) => SymbolResolverNodeKind::Literal,
            SymbolicResolverNode::LocalField { .. } => SymbolResolverNodeKind::LocalField,
            SymbolicResolverNode::RelativeSymbolField { .. } => SymbolResolverNodeKind::RelativeSymbolField,
            SymbolicResolverNode::GlobalSymbolField { .. } => SymbolResolverNodeKind::GlobalSymbolField,
            SymbolicResolverNode::RelativePointerChain { .. } => SymbolResolverNodeKind::RelativePointerChain,
            SymbolicResolverNode::GlobalPointerChain { .. } => SymbolResolverNodeKind::GlobalPointerChain,
            SymbolicResolverNode::TypeSize { .. } => SymbolResolverNodeKind::TypeSize,
            SymbolicResolverNode::Binary { .. } => SymbolResolverNodeKind::Operation,
            SymbolicResolverNode::Conditional { .. } => SymbolResolverNodeKind::Conditional,
        }
    }

    fn resolver_node_kind_label(resolver_node_kind: SymbolResolverNodeKind) -> &'static str {
        match resolver_node_kind {
            SymbolResolverNodeKind::Literal => "Literal",
            SymbolResolverNodeKind::LocalField => "Local Field",
            SymbolResolverNodeKind::RelativeSymbolField => "Relative Symbol Field",
            SymbolResolverNodeKind::GlobalSymbolField => "Global Symbol Field",
            SymbolResolverNodeKind::RelativePointerChain => "Relative Pointer Chain",
            SymbolResolverNodeKind::GlobalPointerChain => "Global Pointer Chain",
            SymbolResolverNodeKind::TypeSize => "Type Size",
            SymbolResolverNodeKind::Operation => "Operation",
            SymbolResolverNodeKind::Conditional => "Conditional",
        }
    }

    fn resolver_node_kind_from_label(label: &str) -> Option<SymbolResolverNodeKind> {
        match label.trim() {
            "Literal" => Some(SymbolResolverNodeKind::Literal),
            "Local Field" => Some(SymbolResolverNodeKind::LocalField),
            "Relative Symbol Field" | "Symbol Field" => Some(SymbolResolverNodeKind::RelativeSymbolField),
            "Global Symbol Field" => Some(SymbolResolverNodeKind::GlobalSymbolField),
            "Relative Pointer Chain" => Some(SymbolResolverNodeKind::RelativePointerChain),
            "Global Pointer Chain" => Some(SymbolResolverNodeKind::GlobalPointerChain),
            "Type Size" => Some(SymbolResolverNodeKind::TypeSize),
            "Operation" => Some(SymbolResolverNodeKind::Operation),
            "Conditional" => Some(SymbolResolverNodeKind::Conditional),
            _ => None,
        }
    }

    fn resolver_operator_from_label(label: &str) -> Option<SymbolicResolverBinaryOperator> {
        SymbolicResolverBinaryOperator::ALL
            .iter()
            .copied()
            .find(|operator| operator.label() == label.trim())
    }

    fn get_node_mut<'resolver>(
        resolver_node: &'resolver mut SymbolicResolverNode,
        node_path: &[usize],
    ) -> Option<&'resolver mut SymbolicResolverNode> {
        if node_path.is_empty() {
            return Some(resolver_node);
        }

        match resolver_node {
            SymbolicResolverNode::Binary { left_node, right_node, .. } => match node_path[0] {
                0 => Self::get_node_mut(left_node, &node_path[1..]),
                1 => Self::get_node_mut(right_node, &node_path[1..]),
                _ => None,
            },
            SymbolicResolverNode::Conditional {
                condition_node,
                true_node,
                false_node,
            } => match node_path[0] {
                0 => Self::get_node_mut(condition_node, &node_path[1..]),
                1 => Self::get_node_mut(true_node, &node_path[1..]),
                2 => Self::get_node_mut(false_node, &node_path[1..]),
                _ => None,
            },
            SymbolicResolverNode::Literal(_)
            | SymbolicResolverNode::LocalField { .. }
            | SymbolicResolverNode::RelativeSymbolField { .. }
            | SymbolicResolverNode::GlobalSymbolField { .. }
            | SymbolicResolverNode::RelativePointerChain { .. }
            | SymbolicResolverNode::GlobalPointerChain { .. }
            | SymbolicResolverNode::TypeSize { .. } => None,
        }
    }

    fn get_node<'resolver>(
        resolver_node: &'resolver SymbolicResolverNode,
        node_path: &[usize],
    ) -> Option<&'resolver SymbolicResolverNode> {
        if node_path.is_empty() {
            return Some(resolver_node);
        }

        match resolver_node {
            SymbolicResolverNode::Binary { left_node, right_node, .. } => match node_path[0] {
                0 => Self::get_node(left_node, &node_path[1..]),
                1 => Self::get_node(right_node, &node_path[1..]),
                _ => None,
            },
            SymbolicResolverNode::Conditional {
                condition_node,
                true_node,
                false_node,
            } => match node_path[0] {
                0 => Self::get_node(condition_node, &node_path[1..]),
                1 => Self::get_node(true_node, &node_path[1..]),
                2 => Self::get_node(false_node, &node_path[1..]),
                _ => None,
            },
            SymbolicResolverNode::Literal(_)
            | SymbolicResolverNode::LocalField { .. }
            | SymbolicResolverNode::RelativeSymbolField { .. }
            | SymbolicResolverNode::GlobalSymbolField { .. }
            | SymbolicResolverNode::RelativePointerChain { .. }
            | SymbolicResolverNode::GlobalPointerChain { .. }
            | SymbolicResolverNode::TypeSize { .. } => None,
        }
    }

    fn clamp_i128_to_i64(value: i128) -> i64 {
        value.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64
    }

    fn read_i64_field_value(valued_struct_field: &ValuedStructField) -> Option<i64> {
        let value_bytes = valued_struct_field.get_data_value()?.get_value_bytes();
        let value_bytes: [u8; 8] = value_bytes.as_slice().try_into().ok()?;

        Some(i64::from_le_bytes(value_bytes))
    }

    fn render_empty_project_message(
        &self,
        user_interface: &mut Ui,
    ) -> Response {
        user_interface
            .allocate_ui_with_layout(
                user_interface.available_size(),
                Layout::centered_and_justified(Direction::TopDown),
                |user_interface| {
                    user_interface.label(RichText::new("Open a project to author reusable symbol resolvers.").color(self.app_context.theme.foreground_preview));
                },
            )
            .response
    }
}

impl Widget for SymbolResolverEditorView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> eframe::egui::Response {
        let Some(project_symbol_catalog) = self.get_opened_project_symbol_catalog() else {
            return self.render_empty_project_message(user_interface);
        };

        if let Some(mut view_data) = self
            .symbol_resolver_editor_view_data
            .write("SymbolResolverEditor synchronize")
        {
            view_data.synchronize(&project_symbol_catalog);
        }

        let (selected_resolver_id, selected_node_path, take_over_state, baseline_draft, draft) = self
            .symbol_resolver_editor_view_data
            .read("SymbolResolverEditor view")
            .map(|view_data| {
                (
                    view_data.get_selected_resolver_id().map(str::to_string),
                    view_data.get_selected_node_path().map(<[usize]>::to_vec),
                    view_data.get_take_over_state().cloned(),
                    view_data.get_baseline_draft().cloned(),
                    view_data.get_draft().cloned(),
                )
            })
            .unwrap_or((None, None, None, None, None));

        let can_handle_window_shortcuts = self
            .app_context
            .window_focus_manager
            .can_window_handle_shortcuts(user_interface.ctx(), Self::WINDOW_ID);
        let validation_result = draft
            .as_ref()
            .map(|draft| SymbolResolverEditorViewData::build_resolver_descriptor(&project_symbol_catalog, draft));
        let has_draft_changes = draft
            .as_ref()
            .zip(baseline_draft.as_ref())
            .map(|(draft, baseline_draft)| draft != baseline_draft)
            .unwrap_or(false);
        let is_creating_resolver = matches!(take_over_state, Some(SymbolResolverEditorTakeOverState::CreateResolver));
        let can_save = draft.is_some() && validation_result.as_ref().is_some_and(Result::is_ok) && (has_draft_changes || is_creating_resolver);
        let mut frame_action = ResolverFrameAction::None;

        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                match (take_over_state.as_ref(), draft.as_ref()) {
                    (Some(take_over_state), _) => {
                        let takeover_action = SymbolResolverTakeoverHostView::new(
                            self.app_context.clone(),
                            self.symbol_resolver_editor_view_data.clone(),
                            &project_symbol_catalog,
                            take_over_state,
                            draft.as_ref(),
                            selected_node_path.as_deref(),
                            validation_result.as_ref(),
                            can_save,
                            can_handle_window_shortcuts,
                        )
                        .show(user_interface);
                        frame_action = self.apply_takeover_action(takeover_action);
                    }
                    _ => {
                        frame_action = self.render_selection_toolbar(user_interface);
                        user_interface.add_space(4.0);
                        let resolver_context_menu_target = self
                            .symbol_resolver_editor_view_data
                            .read("SymbolResolverEditor resolver context menu")
                            .and_then(|view_data| view_data.get_resolver_context_menu_target().cloned());
                        let list_action = SymbolResolverListView::new(
                            self.app_context.clone(),
                            self.symbol_resolver_editor_view_data.clone(),
                            &project_symbol_catalog,
                            selected_resolver_id.as_deref(),
                            resolver_context_menu_target.as_ref(),
                        )
                        .show(user_interface);

                        if !matches!(list_action, SymbolResolverListAction::None) {
                            frame_action = Self::map_list_action(list_action);
                        }
                    }
                }
            })
            .response;

        if can_handle_window_shortcuts
            && take_over_state.is_none()
            && matches!(frame_action, ResolverFrameAction::None)
            && user_interface.input(|input_state| input_state.key_pressed(Key::ArrowUp))
        {
            if let Some(mut view_data) = self
                .symbol_resolver_editor_view_data
                .write("SymbolResolverEditor keyboard navigate up")
            {
                if let Some(selected_resolver_id) = view_data.navigate_resolver_selection(&project_symbol_catalog, ListNavigationDirection::Up) {
                    drop(view_data);
                    self.focus_resolver_in_struct_viewer(&project_symbol_catalog, &selected_resolver_id);
                }
            }
        }

        if can_handle_window_shortcuts
            && take_over_state.is_none()
            && matches!(frame_action, ResolverFrameAction::None)
            && user_interface.input(|input_state| input_state.key_pressed(Key::ArrowDown))
        {
            if let Some(mut view_data) = self
                .symbol_resolver_editor_view_data
                .write("SymbolResolverEditor keyboard navigate down")
            {
                if let Some(selected_resolver_id) = view_data.navigate_resolver_selection(&project_symbol_catalog, ListNavigationDirection::Down) {
                    drop(view_data);
                    self.focus_resolver_in_struct_viewer(&project_symbol_catalog, &selected_resolver_id);
                }
            }
        }

        if can_handle_window_shortcuts
            && take_over_state.is_none()
            && matches!(frame_action, ResolverFrameAction::None)
            && user_interface.input(|input_state| input_state.key_pressed(Key::Enter))
            && selected_resolver_id.is_some()
        {
            if let Some(selected_resolver_id) = selected_resolver_id.as_deref() {
                frame_action = ResolverFrameAction::BeginOpenResolver(selected_resolver_id.to_string());
            }
        }

        if can_handle_window_shortcuts
            && take_over_state.is_none()
            && matches!(frame_action, ResolverFrameAction::None)
            && user_interface.input(|input_state| input_state.key_pressed(Key::Delete))
            && selected_resolver_id.is_some()
        {
            if let Some(selected_resolver_id) = selected_resolver_id.as_deref() {
                frame_action = ResolverFrameAction::RequestDeleteConfirmation(selected_resolver_id.to_string());
            }
        }

        if can_handle_window_shortcuts
            && user_interface.input(|input_state| input_state.key_pressed(Key::Escape))
            && draft.is_some()
            && matches!(frame_action, ResolverFrameAction::None)
        {
            frame_action = ResolverFrameAction::CancelDraft;
        }

        self.apply_frame_action(&project_symbol_catalog, frame_action, draft.as_ref());

        response
    }
}
