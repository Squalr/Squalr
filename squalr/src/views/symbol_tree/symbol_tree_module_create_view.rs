use crate::app_context::AppContext;
use crate::ui::widgets::controls::groupbox::GroupBox;
use crate::views::symbol_tree::symbol_tree_take_over_view_helpers::{
    SYMBOL_TREE_TAKE_OVER_ACTION_BUTTON_SPACING, SYMBOL_TREE_TAKE_OVER_ACTION_BUTTON_WIDTH, SYMBOL_TREE_TAKE_OVER_BOTTOM_PADDING,
    SYMBOL_TREE_TAKE_OVER_GROUPBOX_SIDE_PADDING, SYMBOL_TREE_TAKE_OVER_ROW_HEIGHT, draw_sized_action_button, render_offset_data_value_box,
    render_string_data_value_box,
};
use crate::views::symbol_tree::view_data::symbol_tree_view_data::ModuleRootCreateDraft;
use eframe::egui::{Direction, Layout, RichText, Ui, vec2};
use squalr_engine_api::commands::project_symbols::create_module::project_symbols_create_module_request::ProjectSymbolsCreateModuleRequest;
use squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
use squalr_engine_api::structures::projects::symbol_tree::operations::define_field::parse_define_field_relative_offset;
use std::sync::Arc;

const CREATE_DISPLAY_NAME_DATA_VALUE_BOX_ID: &str = "symbol_tree_create_display_name";
const CREATE_MODULE_SIZE_DATA_VALUE_BOX_ID: &str = "symbol_tree_create_module_size";

#[derive(Clone, Debug)]
pub enum SymbolTreeModuleCreateAction {
    None,
    Cancel,
    Create(ProjectSymbolsCreateModuleRequest),
    DraftChanged(ModuleRootCreateDraft),
}

pub struct SymbolTreeModuleCreateView<'a> {
    app_context: Arc<AppContext>,
    module_root_create_draft: &'a ModuleRootCreateDraft,
}

impl<'a> SymbolTreeModuleCreateView<'a> {
    pub fn new(
        app_context: Arc<AppContext>,
        module_root_create_draft: &'a ModuleRootCreateDraft,
    ) -> Self {
        Self {
            app_context,
            module_root_create_draft,
        }
    }

    pub fn parse_module_root_size(
        size_text: &str,
        size_format: AnonymousValueStringFormat,
    ) -> Option<u64> {
        parse_define_field_relative_offset(size_text, size_format).ok()
    }

    pub fn build_module_root_create_request_from_draft(edited_draft: &ModuleRootCreateDraft) -> Option<ProjectSymbolsCreateModuleRequest> {
        let parsed_size = Self::parse_module_root_size(&edited_draft.size_text, edited_draft.size_format);

        if edited_draft.module_name.trim().is_empty() {
            return None;
        }

        Some(ProjectSymbolsCreateModuleRequest {
            module_name: edited_draft.module_name.trim().to_string(),
            size: parsed_size?,
        })
    }

    pub fn show(
        self,
        user_interface: &mut Ui,
    ) -> SymbolTreeModuleCreateAction {
        let theme = &self.app_context.theme;
        let original_draft = self.module_root_create_draft.clone();
        let mut edited_draft = original_draft.clone();
        let mut action = SymbolTreeModuleCreateAction::None;
        let mut create_module_root_request = None;

        user_interface.allocate_ui_with_layout(
            user_interface.available_size(),
            Layout::centered_and_justified(Direction::TopDown),
            |user_interface| {
                let panel_width = (user_interface.available_width() - SYMBOL_TREE_TAKE_OVER_GROUPBOX_SIDE_PADDING * 2.0).max(0.0);

                user_interface.horizontal(|user_interface| {
                    user_interface.add_space(SYMBOL_TREE_TAKE_OVER_GROUPBOX_SIDE_PADDING);
                    user_interface.add(
                        GroupBox::new_from_theme(theme, "New Module", |user_interface| {
                            Self::render_create_module_root_details(&self.app_context, user_interface, &mut edited_draft);
                            create_module_root_request = Self::build_module_root_create_request_from_draft(&edited_draft);

                            user_interface.add_space(12.0);
                            user_interface.allocate_ui(vec2(user_interface.available_width(), 32.0), |user_interface| {
                                let button_size = vec2(SYMBOL_TREE_TAKE_OVER_ACTION_BUTTON_WIDTH, SYMBOL_TREE_TAKE_OVER_ROW_HEIGHT);
                                let total_button_row_width = button_size.x * 2.0 + SYMBOL_TREE_TAKE_OVER_ACTION_BUTTON_SPACING;
                                let side_spacing = ((user_interface.available_width() - total_button_row_width) * 0.5).max(0.0);

                                user_interface.horizontal(|user_interface| {
                                    user_interface.add_space(side_spacing);
                                    user_interface.spacing_mut().item_spacing.x = SYMBOL_TREE_TAKE_OVER_ACTION_BUTTON_SPACING;

                                    let button_cancel = draw_sized_action_button(
                                        &self.app_context,
                                        user_interface,
                                        "Cancel",
                                        button_size,
                                        theme.background_control_secondary,
                                        theme.background_control_secondary_dark,
                                        true,
                                    );

                                    if button_cancel.clicked() {
                                        action = SymbolTreeModuleCreateAction::Cancel;
                                    }

                                    let can_create_module = create_module_root_request.is_some();
                                    let create_fill = if can_create_module {
                                        theme.background_control_primary
                                    } else {
                                        theme.background_control_secondary
                                    };
                                    let create_stroke = if can_create_module {
                                        theme.background_control_primary_dark
                                    } else {
                                        theme.background_control_secondary_dark
                                    };
                                    let button_create = draw_sized_action_button(
                                        &self.app_context,
                                        user_interface,
                                        "Create",
                                        button_size,
                                        create_fill,
                                        create_stroke,
                                        can_create_module,
                                    );

                                    if button_create.clicked() {
                                        if let Some(project_symbols_create_module_request) = create_module_root_request.clone() {
                                            action = SymbolTreeModuleCreateAction::Create(project_symbols_create_module_request);
                                        }
                                    }
                                });
                            });
                            user_interface.add_space(SYMBOL_TREE_TAKE_OVER_BOTTOM_PADDING);
                        })
                        .desired_width(panel_width),
                    );
                });
            },
        );

        if matches!(action, SymbolTreeModuleCreateAction::None) && edited_draft != original_draft {
            return SymbolTreeModuleCreateAction::DraftChanged(edited_draft);
        }

        action
    }

    fn render_create_module_root_details(
        app_context: &Arc<AppContext>,
        user_interface: &mut Ui,
        edited_draft: &mut ModuleRootCreateDraft,
    ) {
        let theme = &app_context.theme;

        user_interface.label(RichText::new("Module Name").color(theme.foreground));
        user_interface.add_space(2.0);
        render_string_data_value_box(
            app_context.clone(),
            user_interface,
            &mut edited_draft.module_name,
            "",
            CREATE_DISPLAY_NAME_DATA_VALUE_BOX_ID,
            user_interface.available_width(),
        );
        user_interface.add_space(8.0);

        user_interface.label(RichText::new("Initial Module Size").color(theme.foreground));
        user_interface.add_space(2.0);
        render_offset_data_value_box(
            app_context.clone(),
            user_interface,
            &mut edited_draft.size_text,
            &mut edited_draft.size_format,
            "",
            CREATE_MODULE_SIZE_DATA_VALUE_BOX_ID,
            user_interface.available_width(),
        );
    }
}
