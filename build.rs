use std::env;
use std::fs::File;
use std::path::Path;
use syntect::dumps::dump_to_writer;
use syntect::parsing::SyntaxSet;
use syntect::parsing::syntax_definition::SyntaxDefinition;

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

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let dest_path = Path::new(&out_dir).join("syntax_set.pack");

    let mut builder = SyntaxSet::load_defaults_newlines().into_builder();
    let ts_syntax =
        SyntaxDefinition::load_from_str(TYPESCRIPT_SYNTAX_YAML, true, Some("TypeScript"))
            .expect("failed to load TypeScript syntax YAML");
    builder.add(ts_syntax);

    let toml_syntax = SyntaxDefinition::load_from_str(TOML_SYNTAX_YAML, true, Some("TOML"))
        .expect("failed to load TOML syntax YAML");
    builder.add(toml_syntax);

    let syntax_set = builder.build();

    let mut out_file = File::create(&dest_path).expect("failed to create syntax_set.pack");
    dump_to_writer(&syntax_set, &mut out_file).expect("failed to dump syntax set");
}
