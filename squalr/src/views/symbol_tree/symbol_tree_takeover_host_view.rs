use crate::app_context::AppContext;
use crate::views::symbol_tree::symbol_tree_define_field_view::{SymbolTreeDefineFieldAction, SymbolTreeDefineFieldView};
use crate::views::symbol_tree::symbol_tree_delete_confirmation_view::{SymbolTreeDeleteConfirmationAction, SymbolTreeDeleteConfirmationView};
use crate::views::symbol_tree::symbol_tree_module_create_view::{SymbolTreeModuleCreateAction, SymbolTreeModuleCreateView};
use crate::views::symbol_tree::view_data::symbol_tree_view_data::{DefineFieldDraft, ModuleRootCreateDraft, SymbolTreeSelection, SymbolTreeTakeOverState};
use eframe::egui::Ui;
use squalr_engine_api::commands::project_symbols::create_module::project_symbols_create_module_request::ProjectSymbolsCreateModuleRequest;
use squalr_engine_api::commands::project_symbols::delete::project_symbols_delete_request::ProjectSymbolsDeleteModuleRangeMode;
use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
use squalr_engine_api::structures::projects::symbol_tree::operations::define_field::DefineFieldPlan;
use squalr_engine_api::structures::projects::symbol_tree::operations::delete_symbol::build_delete_module_range_confirmation_description;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum SymbolTreeTakeoverHostAction {
    None,
    CancelTakeover,
    CancelModuleRootCreate,
    DeleteSymbolClaim {
        symbol_locator_key: String,
    },
    DeleteModuleRoot {
        module_name: String,
    },
    DeleteModuleRange {
        module_name: String,
        offset: u64,
        length: u64,
        mode: ProjectSymbolsDeleteModuleRangeMode,
    },
    CreateFieldFromUnassignedSegment {
        module_name: String,
        define_field_plan: DefineFieldPlan,
    },
    DefineFieldDraftChanged(DefineFieldDraft),
    CreateModuleRoot(ProjectSymbolsCreateModuleRequest),
    ModuleRootCreateDraftChanged(ModuleRootCreateDraft),
}

pub struct SymbolTreeTakeoverHostResponse {
    pub is_active: bool,
    pub action: SymbolTreeTakeoverHostAction,
}

pub struct SymbolTreeTakeoverHostView<'a> {
    app_context: Arc<AppContext>,
    project_symbol_catalog: &'a ProjectSymbolCatalog,
    selected_entry: Option<&'a SymbolTreeSelection>,
    take_over_state: Option<&'a SymbolTreeTakeOverState>,
    module_root_create_draft: &'a ModuleRootCreateDraft,
    define_field_draft: &'a DefineFieldDraft,
}

impl<'a> SymbolTreeTakeoverHostView<'a> {
    pub fn new(
        app_context: Arc<AppContext>,
        project_symbol_catalog: &'a ProjectSymbolCatalog,
        selected_entry: Option<&'a SymbolTreeSelection>,
        take_over_state: Option<&'a SymbolTreeTakeOverState>,
        module_root_create_draft: &'a ModuleRootCreateDraft,
        define_field_draft: &'a DefineFieldDraft,
    ) -> Self {
        Self {
            app_context,
            project_symbol_catalog,
            selected_entry,
            take_over_state,
            module_root_create_draft,
            define_field_draft,
        }
    }

    pub fn show(
        self,
        user_interface: &mut Ui,
    ) -> SymbolTreeTakeoverHostResponse {
        if let Some(response) = self.show_takeover_state(user_interface) {
            return response;
        }

        if matches!(self.selected_entry, Some(SymbolTreeSelection::CreateModuleRoot)) {
            return self.show_module_root_create(user_interface);
        }

        SymbolTreeTakeoverHostResponse {
            is_active: false,
            action: SymbolTreeTakeoverHostAction::None,
        }
    }

    fn show_takeover_state(
        &self,
        user_interface: &mut Ui,
    ) -> Option<SymbolTreeTakeoverHostResponse> {
        let action = match self.take_over_state? {
            SymbolTreeTakeOverState::DeleteSymbolClaimConfirmation {
                symbol_locator_key,
                display_name,
            } => {
                let description_text = String::from("This removes the authored symbol from the project.");

                user_interface.add_space(8.0);
                match SymbolTreeDeleteConfirmationView::new(self.app_context.clone(), "Delete this symbol", display_name, &description_text, false, "Delete")
                    .show(user_interface)
                {
                    SymbolTreeDeleteConfirmationAction::Confirm => SymbolTreeTakeoverHostAction::DeleteSymbolClaim {
                        symbol_locator_key: symbol_locator_key.clone(),
                    },
                    SymbolTreeDeleteConfirmationAction::Cancel => SymbolTreeTakeoverHostAction::CancelTakeover,
                    SymbolTreeDeleteConfirmationAction::None => SymbolTreeTakeoverHostAction::None,
                }
            }
            SymbolTreeTakeOverState::DeleteModuleRootConfirmation { module_name } => {
                let description_text = String::from("This removes the module root and all symbol claims inside it.");

                user_interface.add_space(8.0);
                match SymbolTreeDeleteConfirmationView::new(self.app_context.clone(), "Delete this module", module_name, &description_text, false, "Delete")
                    .show(user_interface)
                {
                    SymbolTreeDeleteConfirmationAction::Confirm => SymbolTreeTakeoverHostAction::DeleteModuleRoot {
                        module_name: module_name.clone(),
                    },
                    SymbolTreeDeleteConfirmationAction::Cancel => SymbolTreeTakeoverHostAction::CancelTakeover,
                    SymbolTreeDeleteConfirmationAction::None => SymbolTreeTakeoverHostAction::None,
                }
            }
            SymbolTreeTakeOverState::DeleteModuleRangeConfirmation {
                module_name,
                offset,
                length,
                display_name,
                mode,
            } => {
                let delete_confirmation_description = build_delete_module_range_confirmation_description(module_name, *length, *mode);
                let description_text = delete_confirmation_description.text;

                user_interface.add_space(8.0);
                match SymbolTreeDeleteConfirmationView::new(
                    self.app_context.clone(),
                    "Delete this field",
                    display_name,
                    &description_text,
                    delete_confirmation_description.is_warning,
                    "Delete",
                )
                .show(user_interface)
                {
                    SymbolTreeDeleteConfirmationAction::Confirm => SymbolTreeTakeoverHostAction::DeleteModuleRange {
                        module_name: module_name.clone(),
                        offset: *offset,
                        length: *length,
                        mode: *mode,
                    },
                    SymbolTreeDeleteConfirmationAction::Cancel => SymbolTreeTakeoverHostAction::CancelTakeover,
                    SymbolTreeDeleteConfirmationAction::None => SymbolTreeTakeoverHostAction::None,
                }
            }
            SymbolTreeTakeOverState::DefineFieldFromUnassignedSegment { module_name, offset, length } => {
                user_interface.add_space(8.0);
                match SymbolTreeDefineFieldView::new(
                    self.app_context.clone(),
                    self.project_symbol_catalog,
                    module_name,
                    *offset,
                    *length,
                    self.define_field_draft,
                )
                .show(user_interface)
                {
                    SymbolTreeDefineFieldAction::Cancel => SymbolTreeTakeoverHostAction::CancelTakeover,
                    SymbolTreeDefineFieldAction::Create(define_field_plan) => SymbolTreeTakeoverHostAction::CreateFieldFromUnassignedSegment {
                        module_name: module_name.clone(),
                        define_field_plan,
                    },
                    SymbolTreeDefineFieldAction::DraftChanged(define_field_draft) => SymbolTreeTakeoverHostAction::DefineFieldDraftChanged(define_field_draft),
                    SymbolTreeDefineFieldAction::None => SymbolTreeTakeoverHostAction::None,
                }
            }
        };

        Some(SymbolTreeTakeoverHostResponse { is_active: true, action })
    }

    fn show_module_root_create(
        &self,
        user_interface: &mut Ui,
    ) -> SymbolTreeTakeoverHostResponse {
        user_interface.add_space(8.0);
        let action = match SymbolTreeModuleCreateView::new(self.app_context.clone(), self.module_root_create_draft).show(user_interface) {
            SymbolTreeModuleCreateAction::Cancel => SymbolTreeTakeoverHostAction::CancelModuleRootCreate,
            SymbolTreeModuleCreateAction::Create(project_symbols_create_module_request) => {
                SymbolTreeTakeoverHostAction::CreateModuleRoot(project_symbols_create_module_request)
            }
            SymbolTreeModuleCreateAction::DraftChanged(module_root_create_draft) => {
                SymbolTreeTakeoverHostAction::ModuleRootCreateDraftChanged(module_root_create_draft)
            }
            SymbolTreeModuleCreateAction::None => SymbolTreeTakeoverHostAction::None,
        };

        SymbolTreeTakeoverHostResponse { is_active: true, action }
    }
}
