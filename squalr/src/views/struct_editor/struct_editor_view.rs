use crate::app_context::AppContext;
use crate::ui::widgets::controls::data_type_selector::{data_type_selection::DataTypeSelection, data_type_selector_view::DataTypeSelectorView};
use crate::ui::widgets::controls::{button::Button as ThemeButton, data_value_box::data_value_box_view::DataValueBoxView, state_layer::StateLayer};
use crate::views::struct_editor::view_data::struct_editor_view_data::{StructEditorViewData, StructFieldEditDraft, StructLayoutEditDraft};
use eframe::egui::{Align, Align2, Direction, Layout, Response, RichText, ScrollArea, Sense, Ui, Widget, pos2, vec2};
use epaint::{CornerRadius, Stroke, StrokeKind};
use squalr_engine_api::commands::{
    privileged_command_request::PrivilegedCommandRequest, project::save::project_save_request::ProjectSaveRequest,
    registry::set_project_symbols::registry_set_project_symbols_request::RegistrySetProjectSymbolsRequest,
    unprivileged_command_request::UnprivilegedCommandRequest,
};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::{
    data_types::{
        built_in_types::{string::utf8::data_type_string_utf8::DataTypeStringUtf8, u8::data_type_u8::DataTypeU8},
        data_type_ref::DataTypeRef,
    },
    data_values::{anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType},
    projects::project_symbol_catalog::ProjectSymbolCatalog,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct StructEditorView {
    app_context: Arc<AppContext>,
    struct_editor_view_data: Dependency<StructEditorViewData>,
}

impl StructEditorView {
    pub const WINDOW_ID: &'static str = "window_struct_editor";
    const LIST_PANEL_WIDTH: f32 = 280.0;
    const FIELD_ROW_HEIGHT: f32 = 28.0;
    const LIST_ROW_HEIGHT: f32 = 28.0;

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let struct_editor_view_data = app_context
            .dependency_container
            .register(StructEditorViewData::new());

        Self {
            app_context,
            struct_editor_view_data,
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

    fn persist_project_symbol_catalog(
        &self,
        updated_project_symbol_catalog: ProjectSymbolCatalog,
    ) {
        let opened_project_lock = self
            .app_context
            .engine_unprivileged_state
            .get_project_manager()
            .get_opened_project();
        let did_update_project = match opened_project_lock.write() {
            Ok(mut opened_project) => {
                if let Some(opened_project) = opened_project.as_mut() {
                    let project_info = opened_project.get_project_info_mut();

                    *project_info.get_project_symbol_catalog_mut() = updated_project_symbol_catalog.clone();
                    project_info.set_has_unsaved_changes(true);
                    true
                } else {
                    false
                }
            }
            Err(error) => {
                log::error!("Failed to acquire opened project while persisting struct editor changes: {}.", error);
                false
            }
        };

        if !did_update_project {
            return;
        }

        let project_save_request = ProjectSaveRequest {};
        project_save_request.send(&self.app_context.engine_unprivileged_state, |project_save_response| {
            if !project_save_response.success {
                log::error!("Failed to save project after applying struct editor changes.");
            }
        });

        let registry_set_project_symbols_request = RegistrySetProjectSymbolsRequest {
            project_symbol_catalog: updated_project_symbol_catalog,
        };
        let did_dispatch_registry_sync = registry_set_project_symbols_request.send(&self.app_context.engine_unprivileged_state, |_response| {});
        if !did_dispatch_registry_sync {
            log::error!("Failed to dispatch project symbol registry sync after struct editor changes.");
        }
    }

    fn default_data_type_ref(&self) -> DataTypeRef {
        self.app_context
            .engine_unprivileged_state
            .get_registered_data_type_refs()
            .first()
            .cloned()
            .unwrap_or_else(|| DataTypeRef::new(DataTypeU8::DATA_TYPE_ID))
    }

    fn available_data_types(&self) -> Vec<DataTypeRef> {
        let mut available_data_types = self
            .app_context
            .engine_unprivileged_state
            .get_registered_data_type_refs();
        if available_data_types.is_empty() {
            available_data_types.push(DataTypeRef::new(DataTypeU8::DATA_TYPE_ID));
        }

        available_data_types
    }

    fn string_data_type_ref() -> DataTypeRef {
        DataTypeRef::new(DataTypeStringUtf8::DATA_TYPE_ID)
    }

    fn render_text_button(
        &self,
        user_interface: &mut Ui,
        label: &str,
        tooltip_text: &str,
        width: f32,
        height: f32,
        is_disabled: bool,
    ) -> Response {
        let theme = &self.app_context.theme;
        let button_response = user_interface.add_sized(
            vec2(width, height),
            ThemeButton::new_from_theme(theme)
                .with_tooltip_text(tooltip_text)
                .disabled(is_disabled),
        );

        user_interface.painter().text(
            button_response.rect.center(),
            Align2::CENTER_CENTER,
            label,
            theme.font_library.font_noto_sans.font_normal.clone(),
            if is_disabled { theme.foreground_preview } else { theme.foreground },
        );

        button_response
    }

    fn render_string_value_box(
        &self,
        user_interface: &mut Ui,
        value: &mut String,
        preview_text: &str,
        id: &str,
        width: f32,
        height: f32,
    ) {
        let validation_data_type_ref = Self::string_data_type_ref();
        let mut value_string = AnonymousValueString::new(value.clone(), AnonymousValueStringFormat::String, ContainerType::None);

        user_interface.add(
            DataValueBoxView::new(
                self.app_context.clone(),
                &mut value_string,
                &validation_data_type_ref,
                false,
                true,
                preview_text,
                id,
            )
            .allowed_anonymous_value_string_formats(vec![AnonymousValueStringFormat::String])
            .show_format_button(false)
            .normalize_value_format(false)
            .use_format_text_colors(false)
            .width(width)
            .height(height),
        );

        *value = value_string.get_anonymous_value_string().to_string();
    }

    fn render_struct_layout_row(
        &self,
        user_interface: &mut Ui,
        label: &str,
        is_selected: bool,
    ) -> Response {
        let theme = &self.app_context.theme;
        let (row_rect, row_response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), Self::LIST_ROW_HEIGHT), Sense::click());

        if is_selected {
            user_interface
                .painter()
                .rect_filled(row_rect, CornerRadius::ZERO, theme.selected_background);
            user_interface
                .painter()
                .rect_stroke(row_rect, CornerRadius::ZERO, Stroke::new(1.0, theme.selected_border), StrokeKind::Inside);
        }

        StateLayer {
            bounds_min: row_rect.min,
            bounds_max: row_rect.max,
            enabled: true,
            pressed: row_response.is_pointer_button_down_on(),
            has_hover: row_response.hovered(),
            has_focus: false,
            corner_radius: CornerRadius::ZERO,
            border_width: 0.0,
            hover_color: theme.hover_tint,
            pressed_color: theme.pressed_tint,
            border_color: theme.background_control_secondary_dark,
            border_color_focused: theme.background_control_secondary_dark,
        }
        .ui(user_interface);

        user_interface.painter().text(
            pos2(row_rect.min.x + 8.0, row_rect.center().y),
            Align2::LEFT_CENTER,
            label,
            theme.font_library.font_noto_sans.font_normal.clone(),
            if is_selected { theme.foreground } else { theme.foreground_preview },
        );

        row_response
    }

    fn render_list_panel(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        selected_layout_id: Option<&str>,
        filter_text: &str,
        is_creating_new_layout: bool,
    ) {
        user_interface.horizontal(|user_interface| {
            let new_layout_response = self.render_text_button(
                user_interface,
                "New",
                "Create a new reusable struct layout.",
                80.0,
                Self::FIELD_ROW_HEIGHT,
                false,
            );
            if new_layout_response.clicked() {
                StructEditorViewData::begin_create_struct_layout(self.struct_editor_view_data.clone(), project_symbol_catalog, self.default_data_type_ref());
            }

            let usage_count = selected_layout_id
                .map(|selected_layout_id| StructEditorViewData::count_rooted_symbol_usages(project_symbol_catalog, selected_layout_id))
                .unwrap_or(0);
            let can_delete_selected_layout = !is_creating_new_layout && selected_layout_id.is_some() && usage_count == 0;
            let delete_layout_response = self.render_text_button(
                user_interface,
                "Delete",
                "Delete the selected struct layout.",
                80.0,
                Self::FIELD_ROW_HEIGHT,
                !can_delete_selected_layout,
            );
            if delete_layout_response.clicked() {
                if let Some(selected_layout_id) = selected_layout_id {
                    if let Ok(updated_project_symbol_catalog) =
                        StructEditorViewData::remove_struct_layout_from_catalog(project_symbol_catalog, selected_layout_id)
                    {
                        self.persist_project_symbol_catalog(updated_project_symbol_catalog);
                    }
                }
            }
        });

        user_interface.add_space(8.0);
        let mut edited_filter_text = filter_text.to_string();
        self.render_string_value_box(
            user_interface,
            &mut edited_filter_text,
            "Filter struct layouts...",
            "struct_editor_filter_text",
            user_interface.available_width(),
            Self::FIELD_ROW_HEIGHT,
        );
        if edited_filter_text != filter_text {
            StructEditorViewData::set_filter_text(self.struct_editor_view_data.clone(), edited_filter_text);
        }

        user_interface.add_space(8.0);
        ScrollArea::vertical()
            .id_salt("struct_editor_layout_list")
            .auto_shrink([false, false])
            .show(user_interface, |user_interface| {
                for struct_layout_descriptor in project_symbol_catalog
                    .get_struct_layout_descriptors()
                    .iter()
                    .filter(|struct_layout_descriptor| StructEditorViewData::layout_matches_filter(struct_layout_descriptor, filter_text))
                {
                    let struct_layout_id = struct_layout_descriptor.get_struct_layout_id();
                    let usage_count = StructEditorViewData::count_rooted_symbol_usages(project_symbol_catalog, struct_layout_id);
                    let layout_label = if usage_count == 0 {
                        struct_layout_id.to_string()
                    } else {
                        format!("{} ({})", struct_layout_id, usage_count)
                    };
                    let row_response = self.render_struct_layout_row(user_interface, &layout_label, selected_layout_id == Some(struct_layout_id));
                    if row_response.clicked() {
                        StructEditorViewData::select_struct_layout(
                            self.struct_editor_view_data.clone(),
                            project_symbol_catalog,
                            Some(struct_layout_id.to_string()),
                        );
                    }
                }

                if project_symbol_catalog
                    .get_struct_layout_descriptors()
                    .is_empty()
                {
                    user_interface.label(RichText::new("No struct layouts yet.").color(self.app_context.theme.foreground_preview));
                }
            });
    }

    fn render_field_row_header(
        &self,
        user_interface: &mut Ui,
    ) {
        user_interface.horizontal(|user_interface| {
            user_interface.add_sized(vec2(140.0, Self::FIELD_ROW_HEIGHT), eframe::egui::Label::new(RichText::new("Field").strong()));
            user_interface.add_sized(vec2(180.0, Self::FIELD_ROW_HEIGHT), eframe::egui::Label::new(RichText::new("Type").strong()));
            user_interface.add_sized(
                vec2(110.0, Self::FIELD_ROW_HEIGHT),
                eframe::egui::Label::new(RichText::new("Container").strong()),
            );
            user_interface.add_sized(vec2(60.0, Self::FIELD_ROW_HEIGHT), eframe::egui::Label::new(""));
        });
    }

    fn render_field_rows(
        &self,
        user_interface: &mut Ui,
        draft: &mut StructLayoutEditDraft,
    ) {
        let available_data_types = self.available_data_types();
        self.render_field_row_header(user_interface);
        user_interface.add_space(4.0);

        let mut pending_removed_field_index = None;
        ScrollArea::vertical()
            .id_salt("struct_editor_field_rows")
            .auto_shrink([false, false])
            .show(user_interface, |user_interface| {
                for (field_index, field_draft) in draft.field_drafts.iter_mut().enumerate() {
                    user_interface.horizontal(|user_interface| {
                        self.render_string_value_box(
                            user_interface,
                            &mut field_draft.field_name,
                            "field_name",
                            &format!("struct_editor_field_name_{}", field_index),
                            140.0,
                            Self::FIELD_ROW_HEIGHT,
                        );
                        let selector_id = format!("struct_editor_data_type_{}", field_index);
                        user_interface.add_sized(
                            vec2(180.0, Self::FIELD_ROW_HEIGHT),
                            DataTypeSelectorView::new(self.app_context.clone(), &mut field_draft.data_type_selection, &selector_id)
                                .available_data_types(available_data_types.clone())
                                .stacked_list()
                                .width(180.0)
                                .height(Self::FIELD_ROW_HEIGHT),
                        );
                        self.render_string_value_box(
                            user_interface,
                            &mut field_draft.container_suffix,
                            "[] / [4] / *(64)",
                            &format!("struct_editor_container_suffix_{}", field_index),
                            110.0,
                            Self::FIELD_ROW_HEIGHT,
                        );
                        let remove_field_response = self.render_text_button(
                            user_interface,
                            "Remove",
                            "Remove this field from the draft struct layout.",
                            72.0,
                            Self::FIELD_ROW_HEIGHT,
                            false,
                        );
                        if remove_field_response.clicked() {
                            pending_removed_field_index = Some(field_index);
                        }
                    });
                    user_interface.add_space(4.0);
                }
            });

        if let Some(removed_field_index) = pending_removed_field_index {
            draft.field_drafts.remove(removed_field_index);
            if draft.field_drafts.is_empty() {
                draft.field_drafts.push(StructFieldEditDraft {
                    field_name: String::new(),
                    data_type_selection: DataTypeSelection::new(self.default_data_type_ref()),
                    container_suffix: String::new(),
                });
            }
        }

        if self
            .render_text_button(
                user_interface,
                "Add Field",
                "Append a new field to the draft struct layout.",
                96.0,
                Self::FIELD_ROW_HEIGHT,
                false,
            )
            .clicked()
        {
            draft.field_drafts.push(StructFieldEditDraft {
                field_name: String::new(),
                data_type_selection: DataTypeSelection::new(self.default_data_type_ref()),
                container_suffix: String::new(),
            });
        }
    }

    fn render_editor_panel(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        selected_layout_id: Option<&str>,
        draft: Option<&StructLayoutEditDraft>,
    ) {
        let Some(draft) = draft else {
            user_interface.with_layout(Layout::centered_and_justified(Direction::TopDown), |user_interface| {
                user_interface.label(RichText::new("Select a struct layout to edit its fields.").color(self.app_context.theme.foreground_preview));
            });
            return;
        };

        let mut edited_draft = draft.clone();
        let validation_result = StructEditorViewData::build_struct_layout_descriptor(project_symbol_catalog, &edited_draft);
        let usage_count = selected_layout_id
            .map(|selected_layout_id| StructEditorViewData::count_rooted_symbol_usages(project_symbol_catalog, selected_layout_id))
            .unwrap_or(0);
        let is_creating_new_layout = edited_draft.original_layout_id.is_none();

        user_interface.horizontal(|user_interface| {
            user_interface.label(RichText::new("Struct Layout Id").strong());
            user_interface.add_space(8.0);
            self.render_string_value_box(
                user_interface,
                &mut edited_draft.layout_id,
                "module.type",
                "struct_editor_layout_id",
                280.0,
                Self::FIELD_ROW_HEIGHT,
            );
        });
        user_interface.add_space(6.0);

        let status_text = if is_creating_new_layout {
            String::from("Creating a new reusable struct layout.")
        } else if usage_count == 0 {
            String::from("Not used by any rooted symbols yet.")
        } else if usage_count == 1 {
            String::from("Used by 1 rooted symbol.")
        } else {
            format!("Used by {} rooted symbols.", usage_count)
        };
        user_interface.label(RichText::new(status_text).color(self.app_context.theme.foreground_preview));
        user_interface.add_space(12.0);

        self.render_field_rows(user_interface, &mut edited_draft);
        user_interface.add_space(12.0);

        if let Err(validation_error) = validation_result.as_ref() {
            user_interface.label(RichText::new(validation_error).color(self.app_context.theme.error_red));
            user_interface.add_space(8.0);
        }

        user_interface.horizontal(|user_interface| {
            let can_apply = validation_result.is_ok();
            let apply_response = self.render_text_button(
                user_interface,
                "Apply",
                "Apply this struct layout draft.",
                96.0,
                Self::FIELD_ROW_HEIGHT,
                !can_apply,
            );
            if apply_response.clicked() {
                match StructEditorViewData::apply_draft_to_catalog(project_symbol_catalog, &edited_draft) {
                    Ok(updated_project_symbol_catalog) => {
                        self.persist_project_symbol_catalog(updated_project_symbol_catalog.clone());
                        StructEditorViewData::select_struct_layout(
                            self.struct_editor_view_data.clone(),
                            &updated_project_symbol_catalog,
                            Some(edited_draft.layout_id.trim().to_string()),
                        );
                    }
                    Err(error) => {
                        log::error!("Failed to apply struct editor draft: {}.", error);
                    }
                }
            }

            let revert_response = self.render_text_button(
                user_interface,
                "Revert",
                "Revert this draft back to the selected struct layout.",
                96.0,
                Self::FIELD_ROW_HEIGHT,
                false,
            );
            if revert_response.clicked() {
                if let Some(original_layout_id) = edited_draft.original_layout_id.clone() {
                    StructEditorViewData::select_struct_layout(self.struct_editor_view_data.clone(), project_symbol_catalog, Some(original_layout_id));
                } else {
                    StructEditorViewData::begin_create_struct_layout(
                        self.struct_editor_view_data.clone(),
                        project_symbol_catalog,
                        self.default_data_type_ref(),
                    );
                }
            }
        });

        StructEditorViewData::update_draft(self.struct_editor_view_data.clone(), edited_draft);
    }
}

impl Widget for StructEditorView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> eframe::egui::Response {
        let Some(project_symbol_catalog) = self.get_opened_project_symbol_catalog() else {
            return user_interface
                .allocate_ui_with_layout(
                    user_interface.available_size(),
                    Layout::centered_and_justified(Direction::TopDown),
                    |user_interface| {
                        user_interface
                            .label(RichText::new("Open a project to author reusable struct layouts.").color(self.app_context.theme.foreground_preview));
                    },
                )
                .response;
        };

        StructEditorViewData::synchronize(self.struct_editor_view_data.clone(), &project_symbol_catalog);
        let (selected_layout_id, filter_text, draft) = self
            .struct_editor_view_data
            .read("Struct editor view")
            .map(|struct_editor_view_data| {
                (
                    struct_editor_view_data
                        .get_selected_layout_id()
                        .map(str::to_string),
                    struct_editor_view_data.get_filter_text().to_string(),
                    struct_editor_view_data.get_draft().cloned(),
                )
            })
            .unwrap_or((None, String::new(), None));
        let is_creating_new_layout = draft
            .as_ref()
            .is_some_and(|draft| draft.original_layout_id.is_none());

        user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                user_interface.horizontal(|user_interface| {
                    user_interface.allocate_ui_with_layout(
                        vec2(Self::LIST_PANEL_WIDTH.min(user_interface.available_width()), user_interface.available_height()),
                        Layout::top_down(Align::Min),
                        |user_interface| {
                            self.render_list_panel(
                                user_interface,
                                &project_symbol_catalog,
                                selected_layout_id.as_deref(),
                                &filter_text,
                                is_creating_new_layout,
                            );
                        },
                    );

                    let separator_response = user_interface.allocate_response(vec2(8.0, user_interface.available_height()), Sense::hover());
                    user_interface.painter().vline(
                        separator_response.rect.center().x,
                        separator_response.rect.y_range(),
                        (1.0, self.app_context.theme.background_control),
                    );

                    user_interface.allocate_ui_with_layout(
                        vec2(user_interface.available_width(), user_interface.available_height()),
                        Layout::top_down(Align::Min),
                        |user_interface| {
                            self.render_editor_panel(user_interface, &project_symbol_catalog, selected_layout_id.as_deref(), draft.as_ref());
                        },
                    );
                });
            })
            .response
    }
}
