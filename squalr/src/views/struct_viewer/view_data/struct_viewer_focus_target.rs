use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StructViewerFocusTarget {
    ProjectHierarchy { project_item_paths: Vec<PathBuf> },
    SymbolExplorer { selection_key: String },
    SymbolTable { symbol_locator_key: String },
}
