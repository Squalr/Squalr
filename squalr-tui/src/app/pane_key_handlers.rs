use super::app_shell::AppShell;
use crate::state::pane::TuiPane;
use crate::views::element_scanner::pane_state::ElementScannerFocusTarget;
use crate::views::process_selector::pane_state::ProcessSelectorInputMode;
use crate::views::project_explorer::pane_state::{ProjectExplorerFocusTarget, ProjectSelectorInputMode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use squalr_engine::squalr_engine::SqualrEngine;

impl AppShell {
    pub(super) fn handle_focused_pane_event(
        &mut self,
        key_event: KeyEvent,
        squalr_engine: &mut SqualrEngine,
    ) {
        match self.app_state.focused_pane() {
            TuiPane::ProcessSelector => self.handle_process_selector_key_event(key_event, squalr_engine),
            TuiPane::ElementScanner => self.handle_element_scanner_key_event(key_event, squalr_engine),
            TuiPane::ScanResults => self.handle_scan_results_key_event(key_event, squalr_engine),
            TuiPane::ProjectExplorer => self.handle_project_explorer_key_event(key_event, squalr_engine),
            TuiPane::StructViewer => self.handle_struct_viewer_key_event(key_event, squalr_engine),
            TuiPane::Output => self.handle_output_key_event(key_event.code, squalr_engine),
            TuiPane::Settings => self.handle_settings_key_event(key_event, squalr_engine),
        }
    }

    pub(super) fn handle_output_key_event(
        &mut self,
        key_code: KeyCode,
        squalr_engine: &mut SqualrEngine,
    ) {
        match key_code {
            KeyCode::Char('r') => self.refresh_output_log_history_with_feedback(squalr_engine, true),
            KeyCode::Char('x') | KeyCode::Delete => self.app_state.output_pane_state.clear_log_lines(),
            KeyCode::Char('+') | KeyCode::Char('=') => self.app_state.output_pane_state.increase_max_line_count(),
            KeyCode::Char('-') => self.app_state.output_pane_state.decrease_max_line_count(),
            _ => {}
        }
    }

    pub(super) fn handle_settings_key_event(
        &mut self,
        key_event: KeyEvent,
        squalr_engine: &mut SqualrEngine,
    ) {
        match key_event.code {
            KeyCode::Char('r') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.reset_selected_settings_category_to_defaults(squalr_engine);
            }
            KeyCode::Char('r') => self.refresh_all_settings_categories_with_feedback(squalr_engine, true),
            KeyCode::Left => self.app_state.settings_pane_state.cycle_category_backward(),
            KeyCode::Right => self.app_state.settings_pane_state.cycle_category_forward(),
            KeyCode::Char(']') => self.app_state.settings_pane_state.cycle_category_forward(),
            KeyCode::Char('[') => self.app_state.settings_pane_state.cycle_category_backward(),
            KeyCode::Down => self.app_state.settings_pane_state.select_next_field(),
            KeyCode::Up => self.app_state.settings_pane_state.select_previous_field(),
            KeyCode::Home => self.app_state.settings_pane_state.select_first_field(),
            KeyCode::End => self.app_state.settings_pane_state.select_last_field(),
            KeyCode::Esc => {
                self.app_state.settings_pane_state.cancel_pending_numeric_edit();
            }
            KeyCode::Backspace => {
                self.app_state
                    .settings_pane_state
                    .backspace_pending_numeric_edit();
            }
            KeyCode::Char(' ') => {
                if self
                    .app_state
                    .settings_pane_state
                    .toggle_selected_boolean_field()
                {
                    self.apply_selected_settings_category(squalr_engine);
                }
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                if self
                    .app_state
                    .settings_pane_state
                    .step_selected_numeric_field(true)
                {
                    self.apply_selected_settings_category(squalr_engine);
                }
            }
            KeyCode::Char('-') => {
                if self
                    .app_state
                    .settings_pane_state
                    .step_selected_numeric_field(false)
                {
                    self.apply_selected_settings_category(squalr_engine);
                }
            }
            KeyCode::Char('>') | KeyCode::Char('.') => {
                if self
                    .app_state
                    .settings_pane_state
                    .cycle_selected_enum_field(true)
                {
                    self.apply_selected_settings_category(squalr_engine);
                }
            }
            KeyCode::Char('<') | KeyCode::Char(',') => {
                if self
                    .app_state
                    .settings_pane_state
                    .cycle_selected_enum_field(false)
                {
                    self.apply_selected_settings_category(squalr_engine);
                }
            }
            KeyCode::Char('u') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.app_state.settings_pane_state.clear_pending_numeric_edit();
            }
            KeyCode::Enter => {
                if self.app_state.settings_pane_state.commit_pending_numeric_edit() {
                    self.apply_selected_settings_category(squalr_engine);
                } else {
                    self.apply_selected_settings_category(squalr_engine);
                }
            }
            KeyCode::Char(pending_character) => {
                self.app_state
                    .settings_pane_state
                    .append_pending_numeric_edit_character(pending_character);
            }
            _ => {}
        }
    }

    pub(super) fn handle_process_selector_key_event(
        &mut self,
        key_event: KeyEvent,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.process_selector_pane_state.input_mode == ProcessSelectorInputMode::Search {
            match key_event.code {
                KeyCode::Esc => {
                    self.app_state.process_selector_pane_state.cancel_search_input();
                }
                KeyCode::Enter => {
                    self.app_state.process_selector_pane_state.commit_search_input();
                }
                KeyCode::Backspace => {
                    self.app_state
                        .process_selector_pane_state
                        .backspace_pending_search_name();
                }
                KeyCode::Char('u') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.app_state
                        .process_selector_pane_state
                        .clear_pending_search_name();
                }
                KeyCode::Down => {
                    self.app_state.process_selector_pane_state.commit_search_input();
                }
                KeyCode::Char(search_character) => {
                    self.app_state
                        .process_selector_pane_state
                        .append_pending_search_character(search_character);
                }
                _ => {}
            }

            return;
        }

        match key_event.code {
            KeyCode::Char('r') => self.refresh_process_list(squalr_engine),
            KeyCode::Char('w') => {
                let updated_windowed_filter = !self
                    .app_state
                    .process_selector_pane_state
                    .show_windowed_processes_only;
                self.app_state
                    .process_selector_pane_state
                    .set_windowed_filter(updated_windowed_filter);
                self.refresh_process_list(squalr_engine);
            }
            KeyCode::Char('/') => self.app_state.process_selector_pane_state.begin_search_input(),
            KeyCode::Down => self.app_state.process_selector_pane_state.select_next_process(),
            KeyCode::Up => {
                if self
                    .app_state
                    .process_selector_pane_state
                    .selected_process_list_index
                    == Some(0)
                {
                    self.app_state.process_selector_pane_state.begin_search_input();
                } else {
                    self.app_state
                        .process_selector_pane_state
                        .select_previous_process();
                }
            }
            KeyCode::Home => self
                .app_state
                .process_selector_pane_state
                .select_first_process(),
            KeyCode::End => self.app_state.process_selector_pane_state.select_last_process(),
            KeyCode::Enter | KeyCode::Char('o') => self.open_selected_process(squalr_engine),
            _ => {}
        }
    }

    pub(super) fn handle_element_scanner_key_event(
        &mut self,
        key_event: KeyEvent,
        squalr_engine: &mut SqualrEngine,
    ) {
        match key_event.code {
            KeyCode::Char('s') => self.start_element_scan(squalr_engine),
            KeyCode::Char('n') => self.reset_scan_state(squalr_engine),
            KeyCode::Char('c') => self.collect_scan_values(squalr_engine),
            KeyCode::Right => self
                .app_state
                .element_scanner_pane_state
                .select_data_type_right(),
            KeyCode::Left => self
                .app_state
                .element_scanner_pane_state
                .select_data_type_left(),
            KeyCode::Down => self.app_state.element_scanner_pane_state.move_focus_down(),
            KeyCode::Up => self.app_state.element_scanner_pane_state.move_focus_up(),
            KeyCode::Char(' ') | KeyCode::Enter => {
                if self.app_state.element_scanner_pane_state.focus_target == ElementScannerFocusTarget::DataTypes
                    && self
                        .app_state
                        .element_scanner_pane_state
                        .toggle_hovered_data_type_selection()
                {
                    self.sync_scan_results_type_filters_from_element_scanner();
                }
            }
            KeyCode::Char(']') => {
                if self.app_state.element_scanner_pane_state.focus_target != ElementScannerFocusTarget::Constraints {
                    return;
                }
                self.app_state
                    .element_scanner_pane_state
                    .select_next_constraint();
            }
            KeyCode::Char('[') => {
                if self.app_state.element_scanner_pane_state.focus_target != ElementScannerFocusTarget::Constraints {
                    return;
                }
                self.app_state
                    .element_scanner_pane_state
                    .select_previous_constraint();
            }
            KeyCode::Char('m') => {
                if self.app_state.element_scanner_pane_state.focus_target != ElementScannerFocusTarget::Constraints {
                    return;
                }
                self.app_state
                    .element_scanner_pane_state
                    .cycle_selected_constraint_compare_type_forward();
            }
            KeyCode::Char('M') => {
                if self.app_state.element_scanner_pane_state.focus_target != ElementScannerFocusTarget::Constraints {
                    return;
                }
                self.app_state
                    .element_scanner_pane_state
                    .cycle_selected_constraint_compare_type_backward();
            }
            KeyCode::Char('a') => {
                if self.app_state.element_scanner_pane_state.focus_target != ElementScannerFocusTarget::Constraints {
                    return;
                }
                if !self.app_state.element_scanner_pane_state.add_constraint() {
                    self.app_state.element_scanner_pane_state.status_message = "Maximum of five constraints reached.".to_string();
                }
            }
            KeyCode::Char('x') => {
                if self.app_state.element_scanner_pane_state.focus_target != ElementScannerFocusTarget::Constraints {
                    return;
                }
                if !self
                    .app_state
                    .element_scanner_pane_state
                    .remove_selected_constraint()
                {
                    self.app_state.element_scanner_pane_state.status_message = "At least one constraint is required.".to_string();
                }
            }
            KeyCode::Backspace => {
                if self.app_state.element_scanner_pane_state.focus_target != ElementScannerFocusTarget::Constraints {
                    return;
                }
                self.app_state
                    .element_scanner_pane_state
                    .backspace_selected_constraint_value();
            }
            KeyCode::Char('u') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                if self.app_state.element_scanner_pane_state.focus_target != ElementScannerFocusTarget::Constraints {
                    return;
                }
                self.app_state
                    .element_scanner_pane_state
                    .clear_selected_constraint_value();
            }
            KeyCode::Char(scan_value_character) => {
                if self.app_state.element_scanner_pane_state.focus_target != ElementScannerFocusTarget::Constraints {
                    return;
                }
                self.app_state
                    .element_scanner_pane_state
                    .append_selected_constraint_value_character(scan_value_character);
            }
            _ => {}
        }
    }

    pub(super) fn handle_scan_results_key_event(
        &mut self,
        key_event: KeyEvent,
        squalr_engine: &mut SqualrEngine,
    ) {
        let is_range_extend_modifier_active = key_event.modifiers.contains(KeyModifiers::SHIFT);

        match key_event.code {
            KeyCode::Char('r') => self.query_scan_results_current_page(squalr_engine),
            KeyCode::Char('R') => self.refresh_scan_results_page(squalr_engine),
            KeyCode::Char(']') => self.query_next_scan_results_page(squalr_engine),
            KeyCode::Char('[') => self.query_previous_scan_results_page(squalr_engine),
            KeyCode::Down => {
                if is_range_extend_modifier_active {
                    self.app_state
                        .scan_results_pane_state
                        .set_selected_range_end_to_current();
                }
                self.app_state
                    .scan_results_pane_state
                    .select_next_result(is_range_extend_modifier_active);
            }
            KeyCode::Up => {
                if is_range_extend_modifier_active {
                    self.app_state
                        .scan_results_pane_state
                        .set_selected_range_end_to_current();
                }
                self.app_state
                    .scan_results_pane_state
                    .select_previous_result(is_range_extend_modifier_active);
            }
            KeyCode::Home => {
                if is_range_extend_modifier_active {
                    self.app_state
                        .scan_results_pane_state
                        .set_selected_range_end_to_current();
                }
                self.app_state
                    .scan_results_pane_state
                    .select_first_result(is_range_extend_modifier_active);
            }
            KeyCode::End => {
                if is_range_extend_modifier_active {
                    self.app_state
                        .scan_results_pane_state
                        .set_selected_range_end_to_current();
                }
                self.app_state
                    .scan_results_pane_state
                    .select_last_result(is_range_extend_modifier_active);
            }
            KeyCode::Char('f') => self.toggle_selected_scan_results_frozen_state(squalr_engine),
            KeyCode::Char('a') => self.add_selected_scan_results_to_project(squalr_engine),
            KeyCode::Char('x') | KeyCode::Delete => self.delete_selected_scan_results(squalr_engine),
            KeyCode::Enter => self.commit_selected_scan_results_value_edit(squalr_engine),
            KeyCode::Backspace => self
                .app_state
                .scan_results_pane_state
                .backspace_pending_value_edit(),
            KeyCode::Char('u') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.app_state
                    .scan_results_pane_state
                    .clear_pending_value_edit();
            }
            KeyCode::Char('y') => self
                .app_state
                .scan_results_pane_state
                .sync_pending_value_edit_from_selection(),
            KeyCode::Char(scan_value_character) => self
                .app_state
                .scan_results_pane_state
                .append_pending_value_edit_character(scan_value_character),
            _ => {}
        }
    }

    pub(super) fn handle_project_explorer_key_event(
        &mut self,
        key_event: KeyEvent,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.project_explorer_pane_state.input_mode != ProjectSelectorInputMode::None {
            match self.app_state.project_explorer_pane_state.input_mode {
                ProjectSelectorInputMode::Search => match key_event.code {
                    KeyCode::Esc => self.app_state.project_explorer_pane_state.cancel_search_input(),
                    KeyCode::Enter => self.app_state.project_explorer_pane_state.commit_search_input(),
                    KeyCode::Backspace => self
                        .app_state
                        .project_explorer_pane_state
                        .backspace_pending_search_name(),
                    KeyCode::Char('u') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.app_state
                            .project_explorer_pane_state
                            .clear_pending_search_name();
                    }
                    KeyCode::Down => {
                        self.app_state.project_explorer_pane_state.commit_search_input();
                    }
                    KeyCode::Char(search_character) => {
                        self.app_state
                            .project_explorer_pane_state
                            .append_pending_search_character(search_character);
                    }
                    _ => {}
                },
                ProjectSelectorInputMode::CreatingProject | ProjectSelectorInputMode::RenamingProject | ProjectSelectorInputMode::CreatingProjectDirectory => {
                    match key_event.code {
                        KeyCode::Esc => self
                            .app_state
                            .project_explorer_pane_state
                            .cancel_project_name_input(),
                        KeyCode::Backspace => self
                            .app_state
                            .project_explorer_pane_state
                            .backspace_pending_project_name(),
                        KeyCode::Char('u') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                            self.app_state
                                .project_explorer_pane_state
                                .clear_pending_project_name();
                        }
                        KeyCode::Enter => self.commit_project_selector_input(squalr_engine),
                        KeyCode::Char(project_name_character) => {
                            self.app_state
                                .project_explorer_pane_state
                                .append_pending_project_name_character(project_name_character);
                        }
                        _ => {}
                    }
                }
                ProjectSelectorInputMode::None => {}
            }

            return;
        }

        match self.app_state.project_explorer_pane_state.focus_target {
            ProjectExplorerFocusTarget::ProjectList => self.handle_project_list_key_event(key_event.code, squalr_engine),
            ProjectExplorerFocusTarget::ProjectHierarchy => self.handle_project_hierarchy_key_event(key_event.code, squalr_engine),
        }
    }

    pub(super) fn handle_project_list_key_event(
        &mut self,
        key_code: KeyCode,
        squalr_engine: &mut SqualrEngine,
    ) {
        match key_code {
            KeyCode::Char('r') => self.refresh_project_list(squalr_engine),
            KeyCode::Char('/') => self.app_state.project_explorer_pane_state.begin_search_input(),
            KeyCode::Down => self.app_state.project_explorer_pane_state.select_next_project(),
            KeyCode::Up => self
                .app_state
                .project_explorer_pane_state
                .select_previous_project(),
            KeyCode::Home => self
                .app_state
                .project_explorer_pane_state
                .select_first_project(),
            KeyCode::End => self.app_state.project_explorer_pane_state.select_last_project(),
            KeyCode::Enter | KeyCode::Char('o') => self.open_selected_project(squalr_engine),
            KeyCode::Char('n') => self
                .app_state
                .project_explorer_pane_state
                .begin_create_project_input(),
            KeyCode::Char('e') => {
                if !self
                    .app_state
                    .project_explorer_pane_state
                    .begin_rename_selected_project_input()
                {
                    self.app_state.project_explorer_pane_state.status_message = "No project is selected for rename.".to_string();
                }
            }
            KeyCode::Char('x') | KeyCode::Delete => self.delete_selected_project(squalr_engine),
            KeyCode::Char('c') => self.close_active_project(squalr_engine),
            _ => {}
        }
    }

    pub(super) fn handle_project_hierarchy_key_event(
        &mut self,
        key_code: KeyCode,
        squalr_engine: &mut SqualrEngine,
    ) {
        match key_code {
            KeyCode::Char('r') => self.refresh_project_items_list(squalr_engine),
            KeyCode::Down => {
                self.app_state
                    .project_explorer_pane_state
                    .select_next_project_item();
                self.sync_struct_viewer_focus_from_project_items();
            }
            KeyCode::Up => {
                self.app_state
                    .project_explorer_pane_state
                    .select_previous_project_item();
                self.sync_struct_viewer_focus_from_project_items();
            }
            KeyCode::Home => {
                self.app_state
                    .project_explorer_pane_state
                    .select_first_project_item();
                self.sync_struct_viewer_focus_from_project_items();
            }
            KeyCode::End => {
                self.app_state
                    .project_explorer_pane_state
                    .select_last_project_item();
                self.sync_struct_viewer_focus_from_project_items();
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if !self
                    .app_state
                    .project_explorer_pane_state
                    .expand_selected_project_item_directory()
                {
                    self.app_state.project_explorer_pane_state.status_message = "No expandable directory is selected.".to_string();
                }
                self.sync_struct_viewer_focus_from_project_items();
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if !self
                    .app_state
                    .project_explorer_pane_state
                    .collapse_selected_project_item_directory_or_select_parent()
                {
                    self.app_state.project_explorer_pane_state.status_message = "No collapsible directory is selected.".to_string();
                }
                self.sync_struct_viewer_focus_from_project_items();
            }
            KeyCode::Char(' ') => {
                self.toggle_selected_project_item_activation(squalr_engine);
                self.sync_struct_viewer_focus_from_project_items();
            }
            KeyCode::Char('n') => {
                if !self
                    .app_state
                    .project_explorer_pane_state
                    .begin_create_project_directory_input()
                {
                    self.app_state.project_explorer_pane_state.status_message = "No project item directory target is selected.".to_string();
                }
            }
            KeyCode::Char('m') => {
                if self
                    .app_state
                    .project_explorer_pane_state
                    .stage_selected_project_item_for_move()
                {
                    self.app_state.project_explorer_pane_state.status_message =
                        "Staged selected project item for move. Select destination and press b.".to_string();
                } else {
                    self.app_state.project_explorer_pane_state.status_message = "No project item is selected for move.".to_string();
                }
            }
            KeyCode::Char('b') => self.move_staged_project_items_to_selected_directory(squalr_engine),
            KeyCode::Char('u') => {
                self.app_state
                    .project_explorer_pane_state
                    .clear_pending_move_source_paths();
                self.app_state.project_explorer_pane_state.status_message = "Cleared staged project item move.".to_string();
            }
            KeyCode::Char('[') => self.reorder_selected_project_item(squalr_engine, true),
            KeyCode::Char(']') => self.reorder_selected_project_item(squalr_engine, false),
            KeyCode::Char('x') | KeyCode::Delete => self.delete_selected_project_item_with_confirmation(squalr_engine),
            KeyCode::Char('c') => self.close_active_project(squalr_engine),
            _ => {}
        }
    }

    pub(super) fn handle_struct_viewer_key_event(
        &mut self,
        key_event: KeyEvent,
        squalr_engine: &mut SqualrEngine,
    ) {
        let apply_edit_input_guard = |struct_viewer_pane_state: &mut crate::views::struct_viewer::pane_state::StructViewerPaneState| -> bool {
            if let Some(block_reason) = struct_viewer_pane_state.selected_field_edit_block_reason() {
                struct_viewer_pane_state.status_message = block_reason;
                return false;
            }

            true
        };
        match key_event.code {
            KeyCode::Char('r') => self.refresh_struct_viewer_focus_from_source(),
            KeyCode::Down => self.app_state.struct_viewer_pane_state.select_next_field(),
            KeyCode::Up => self.app_state.struct_viewer_pane_state.select_previous_field(),
            KeyCode::Char('[') => {
                let selected_field_name = self
                    .app_state
                    .struct_viewer_pane_state
                    .selected_field_name
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string());
                match self
                    .app_state
                    .struct_viewer_pane_state
                    .cycle_selected_field_display_format_backward()
                {
                    Ok(active_display_format) => {
                        self.app_state.struct_viewer_pane_state.status_message =
                            format!("Set display format for field '{}' to {}.", selected_field_name, active_display_format);
                    }
                    Err(error) => {
                        self.app_state.struct_viewer_pane_state.status_message = error;
                    }
                }
            }
            KeyCode::Char(']') => {
                let selected_field_name = self
                    .app_state
                    .struct_viewer_pane_state
                    .selected_field_name
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string());
                match self
                    .app_state
                    .struct_viewer_pane_state
                    .cycle_selected_field_display_format_forward()
                {
                    Ok(active_display_format) => {
                        self.app_state.struct_viewer_pane_state.status_message =
                            format!("Set display format for field '{}' to {}.", selected_field_name, active_display_format);
                    }
                    Err(error) => {
                        self.app_state.struct_viewer_pane_state.status_message = error;
                    }
                }
            }
            KeyCode::Enter => self.commit_struct_viewer_field_edit(squalr_engine),
            KeyCode::Backspace => {
                if apply_edit_input_guard(&mut self.app_state.struct_viewer_pane_state) {
                    self.app_state.struct_viewer_pane_state.backspace_pending_edit();
                }
            }
            KeyCode::Char('u') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                if apply_edit_input_guard(&mut self.app_state.struct_viewer_pane_state) {
                    self.app_state.struct_viewer_pane_state.clear_pending_edit();
                }
            }
            KeyCode::Char(pending_character) => {
                if apply_edit_input_guard(&mut self.app_state.struct_viewer_pane_state) {
                    self.app_state
                        .struct_viewer_pane_state
                        .append_pending_edit_character(pending_character);
                }
            }
            _ => {}
        }
    }
}
