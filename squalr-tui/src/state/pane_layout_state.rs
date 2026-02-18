use crate::state::pane::TuiPane;
use crate::state::workspace_page::TuiWorkspacePage;

/// Stores focus and visibility for top-level panes.
#[derive(Clone, Debug)]
pub struct PaneLayoutState {
    pub active_workspace_page: TuiWorkspacePage,
    pub focused_pane: TuiPane,
    pub pane_visibility: [bool; 7],
}

impl PaneLayoutState {
    pub fn set_active_workspace_page(
        &mut self,
        active_workspace_page: TuiWorkspacePage,
    ) {
        self.active_workspace_page = active_workspace_page;
        self.pane_visibility = Self::pane_visibility_for_workspace_page(active_workspace_page);

        if !self.is_pane_visible(self.focused_pane) {
            if let Some(first_visible_pane) = active_workspace_page.visible_panes().first().copied() {
                self.focused_pane = first_visible_pane;
            }
        }
    }

    pub fn is_pane_visible(
        &self,
        pane: TuiPane,
    ) -> bool {
        self.pane_visibility[pane.to_index()]
    }

    pub fn visible_panes_in_order(&self) -> Vec<TuiPane> {
        self.active_workspace_page.visible_panes().to_vec()
    }
}

impl Default for PaneLayoutState {
    fn default() -> Self {
        let active_workspace_page = TuiWorkspacePage::default();
        Self {
            active_workspace_page,
            focused_pane: TuiPane::ProcessSelector,
            pane_visibility: Self::pane_visibility_for_workspace_page(active_workspace_page),
        }
    }
}

impl PaneLayoutState {
    fn pane_visibility_for_workspace_page(active_workspace_page: TuiWorkspacePage) -> [bool; 7] {
        let mut pane_visibility = [false; 7];
        for pane in active_workspace_page.visible_panes() {
            pane_visibility[pane.to_index()] = true;
        }

        pane_visibility
    }
}
