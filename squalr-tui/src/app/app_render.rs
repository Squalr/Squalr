use crate::app::AppShell;
use crate::state::pane::TuiPane;
use crate::state::pane_entry_row::PaneEntryRow;
use crate::state::workspace_page::TuiWorkspacePage;
use crate::theme::TuiTheme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

impl AppShell {
    pub(super) fn draw_pane_layout(
        &self,
        frame: &mut ratatui::Frame<'_>,
        body_area: Rect,
    ) {
        match self.app_state.active_workspace_page() {
            TuiWorkspacePage::ProjectWorkspace => self.draw_project_workspace_layout(frame, body_area),
            TuiWorkspacePage::ScannerWorkspace => self.draw_scanner_workspace_layout(frame, body_area),
            TuiWorkspacePage::SettingsWorkspace => self.draw_settings_workspace_layout(frame, body_area),
        }
    }

    fn draw_project_workspace_layout(
        &self,
        frame: &mut ratatui::Frame<'_>,
        body_area: Rect,
    ) {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(body_area);

        if self
            .app_state
            .process_selector_pane_state
            .is_process_selector_view_active
        {
            self.draw_single_pane(frame, rows[0], TuiPane::ProcessSelector);
        } else {
            self.draw_single_pane(frame, rows[0], TuiPane::ProjectExplorer);
        }
        self.draw_single_pane(frame, rows[1], TuiPane::Output);
    }

    fn draw_scanner_workspace_layout(
        &self,
        frame: &mut ratatui::Frame<'_>,
        body_area: Rect,
    ) {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(body_area);
        let workspace_columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(rows[0]);

        self.draw_single_pane(frame, workspace_columns[0], TuiPane::ElementScanner);
        self.draw_single_pane(frame, workspace_columns[1], TuiPane::ScanResults);
        self.draw_single_pane(frame, rows[1], TuiPane::Output);
    }

    fn draw_settings_workspace_layout(
        &self,
        frame: &mut ratatui::Frame<'_>,
        body_area: Rect,
    ) {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(body_area);

        self.draw_single_pane(frame, rows[0], TuiPane::Settings);
        self.draw_single_pane(frame, rows[1], TuiPane::Output);
    }

    fn draw_single_pane(
        &self,
        frame: &mut ratatui::Frame<'_>,
        pane_area: Rect,
        pane: TuiPane,
    ) {
        let is_focused = self.app_state.focused_pane() == pane;
        let mut title = pane.title().to_string();
        if is_focused {
            title.push_str(" *");
        }

        let pane_content_height = pane_area.height.saturating_sub(2) as usize;
        let pane_content_width = pane_area.width.saturating_sub(2) as usize;
        let fitted_summary_lines = self
            .fit_summary_lines_to_width(self.app_state.pane_summary_lines(pane, pane_content_height), pane_content_width)
            .into_iter()
            .collect::<Vec<String>>();
        let clamped_summary_lines = Self::clamp_summary_lines_for_entry_safeguard(pane, fitted_summary_lines, pane_content_height);
        let (summary_lines, entry_row_capacity) =
            Self::reconcile_row_telemetry_for_capacity(pane, clamped_summary_lines, pane_content_height, |pane_entry_row_capacity| {
                self.app_state
                    .pane_row_telemetry_line(pane, pane_entry_row_capacity)
            });
        let summary_lines = Self::fold_scan_results_visible_rows_into_page_line(pane, summary_lines, entry_row_capacity);
        let display_summary_lines = self.fit_summary_lines_to_width(summary_lines, pane_content_width);
        let pane_lines: Vec<Line<'static>> = display_summary_lines.into_iter().map(Line::from).collect();
        let entry_rows = self.app_state.pane_entry_rows(pane, entry_row_capacity);
        let pane_lines = self.append_entry_row_lines(pane_lines, entry_rows, pane_content_width);

        let pane_widget = Paragraph::new(pane_lines)
            .style(TuiTheme::panel_text_style())
            .block(TuiTheme::pane_block(&title, pane, is_focused));
        frame.render_widget(pane_widget, pane_area);
    }

    fn append_entry_row_lines(
        &self,
        mut pane_lines: Vec<Line<'static>>,
        entry_rows: Vec<PaneEntryRow>,
        content_width: usize,
    ) -> Vec<Line<'static>> {
        if entry_rows.is_empty() {
            return pane_lines;
        }

        if !pane_lines.is_empty() {
            pane_lines.push(Line::from(String::new()));
        }
        for entry_row in entry_rows {
            pane_lines.push(Self::render_entry_row(entry_row, content_width));
        }

        pane_lines
    }

    fn render_entry_row(
        entry_row: PaneEntryRow,
        content_width: usize,
    ) -> Line<'static> {
        let marker_style = TuiTheme::pane_entry_marker_style(entry_row.tone);
        let primary_style = TuiTheme::pane_entry_primary_style(entry_row.tone);
        let secondary_style = TuiTheme::pane_entry_secondary_style(entry_row.tone);
        let marker_text = Self::format_marker_prefix(entry_row.marker_text, content_width);
        let marker_prefix_width = marker_text.chars().count();
        let available_content_width = content_width.saturating_sub(marker_prefix_width);
        let (primary_text, secondary_text) = Self::fit_entry_row_content(entry_row.primary_text, entry_row.secondary_text, available_content_width);
        let mut entry_spans = vec![
            Span::styled(marker_text, marker_style),
            Span::styled(primary_text, primary_style),
        ];

        if let Some(secondary_text) = secondary_text {
            entry_spans.push(Span::raw("  "));
            entry_spans.push(Span::styled(secondary_text, secondary_style));
        }

        Line::from(entry_spans)
    }

    fn fit_summary_lines_to_width(
        &self,
        summary_lines: Vec<String>,
        content_width: usize,
    ) -> Vec<String> {
        summary_lines
            .into_iter()
            .map(|summary_line| Self::truncate_line_with_ellipsis(summary_line, content_width))
            .collect()
    }

    fn replace_row_telemetry_line(
        mut summary_lines: Vec<String>,
        row_telemetry_line: Option<String>,
    ) -> Vec<String> {
        let Some(row_telemetry_line) = row_telemetry_line else {
            return summary_lines;
        };
        let Some(row_summary_line_index) = summary_lines
            .iter()
            .position(|summary_line| summary_line.starts_with("[ROWS]"))
        else {
            return summary_lines;
        };
        summary_lines[row_summary_line_index] = row_telemetry_line;
        summary_lines
    }

    fn upsert_row_telemetry_line(
        pane: TuiPane,
        mut summary_lines: Vec<String>,
        row_telemetry_line: Option<String>,
        pane_content_height: usize,
    ) -> Vec<String> {
        let Some(row_telemetry_line) = row_telemetry_line else {
            return summary_lines;
        };
        let Some(row_summary_line_index) = summary_lines
            .iter()
            .position(|summary_line| summary_line.starts_with("[ROWS]"))
        else {
            if !Self::is_entry_heavy_pane(pane) || pane_content_height == 0 {
                return summary_lines;
            }
            if summary_lines.is_empty() {
                summary_lines.push(row_telemetry_line);
            } else {
                let last_summary_line_index = summary_lines.len() - 1;
                summary_lines[last_summary_line_index] = row_telemetry_line;
            }
            return summary_lines;
        };
        summary_lines[row_summary_line_index] = row_telemetry_line;
        summary_lines
    }

    fn strip_row_telemetry_line(summary_lines: Vec<String>) -> Vec<String> {
        summary_lines
            .into_iter()
            .filter(|summary_line| !summary_line.starts_with("[ROWS]"))
            .collect()
    }

    fn reconcile_row_telemetry_for_capacity<F>(
        pane: TuiPane,
        summary_lines: Vec<String>,
        pane_content_height: usize,
        row_telemetry_line_builder: F,
    ) -> (Vec<String>, usize)
    where
        F: Fn(usize) -> Option<String>,
    {
        let baseline_entry_row_capacity = Self::pane_entry_row_capacity(pane, pane_content_height, summary_lines.len());
        if baseline_entry_row_capacity == 0 {
            return (Self::strip_row_telemetry_line(summary_lines), 0);
        }

        let summary_lines_with_telemetry = Self::upsert_row_telemetry_line(
            pane,
            summary_lines.clone(),
            row_telemetry_line_builder(baseline_entry_row_capacity),
            pane_content_height,
        );
        let telemetry_entry_row_capacity = Self::pane_entry_row_capacity(pane, pane_content_height, summary_lines_with_telemetry.len());
        if telemetry_entry_row_capacity == 0 {
            return (Self::strip_row_telemetry_line(summary_lines), baseline_entry_row_capacity);
        }

        (
            Self::replace_row_telemetry_line(summary_lines_with_telemetry, row_telemetry_line_builder(telemetry_entry_row_capacity)),
            telemetry_entry_row_capacity,
        )
    }

    fn clamp_summary_lines_for_entry_safeguard(
        pane: TuiPane,
        mut summary_lines: Vec<String>,
        pane_content_height: usize,
    ) -> Vec<String> {
        let minimum_entry_row_count = Self::minimum_entry_row_count_for_pane(pane);
        if minimum_entry_row_count == 0 {
            return summary_lines;
        }

        let maximum_summary_line_count = pane_content_height.saturating_sub(minimum_entry_row_count.saturating_add(1));
        if summary_lines.len() > maximum_summary_line_count {
            summary_lines.truncate(maximum_summary_line_count);
        }

        summary_lines
    }

    fn pane_entry_row_capacity(
        pane: TuiPane,
        pane_content_height: usize,
        summary_line_count: usize,
    ) -> usize {
        let separator_line_count = usize::from(summary_line_count > 0);
        let computed_entry_row_capacity = pane_content_height.saturating_sub(summary_line_count.saturating_add(separator_line_count));
        let minimum_entry_row_count = Self::minimum_entry_row_count_for_pane(pane);
        if minimum_entry_row_count == 0 {
            return computed_entry_row_capacity;
        }

        if pane_content_height < minimum_entry_row_count {
            return computed_entry_row_capacity;
        }

        if computed_entry_row_capacity == 0 {
            return 0;
        }

        computed_entry_row_capacity.max(minimum_entry_row_count)
    }

    fn is_entry_heavy_pane(pane: TuiPane) -> bool {
        matches!(pane, TuiPane::ProcessSelector | TuiPane::ScanResults | TuiPane::ProjectExplorer)
    }

    fn minimum_entry_row_count_for_pane(pane: TuiPane) -> usize {
        usize::from(Self::is_entry_heavy_pane(pane))
    }

    fn format_marker_prefix(
        marker_text: String,
        content_width: usize,
    ) -> String {
        if content_width == 0 {
            return String::new();
        }

        if content_width == 1 {
            return Self::single_column_marker(marker_text);
        }

        if content_width == 2 {
            let truncated_marker = Self::truncate_line_with_ellipsis(marker_text, 2);
            return format!("{:>2}", truncated_marker);
        }

        let truncated_marker = Self::truncate_line_with_ellipsis(marker_text, 2);
        format!("{:>2} ", truncated_marker)
    }

    fn single_column_marker(marker_text: String) -> String {
        if marker_text.is_empty() {
            return String::new();
        }

        if let Some(first_visible_marker) = marker_text
            .chars()
            .find(|marker_character| !marker_character.is_whitespace())
        {
            return first_visible_marker.to_string();
        }

        " ".to_string()
    }

    fn fit_entry_row_content(
        primary_text: String,
        secondary_text: Option<String>,
        available_content_width: usize,
    ) -> (String, Option<String>) {
        if available_content_width == 0 {
            return (String::new(), None);
        }

        let truncated_primary = Self::truncate_line_with_ellipsis(primary_text.clone(), available_content_width);
        let Some(secondary_text) = secondary_text else {
            return (truncated_primary, None);
        };

        let secondary_separator_width = 2usize;
        let minimum_secondary_width = 4usize;
        if available_content_width <= secondary_separator_width + minimum_secondary_width {
            return (truncated_primary, None);
        }

        let primary_text_length = primary_text.chars().count();
        let secondary_text_length = secondary_text.chars().count();
        if primary_text_length + secondary_separator_width + secondary_text_length <= available_content_width {
            return (primary_text, Some(secondary_text));
        }

        if available_content_width <= 24 {
            return (truncated_primary, None);
        }

        let primary_width = available_content_width.saturating_sub(secondary_separator_width + minimum_secondary_width);
        if primary_width == 0 {
            return (truncated_primary, None);
        }

        let secondary_width = available_content_width.saturating_sub(primary_width + secondary_separator_width);
        if secondary_width == 0 {
            return (Self::truncate_line_with_ellipsis(primary_text, available_content_width), None);
        }

        let fitted_primary = Self::truncate_line_with_ellipsis(primary_text, primary_width);
        let fitted_secondary = Self::truncate_line_with_ellipsis(secondary_text, secondary_width);
        (fitted_primary, Some(fitted_secondary))
    }

    fn truncate_line_with_ellipsis(
        text: String,
        max_width: usize,
    ) -> String {
        if max_width == 0 {
            return String::new();
        }

        let text_character_count = text.chars().count();
        if text_character_count <= max_width {
            return text;
        }

        if max_width == 1 {
            return ".".to_string();
        }

        let kept_text: String = text.chars().take(max_width - 1).collect();
        format!("{}.", kept_text)
    }

    fn fold_scan_results_visible_rows_into_page_line(
        pane: TuiPane,
        mut summary_lines: Vec<String>,
        entry_row_capacity: usize,
    ) -> Vec<String> {
        if pane != TuiPane::ScanResults {
            return summary_lines;
        }

        let Some(page_summary_line_index) = summary_lines
            .iter()
            .position(|summary_line| summary_line.starts_with("[PAGE]"))
        else {
            return summary_lines;
        };
        let page_summary_line = summary_lines[page_summary_line_index]
            .trim_end_matches('.')
            .to_string();
        summary_lines[page_summary_line_index] = format!("{} | visible_rows={}.", page_summary_line, entry_row_capacity);

        summary_lines
    }
}
