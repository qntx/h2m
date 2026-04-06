//! GFM extension tests (strikethrough, tables, task lists).

use h2m::convert_gfm;
use pretty_assertions::assert_eq;

#[test]
fn gfm_del_tag() {
    assert_eq!(convert_gfm("<p><del>removed</del></p>"), "~~removed~~");
}

#[test]
fn gfm_s_tag() {
    assert_eq!(convert_gfm("<p><s>removed</s></p>"), "~~removed~~");
}

#[test]
fn gfm_strike_tag() {
    assert_eq!(
        convert_gfm("<p><strike>removed</strike></p>"),
        "~~removed~~"
    );
}

#[test]
fn gfm_table_basic_structure() {
    let html = "<table><thead><tr><th>Name</th><th>Age</th></tr></thead>\
                <tbody><tr><td>Alice</td><td>30</td></tr></tbody></table>";
    let md = convert_gfm(html);
    let lines: Vec<&str> = md.lines().collect();
    assert!(lines.len() >= 3, "table should have at least 3 lines");
    assert!(lines[0].contains("Name"));
    assert!(lines[0].contains("Age"));
    assert!(lines[1].contains("---"));
    assert!(lines[2].contains("Alice"));
    assert!(lines[2].contains("30"));
}

#[test]
fn gfm_table_alignment() {
    let html = r#"<table><thead><tr>
        <th align="left">L</th><th align="center">C</th><th align="right">R</th>
    </tr></thead><tbody><tr><td>a</td><td>b</td><td>c</td></tr></tbody></table>"#;
    let md = convert_gfm(html);
    let lines: Vec<&str> = md.lines().collect();
    assert!(lines.len() >= 2, "table should have a separator row");
    let sep = lines[1];
    assert!(sep.contains(":--"), "left alignment marker");
    assert!(
        sep.contains(":-") && sep.contains("-:"),
        "center alignment marker"
    );
    assert!(sep.contains("--:"), "right alignment marker");
}

#[test]
fn gfm_task_list_checked_and_unchecked() {
    let html = r#"<ul>
        <li><input type="checkbox" checked/> done</li>
        <li><input type="checkbox"/> todo</li>
    </ul>"#;
    let md = convert_gfm(html);
    let lines: Vec<&str> = md.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("[x]") && lines[0].contains("done"));
    assert!(lines[1].contains("[ ]") && lines[1].contains("todo"));
}

#[test]
fn convert_gfm_includes_all_extensions() {
    let md = convert_gfm("<p><del>strike</del></p>");
    assert_eq!(md, "~~strike~~");
}
