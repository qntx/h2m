//! GFM table (`<table>`) conversion rules.
//!
//! **Known limitation:** column width is calculated using byte length, not
//! Unicode display width. Tables containing wide characters (CJK, emoji) may
//! have misaligned columns in the rendered Markdown source. Most Markdown
//! renderers still display these tables correctly.

use scraper::ElementRef;

use crate::context::Context;
use crate::converter::{Action, Rule};
use crate::dom;

/// Handles `<table>` elements, rendering them as GFM pipe tables.
#[derive(Debug, Clone, Copy)]
pub struct TableRule;

impl Rule for TableRule {
    fn tags(&self) -> &'static [&'static str] {
        &["table"]
    }

    fn apply(&self, _content: &str, element: &ElementRef<'_>, _ctx: &mut Context<'_>) -> Action {
        let rows = collect_rows(element);
        if rows.is_empty() {
            return Action::Skip;
        }

        let col_count = rows.iter().map(|r| r.cells.len()).max().unwrap_or(0);
        if col_count == 0 {
            return Action::Skip;
        }

        // Compute max width per column (minimum 3 for "---").
        let mut col_widths = vec![3usize; col_count];
        for row in &rows {
            for (j, cell) in row.cells.iter().enumerate() {
                col_widths[j] = col_widths[j].max(cell.text.len().max(3));
            }
        }

        let empty_header;
        let (header, body_rows) = if rows.first().is_some_and(|r| r.is_header) {
            (&rows[0], &rows[1..])
        } else {
            empty_header = TableRow {
                is_header: true,
                cells: vec![
                    TableCell {
                        text: String::new(),
                        alignment: Alignment::None,
                    };
                    col_count
                ],
            };
            (&empty_header, rows.as_slice())
        };

        let mut output = String::new();
        write_row(&mut output, header, &col_widths);
        output.push('\n');
        write_separator(&mut output, &header.cells, &col_widths);
        for row in body_rows {
            output.push('\n');
            write_row(&mut output, row, &col_widths);
        }

        Action::Replace(format!("\n\n{output}\n\n"))
    }
}

/// Handles `<thead>`, `<tbody>`, `<tfoot>` — transparent passthrough.
#[derive(Debug, Clone, Copy)]
pub struct TableSectionRule;

impl Rule for TableSectionRule {
    fn tags(&self) -> &'static [&'static str] {
        &["thead", "tbody", "tfoot"]
    }

    fn apply(&self, content: &str, _element: &ElementRef<'_>, _ctx: &mut Context<'_>) -> Action {
        Action::Replace(content.to_owned())
    }
}

/// Handles `<tr>` — transparent passthrough.
#[derive(Debug, Clone, Copy)]
pub struct TableRowRule;

impl Rule for TableRowRule {
    fn tags(&self) -> &'static [&'static str] {
        &["tr"]
    }

    fn apply(&self, content: &str, _element: &ElementRef<'_>, _ctx: &mut Context<'_>) -> Action {
        Action::Replace(content.to_owned())
    }
}

/// Handles `<td>` and `<th>` — transparent passthrough.
#[derive(Debug, Clone, Copy)]
pub struct TableCellRule;

impl Rule for TableCellRule {
    fn tags(&self) -> &'static [&'static str] {
        &["td", "th"]
    }

    fn apply(&self, content: &str, _element: &ElementRef<'_>, _ctx: &mut Context<'_>) -> Action {
        Action::Replace(content.to_owned())
    }
}

/// Column alignment parsed from `align` attribute or `text-align` style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Alignment {
    /// No explicit alignment.
    None,
    /// Left-aligned (`:---`).
    Left,
    /// Center-aligned (`:---:`).
    Center,
    /// Right-aligned (`---:`).
    Right,
}

/// A single table cell with its text content and alignment.
#[derive(Debug, Clone)]
struct TableCell {
    /// The plain-text content of the cell.
    text: String,
    /// The alignment of the cell's column.
    alignment: Alignment,
}

/// A row of table cells.
#[derive(Debug, Clone)]
struct TableRow {
    /// Whether this row belongs to a `<thead>` or consists entirely of `<th>` cells.
    is_header: bool,
    /// The cells in this row.
    cells: Vec<TableCell>,
}

/// Collects all rows from a `<table>` element.
fn collect_rows(table: &ElementRef<'_>) -> Vec<TableRow> {
    let mut rows = Vec::new();
    collect_rows_recursive(table, &mut rows);
    rows
}

/// Recursively collects rows from table sections (`<thead>`, `<tbody>`, `<tfoot>`).
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
                let text = dom::collect_text(&child);
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

/// Parses column alignment from an `align` attribute or `text-align` CSS style.
#[inline]
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

/// Writes a single row as a GFM pipe-table line.
fn write_row(out: &mut String, row: &TableRow, col_widths: &[usize]) {
    out.push('|');
    for (i, width) in col_widths.iter().enumerate() {
        let text = row.cells.get(i).map_or("", |c| c.text.as_str());
        out.push(' ');
        out.push_str(text);
        for _ in text.len()..*width {
            out.push(' ');
        }
        out.push_str(" |");
    }
}

/// Writes the separator line between header and body rows.
fn write_separator(out: &mut String, header_cells: &[TableCell], col_widths: &[usize]) {
    out.push('|');
    for (i, width) in col_widths.iter().enumerate() {
        let alignment = header_cells.get(i).map_or(Alignment::None, |c| c.alignment);
        out.push(' ');
        match alignment {
            Alignment::Left => {
                out.push(':');
                for _ in 0..width.saturating_sub(1) {
                    out.push('-');
                }
            }
            Alignment::Center => {
                out.push(':');
                for _ in 0..width.saturating_sub(2).max(1) {
                    out.push('-');
                }
                out.push(':');
            }
            Alignment::Right => {
                for _ in 0..width.saturating_sub(1) {
                    out.push('-');
                }
                out.push(':');
            }
            Alignment::None => {
                for _ in 0..*width {
                    out.push('-');
                }
            }
        }
        out.push_str(" |");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alignment_from_attr() {
        assert_eq!(parse_alignment(Some("left"), None), Alignment::Left);
        assert_eq!(parse_alignment(Some("center"), None), Alignment::Center);
        assert_eq!(parse_alignment(Some("right"), None), Alignment::Right);
    }

    #[test]
    fn alignment_attr_case_insensitive() {
        assert_eq!(parse_alignment(Some("LEFT"), None), Alignment::Left);
        assert_eq!(parse_alignment(Some("Center"), None), Alignment::Center);
    }

    #[test]
    fn alignment_from_style() {
        assert_eq!(
            parse_alignment(None, Some("text-align: center")),
            Alignment::Center
        );
        assert_eq!(
            parse_alignment(None, Some("text-align: right")),
            Alignment::Right
        );
        assert_eq!(
            parse_alignment(None, Some("text-align: left")),
            Alignment::Left
        );
    }

    #[test]
    fn alignment_attr_takes_precedence_over_style() {
        assert_eq!(
            parse_alignment(Some("left"), Some("text-align: right")),
            Alignment::Left
        );
    }

    #[test]
    fn alignment_none_fallback() {
        assert_eq!(parse_alignment(None, None), Alignment::None);
        assert_eq!(parse_alignment(Some("invalid"), None), Alignment::None);
        assert_eq!(parse_alignment(None, Some("color: red")), Alignment::None);
    }
}
