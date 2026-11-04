use super::*;
use crate::features::notebook::types::{CellOutput, NotebookCell};

#[test]
fn test_parse_valid_notebook_minimal() {
    let json = serde_json::json!({
        "cells": [],
        "metadata": {
            "language_info": {
                "name": "python"
            }
        },
        "nbformat": 4,
        "nbformat_minor": 2
    })
    .to_string();

    let res = parse_notebook(&json);
    assert!(res.is_ok());
    let data = res.expect("valid minimal notebook");
    assert_eq!(data.total_cells, 0);
    assert_eq!(data.kernel_language, "python");
}

#[test]
fn test_parse_markdown_and_code_cells() {
    let json = serde_json::json!({
        "cells": [
            {
                "cell_type": "markdown",
                "metadata": {},
                "source": ["# Title\n", "This is **bold** text."]
            },
            {
                "cell_type": "code",
                "execution_count": 1,
                "metadata": {},
                "source": "x = 42\nprint(x)",
                "outputs": [
                    {
                        "output_type": "stream",
                        "name": "stdout",
                        "text": ["42\n"]
                    }
                ]
            }
        ],
        "metadata": {
            "kernelspec": {
                "language": "python"
            }
        },
        "nbformat": 4,
        "nbformat_minor": 5
    })
    .to_string();

    let data = parse_notebook(&json).expect("parse valid cells");
    assert_eq!(data.total_cells, 2);

    match &data.cells[0] {
        NotebookCell::Markdown { source, blocks } => {
            assert!(source.contains("# Title"));
            assert!(!blocks.is_empty());
        }
        _ => panic!("Expected Markdown cell"),
    }

    match &data.cells[1] {
        NotebookCell::Code {
            source,
            execution_count,
            outputs,
            language,
        } => {
            assert_eq!(source, "x = 42\nprint(x)");
            assert_eq!(*execution_count, Some(1));
            assert_eq!(language, "python");
            assert_eq!(outputs.len(), 1);
            match &outputs[0] {
                CellOutput::Text {
                    text,
                    is_error,
                    truncated,
                } => {
                    assert_eq!(text, "42\n");
                    assert!(!is_error);
                    assert!(!truncated);
                }
                _ => panic!("Expected Text output"),
            }
        }
        _ => panic!("Expected Code cell"),
    }
}

#[test]
fn test_parse_error_output_with_ansi() {
    let json = serde_json::json!({
        "cells": [
            {
                "cell_type": "code",
                "execution_count": 2,
                "metadata": {},
                "source": "1 / 0",
                "outputs": [
                    {
                        "output_type": "error",
                        "ename": "ZeroDivisionError",
                        "evalue": "division by zero",
                        "traceback": [
                            "\u{001b}[0;31mTraceback (most recent call last):\u{001b}[0m\n",
                            "\u{001b}[1;32mZeroDivisionError: division by zero\u{001b}[0m\n"
                        ]
                    }
                ]
            }
        ],
        "metadata": {}
    })
    .to_string();

    let data = parse_notebook(&json).expect("parse error cell");
    match &data.cells[0] {
        NotebookCell::Code { outputs, .. } => {
            assert_eq!(outputs.len(), 1);
            match &outputs[0] {
                CellOutput::Text { text, is_error, .. } => {
                    assert!(is_error);
                    assert!(!text.contains('\u{1b}'));
                    assert!(text.contains("ZeroDivisionError"));
                }
                _ => panic!("Expected error text output"),
            }
        }
        _ => panic!("Expected Code cell"),
    }
}

#[test]
fn test_parse_image_output_base64() {
    // 1x1 transparent PNG in base64: iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII=
    let b64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII=";
    let json = serde_json::json!({
        "cells": [
            {
                "cell_type": "code",
                "execution_count": 3,
                "metadata": {},
                "source": "plot()",
                "outputs": [
                    {
                        "output_type": "display_data",
                        "data": {
                            "image/png": b64
                        }
                    }
                ]
            }
        ],
        "metadata": {}
    })
    .to_string();

    let data = parse_notebook(&json).expect("parse image output");
    match &data.cells[0] {
        NotebookCell::Code { outputs, .. } => {
            assert_eq!(outputs.len(), 1);
            match &outputs[0] {
                CellOutput::Image { data, format_hint } => {
                    assert_eq!(format_hint, "png");
                    assert!(!data.is_empty());
                }
                _ => panic!("Expected Image output"),
            }
        }
        _ => panic!("Expected Code cell"),
    }
}

#[test]
fn test_output_truncation_over_max_lines() {
    let mut long_text = String::new();
    for i in 1..=250 {
        long_text.push_str(&format!("Line {i}\n"));
    }

    let json = serde_json::json!({
        "cells": [
            {
                "cell_type": "code",
                "source": "for i in range(250): print(i)",
                "outputs": [
                    {
                        "output_type": "stream",
                        "text": long_text
                    }
                ]
            }
        ],
        "metadata": {}
    })
    .to_string();

    let data = parse_notebook(&json).expect("parse long output");
    match &data.cells[0] {
        NotebookCell::Code { outputs, .. } => match &outputs[0] {
            CellOutput::Text {
                text, truncated, ..
            } => {
                assert!(truncated);
                assert_eq!(text.lines().count(), 200);
            }
            _ => panic!("Expected Text output"),
        },
        _ => panic!("Expected Code cell"),
    }
}

#[test]
fn test_missing_cells_error() {
    let invalid_json = r#"{"metadata": {}}"#;
    assert!(parse_notebook(invalid_json).is_err());
}

#[test]
fn test_parse_sample_fixture_file() {
    let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("testing-file")
        .join("sample.ipynb");
    let content = std::fs::read_to_string(&fixture_path).expect("read sample fixture");
    let data = parse_notebook(&content).expect("parse sample fixture notebook");
    assert_eq!(data.total_cells, 4);
    assert_eq!(data.kernel_language, "python");
}

#[test]
fn test_parser_registry_e2e_ipynb() {
    use crate::core::FilePreviewer;
    use crate::engine::registry::build_registry;

    let registry = build_registry();
    let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("testing-file")
        .join("sample.ipynb");
    let preview = FilePreviewer::parse(&registry, &fixture_path).expect("parse via registry");

    match preview {
        crate::core::PreviewData::Notebook(data) => {
            assert_eq!(data.total_cells, 4);
            assert_eq!(data.kernel_language, "python");
        }
        _ => panic!("Expected PreviewData::Notebook"),
    }
}
