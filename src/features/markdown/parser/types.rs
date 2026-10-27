#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertKind {
    Note,
    Tip,
    Important,
    Warning,
    Caution,
}

impl AlertKind {
    pub fn title(self) -> &'static str {
        match self {
            Self::Note => "NOTE",
            Self::Tip => "TIP",
            Self::Important => "IMPORTANT",
            Self::Warning => "WARNING",
            Self::Caution => "CAUTION",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inline {
    Text(String),
    Bold(Vec<Inline>),
    Italic(Vec<Inline>),
    Strikethrough(Vec<Inline>),
    Underline(Vec<Inline>),
    Code(String),
    Link {
        text: Vec<Inline>,
        url: String,
    },
    Image {
        alt: String,
        url: String,
        width: Option<u32>,
        height: Option<u32>,
    },
    InlineMath(String),
    DisplayMath(String),
    FootnoteReference(String),
    SoftBreak,
    Ruby {
        text: String,
        ruby: String,
    },
}

#[derive(Debug, Clone)]
pub struct TableCell {
    pub content: Vec<Inline>,
}

#[derive(Debug, Clone)]
pub struct TableBlock {
    pub headers: Vec<TableCell>,
    pub rows: Vec<Vec<TableCell>>,
    pub column_weights: Vec<u16>,
    pub alignments: Vec<pulldown_cmark::Alignment>,
}

#[derive(Debug, Clone)]
pub enum Block {
    Frontmatter(Vec<(String, String)>),
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
        link_url: Option<String>,
        width: Option<u32>,
        height: Option<u32>,
    },
    List {
        ordered: bool,
        start_number: u64,
        items: Vec<ListItem>,
    },
    Quote(Vec<Block>),
    Alert {
        kind: AlertKind,
        content: Vec<Block>,
    },
    FootnoteDefinition {
        label: String,
        content: Vec<Block>,
    },
    HorizontalRule,
    Html(String),
    Math(String),
}

#[derive(Debug, Clone)]
pub struct ListItem {
    pub is_task: Option<bool>,
    pub content: Vec<Inline>,
    pub sub_blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockLayout {
    pub estimated_height: f32,
    pub measured_height: Option<f32>,
}

impl BlockLayout {
    #[inline]
    pub fn new(estimated_height: f32) -> Self {
        Self {
            estimated_height,
            measured_height: None,
        }
    }

    #[inline]
    pub fn effective_height(&self) -> f32 {
        self.measured_height.unwrap_or(self.estimated_height)
    }

    #[inline]
    pub fn invalidate_measurement(&mut self) {
        self.measured_height = None;
    }
}

pub fn collect_image_paths(block: &Block, out: &mut Vec<String>) {
    match block {
        Block::Image { path, .. } => {
            out.push(path.clone());
        }
        Block::List { items, .. } => {
            for item in items {
                for sub in &item.sub_blocks {
                    collect_image_paths(sub, out);
                }
            }
        }
        Block::Quote(blocks)
        | Block::Alert {
            content: blocks, ..
        }
        | Block::FootnoteDefinition {
            content: blocks, ..
        } => {
            for b in blocks {
                collect_image_paths(b, out);
            }
        }
        Block::Frontmatter(_)
        | Block::Heading { .. }
        | Block::Paragraph(_)
        | Block::CodeBlock { .. }
        | Block::Table(_)
        | Block::Mermaid { .. }
        | Block::HorizontalRule
        | Block::Html(_)
        | Block::Math(_) => {}
    }
}

pub fn collect_all_image_paths(blocks: &[Block]) -> Vec<String> {
    let mut paths = Vec::new();
    for block in blocks {
        collect_image_paths(block, &mut paths);
    }
    paths.sort();
    paths.dedup();
    paths
}
