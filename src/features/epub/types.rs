use crate::features::markdown::MarkdownState;
use crate::parsers::markdown::Block;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct EpubChapterInfo {
    pub title: String,
    pub level: u8,
    pub anchor: Option<String>,
    pub file_href: String,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone)]
pub struct EpubState {
    pub title: String,
    pub author: String,
    pub chapters: Vec<EpubChapterInfo>,
    pub active_chapter: usize,
    pub sidebar_visible: bool,
    pub sidebar_width: f32,
    pub sidebar_resizing: bool,
    pub sidebar_drag_start_x: Option<f32>,
    pub sidebar_drag_start_width: f32,
    pub scroll_y: f32,
    pub collapsed_chapters: HashSet<usize>,
    pub markdown_state: MarkdownState,
}

impl Default for EpubState {
    fn default() -> Self {
        Self {
            title: String::new(),
            author: String::new(),
            chapters: Vec::new(),
            active_chapter: 0,
            sidebar_visible: false,
            sidebar_width: 240.0,
            sidebar_resizing: false,
            sidebar_drag_start_x: None,
            sidebar_drag_start_width: 240.0,
            scroll_y: 0.0,
            collapsed_chapters: HashSet::new(),
            markdown_state: MarkdownState::default(),
        }
    }
}
