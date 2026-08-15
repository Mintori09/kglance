#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Struct,
    Class,
    Enum,
    Trait,
    Module,
    Type,
    Const,
}

impl SymbolKind {
    pub fn badge_label(&self) -> &'static str {
        match self {
            Self::Function => "fn",
            Self::Struct => "struct",
            Self::Class => "class",
            Self::Enum => "enum",
            Self::Trait => "trait",
            Self::Module => "mod",
            Self::Type => "type",
            Self::Const => "const",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub line_number: usize, // 1-indexed
    pub indent_level: usize,
}

pub fn extract_symbols(content: &str, extension: &str) -> Vec<CodeSymbol> {
    let ext = extension.trim_start_matches('.').to_lowercase();
    match ext.as_str() {
        "rs" => extract_rust_symbols(content),
        "py" | "pyw" => extract_python_symbols(content),
        "js" | "mjs" | "cjs" | "ts" | "mts" | "cts" | "jsx" | "tsx" => {
            extract_ts_js_symbols(content)
        }
        "go" => extract_go_symbols(content),
        "c" | "h" | "cpp" | "hpp" | "cc" | "cxx" => extract_cpp_symbols(content),
        _ => extract_generic_symbols(content),
    }
}

fn calculate_indent(line: &str) -> usize {
    let spaces = line.chars().take_while(|&c| c == ' ' || c == '\t').count();
    spaces / 4
}

fn extract_name_until(line: &str, delimiters: &[char]) -> String {
    let trimmed = line.trim();
    let end_idx = trimmed
        .find(|c: char| delimiters.contains(&c))
        .unwrap_or(trimmed.len());
    trimmed[..end_idx].trim().to_string()
}

fn extract_rust_symbols(content: &str) -> Vec<CodeSymbol> {
    let mut symbols = Vec::new();

    for (idx, raw_line) in content.lines().enumerate() {
        let line = raw_line.trim();
        let indent = calculate_indent(raw_line);
        let line_number = idx + 1;

        if line.starts_with("//") || line.starts_with("/*") || line.starts_with('*') {
            continue;
        }

        let clean_line = line.strip_prefix("pub ").unwrap_or(line);
        let clean_line = clean_line
            .strip_prefix("(crate) ")
            .unwrap_or(clean_line)
            .trim_start();

        if let Some(rest) = clean_line.strip_prefix("fn ") {
            let name = extract_name_until(rest, &['(', '<', '{']);
            if !name.is_empty() {
                symbols.push(CodeSymbol {
                    name,
                    kind: SymbolKind::Function,
                    line_number,
                    indent_level: indent,
                });
            }
        } else if let Some(rest) = clean_line.strip_prefix("async fn ") {
            let name = extract_name_until(rest, &['(', '<', '{']);
            if !name.is_empty() {
                symbols.push(CodeSymbol {
                    name,
                    kind: SymbolKind::Function,
                    line_number,
                    indent_level: indent,
                });
            }
        } else if let Some(rest) = clean_line.strip_prefix("struct ") {
            let name = extract_name_until(rest, &['<', '{', '(', ';']);
            if !name.is_empty() {
                symbols.push(CodeSymbol {
                    name,
                    kind: SymbolKind::Struct,
                    line_number,
                    indent_level: indent,
                });
            }
        } else if let Some(rest) = clean_line.strip_prefix("enum ") {
            let name = extract_name_until(rest, &['<', '{', ';']);
            if !name.is_empty() {
                symbols.push(CodeSymbol {
                    name,
                    kind: SymbolKind::Enum,
                    line_number,
                    indent_level: indent,
                });
            }
        } else if let Some(rest) = clean_line.strip_prefix("trait ") {
            let name = extract_name_until(rest, &['<', '{', ':']);
            if !name.is_empty() {
                symbols.push(CodeSymbol {
                    name,
                    kind: SymbolKind::Trait,
                    line_number,
                    indent_level: indent,
                });
            }
        } else if let Some(rest) = clean_line.strip_prefix("impl") {
            let rest = rest.trim_start();
            let name = extract_name_until(rest, &['{', '<', '\n']);
            if !name.is_empty() {
                symbols.push(CodeSymbol {
                    name: format!("impl {}", name),
                    kind: SymbolKind::Trait,
                    line_number,
                    indent_level: indent,
                });
            }
        } else if let Some(rest) = clean_line.strip_prefix("mod ") {
            let name = extract_name_until(rest, &[';', '{']);
            if !name.is_empty() {
                symbols.push(CodeSymbol {
                    name,
                    kind: SymbolKind::Module,
                    line_number,
                    indent_level: indent,
                });
            }
        } else if let Some(rest) = clean_line.strip_prefix("type ") {
            let name = extract_name_until(rest, &['=', '<', ';']);
            if !name.is_empty() {
                symbols.push(CodeSymbol {
                    name,
                    kind: SymbolKind::Type,
                    line_number,
                    indent_level: indent,
                });
            }
        }
    }

    symbols
}

fn extract_python_symbols(content: &str) -> Vec<CodeSymbol> {
    let mut symbols = Vec::new();

    for (idx, raw_line) in content.lines().enumerate() {
        let line = raw_line.trim();
        let indent = calculate_indent(raw_line);
        let line_number = idx + 1;

        if line.starts_with('#') {
            continue;
        }

        if let Some(rest) = line.strip_prefix("def ") {
            let name = extract_name_until(rest, &['(', ':']);
            if !name.is_empty() {
                symbols.push(CodeSymbol {
                    name,
                    kind: SymbolKind::Function,
                    line_number,
                    indent_level: indent,
                });
            }
        } else if let Some(rest) = line.strip_prefix("async def ") {
            let name = extract_name_until(rest, &['(', ':']);
            if !name.is_empty() {
                symbols.push(CodeSymbol {
                    name,
                    kind: SymbolKind::Function,
                    line_number,
                    indent_level: indent,
                });
            }
        } else if let Some(rest) = line.strip_prefix("class ") {
            let name = extract_name_until(rest, &['(', ':']);
            if !name.is_empty() {
                symbols.push(CodeSymbol {
                    name,
                    kind: SymbolKind::Class,
                    line_number,
                    indent_level: indent,
                });
            }
        }
    }

    symbols
}

fn extract_ts_js_symbols(content: &str) -> Vec<CodeSymbol> {
    let mut symbols = Vec::new();

    for (idx, raw_line) in content.lines().enumerate() {
        let line = raw_line.trim();
        let indent = calculate_indent(raw_line);
        let line_number = idx + 1;

        if line.starts_with("//") || line.starts_with("/*") || line.starts_with('*') {
            continue;
        }

        let clean_line = line.strip_prefix("export ").unwrap_or(line);
        let clean_line = clean_line.strip_prefix("default ").unwrap_or(clean_line);

        if let Some(rest) = clean_line.strip_prefix("function ") {
            let name = extract_name_until(rest, &['(', '<']);
            if !name.is_empty() {
                symbols.push(CodeSymbol {
                    name,
                    kind: SymbolKind::Function,
                    line_number,
                    indent_level: indent,
                });
            }
        } else if let Some(rest) = clean_line.strip_prefix("async function ") {
            let name = extract_name_until(rest, &['(', '<']);
            if !name.is_empty() {
                symbols.push(CodeSymbol {
                    name,
                    kind: SymbolKind::Function,
                    line_number,
                    indent_level: indent,
                });
            }
        } else if let Some(rest) = clean_line.strip_prefix("class ") {
            let name = extract_name_until(rest, &['<', '{', ' ']);
            if !name.is_empty() {
                symbols.push(CodeSymbol {
                    name,
                    kind: SymbolKind::Class,
                    line_number,
                    indent_level: indent,
                });
            }
        } else if let Some(rest) = clean_line.strip_prefix("interface ") {
            let name = extract_name_until(rest, &['<', '{', ' ']);
            if !name.is_empty() {
                symbols.push(CodeSymbol {
                    name,
                    kind: SymbolKind::Trait,
                    line_number,
                    indent_level: indent,
                });
            }
        } else if let Some(rest) = clean_line.strip_prefix("type ") {
            let name = extract_name_until(rest, &['<', '=', ' ']);
            if !name.is_empty() {
                symbols.push(CodeSymbol {
                    name,
                    kind: SymbolKind::Type,
                    line_number,
                    indent_level: indent,
                });
            }
        } else if (clean_line.starts_with("const ") || clean_line.starts_with("let "))
            && (clean_line.contains("=>") || clean_line.contains("function"))
        {
            let rest = clean_line
                .strip_prefix("const ")
                .or_else(|| clean_line.strip_prefix("let "))
                .unwrap_or("");
            let name = extract_name_until(rest, &[':', '=', ' ']);
            if !name.is_empty() {
                symbols.push(CodeSymbol {
                    name,
                    kind: SymbolKind::Function,
                    line_number,
                    indent_level: indent,
                });
            }
        }
    }

    symbols
}

fn extract_go_symbols(content: &str) -> Vec<CodeSymbol> {
    let mut symbols = Vec::new();

    for (idx, raw_line) in content.lines().enumerate() {
        let line = raw_line.trim();
        let indent = calculate_indent(raw_line);
        let line_number = idx + 1;

        if line.starts_with("//") {
            continue;
        }

        if let Some(rest) = line.strip_prefix("func ") {
            let rest = rest.trim_start();
            let (kind, name) = if rest.starts_with('(') {
                // Method with receiver: func (s *Struct) MethodName()
                if let Some(after_paren) = rest.find(')') {
                    let method_part = rest[after_paren + 1..].trim_start();
                    let name = extract_name_until(method_part, &['(', '<']);
                    (SymbolKind::Function, name)
                } else {
                    (SymbolKind::Function, extract_name_until(rest, &['(', '<']))
                }
            } else {
                (SymbolKind::Function, extract_name_until(rest, &['(', '<']))
            };

            if !name.is_empty() {
                symbols.push(CodeSymbol {
                    name,
                    kind,
                    line_number,
                    indent_level: indent,
                });
            }
        } else if let Some(rest) = line.strip_prefix("type ") {
            let name = extract_name_until(rest, &[' ']);
            let kind = if line.contains("struct") {
                SymbolKind::Struct
            } else if line.contains("interface") {
                SymbolKind::Trait
            } else {
                SymbolKind::Type
            };
            if !name.is_empty() {
                symbols.push(CodeSymbol {
                    name,
                    kind,
                    line_number,
                    indent_level: indent,
                });
            }
        }
    }

    symbols
}

fn extract_cpp_symbols(content: &str) -> Vec<CodeSymbol> {
    let mut symbols = Vec::new();

    for (idx, raw_line) in content.lines().enumerate() {
        let line = raw_line.trim();
        let indent = calculate_indent(raw_line);
        let line_number = idx + 1;

        if line.starts_with("//") || line.starts_with("/*") || line.starts_with('*') {
            continue;
        }

        if let Some(rest) = line.strip_prefix("class ") {
            let name = extract_name_until(rest, &[':', '{', ';', ' ']);
            if !name.is_empty() {
                symbols.push(CodeSymbol {
                    name,
                    kind: SymbolKind::Class,
                    line_number,
                    indent_level: indent,
                });
            }
        } else if let Some(rest) = line.strip_prefix("struct ") {
            let name = extract_name_until(rest, &[':', '{', ';', ' ']);
            if !name.is_empty() {
                symbols.push(CodeSymbol {
                    name,
                    kind: SymbolKind::Struct,
                    line_number,
                    indent_level: indent,
                });
            }
        } else if let Some(rest) = line.strip_prefix("enum ") {
            let name = extract_name_until(rest, &[':', '{', ';', ' ']);
            if !name.is_empty() {
                symbols.push(CodeSymbol {
                    name,
                    kind: SymbolKind::Enum,
                    line_number,
                    indent_level: indent,
                });
            }
        }
    }

    symbols
}

fn extract_generic_symbols(content: &str) -> Vec<CodeSymbol> {
    let mut symbols = Vec::new();

    for (idx, raw_line) in content.lines().enumerate() {
        let line = raw_line.trim();
        let indent = calculate_indent(raw_line);
        let line_number = idx + 1;

        if line.starts_with('#') || line.starts_with("//") {
            continue;
        }

        if let Some(rest) = line.strip_prefix("function ") {
            let name = extract_name_until(rest, &['(', ' ']);
            if !name.is_empty() {
                symbols.push(CodeSymbol {
                    name,
                    kind: SymbolKind::Function,
                    line_number,
                    indent_level: indent,
                });
            }
        }
    }

    symbols
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_rust_symbols() {
        let code = r#"
pub struct UserConfig {
    pub id: u64,
}

impl UserConfig {
    pub fn new() -> Self {
        Self { id: 0 }
    }
}

pub enum Status {
    Active,
    Inactive,
}

fn calculate_total(a: i32, b: i32) -> i32 {
    a + b
}
"#;
        let symbols = extract_symbols(code, "rs");
        assert_eq!(symbols.len(), 5);
        assert_eq!(symbols[0].name, "UserConfig");
        assert_eq!(symbols[0].kind, SymbolKind::Struct);
        assert_eq!(symbols[0].line_number, 2);
        assert_eq!(symbols[1].name, "impl UserConfig");
        assert_eq!(symbols[2].name, "new");
        assert_eq!(symbols[2].kind, SymbolKind::Function);
        assert_eq!(symbols[3].name, "Status");
        assert_eq!(symbols[3].kind, SymbolKind::Enum);
        assert_eq!(symbols[4].name, "calculate_total");
        assert_eq!(symbols[4].kind, SymbolKind::Function);
    }

    #[test]
    fn extracts_python_symbols() {
        let code = r#"
class PreviewEngine:
    def __init__(self):
        pass

    async def fetch_data(self):
        return True

def standalone_helper():
    pass
"#;
        let symbols = extract_symbols(code, "py");
        assert_eq!(symbols.len(), 4);
        assert_eq!(symbols[0].name, "PreviewEngine");
        assert_eq!(symbols[0].kind, SymbolKind::Class);
        assert_eq!(symbols[1].name, "__init__");
        assert_eq!(symbols[1].kind, SymbolKind::Function);
        assert_eq!(symbols[2].name, "fetch_data");
        assert_eq!(symbols[2].kind, SymbolKind::Function);
        assert_eq!(symbols[3].name, "standalone_helper");
        assert_eq!(symbols[3].kind, SymbolKind::Function);
    }

    #[test]
    fn extracts_typescript_symbols() {
        let code = r#"
export interface Config {
    port: number;
}

export class Server {
    start() {}
}

export const runHandler = () => {
    console.log("running");
};
"#;
        let symbols = extract_symbols(code, "ts");
        assert_eq!(symbols.len(), 3);
        assert_eq!(symbols[0].name, "Config");
        assert_eq!(symbols[0].kind, SymbolKind::Trait);
        assert_eq!(symbols[1].name, "Server");
        assert_eq!(symbols[1].kind, SymbolKind::Class);
        assert_eq!(symbols[2].name, "runHandler");
        assert_eq!(symbols[2].kind, SymbolKind::Function);
    }

    #[test]
    fn extracts_go_symbols() {
        let code = r#"
type Server struct {
    port int
}

func (s *Server) Start() error {
    return nil
}

func Helper() {}
"#;
        let symbols = extract_symbols(code, "go");
        assert_eq!(symbols.len(), 3);
        assert_eq!(symbols[0].name, "Server");
        assert_eq!(symbols[0].kind, SymbolKind::Struct);
        assert_eq!(symbols[1].name, "Start");
        assert_eq!(symbols[1].kind, SymbolKind::Function);
        assert_eq!(symbols[2].name, "Helper");
        assert_eq!(symbols[2].kind, SymbolKind::Function);
    }
}
