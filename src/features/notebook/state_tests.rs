use super::*;
use crate::parsers::markdown::Inline;

#[test]
fn test_extract_notebook_toc_empty() {
    let cells = vec![];
    let toc = extract_notebook_toc(&cells);
    assert!(toc.is_empty());
}

#[test]
fn test_extract_notebook_toc_with_markdown_headings() {
    let cells = vec![
        NotebookCell::Markdown {
            source: "# Introduction\nSome text".to_string(),
            blocks: vec![
                Block::Heading {
                    level: 1,
                    content: vec![Inline::Text("Introduction".to_string())],
                },
                Block::Paragraph(vec![Inline::Text("Some text".to_string())]),
            ],
        },
        NotebookCell::Code {
            source: "print('hello')".to_string(),
            execution_count: Some(1),
            outputs: vec![],
            language: Some("python".to_string()),
        },
        NotebookCell::Markdown {
            source: "## Section 1\nDetails\n### Subsection 1.1".to_string(),
            blocks: vec![
                Block::Heading {
                    level: 2,
                    content: vec![Inline::Text("Section 1".to_string())],
                },
                Block::Paragraph(vec![Inline::Text("Details".to_string())]),
                Block::Heading {
                    level: 3,
                    content: vec![Inline::Text("Subsection 1.1".to_string())],
                },
            ],
        },
    ];

    let toc = extract_notebook_toc(&cells);
    assert_eq!(toc.len(), 3);

    assert_eq!(toc[0].level, 1);
    assert_eq!(toc[0].text, "Introduction");
    assert_eq!(toc[0].block_index, 0);
    assert!(toc[0].y_offset >= CONTENT_PADDING);

    assert_eq!(toc[1].level, 2);
    assert_eq!(toc[1].text, "Section 1");
    assert_eq!(toc[1].block_index, 2);
    assert!(toc[1].y_offset > toc[0].y_offset);

    assert_eq!(toc[2].level, 3);
    assert_eq!(toc[2].text, "Subsection 1.1");
    assert_eq!(toc[2].block_index, 4);
    assert!(toc[2].y_offset > toc[1].y_offset);
}

#[test]
fn test_populate_state_sets_toc_for_notebook() {
    let mut state = KglanceState::default();
    let data = NotebookData {
        cells: vec![
            NotebookCell::Markdown {
                source: "# Overview".to_string(),
                blocks: vec![Block::Heading {
                    level: 1,
                    content: vec![Inline::Text("Overview".to_string())],
                }],
            },
            NotebookCell::Code {
                source: "x = 42".to_string(),
                execution_count: None,
                outputs: vec![],
                language: Some("python".to_string()),
            },
        ],
        kernel_language: "python".to_string(),
        total_cells: 2,
        fallback_notice: None,
    };

    populate_state(&mut state, &data);

    assert_eq!(state.markdown.toc.len(), 1);
    assert_eq!(state.markdown.toc[0].text, "Overview");
    assert_eq!(state.markdown.toc[0].level, 1);
    assert!(!state.markdown.toc_visible);
}
