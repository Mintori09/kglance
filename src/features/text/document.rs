use std::sync::Arc;

/// Represents immutable source code text with byte offset indexing for virtualization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeDocument {
    /// Full file content (safely shared between threads/UI, zero-clone).
    raw_text: Arc<str>,
    /// Starting byte offset of each line (line_offsets[0] = 0).
    line_offsets: Vec<usize>,
    /// Total logical lines.
    total_lines: usize,
    /// Max digits for calculating gutter width (e.g., 1000 lines => 4).
    max_digits: usize,
}

impl CodeDocument {
    /// Initializes CodeDocument from text content with SIMD newline indexing.
    pub fn new(content: impl Into<Arc<str>>) -> Self {
        let raw_text: Arc<str> = content.into();
        let bytes = raw_text.as_bytes();

        // Estimate offset vector capacity (average 40 bytes/line)
        let mut line_offsets = Vec::with_capacity(bytes.len() / 40 + 2);
        line_offsets.push(0);

        // High-speed '\n' scan (> 2.5 GB/s) using memchr
        for pos in memchr::memchr_iter(b'\n', bytes) {
            let next_offset = pos + 1;
            if next_offset < bytes.len() {
                line_offsets.push(next_offset);
            }
        }

        let total_lines = if bytes.is_empty() {
            1
        } else {
            line_offsets.len().max(1)
        };

        let max_digits = if total_lines > 0 {
            total_lines.to_string().len()
        } else {
            1
        };

        Self {
            raw_text,
            line_offsets,
            total_lines,
            max_digits,
        }
    }

    /// Returns the original text wrapped in an Arc.
    #[inline]
    pub fn raw(&self) -> &Arc<str> {
        &self.raw_text
    }

    /// Returns a string slice &str of the entire document.
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.raw_text
    }

    /// Total number of lines in the document.
    #[inline]
    pub fn total_lines(&self) -> usize {
        self.total_lines
    }

    /// Maximum number of digits in line count (for gutter).
    #[inline]
    pub fn max_digits(&self) -> usize {
        self.max_digits
    }

    /// Returns the byte offset slice for all lines.
    #[inline]
    pub fn line_offsets(&self) -> &[usize] {
        &self.line_offsets
    }

    /// Returns the content of line `line_idx` (0-indexed), stripped of trailing `\r` and `\n`.
    pub fn get_line(&self, line_idx: usize) -> &str {
        if line_idx >= self.total_lines || self.raw_text.is_empty() {
            return "";
        }

        let start = self.line_offsets.get(line_idx).copied().unwrap_or(0);
        let end = if line_idx + 1 < self.line_offsets.len() {
            self.line_offsets[line_idx + 1]
        } else {
            self.raw_text.len()
        };

        if start >= self.raw_text.len() {
            return "";
        }

        let raw_slice = &self.raw_text[start..end.min(self.raw_text.len())];
        raw_slice.trim_end_matches(['\r', '\n'])
    }

    /// Returns the raw line content including newline characters.
    pub fn get_line_raw(&self, line_idx: usize) -> &str {
        if line_idx >= self.total_lines || self.raw_text.is_empty() {
            return "";
        }

        let start = self.line_offsets.get(line_idx).copied().unwrap_or(0);
        let end = if line_idx + 1 < self.line_offsets.len() {
            self.line_offsets[line_idx + 1]
        } else {
            self.raw_text.len()
        };

        if start >= self.raw_text.len() {
            return "";
        }

        &self.raw_text[start..end.min(self.raw_text.len())]
    }

    /// Extracts the text range between (start_line, start_col) and (end_line, end_col) for copying.
    pub fn extract_range(
        &self,
        mut start_line: usize,
        mut start_col: usize,
        mut end_line: usize,
        mut end_col: usize,
    ) -> String {
        if self.raw_text.is_empty() || self.total_lines == 0 {
            return String::new();
        }

        // Normalize order (start <= end)
        if (start_line, start_col) > (end_line, end_col) {
            std::mem::swap(&mut start_line, &mut end_line);
            std::mem::swap(&mut start_col, &mut end_col);
        }

        start_line = start_line.min(self.total_lines.saturating_sub(1));
        end_line = end_line.min(self.total_lines.saturating_sub(1));

        if start_line == end_line {
            let line = self.get_line(start_line);
            let char_indices: Vec<(usize, char)> = line.char_indices().collect();
            let start_byte = char_indices
                .get(start_col)
                .map(|(i, _)| *i)
                .unwrap_or(line.len());
            let end_byte = char_indices
                .get(end_col)
                .map(|(i, _)| *i)
                .unwrap_or(line.len());
            return line[start_byte..end_byte.max(start_byte)].to_string();
        }

        let mut result = String::new();
        for line_idx in start_line..=end_line {
            let line = self.get_line(line_idx);
            let char_indices: Vec<(usize, char)> = line.char_indices().collect();

            if line_idx == start_line {
                let start_byte = char_indices
                    .get(start_col)
                    .map(|(i, _)| *i)
                    .unwrap_or(line.len());
                result.push_str(&line[start_byte..]);
                result.push('\n');
            } else if line_idx == end_line {
                let end_byte = char_indices
                    .get(end_col)
                    .map(|(i, _)| *i)
                    .unwrap_or(line.len());
                result.push_str(&line[..end_byte]);
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }
        result
    }
}

impl Default for CodeDocument {
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_document_basic() {
        let code = "fn main() {\n    println!(\"hello\");\n}\n";
        let doc = CodeDocument::new(code);

        assert_eq!(doc.total_lines(), 3);
        assert_eq!(doc.get_line(0), "fn main() {");
        assert_eq!(doc.get_line(1), "    println!(\"hello\");");
        assert_eq!(doc.get_line(2), "}");
        assert_eq!(doc.max_digits(), 1);
    }

    #[test]
    fn test_code_document_crlf() {
        let code = "line 1\r\nline 2\r\nline 3";
        let doc = CodeDocument::new(code);

        assert_eq!(doc.total_lines(), 3);
        assert_eq!(doc.get_line(0), "line 1");
        assert_eq!(doc.get_line(1), "line 2");
        assert_eq!(doc.get_line(2), "line 3");
    }

    #[test]
    fn test_code_document_extract_range() {
        let code = "hello world\nrust language\niced gui";
        let doc = CodeDocument::new(code);

        // Same line: "world"
        let part1 = doc.extract_range(0, 6, 0, 11);
        assert_eq!(part1, "world");

        // Multi-line: "world\nrust"
        let part2 = doc.extract_range(0, 6, 1, 4);
        assert_eq!(part2, "world\nrust");
    }

    #[test]
    fn test_empty_document() {
        let doc = CodeDocument::new("");
        assert_eq!(doc.total_lines(), 1);
        assert_eq!(doc.get_line(0), "");
    }
}
