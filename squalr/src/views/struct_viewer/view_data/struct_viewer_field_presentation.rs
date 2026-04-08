#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StructViewerFieldEditorKind {
    ValueBox,
    DataTypeSelector,
    ContainerTypeSelector,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructViewerFieldPresentation {
    display_name: String,
    editor_kind: StructViewerFieldEditorKind,
}

impl StructViewerFieldPresentation {
    pub fn new(
        display_name: String,
        editor_kind: StructViewerFieldEditorKind,
    ) -> Self {
        Self { display_name, editor_kind }
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn editor_kind(&self) -> &StructViewerFieldEditorKind {
        &self.editor_kind
    }
}
