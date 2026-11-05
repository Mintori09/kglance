use super::resolve_column_plan;
use crate::parsers::markdown::{TableBlock, TableCell};

fn make_test_table(rows_data: &[(&[&str], &[&str])]) -> TableBlock {
    let headers = if let Some((h, _)) = rows_data.first() {
        h.iter()
            .map(|text| TableCell {
                content: vec![crate::parsers::markdown::Inline::Text(text.to_string())],
            })
            .collect()
    } else {
        Vec::new()
    };

    let rows = rows_data
        .iter()
        .map(|(_, r)| {
            r.iter()
                .map(|text| TableCell {
                    content: vec![crate::parsers::markdown::Inline::Text(text.to_string())],
                })
                .collect::<Vec<TableCell>>()
        })
        .collect();

    TableBlock {
        headers,
        rows,
        alignments: Vec::new(),
        column_weights: Vec::new(),
    }
}

#[test]
fn test_two_columns_one_long_text_fits_content_width() {
    let long_text = ".rs, .py, .js, .ts, .jsx, .tsx, .html, .css, .scss, .json, .toml, .yml/.yaml, .xml, .sh, .bash, .zsh, .fish, .c, .h, .cpp, .hpp, .java, .kt, .swift, .go, .rb, .php, .pl, .pm, .lua, .r, .sql, .graphql, .proto, .tex, .bib, .dockerfile, .makefile, .cmake, .gradle, .cfg, .ini, .conf, .txt, .log, .diff, .patch, .vim, .ps1, .bat";
    let table = make_test_table(&[(&["Category", "Formats"], &["Code", long_text])]);

    let plan = resolve_column_plan(&table, 14.0, 800.0);
    assert!(!plan.overflows);
    assert_eq!(plan.col_widths.len(), 2);
    assert!((plan.total_width - 800.0).abs() < 1.0);
    // The short column (Category) should be bounded and not excessively wide
    assert!(plan.col_widths[0] < 200.0);
    // The long column should get the bulk of the remaining width (> 600px)
    assert!(plan.col_widths[1] > 600.0);
}

#[test]
fn test_many_columns_overflows_when_narrow() {
    let headers = [
        "Col 1", "Col 2", "Col 3", "Col 4", "Col 5", "Col 6", "Col 7", "Col 8",
    ];
    let row = [
        "Val 1", "Val 2", "Val 3", "Val 4", "Val 5", "Val 6", "Val 7", "Val 8",
    ];
    let table = make_test_table(&[(&headers, &row)]);

    let plan = resolve_column_plan(&table, 14.0, 500.0);
    assert!(plan.overflows);
    assert!(plan.total_width > 500.0);
}
