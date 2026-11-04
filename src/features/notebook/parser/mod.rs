pub mod ansi;
#[cfg(test)]
mod tests;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use serde_json::Value;

use crate::features::common::parser::traits::ParseError;
use crate::features::notebook::types::{CellOutput, NotebookCell, NotebookData};
use crate::parsers::markdown::parse_to_blocks;

const MAX_OUTPUT_LINES: usize = 200;

pub fn parse_notebook(content: &str) -> Result<NotebookData, ParseError> {
    let json_val: Value = serde_json::from_str(content)
        .map_err(|e| ParseError::ParseFailed(format!("Invalid JSON in notebook: {e}")))?;

    let cells_array = json_val
        .get("cells")
        .and_then(Value::as_array)
        .ok_or_else(|| ParseError::ParseFailed("Missing 'cells' array in notebook".into()))?;

    let kernel_language = json_val
        .get("metadata")
        .and_then(|m| {
            m.get("language_info")
                .and_then(|l| l.get("name"))
                .and_then(Value::as_str)
                .or_else(|| {
                    m.get("kernelspec")
                        .and_then(|k| k.get("language"))
                        .and_then(Value::as_str)
                })
        })
        .unwrap_or("python")
        .to_string();

    let mut cells = Vec::with_capacity(cells_array.len());

    for cell in cells_array {
        let cell_type = cell
            .get("cell_type")
            .and_then(Value::as_str)
            .unwrap_or("raw");
        let source = extract_multiline_string(cell.get("source"));

        match cell_type {
            "markdown" => {
                let blocks = parse_to_blocks(&source);
                cells.push(NotebookCell::Markdown { source, blocks });
            }
            "code" => {
                let execution_count = cell.get("execution_count").and_then(Value::as_i64);
                let outputs = cell
                    .get("outputs")
                    .and_then(Value::as_array)
                    .map(|arr| parse_cell_outputs(arr))
                    .unwrap_or_default();

                cells.push(NotebookCell::Code {
                    source,
                    execution_count,
                    outputs,
                    language: kernel_language.clone(),
                });
            }
            _ => {
                cells.push(NotebookCell::Raw { source });
            }
        }
    }

    let total_cells = cells.len();

    Ok(NotebookData {
        cells,
        kernel_language,
        total_cells,
        fallback_notice: None,
    })
}

fn parse_cell_outputs(outputs_arr: &[Value]) -> Vec<CellOutput> {
    let mut outputs = Vec::new();

    for out in outputs_arr {
        let output_type = out
            .get("output_type")
            .and_then(Value::as_str)
            .unwrap_or_default();

        match output_type {
            "stream" => {
                let raw_text = extract_multiline_string(out.get("text"));
                let clean_text = ansi::strip_ansi_codes(&raw_text);
                let (text, truncated) = truncate_lines(&clean_text, MAX_OUTPUT_LINES);
                outputs.push(CellOutput::Text {
                    text,
                    is_error: false,
                    truncated,
                });
            }
            "error" => {
                let mut error_text = String::new();
                if let Some(tb) = out.get("traceback").and_then(Value::as_array) {
                    for line in tb {
                        if let Some(s) = line.as_str() {
                            error_text.push_str(s);
                            if !s.ends_with('\n') {
                                error_text.push('\n');
                            }
                        }
                    }
                } else {
                    let ename = out
                        .get("ename")
                        .and_then(Value::as_str)
                        .unwrap_or("UnknownError");
                    let evalue = out.get("evalue").and_then(Value::as_str).unwrap_or("");
                    error_text = format!("{ename}: {evalue}\n");
                }

                let clean_text = ansi::strip_ansi_codes(&error_text);
                let (text, truncated) = truncate_lines(&clean_text, MAX_OUTPUT_LINES);
                outputs.push(CellOutput::Text {
                    text,
                    is_error: true,
                    truncated,
                });
            }
            "execute_result" | "display_data" => {
                if let Some(data) = out.get("data").and_then(Value::as_object) {
                    if let Some(b64_val) = data.get("image/png")
                        && let Some(img_output) = decode_base64_image(b64_val, "png")
                    {
                        outputs.push(img_output);
                        continue;
                    }

                    if let Some(b64_val) = data.get("image/jpeg")
                        && let Some(img_output) = decode_base64_image(b64_val, "jpeg")
                    {
                        outputs.push(img_output);
                        continue;
                    }

                    if let Some(text_val) = data.get("text/plain") {
                        let raw_text = extract_multiline_string(Some(text_val));
                        let clean_text = ansi::strip_ansi_codes(&raw_text);
                        let (text, truncated) = truncate_lines(&clean_text, MAX_OUTPUT_LINES);
                        outputs.push(CellOutput::Text {
                            text,
                            is_error: false,
                            truncated,
                        });
                    }
                }
            }
            _ => {}
        }
    }

    outputs
}

fn decode_base64_image(val: &Value, format_hint: &str) -> Option<CellOutput> {
    let b64_str = match val {
        Value::String(s) => s.clone(),
        Value::Array(arr) => {
            let mut s = String::new();
            for item in arr {
                if let Some(sub) = item.as_str() {
                    s.push_str(sub);
                }
            }
            s
        }
        _ => return None,
    };

    let cleaned: String = b64_str.chars().filter(|c| !c.is_whitespace()).collect();
    let data = BASE64_STANDARD.decode(cleaned).ok()?;

    Some(CellOutput::Image {
        data,
        format_hint: format_hint.to_string(),
    })
}

fn truncate_lines(text: &str, max_lines: usize) -> (String, bool) {
    let mut lines = text.lines();
    let mut out = String::new();
    let mut truncated = false;

    for (count, line) in lines.by_ref().enumerate() {
        if count >= max_lines {
            truncated = true;
            break;
        }
        out.push_str(line);
        out.push('\n');
    }

    if truncated {
        (out, true)
    } else {
        (text.to_string(), false)
    }
}

fn extract_multiline_string(val: Option<&Value>) -> String {
    match val {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Array(arr)) => {
            let mut out = String::new();
            for item in arr {
                if let Some(s) = item.as_str() {
                    out.push_str(s);
                }
            }
            out
        }
        _ => String::new(),
    }
}
