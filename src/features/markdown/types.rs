#[derive(Debug)]
pub struct ImageRef {
    pub alt_text: String,
    pub path: String,
}

#[derive(Debug, Clone)]
pub enum Inline {
    Text(String),
    Bold(Vec<Inline>),
    Italic(Vec<Inline>),
    Strikethrough(Vec<Inline>),
    Code(String),
    Link { text: Vec<Inline>, url: String },
    Image { alt: String, url: String },
    InlineMath(String),
    DisplayMath(String),
    SoftBreak,
}

#[derive(Debug, Clone)]
pub struct TableCell {
    pub content: Vec<Inline>,
}

#[derive(Debug, Clone)]
pub struct TableBlock {
    pub headers: Vec<TableCell>,
    pub rows: Vec<Vec<TableCell>>,
}

#[derive(Debug, Clone)]
pub enum Block {
    Heading {
        level: u8,
        content: Vec<Inline>,
    },
    Paragraph(Vec<Inline>),
    CodeBlock {
        lang: Option<String>,
        title: Option<String>,
        code: String,
    },
    Table(TableBlock),
    Mermaid {
        lines: Vec<String>,
        rendered: Option<Vec<u8>>,
    },
    Image {
        alt: String,
        path: String,
    },
    List {
        ordered: bool,
        start_number: u64,
        items: Vec<ListItem>,
    },
    Quote(Vec<Block>),
    HorizontalRule,
    Html(String),
}

#[derive(Debug, Clone)]
pub struct ListItem {
    pub is_task: Option<bool>,
    pub content: Vec<Inline>,
    pub sub_blocks: Vec<Block>,
}
