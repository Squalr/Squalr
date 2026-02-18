/// Describes visual tone for a pane entry row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaneEntryRowTone {
    Selected,
    Normal,
    Disabled,
}

/// Describes a reusable row primitive for list-heavy panes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaneEntryRow {
    pub marker_text: String,
    pub primary_text: String,
    pub secondary_text: Option<String>,
    pub tone: PaneEntryRowTone,
}

impl PaneEntryRow {
    pub fn selected(
        marker_text: String,
        primary_text: String,
        secondary_text: Option<String>,
    ) -> Self {
        Self {
            marker_text,
            primary_text,
            secondary_text,
            tone: PaneEntryRowTone::Selected,
        }
    }

    pub fn normal(
        marker_text: String,
        primary_text: String,
        secondary_text: Option<String>,
    ) -> Self {
        Self {
            marker_text,
            primary_text,
            secondary_text,
            tone: PaneEntryRowTone::Normal,
        }
    }

    pub fn disabled(
        marker_text: String,
        primary_text: String,
        secondary_text: Option<String>,
    ) -> Self {
        Self {
            marker_text,
            primary_text,
            secondary_text,
            tone: PaneEntryRowTone::Disabled,
        }
    }
}
