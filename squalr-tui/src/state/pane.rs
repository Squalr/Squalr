/// Enumerates all top-level panes in keyboard focus order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TuiPane {
    ProcessSelector,
    ElementScanner,
    ScanResults,
    ProjectExplorer,
    StructViewer,
    Output,
    Settings,
}

impl TuiPane {
    pub fn title(self) -> &'static str {
        match self {
            TuiPane::ProcessSelector => "Process Selector",
            TuiPane::ElementScanner => "Element Scanner",
            TuiPane::ScanResults => "Scan Results",
            TuiPane::ProjectExplorer => "Project Explorer",
            TuiPane::StructViewer => "Struct Viewer",
            TuiPane::Output => "Output",
            TuiPane::Settings => "Settings",
        }
    }

    pub fn to_index(self) -> usize {
        match self {
            TuiPane::ProcessSelector => 0,
            TuiPane::ElementScanner => 1,
            TuiPane::ScanResults => 2,
            TuiPane::ProjectExplorer => 3,
            TuiPane::StructViewer => 4,
            TuiPane::Output => 5,
            TuiPane::Settings => 6,
        }
    }
}

impl Default for TuiPane {
    fn default() -> Self {
        TuiPane::ProcessSelector
    }
}
