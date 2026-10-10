use std::sync::OnceLock;
use syntect::highlighting::ThemeSet;
use syntect::parsing::syntax_definition::SyntaxDefinition;
use syntect::parsing::{SyntaxReference, SyntaxSet};

const TYPESCRIPT_SYNTAX_YAML: &str = r#"
%YAML 1.2
---
name: TypeScript
file_extensions:
  - ts
  - tsx
  - mts
  - cts
scope: source.ts
first_line_match: ^#!.*\b(node|deno|bun|ts-node)\b

contexts:
  main:
    # Comments
    - match: '/\*\*'
      scope: punctuation.definition.comment.begin.ts
      push: doc-comment
    - match: '/\*'
      scope: punctuation.definition.comment.begin.ts
      push: block-comment
    - match: '//.*$'
      scope: comment.line.double-slash.ts

    # Strings
    - match: '"'
      scope: punctuation.definition.string.begin.ts
      push: double-quoted-string
    - match: "'"
      scope: punctuation.definition.string.begin.ts
      push: single-quoted-string
    - match: '`'
      scope: punctuation.definition.string.begin.ts
      push: template-string

    # Numbers
    - match: '\b0[xX][0-9a-fA-F][0-9a-fA-F_]*n?\b'
      scope: constant.numeric.hex.ts
    - match: '\b0[bB][01][01_]*n?\b'
      scope: constant.numeric.binary.ts
    - match: '\b0[oO][0-7][0-7_]*n?\b'
      scope: constant.numeric.octal.ts
    - match: '\b[0-9][0-9_]*(\.[0-9][0-9_]*)?([eE][+-]?[0-9][0-9_]*)?n?\b'
      scope: constant.numeric.decimal.ts

    # Storage types
    - match: '\b(const|let|var|function|class|interface|type|enum|namespace|module|constructor)\b'
      scope: storage.type.ts

    # Storage modifiers
    - match: '\b(async|await|yield)\b'
      scope: storage.modifier.async.ts
    - match: '\b(public|private|protected|readonly|override|static|abstract|declare|get|set)\b'
      scope: storage.modifier.access.ts

    # Control flow keywords
    - match: '\b(if|else|switch|case|default|break|continue|return|throw|try|catch|finally)\b'
      scope: keyword.control.flow.ts
    - match: '\b(while|do|for|of|in)\b'
      scope: keyword.control.loop.ts

    # Module imports / exports
    - match: '\b(import|export|from|as|default)\b'
      scope: keyword.control.import-export.ts

    # Special operators & expressions
    - match: '\b(new|delete|void|typeof|instanceof|keyof|is|infer|satisfies|asserts)\b'
      scope: keyword.operator.expression.ts

    # Primitive types
    - match: '\b(string|number|boolean|symbol|bigint|any|unknown|never|void|object|undefined|null)\b'
      scope: support.type.primitive.ts

    # Language constants
    - match: '\b(true|false|null|undefined|NaN|Infinity)\b'
      scope: constant.language.ts

    # Special variables
    - match: '\b(this|super|arguments)\b'
      scope: variable.language.ts

    # Decorators (@decorator)
    - match: '@[a-zA-Z_$][a-zA-Z0-9_$]*'
      scope: entity.name.function.decorator.ts

    # JSX / TSX tags
    - match: '</?[a-zA-Z_$][a-zA-Z0-9_$-]*'
      scope: entity.name.tag.tsx
    - match: '/?>'
      scope: entity.name.tag.tsx

    # Arrow function operator
    - match: '=>'
      scope: storage.type.function.arrow.ts

    # Function calls: foo(...)
    - match: '\b([a-zA-Z_$][a-zA-Z0-9_$]*)\s*(?=\()'
      scope: entity.name.function.ts

    # Properties: .length, .map, .resolve
    - match: '\.([a-zA-Z_$][a-zA-Z0-9_$]*)'
      captures:
        1: variable.other.property.ts

    # Object keys or type labels: key:
    - match: '\b([a-zA-Z_$][a-zA-Z0-9_$]*)\s*(?=:)'
      captures:
        1: variable.other.declaration.ts

    # Type / Class names (PascalCase)
    - match: '\b[A-Z][a-zA-Z0-9_$]*\b'
      scope: entity.name.type.ts

    # Operators
    - match: '===|!==|==|!=|<=|>=|&&|\|\||\?\?|\?\.|[+*/%&|^~!<>=?:-]'
      scope: keyword.operator.ts

    # Punctuation
    - match: '[{}()\\[\\],;.]'
      scope: punctuation.terminator.ts

  doc-comment:
    - meta_scope: comment.block.documentation.ts
    - match: '\*/'
      scope: punctuation.definition.comment.end.ts
      pop: true
    - match: '@[a-zA-Z_]+'
      scope: keyword.other.documentation.ts

  block-comment:
    - meta_scope: comment.block.ts
    - match: '\*/'
      scope: punctuation.definition.comment.end.ts
      pop: true

  double-quoted-string:
    - meta_scope: string.quoted.double.ts
    - match: '\\(x[0-9a-fA-F]{2}|u[0-9a-fA-F]{4}|u\{[0-9a-fA-F]+\}|[0-7]{1,3}|.)'
      scope: constant.character.escape.ts
    - match: '"'
      scope: punctuation.definition.string.end.ts
      pop: true

  single-quoted-string:
    - meta_scope: string.quoted.single.ts
    - match: '\\(x[0-9a-fA-F]{2}|u[0-9a-fA-F]{4}|u\{[0-9a-fA-F]+\}|[0-7]{1,3}|.)'
      scope: constant.character.escape.ts
    - match: "'"
      scope: punctuation.definition.string.end.ts
      pop: true

  template-string:
    - meta_scope: string.template.ts
    - match: '\\(x[0-9a-fA-F]{2}|u[0-9a-fA-F]{4}|u\{[0-9a-fA-F]+\}|[0-7]{1,3}|.)'
      scope: constant.character.escape.ts
    - match: '\$\{'
      scope: punctuation.definition.template-expression.begin.ts
      push: template-expression
    - match: '`'
      scope: punctuation.definition.string.end.ts
      pop: true

  template-expression:
    - match: '\}'
      scope: punctuation.definition.template-expression.end.ts
      pop: true
    - include: main
"#;

const TOML_SYNTAX_YAML: &str = r#"
%YAML 1.2
---
name: TOML
file_extensions:
  - toml
  - Cargo.lock
  - Gopkg.lock
  - Pipfile
  - poetry.lock
scope: source.toml

contexts:
  main:
    # Comments
    - match: '#.*$'
      scope: comment.line.number-sign.toml

    # Table headers: [[table.array]] or [table]
    - match: '^\s*(\[\[)(.+?)(\]\])'
      captures:
        1: punctuation.definition.tag.begin.toml
        2: entity.name.section.toml
        3: punctuation.definition.tag.end.toml
    - match: '^\s*(\[)(.+?)(\])'
      captures:
        1: punctuation.definition.tag.begin.toml
        2: entity.name.section.toml
        3: punctuation.definition.tag.end.toml

    # Keys: key = or "key" = or 'key' =
    - match: '([a-zA-Z0-9_.-]+)\s*(=)'
      captures:
        1: variable.other.property.toml
        2: keyword.operator.assignment.toml

    # Strings: Multiline or Basic
    - match: '"""'
      scope: punctuation.definition.string.begin.toml
      push: ml-basic-string
    - match: "'''"
      scope: punctuation.definition.string.begin.toml
      push: ml-literal-string
    - match: '"'
      scope: punctuation.definition.string.begin.toml
      push: basic-string
    - match: "'"
      scope: punctuation.definition.string.begin.toml
      push: literal-string

    # Booleans
    - match: '\b(true|false)\b'
      scope: constant.language.boolean.toml

    # Datetimes
    - match: '\b\d{4}-\d{2}-\d{2}(?:[Tt ]\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:[Zz]|[+-]\d{2}:\d{2})?)?\b'
      scope: constant.other.datetime.toml

    # Numbers: Float, Hex, Oct, Bin, Int
    - match: '\b0[xX][0-9a-fA-F_]+\b'
      scope: constant.numeric.hex.toml
    - match: '\b0[oO][0-7_]+\b'
      scope: constant.numeric.octal.toml
    - match: '\b0[bB][01_]+\b'
      scope: constant.numeric.binary.toml
    - match: '[-+]?(?:0|[1-9][0-9_]*)(?:\.[0-9_]+)?(?:[eE][-+]?[0-9_]+)?\b'
      scope: constant.numeric.decimal.toml

    # Punctuation / delimiters
    - match: '[{}\\[\\],=]'
      scope: punctuation.terminator.toml

  basic-string:
    - meta_scope: string.quoted.double.toml
    - match: '\\(?:[btnfru"\\/]|u[0-9a-fA-F]{4}|U[0-9a-fA-F]{8})'
      scope: constant.character.escape.toml
    - match: '"'
      scope: punctuation.definition.string.end.toml
      pop: true

  literal-string:
    - meta_scope: string.quoted.single.toml
    - match: "'"
      scope: punctuation.definition.string.end.toml
      pop: true

  ml-basic-string:
    - meta_scope: string.quoted.triple.double.toml
    - match: '\\(?:[btnfru"\\/]|u[0-9a-fA-F]{4}|U[0-9a-fA-F]{8})'
      scope: constant.character.escape.toml
    - match: '"""'
      scope: punctuation.definition.string.end.toml
      pop: true

  ml-literal-string:
    - meta_scope: string.quoted.triple.single.toml
    - match: "'''"
      scope: punctuation.definition.string.end.toml
      pop: true
"#;

fn build_global_syntax_set() -> SyntaxSet {
    let mut builder = SyntaxSet::load_defaults_newlines().into_builder();
    if let Ok(ts_syntax) =
        SyntaxDefinition::load_from_str(TYPESCRIPT_SYNTAX_YAML, true, Some("TypeScript"))
    {
        builder.add(ts_syntax);
    }
    if let Ok(toml_syntax) = SyntaxDefinition::load_from_str(TOML_SYNTAX_YAML, true, Some("TOML")) {
        builder.add(toml_syntax);
    }
    builder.build()
}

/// Global syntax set lazily initialized once with TypeScript and common language extensions.
pub fn global_syntax_set() -> &'static SyntaxSet {
    static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
    SYNTAX_SET.get_or_init(build_global_syntax_set)
}

struct ThemePaletteDef {
    name: &'static str,
    bg: (u8, u8, u8),
    fg: (u8, u8, u8),
    comment: (u8, u8, u8),
    string: (u8, u8, u8),
    keyword: (u8, u8, u8),
    storage: (u8, u8, u8),
    modifier: (u8, u8, u8),
    function: (u8, u8, u8),
    type_name: (u8, u8, u8),
    number: (u8, u8, u8),
    tag: (u8, u8, u8),
    operator: (u8, u8, u8),
    property: (u8, u8, u8),
    punctuation: (u8, u8, u8),
}

fn create_custom_syntect_theme(def: &ThemePaletteDef) -> syntect::highlighting::Theme {
    use std::str::FromStr;
    use syntect::highlighting::{
        Color as SyntectColor, ScopeSelectors, StyleModifier, Theme, ThemeItem, ThemeSettings,
    };

    let syn_color = |(r, g, b): (u8, u8, u8)| SyntectColor { r, g, b, a: 255 };

    let mut scopes = Vec::new();
    let scope_mappings: [(&str, (u8, u8, u8)); 15] = [
        ("comment, punctuation.definition.comment", def.comment),
        ("string, punctuation.definition.string", def.string),
        (
            "keyword.control, keyword.control.flow, keyword.control.loop, keyword.control.import-export",
            def.keyword,
        ),
        ("storage.type, storage.type.function", def.storage),
        (
            "storage.modifier, storage.modifier.async, storage.modifier.access",
            def.modifier,
        ),
        (
            "keyword.operator, keyword.operator.expression, keyword.operator.assignment",
            def.operator,
        ),
        (
            "entity.name.function, support.function, entity.name.function.decorator",
            def.function,
        ),
        (
            "entity.name.type, entity.name.class, support.type, support.class, support.type.primitive",
            def.type_name,
        ),
        (
            "constant.numeric, constant.language, constant.character, constant.other, constant.language.boolean",
            def.number,
        ),
        (
            "variable.other.property, variable.other.declaration, entity.name.section, variable.other.member, variable.language",
            def.property,
        ),
        ("entity.name.tag", def.tag),
        (
            "punctuation.terminator, punctuation.separator, punctuation.definition.tag",
            def.punctuation,
        ),
        ("meta.structure.dictionary.key string", def.property),
        ("entity.name.section.toml", def.type_name),
        ("variable.other.property.toml", def.property),
    ];

    for (scope_str, rgb) in scope_mappings {
        if let Ok(selectors) = ScopeSelectors::from_str(scope_str) {
            scopes.push(ThemeItem {
                scope: selectors,
                style: StyleModifier {
                    foreground: Some(syn_color(rgb)),
                    background: None,
                    font_style: None,
                },
            });
        }
    }

    let settings = ThemeSettings {
        background: Some(syn_color(def.bg)),
        foreground: Some(syn_color(def.fg)),
        ..Default::default()
    };

    Theme {
        name: Some(def.name.to_string()),
        author: Some("Kglance".to_string()),
        settings,
        scopes,
    }
}

const CUSTOM_THEME_DEFS: [ThemePaletteDef; 8] = [
    // 1. Dark (Base16 Eighties tuned)
    ThemePaletteDef {
        name: "dark",
        bg: (45, 45, 45),
        fg: (211, 208, 200),
        comment: (116, 115, 105),
        string: (153, 204, 153),
        keyword: (204, 153, 204),
        storage: (102, 153, 204),
        modifier: (204, 153, 204),
        function: (102, 153, 204),
        type_name: (255, 204, 102),
        number: (249, 145, 87),
        tag: (242, 119, 122),
        operator: (102, 204, 204),
        property: (102, 153, 204),
        punctuation: (211, 208, 200),
    },
    // 2. Light (GitHub tuned - high contrast dark text on white)
    ThemePaletteDef {
        name: "light",
        bg: (255, 255, 255),
        fg: (36, 41, 46),
        comment: (106, 115, 125),
        string: (3, 47, 98),
        keyword: (215, 58, 73),
        storage: (0, 92, 197),
        modifier: (215, 58, 73),
        function: (111, 66, 193),
        type_name: (227, 98, 9),
        number: (0, 92, 197),
        tag: (34, 134, 58),
        operator: (215, 58, 73),
        property: (0, 92, 197),
        punctuation: (68, 77, 86),
    },
    // 3. Catppuccin Mocha
    ThemePaletteDef {
        name: "catppuccin-mocha",
        bg: (30, 30, 46),
        fg: (205, 214, 244),
        comment: (108, 112, 134),
        string: (166, 227, 161),
        keyword: (203, 166, 247),
        storage: (137, 180, 250),
        modifier: (203, 166, 247),
        function: (137, 220, 235),
        type_name: (249, 226, 175),
        number: (250, 179, 135),
        tag: (243, 139, 168),
        operator: (148, 226, 213),
        property: (180, 190, 254),
        punctuation: (147, 153, 178),
    },
    // 4. Catppuccin Latte (high contrast dark text on cream)
    ThemePaletteDef {
        name: "catppuccin-latte",
        bg: (239, 241, 245),
        fg: (76, 79, 105),
        comment: (156, 160, 176),
        string: (64, 160, 43),
        keyword: (136, 57, 239),
        storage: (30, 102, 245),
        modifier: (136, 57, 239),
        function: (23, 146, 153),
        type_name: (223, 142, 29),
        number: (254, 100, 11),
        tag: (230, 69, 83),
        operator: (4, 165, 229),
        property: (30, 102, 245),
        punctuation: (92, 95, 119),
    },
    // 5. Tokyo Night
    ThemePaletteDef {
        name: "tokyo-night",
        bg: (26, 27, 38),
        fg: (169, 177, 214),
        comment: (86, 95, 137),
        string: (158, 206, 106),
        keyword: (187, 154, 247),
        storage: (122, 162, 247),
        modifier: (187, 154, 247),
        function: (122, 162, 247),
        type_name: (42, 195, 222),
        number: (255, 158, 100),
        tag: (247, 118, 142),
        operator: (137, 221, 255),
        property: (122, 162, 247),
        punctuation: (154, 165, 206),
    },
    // 6. Gruvbox Dark
    ThemePaletteDef {
        name: "gruvbox-dark",
        bg: (40, 40, 40),
        fg: (235, 219, 178),
        comment: (146, 131, 116),
        string: (184, 187, 38),
        keyword: (251, 73, 52),
        storage: (254, 128, 25),
        modifier: (251, 73, 52),
        function: (131, 165, 152),
        type_name: (250, 189, 47),
        number: (211, 134, 155),
        tag: (254, 128, 25),
        operator: (254, 128, 25),
        property: (131, 165, 152),
        punctuation: (213, 196, 161),
    },
    // 7. Nord
    ThemePaletteDef {
        name: "nord",
        bg: (46, 52, 64),
        fg: (216, 222, 233),
        comment: (97, 110, 136),
        string: (163, 190, 140),
        keyword: (129, 161, 193),
        storage: (136, 192, 208),
        modifier: (129, 161, 193),
        function: (136, 192, 208),
        type_name: (143, 188, 187),
        number: (180, 142, 173),
        tag: (191, 97, 106),
        operator: (129, 161, 193),
        property: (136, 192, 208),
        punctuation: (229, 233, 240),
    },
    // 8. Dracula
    ThemePaletteDef {
        name: "dracula",
        bg: (40, 42, 54),
        fg: (248, 248, 242),
        comment: (98, 114, 164),
        string: (241, 250, 140),
        keyword: (255, 121, 198),
        storage: (139, 233, 253),
        modifier: (255, 121, 198),
        function: (80, 250, 123),
        type_name: (139, 233, 253),
        number: (189, 147, 249),
        tag: (255, 121, 198),
        operator: (255, 121, 198),
        property: (80, 250, 123),
        punctuation: (248, 248, 242),
    },
];

/// Global theme set lazily initialized once with customized palettes.
pub fn global_theme_set() -> &'static ThemeSet {
    static THEME_SET: OnceLock<ThemeSet> = OnceLock::new();
    THEME_SET.get_or_init(|| {
        let mut ts = ThemeSet::load_defaults();

        for def in &CUSTOM_THEME_DEFS {
            let theme = create_custom_syntect_theme(def);
            ts.themes.insert(def.name.to_string(), theme);
        }
        ts
    })
}

/// Finds matching syntax based on file extension or language name.
pub fn find_syntax_for_extension(ext: &str) -> &'static SyntaxReference {
    let ss = global_syntax_set();
    let clean_ext = ext.trim_start_matches('.').to_ascii_lowercase();

    ss.find_syntax_by_extension(&clean_ext)
        .or_else(|| ss.find_syntax_by_token(&clean_ext))
        .or_else(|| ss.find_syntax_by_name(&clean_ext))
        .or_else(|| {
            let alias = match clean_ext.as_str() {
                "ts" | "typescript" | "mts" | "cts" | "tsx" => "ts",
                "js" | "javascript" | "mjs" | "cjs" | "jsx" => "js",
                "rs" | "rust" => "rs",
                "py" | "python" | "pyw" | "pyi" => "py",
                "sh" | "bash" | "zsh" | "fish" | "shell" | "ksh" | "csh" | "tcsh" | "zshrc"
                | "bashrc" | "profile" | "env" | "envrc" => "sh",
                "yml" | "yaml" => "yaml",
                "html" | "htm" | "xhtml" | "vue" | "svelte" => "html",
                "xml" | "svg" | "xaml" | "plist" | "ui" => "xml",
                "css" | "scss" | "sass" | "less" => "css",
                "c" | "h" | "ino" => "c",
                "cpp" | "hpp" | "cc" | "hh" | "cxx" | "hxx" | "c++" | "h++" => "cpp",
                "cs" | "csharp" => "cs",
                "json" | "jsonc" | "json5" | "jsonl" | "ndjson" | "geojson" | "ipynb" => "json",
                "md" | "markdown" | "mdown" | "mkdn" => "md",
                "toml" | "cargo.lock" | "gopkg.lock" | "pipfile" | "poetry.lock" => "toml",
                "kt" | "kts" | "kotlin" => "java",
                "swift" => "rs",
                "go" | "golang" => "go",
                "sql" | "psql" | "pgsql" | "mysql" => "sql",
                "dockerfile" | "containerfile" => "sh",
                "cmake" | "cmakelists.txt" => "make",
                "make" | "makefile" | "gnumakefile" | "justfile" | "just" => "make",
                "ini" | "conf" | "config" | "cfg" | "properties" | "desktop" | "gitignore"
                | "gitconfig" | "gitmodules" | "gitattributes" => "yaml",
                "lua" => "lua",
                "rb" | "ruby" | "gemfile" | "rakefile" => "ruby",
                "php" | "phtml" => "php",
                "r" => "r",
                "erl" | "hrl" => "erlang",
                "hs" | "lhs" => "haskell",
                "scala" | "sbt" => "scala",
                "clj" | "cljs" | "cljc" | "edn" => "clojure",
                "nix" => "c",
                "typ" | "typst" => "md",
                "proto" | "protobuf" => "c",
                "graphql" | "gql" => "json",
                _ => return None,
            };
            ss.find_syntax_by_extension(alias)
                .or_else(|| ss.find_syntax_by_token(alias))
                .or_else(|| ss.find_syntax_by_name(alias))
        })
        .unwrap_or_else(|| ss.find_syntax_plain_text())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typescript_syntax_loaded() {
        let load_res =
            SyntaxDefinition::load_from_str(TYPESCRIPT_SYNTAX_YAML, true, Some("TypeScript"));
        if let Err(e) = load_res {
            panic!("Load error: {e:?}");
        }
        let ss = global_syntax_set();
        let ts_syntax = ss.find_syntax_by_extension("ts");
        assert!(ts_syntax.is_some());
        assert_eq!(ts_syntax.unwrap().name, "TypeScript");

        let tsx_syntax = ss.find_syntax_by_extension("tsx");
        assert!(tsx_syntax.is_some());
        assert_eq!(tsx_syntax.unwrap().name, "TypeScript");

        let mts_syntax = ss.find_syntax_by_extension("mts");
        assert!(mts_syntax.is_some());
        assert_eq!(mts_syntax.unwrap().name, "TypeScript");

        let cts_syntax = ss.find_syntax_by_extension("cts");
        assert!(cts_syntax.is_some());
        assert_eq!(cts_syntax.unwrap().name, "TypeScript");
    }

    #[test]
    fn test_find_syntax_for_extension_fallbacks() {
        let ts = find_syntax_for_extension("ts");
        assert_eq!(ts.name, "TypeScript");

        let typescript = find_syntax_for_extension("typescript");
        assert_eq!(typescript.name, "TypeScript");

        let js = find_syntax_for_extension("js");
        assert_eq!(js.name, "JavaScript");

        let mjs = find_syntax_for_extension("mjs");
        assert_eq!(mjs.name, "JavaScript");

        let pyw = find_syntax_for_extension("pyw");
        assert_eq!(pyw.name, "Python");

        let unknown = find_syntax_for_extension("unknown_format_xyz");
        assert_eq!(unknown.name, "Plain Text");

        let toml = find_syntax_for_extension("toml");
        assert_eq!(toml.name, "TOML");

        let cargo_lock = find_syntax_for_extension("Cargo.lock");
        assert_eq!(cargo_lock.name, "TOML");

        let rust = find_syntax_for_extension("rs");
        assert_eq!(rust.name, "Rust");

        let json = find_syntax_for_extension("json");
        assert_eq!(json.name, "JSON");

        let cpp = find_syntax_for_extension("cpp");
        assert_eq!(cpp.name, "C++");

        let sh = find_syntax_for_extension("sh");
        assert!(sh.name.contains("Shell") || sh.name == "sh" || sh.name == "Bash");
    }

    #[test]
    fn test_toml_syntax_loaded() {
        let load_res = SyntaxDefinition::load_from_str(TOML_SYNTAX_YAML, true, Some("TOML"));
        assert!(load_res.is_ok(), "TOML syntax YAML must be valid");
        let ss = global_syntax_set();
        let toml_syntax = ss.find_syntax_by_extension("toml");
        assert!(toml_syntax.is_some());
        assert_eq!(toml_syntax.unwrap().name, "TOML");
    }

    #[test]
    fn test_all_app_themes_have_syntect_themes() {
        let ts = global_theme_set();
        let all_themes = [
            crate::ui::theme::AppTheme::Dark,
            crate::ui::theme::AppTheme::Light,
            crate::ui::theme::AppTheme::CatppuccinMocha,
            crate::ui::theme::AppTheme::CatppuccinLatte,
            crate::ui::theme::AppTheme::TokyoNight,
            crate::ui::theme::AppTheme::GruvboxDark,
            crate::ui::theme::AppTheme::Nord,
            crate::ui::theme::AppTheme::Dracula,
        ];

        for theme in all_themes {
            let syn_name = theme.syntect_theme();
            assert!(
                ts.themes.contains_key(syn_name),
                "ThemeSet must contain syntect theme '{syn_name}' for AppTheme '{theme:?}'"
            );
        }
    }
}
