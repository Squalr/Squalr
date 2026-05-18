use crate::{
    app_context::AppContext,
    ui::{draw::icon_draw::IconDraw, list_navigation::ListNavigationDirection, widgets::controls::button::Button as ThemeButton},
    views::{
        struct_viewer::view_data::struct_viewer_view_data::StructViewerViewData,
        symbol_resolver_editor::symbol_resolver_command_dispatcher::SymbolResolverCommandDispatcher,
        symbol_resolver_editor::symbol_resolver_details_focus::SymbolResolverDetailsFocus,
        symbol_resolver_editor::symbol_resolver_list_view::{SymbolResolverListAction, SymbolResolverListView},
        symbol_resolver_editor::symbol_resolver_takeover_host_view::{SymbolResolverTakeoverAction, SymbolResolverTakeoverHostView},
        symbol_resolver_editor::view_data::symbol_resolver_editor_view_data::{
            SymbolResolverEditDraft, SymbolResolverEditorTakeOverState, SymbolResolverEditorViewData,
        },
    },
};
use eframe::egui::{Align, Direction, Key, Layout, Response, RichText, Sense, TextureHandle, Ui, UiBuilder, Widget, vec2};
use epaint::{Color32, CornerRadius};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::registries::symbols::symbolic_resolver_descriptor::SymbolicResolverDescriptor;
use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
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

    fn persist_symbol_resolver_descriptor(
        &self,
        original_resolver_id: Option<String>,
        resolver_descriptor: &SymbolicResolverDescriptor,
    ) {
        self.command_dispatcher()
            .persist_resolver_descriptor(original_resolver_id, resolver_descriptor);
    }

    fn delete_symbol_resolver(
        &self,
        resolver_id: &str,
    ) {
        self.command_dispatcher().delete_resolver(resolver_id);
    }

    fn command_dispatcher(&self) -> SymbolResolverCommandDispatcher {
        SymbolResolverCommandDispatcher::new(self.app_context.clone())
    }

    fn details_focus(&self) -> SymbolResolverDetailsFocus {
        SymbolResolverDetailsFocus::new(
            self.app_context.clone(),
            self.symbol_resolver_editor_view_data.clone(),
            self.struct_viewer_view_data.clone(),
        )
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
        self.details_focus()
            .focus_resolver(project_symbol_catalog, resolver_id);
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
                self.details_focus().clear_if_symbol_resolver_focused();
            }
            ResolverFrameAction::BeginRenameResolver(resolver_id) => {
                if let Some(mut view_data) = self
                    .symbol_resolver_editor_view_data
                    .write("SymbolResolverEditor begin rename resolver")
                {
                    view_data.begin_rename_resolver(project_symbol_catalog, &resolver_id);
                }
                self.details_focus().clear_if_symbol_resolver_focused();
            }
            ResolverFrameAction::BeginOpenResolver(resolver_id) => {
                if let Some(mut view_data) = self
                    .symbol_resolver_editor_view_data
                    .write("SymbolResolverEditor begin open resolver")
                {
                    view_data.begin_open_resolver(project_symbol_catalog, &resolver_id);
                }
                self.details_focus().focus_current_selection();
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
                self.details_focus()
                    .focus_resolver(project_symbol_catalog, &resolver_id);
            }
            ResolverFrameAction::RequestDeleteConfirmation(resolver_id) => {
                if let Some(mut view_data) = self
                    .symbol_resolver_editor_view_data
                    .write("SymbolResolverEditor begin delete confirmation")
                {
                    view_data.begin_delete_confirmation(&resolver_id);
                }
                self.details_focus().clear_if_symbol_resolver_focused();
            }
            ResolverFrameAction::ConfirmDeleteResolver(resolver_id) => {
                self.delete_symbol_resolver(&resolver_id);
                if let Some(mut view_data) = self
                    .symbol_resolver_editor_view_data
                    .write("SymbolResolverEditor delete resolver")
                {
                    view_data.cancel_take_over_state();
                }
                self.details_focus().clear_if_symbol_resolver_focused();
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
                self.details_focus().clear_if_symbol_resolver_focused();
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
                self.details_focus().focus_current_selection();

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
                self.details_focus().clear_if_symbol_resolver_focused();
            }
            Err(error) => {
                log::error!("Failed to apply symbol resolver draft: {}.", error);
            }
        }
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
                    self.details_focus()
                        .focus_resolver(&project_symbol_catalog, &selected_resolver_id);
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
                    self.details_focus()
                        .focus_resolver(&project_symbol_catalog, &selected_resolver_id);
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
