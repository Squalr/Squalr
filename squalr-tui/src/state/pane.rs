/// Enumerates all top-level panes in keyboard focus order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TuiPane {
    ProcessSelector,
    ElementScanner,
    ScanResults,
    ProjectExplorer,
    StructViewer,
    MemoryViewer,
    MemoryInterpretation,
    CodeViewer,
    Output,
    Settings,
    Plugins,
}

impl TuiPane {
    pub fn title(self) -> &'static str {
        match self {
            TuiPane::ProcessSelector => "Process Selector",
            TuiPane::ElementScanner => "Element Scanner",
            TuiPane::ScanResults => "Scan Results",
            TuiPane::ProjectExplorer => "Project Explorer",
            TuiPane::StructViewer => "Struct Viewer",
            TuiPane::MemoryViewer => "Memory Viewer",
            TuiPane::MemoryInterpretation => "Interpretation",
            TuiPane::CodeViewer => "Code Viewer",
            TuiPane::Output => "Output",
            TuiPane::Settings => "Settings",
            TuiPane::Plugins => "Plugins",
        }
    }

    pub fn to_index(self) -> usize {
        match self {
            TuiPane::ProcessSelector => 0,
            TuiPane::ElementScanner => 1,
            TuiPane::ScanResults => 2,
            TuiPane::ProjectExplorer => 3,
            TuiPane::StructViewer => 4,
            TuiPane::MemoryViewer => 5,
            TuiPane::MemoryInterpretation => 6,
            TuiPane::CodeViewer => 7,
            TuiPane::Output => 8,
            TuiPane::Settings => 9,
            TuiPane::Plugins => 10,
        }
    }
}

impl Default for TuiPane {
    fn default() -> Self {
        TuiPane::ProcessSelector
    }
}
