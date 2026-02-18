use crate::state::pane::TuiPane;
use crate::state::pane_entry_row::PaneEntryRow;
use crate::state::pane_layout_state::PaneLayoutState;
use crate::state::workspace_page::TuiWorkspacePage;
use crate::views::element_scanner::pane_state::ElementScannerPaneState;
use crate::views::output::pane_state::OutputPaneState;
use crate::views::output::summary::OUTPUT_FIXED_SUMMARY_LINE_COUNT;
use crate::views::process_selector::pane_state::ProcessSelectorPaneState;
use crate::views::project_explorer::pane_state::{ProjectExplorerFocusTarget, ProjectExplorerPaneState};
use crate::views::scan_results::pane_state::ScanResultsPaneState;
use crate::views::settings::pane_state::SettingsPaneState;
use crate::views::struct_viewer::pane_state::StructViewerPaneState;
use crate::views::struct_viewer::summary::STRUCT_VIEWER_FIXED_SUMMARY_LINE_COUNT;

/// Root state container for TUI panes.
#[derive(Clone, Debug, Default)]
pub struct TuiAppState {
    pub pane_layout_state: PaneLayoutState,
    pub process_selector_pane_state: ProcessSelectorPaneState,
    pub element_scanner_pane_state: ElementScannerPaneState,
    pub scan_results_pane_state: ScanResultsPaneState,
    pub project_explorer_pane_state: ProjectExplorerPaneState,
    pub struct_viewer_pane_state: StructViewerPaneState,
    pub output_pane_state: OutputPaneState,
    pub settings_pane_state: SettingsPaneState,
}

impl TuiAppState {
    pub fn active_workspace_page(&self) -> TuiWorkspacePage {
        self.pane_layout_state.active_workspace_page
    }

    pub fn set_active_workspace_page(
        &mut self,
        active_workspace_page: TuiWorkspacePage,
    ) {
        self.pane_layout_state
            .set_active_workspace_page(active_workspace_page);

        if !self.is_pane_visible(self.focused_pane()) {
            if let Some(first_visible_pane) = self.visible_panes_in_order().first().copied() {
                self.pane_layout_state.focused_pane = first_visible_pane;
            }
        }
    }

    pub fn focused_pane(&self) -> TuiPane {
        self.pane_layout_state.focused_pane
    }

    pub fn set_focused_pane(
        &mut self,
        focused_pane: TuiPane,
    ) {
        if self.is_pane_visible(focused_pane) {
            self.pane_layout_state.focused_pane = focused_pane;
        }
    }

    pub fn is_pane_visible(
        &self,
        pane: TuiPane,
    ) -> bool {
        self.visible_panes_in_order().contains(&pane)
    }

    pub fn visible_panes_in_order(&self) -> Vec<TuiPane> {
        match self.active_workspace_page() {
            TuiWorkspacePage::ProjectWorkspace => {
                if self.process_selector_pane_state.is_process_selector_view_active {
                    vec![TuiPane::ProcessSelector, TuiPane::Output]
                } else {
                    vec![TuiPane::ProjectExplorer, TuiPane::Output]
                }
            }
            _ => self.pane_layout_state.visible_panes_in_order(),
        }
    }

    pub fn cycle_focus_forward(&mut self) {
        self.cycle_focus(true);
    }

    pub fn cycle_focus_backward(&mut self) {
        self.cycle_focus(false);
    }

    pub fn pane_summary_lines(
        &self,
        pane: TuiPane,
        pane_content_height: usize,
    ) -> Vec<String> {
        match pane {
            TuiPane::ProcessSelector => self.process_selector_pane_state.summary_lines(),
            TuiPane::ElementScanner => self
                .element_scanner_pane_state
                .summary_lines_with_capacity(pane_content_height),
            TuiPane::ScanResults => self.scan_results_pane_state.summary_lines(),
            TuiPane::ProjectExplorer => self.project_explorer_pane_state.summary_lines(),
            TuiPane::StructViewer => self
                .struct_viewer_pane_state
                .summary_lines(pane_content_height.saturating_sub(STRUCT_VIEWER_FIXED_SUMMARY_LINE_COUNT)),
            TuiPane::Output => self
                .output_pane_state
                .summary_lines(pane_content_height.saturating_sub(OUTPUT_FIXED_SUMMARY_LINE_COUNT)),
            TuiPane::Settings => self
                .settings_pane_state
                .summary_lines_with_capacity(pane_content_height),
        }
    }

    pub fn pane_entry_rows(
        &self,
        pane: TuiPane,
        pane_entry_row_capacity: usize,
    ) -> Vec<PaneEntryRow> {
        match pane {
            TuiPane::ProcessSelector => self
                .process_selector_pane_state
                .visible_process_entry_rows(pane_entry_row_capacity),
            TuiPane::ScanResults => self
                .scan_results_pane_state
                .visible_scan_result_rows(pane_entry_row_capacity),
            TuiPane::ProjectExplorer => {
                let (project_entry_row_capacity, project_item_entry_row_capacity) = self.project_explorer_entry_row_capacities(pane_entry_row_capacity);
                let mut entry_rows = self
                    .project_explorer_pane_state
                    .visible_project_entry_rows(project_entry_row_capacity);
                entry_rows.extend(
                    self.project_explorer_pane_state
                        .visible_project_item_entry_rows(project_item_entry_row_capacity),
                );
                entry_rows
            }
            _ => Vec::new(),
        }
    }

    pub fn pane_row_telemetry_line(
        &self,
        pane: TuiPane,
        pane_entry_row_capacity: usize,
    ) -> Option<String> {
        match pane {
            TuiPane::ProcessSelector => Some(format!("[ROWS] visible={}.", pane_entry_row_capacity)),
            TuiPane::ProjectExplorer => {
                let (project_entry_row_capacity, project_item_entry_row_capacity) = self.project_explorer_entry_row_capacities(pane_entry_row_capacity);
                Some(format!(
                    "[ROWS] projects={} | hierarchy={}.",
                    project_entry_row_capacity, project_item_entry_row_capacity
                ))
            }
            _ => None,
        }
    }

    fn project_explorer_entry_row_capacities(
        &self,
        total_entry_row_capacity: usize,
    ) -> (usize, usize) {
        if total_entry_row_capacity == 0 {
            return (0, 0);
        }

        let project_entry_count = self.project_explorer_pane_state.project_entries.len();
        let project_item_entry_count = self
            .project_explorer_pane_state
            .project_item_visible_entries
            .len();
        if self.project_explorer_pane_state.focus_target == ProjectExplorerFocusTarget::ProjectHierarchy {
            return (0, total_entry_row_capacity);
        }

        if project_entry_count == 0 {
            return (0, total_entry_row_capacity);
        }

        if project_item_entry_count == 0 {
            return (total_entry_row_capacity, 0);
        }

        if total_entry_row_capacity == 1 {
            return (1, 0);
        }

        let mut project_entry_row_capacity = ((total_entry_row_capacity as f32) * 0.33).round() as usize;
        project_entry_row_capacity = project_entry_row_capacity.clamp(1, total_entry_row_capacity.saturating_sub(1));
        let project_item_entry_row_capacity = total_entry_row_capacity.saturating_sub(project_entry_row_capacity);

        (project_entry_row_capacity, project_item_entry_row_capacity)
    }

    fn cycle_focus(
        &mut self,
        is_forward_direction: bool,
    ) {
        let visible_panes = self.visible_panes_in_order();
        if visible_panes.is_empty() {
            return;
        }

        let focused_pane = self.focused_pane();
        let focused_visible_index = visible_panes
            .iter()
            .position(|visible_pane| *visible_pane == focused_pane)
            .unwrap_or(0);

        let next_visible_index = if is_forward_direction {
            (focused_visible_index + 1) % visible_panes.len()
        } else if focused_visible_index == 0 {
            visible_panes.len() - 1
        } else {
            focused_visible_index - 1
        };

        self.pane_layout_state.focused_pane = visible_panes[next_visible_index];
    }
}

#[cfg(test)]
mod tests {
    use super::TuiAppState;
    use crate::state::pane::TuiPane;
    use crate::state::workspace_page::TuiWorkspacePage;

    #[test]
    fn switching_workspace_page_rehomes_focus_if_current_pane_is_hidden() {
        let mut tui_app_state = TuiAppState::default();
        tui_app_state
            .process_selector_pane_state
            .is_process_selector_view_active = false;
        tui_app_state.set_focused_pane(TuiPane::ProjectExplorer);

        tui_app_state.set_active_workspace_page(TuiWorkspacePage::ScannerWorkspace);

        assert_eq!(tui_app_state.focused_pane(), TuiPane::ElementScanner);
    }

    #[test]
    fn switching_workspace_page_keeps_focus_for_shared_output_pane() {
        let mut tui_app_state = TuiAppState::default();
        tui_app_state.set_focused_pane(TuiPane::Output);

        tui_app_state.set_active_workspace_page(TuiWorkspacePage::ScannerWorkspace);
        assert_eq!(tui_app_state.focused_pane(), TuiPane::Output);

        tui_app_state.set_active_workspace_page(TuiWorkspacePage::SettingsWorkspace);
        assert_eq!(tui_app_state.focused_pane(), TuiPane::Output);
    }

    #[test]
    fn project_workspace_focus_cycle_loops_in_process_selector_view_order() {
        let mut tui_app_state = TuiAppState::default();

        assert_eq!(tui_app_state.focused_pane(), TuiPane::ProcessSelector);
        tui_app_state.cycle_focus_forward();
        assert_eq!(tui_app_state.focused_pane(), TuiPane::Output);
        tui_app_state.cycle_focus_forward();
        assert_eq!(tui_app_state.focused_pane(), TuiPane::ProcessSelector);

        tui_app_state.cycle_focus_backward();
        assert_eq!(tui_app_state.focused_pane(), TuiPane::Output);
    }

    #[test]
    fn project_workspace_focus_cycle_loops_in_project_explorer_view_order() {
        let mut tui_app_state = TuiAppState::default();
        tui_app_state
            .process_selector_pane_state
            .is_process_selector_view_active = false;
        tui_app_state.set_focused_pane(TuiPane::ProjectExplorer);

        assert_eq!(tui_app_state.focused_pane(), TuiPane::ProjectExplorer);
        tui_app_state.cycle_focus_forward();
        assert_eq!(tui_app_state.focused_pane(), TuiPane::Output);
        tui_app_state.cycle_focus_forward();
        assert_eq!(tui_app_state.focused_pane(), TuiPane::ProjectExplorer);

        tui_app_state.cycle_focus_backward();
        assert_eq!(tui_app_state.focused_pane(), TuiPane::Output);
    }

    #[test]
    fn scanner_workspace_focus_cycle_loops_in_page_order() {
        let mut tui_app_state = TuiAppState::default();
        tui_app_state.set_active_workspace_page(TuiWorkspacePage::ScannerWorkspace);

        assert_eq!(tui_app_state.focused_pane(), TuiPane::ElementScanner);
        tui_app_state.cycle_focus_forward();
        assert_eq!(tui_app_state.focused_pane(), TuiPane::ScanResults);
        tui_app_state.cycle_focus_forward();
        assert_eq!(tui_app_state.focused_pane(), TuiPane::Output);
        tui_app_state.cycle_focus_forward();
        assert_eq!(tui_app_state.focused_pane(), TuiPane::ElementScanner);

        tui_app_state.cycle_focus_backward();
        assert_eq!(tui_app_state.focused_pane(), TuiPane::Output);
    }

    #[test]
    fn settings_workspace_focus_cycle_loops_in_page_order() {
        let mut tui_app_state = TuiAppState::default();
        tui_app_state.set_active_workspace_page(TuiWorkspacePage::SettingsWorkspace);

        assert_eq!(tui_app_state.focused_pane(), TuiPane::Settings);
        tui_app_state.cycle_focus_forward();
        assert_eq!(tui_app_state.focused_pane(), TuiPane::Output);
        tui_app_state.cycle_focus_forward();
        assert_eq!(tui_app_state.focused_pane(), TuiPane::Settings);

        tui_app_state.cycle_focus_backward();
        assert_eq!(tui_app_state.focused_pane(), TuiPane::Output);
    }
}
