//! GFM table (`<table>`) conversion rules.

use ego_tree::NodeRef;
use scraper::ElementRef;
use scraper::node::Node;

use crate::context::ConversionContext;
use crate::rule::{Rule, RuleAction};

/// Handles `<table>` elements, rendering them as GFM pipe tables.
#[derive(Debug, Clone, Copy)]
pub struct TableRule;

impl Rule for TableRule {
    fn tags(&self) -> &'static [&'static str] {
        &["table"]
    }

    fn apply(
        &self,
        _content: &str,
        element: &ElementRef<'_>,
        _ctx: &ConversionContext,
    ) -> RuleAction {
        let rows = collect_rows(element);
        if rows.is_empty() {
            return RuleAction::Skip;
        }

        // Determine column count from the widest row.
        let col_count = rows.iter().map(|r| r.cells.len()).max().unwrap_or(0);
        if col_count == 0 {
            return RuleAction::Skip;
        }

        // Compute max width per column.
        let mut col_widths = vec![3usize; col_count]; // minimum "---"
        for row in &rows {
            for (j, cell) in row.cells.iter().enumerate() {
                let w = cell.text.len().max(3);
                if w > col_widths[j] {
                    col_widths[j] = w;
                }
            }
        }

        let mut output = String::new();

        // Determine the header row.
        let (header, body_rows) = if rows.first().is_some_and(|r| r.is_header) {
            (&rows[0], &rows[1..])
        } else {
            // Synthesize an empty header row.
            let empty_header = TableRow {
                is_header: true,
                cells: (0..col_count)
                    .map(|_| TableCell {
                        text: String::new(),
                        alignment: Alignment::None,
                    })
                    .collect(),
            };
            // We can't return a reference to a local, so handle inline.
            output.push_str(&format_row(&empty_header, &col_widths));
            output.push('\n');
            output.push_str(&format_separator(&empty_header.cells, &col_widths));
            output.push('\n');
            for row in &rows {
                output.push_str(&format_row(row, &col_widths));
                output.push('\n');
            }
            return RuleAction::Replace(format!("\n\n{}\n", output.trim_end()));
        };

        output.push_str(&format_row(header, &col_widths));
        output.push('\n');
        output.push_str(&format_separator(&header.cells, &col_widths));
        for row in body_rows {
            output.push('\n');
            output.push_str(&format_row(row, &col_widths));
        }

        RuleAction::Replace(format!("\n\n{output}\n\n"))
    }
}

/// Handles `<thead>`, `<tbody>`, `<tfoot>` — transparent passthrough.
#[derive(Debug, Clone, Copy)]
pub struct TableSectionRule;

impl Rule for TableSectionRule {
    fn tags(&self) -> &'static [&'static str] {
        &["thead", "tbody", "tfoot"]
    }

    fn apply(
        &self,
        content: &str,
        _element: &ElementRef<'_>,
        _ctx: &ConversionContext,
    ) -> RuleAction {
        RuleAction::Replace(content.to_owned())
    }
}

/// Handles `<tr>` — transparent passthrough (table assembly happens in
/// [`TableRule`]).
#[derive(Debug, Clone, Copy)]
pub struct TableRowRule;

impl Rule for TableRowRule {
    fn tags(&self) -> &'static [&'static str] {
        &["tr"]
    }

    fn apply(
        &self,
        content: &str,
        _element: &ElementRef<'_>,
        _ctx: &ConversionContext,
    ) -> RuleAction {
        RuleAction::Replace(content.to_owned())
    }
}

/// Handles `<td>` and `<th>` — transparent passthrough.
#[derive(Debug, Clone, Copy)]
pub struct TableCellRule;

impl Rule for TableCellRule {
    fn tags(&self) -> &'static [&'static str] {
        &["td", "th"]
    }

    fn apply(
        &self,
        content: &str,
        _element: &ElementRef<'_>,
        _ctx: &ConversionContext,
    ) -> RuleAction {
        RuleAction::Replace(content.to_owned())
    }
}

// ── Internal table model ─────────────────────────────────────────────────

/// Cell alignment in a table column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Alignment {
    /// No explicit alignment.
    None,
    /// Left-aligned.
    Left,
    /// Center-aligned.
    Center,
    /// Right-aligned.
    Right,
}

/// A single cell in a table row.
#[derive(Debug, Clone)]
struct TableCell {
    /// The text content of the cell.
    text: String,
    /// The alignment derived from the element's `align` or `style` attribute.
    alignment: Alignment,
}

/// A table row with its cells.
#[derive(Debug, Clone)]
struct TableRow {
    /// Whether this row is part of the header (`<thead>` or all `<th>`).
    is_header: bool,
    /// The cells in this row.
    cells: Vec<TableCell>,
}

/// Collects all rows and cells from a `<table>` element by walking the DOM.
fn collect_rows(table: &ElementRef<'_>) -> Vec<TableRow> {
    let mut rows = Vec::new();
    collect_rows_recursive(table, &mut rows);
    rows
}

/// Recursively walks table children to find `<tr>` elements.
fn collect_rows_recursive(parent: &ElementRef<'_>, rows: &mut Vec<TableRow>) {
    for child in parent.children() {
        if let Some(el) = child.value().as_element() {
            let tag = el.name();
            if tag == "tr" {
                if let Some(tr) = ElementRef::wrap(child) {
                    rows.push(collect_cells(&tr));
                }
            } else if matches!(tag, "thead" | "tbody" | "tfoot")
                && let Some(section) = ElementRef::wrap(child)
            {
                collect_rows_recursive(&section, rows);
            }
        }
    }
}

/// Collects cells from a `<tr>` element.
fn collect_cells(tr: &ElementRef<'_>) -> TableRow {
    let mut cells = Vec::new();
    let mut all_th = true;
    let in_thead = tr
        .parent()
        .and_then(|p| p.value().as_element())
        .is_some_and(|e| e.name() == "thead");

    for child in tr.children() {
        if let Some(el) = child.value().as_element() {
            let tag = el.name();
            if tag == "td" || tag == "th" {
                if tag == "td" {
                    all_th = false;
                }
                let alignment = parse_alignment(el.attr("align"), el.attr("style"));
                let text = collect_text_content(&child);
                cells.push(TableCell {
                    text: text.trim().replace('\n', " "),
                    alignment,
                });
            }
        }
    }

    TableRow {
        is_header: in_thead || all_th,
        cells,
    }
}

/// Recursively collects text content from a node and its descendants.
fn collect_text_content(node: &NodeRef<'_, Node>) -> String {
    let mut text = String::new();
    match node.value() {
        Node::Text(t) => text.push_str(t),
        _ => {
            for child in node.children() {
                text.push_str(&collect_text_content(&child));
            }
        }
    }
    text
}

/// Parses alignment from `align` attribute or inline `style`.
fn parse_alignment(align: Option<&str>, style: Option<&str>) -> Alignment {
    if let Some(a) = align {
        return match a.to_ascii_lowercase().as_str() {
            "left" => Alignment::Left,
            "center" => Alignment::Center,
            "right" => Alignment::Right,
            _ => Alignment::None,
        };
    }
    if let Some(s) = style
        && s.contains("text-align")
    {
        if s.contains("center") {
            return Alignment::Center;
        }
        if s.contains("right") {
            return Alignment::Right;
        }
        if s.contains("left") {
            return Alignment::Left;
        }
    }
    Alignment::None
}

/// Formats a single table row as a pipe-delimited string.
fn format_row(row: &TableRow, col_widths: &[usize]) -> String {
    let mut out = String::from("|");
    for (i, width) in col_widths.iter().enumerate() {
        let text = row.cells.get(i).map_or("", |c| c.text.as_str());
        out.push(' ');
        out.push_str(text);
        // Pad to column width.
        for _ in text.len()..*width {
            out.push(' ');
        }
        out.push_str(" |");
    }
    out
}

/// Formats the separator row (`|---|---|`).
fn format_separator(header_cells: &[TableCell], col_widths: &[usize]) -> String {
    let mut out = String::from("|");
    for (i, width) in col_widths.iter().enumerate() {
        let alignment = header_cells.get(i).map_or(Alignment::None, |c| c.alignment);
        let sep = match alignment {
            Alignment::Left => format!(":{}", "-".repeat(width.saturating_sub(1))),
            Alignment::Center => {
                format!(":{}:", "-".repeat(width.saturating_sub(2).max(1)))
            }
            Alignment::Right => format!("{}:", "-".repeat(width.saturating_sub(1))),
            Alignment::None => "-".repeat(*width),
        };
        out.push(' ');
        out.push_str(&sep);
        out.push_str(" |");
    }
    out
}
