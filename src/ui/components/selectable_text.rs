use std::hash::Hasher;

use iced::advanced::graphics::core::event::Event;
use iced::advanced::graphics::core::layout::{self, Layout};
use iced::advanced::graphics::core::mouse::{self, click};
use iced::advanced::graphics::core::renderer;
use iced::advanced::graphics::core::widget::{Tree, Widget, tree};
use iced::advanced::graphics::core::{Clipboard, Element, Shell};
use iced::advanced::graphics::text::cosmic_text;
use iced::advanced::text::{Paragraph, Text};
use iced::widget::text::Span;
use iced::{
    Background, Border, Color, Font, Length, Pixels, Point, Rectangle, Shadow, Size, Vector,
    alignment,
};

#[path = "selectable_text_tooltip.rs"]
mod tooltip;
pub use tooltip::TextTooltip;
use tooltip::{HoveredTooltip, compute_inline_rect, create_tooltip_overlay};

use crate::ui::theme::AppTheme;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextLink {
    pub start: usize,
    pub end: usize,
    pub url: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextRuby {
    pub start: usize,
    pub end: usize,
    pub ruby: String,
}

pub struct SelectableText<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Renderer: iced::advanced::text::Renderer<Paragraph = iced::advanced::graphics::text::Paragraph>,
    Renderer::Font: From<Font>,
{
    spans: Vec<Span<'a, (), Font>>,
    links: Vec<TextLink>,
    rubies: Vec<TextRuby>,
    tooltips: Vec<TextTooltip>,
    font_size: f32,
    default_text_color: Color,
    selection_color: Color,
    app_theme: Option<AppTheme>,
    block_index: Option<usize>,
    selection_range: Option<crate::core::SelectionRange>,
    on_selection_change: Option<Box<dyn Fn(Option<String>) -> Message + 'a>>,
    on_link_click: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_drag_start: Option<Box<dyn Fn(usize, usize) -> Message + 'a>>,
    on_drag_update: Option<Box<dyn Fn(usize, usize) -> Message + 'a>>,
    on_drag_end: Option<Box<dyn Fn() -> Message + 'a>>,
    on_clear_selection: Option<Box<dyn Fn() -> Message + 'a>>,
    drag_active: bool,
    width: Length,
    align_x: alignment::Horizontal,
    _phantom: std::marker::PhantomData<(Theme, Renderer)>,
}

impl<'a, Message, Theme, Renderer> SelectableText<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::text::Renderer<Paragraph = iced::advanced::graphics::text::Paragraph>,
    Renderer::Font: From<Font>,
{
    pub fn new(spans: Vec<Span<'a, (), Font>>, font_size: f32) -> Self {
        Self {
            spans,
            links: Vec::new(),
            rubies: Vec::new(),
            tooltips: Vec::new(),
            font_size,
            default_text_color: Color::BLACK,
            selection_color: Color::from_rgba(0.2, 0.4, 0.8, 0.3),
            app_theme: None,
            block_index: None,
            selection_range: None,
            on_selection_change: None,
            on_link_click: None,
            on_drag_start: None,
            on_drag_update: None,
            on_drag_end: None,
            on_clear_selection: None,
            drag_active: false,
            width: Length::Fill,
            align_x: alignment::Horizontal::Left,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn align_x(mut self, align: alignment::Horizontal) -> Self {
        self.align_x = align;
        self
    }

    pub fn links(mut self, links: Vec<TextLink>) -> Self {
        self.links = links;
        self
    }

    pub fn rubies(mut self, rubies: Vec<TextRuby>) -> Self {
        self.rubies = rubies;
        self
    }

    pub fn tooltips(mut self, tooltips: Vec<TextTooltip>) -> Self {
        self.tooltips = tooltips;
        self
    }

    pub fn app_theme(mut self, theme: AppTheme) -> Self {
        self.app_theme = Some(theme);
        self
    }

    pub fn top_padding(&self) -> f32 {
        if self.rubies.is_empty() {
            0.0
        } else {
            ruby_top_padding(self.font_size)
        }
    }

    pub fn on_link_click<F>(mut self, f: F) -> Self
    where
        F: Fn(String) -> Message + 'a,
    {
        self.on_link_click = Some(Box::new(f));
        self
    }

    pub fn default_text_color(mut self, color: Color) -> Self {
        self.default_text_color = color;
        self
    }

    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    pub fn block_index(mut self, index: usize) -> Self {
        self.block_index = Some(index);
        self
    }

    pub fn selection_range(mut self, range: Option<crate::core::SelectionRange>) -> Self {
        self.selection_range = range;
        self
    }

    pub fn on_selection_change<F>(mut self, f: F) -> Self
    where
        F: Fn(Option<String>) -> Message + 'a,
    {
        self.on_selection_change = Some(Box::new(f));
        self
    }

    pub fn on_drag_start<F>(mut self, f: F) -> Self
    where
        F: Fn(usize, usize) -> Message + 'a,
    {
        self.on_drag_start = Some(Box::new(f));
        self
    }

    pub fn on_drag_update<F>(mut self, f: F) -> Self
    where
        F: Fn(usize, usize) -> Message + 'a,
    {
        self.on_drag_update = Some(Box::new(f));
        self
    }

    pub fn on_drag_end<F>(mut self, f: F) -> Self
    where
        F: Fn() -> Message + 'a,
    {
        self.on_drag_end = Some(Box::new(f));
        self
    }

    pub fn on_clear_selection<F>(mut self, f: F) -> Self
    where
        F: Fn() -> Message + 'a,
    {
        self.on_clear_selection = Some(Box::new(f));
        self
    }

    pub fn drag_active(mut self, active: bool) -> Self {
        self.drag_active = active;
        self
    }

    #[must_use]
    pub fn extract_plain_text(&self) -> String {
        let mut text = String::new();
        for span in &self.spans {
            text.push_str(&span.text);
        }
        text
    }
}

#[derive(Default)]
struct State {
    paragraph: Option<iced::advanced::graphics::text::Paragraph>,
    last_bounds_width: f32,
    last_font_size: f32,
    last_spans_hash: u64,
    last_size: Size,
    selection: Option<(usize, usize)>,
    is_selecting: bool,

    is_mouse_held: bool,
    drag_start: Option<usize>,
    last_click: Option<mouse::Click>,
    plain_text: String,
    line_offsets: Vec<usize>,

    press_pos: Option<Point>,
    pressed_link_url: Option<String>,
    is_drag: bool,
    hovered_tooltip: Option<HoveredTooltip>,
}

pub fn ruby_top_padding(font_size: f32) -> f32 {
    ruby_box_height(font_size) + 2.0
}

fn ruby_font_size(font_size: f32) -> f32 {
    (font_size * 0.52).max(8.0)
}

pub fn ruby_box_height(font_size: f32) -> f32 {
    ruby_font_size(font_size) * 1.2
}

fn compute_line_starts(buffer: &cosmic_text::Buffer) -> Vec<usize> {
    let mut starts = Vec::with_capacity(buffer.lines.len() + 1);
    let mut current = 0;
    for line in &buffer.lines {
        starts.push(current);
        current += line.text().len() + line.ending().as_str().len();
    }
    starts.push(current);
    starts
}

fn point_to_offset(
    buffer: &cosmic_text::Buffer,
    line_offsets: &[usize],
    plain_text: &str,
    bounds: Rectangle,
    cursor_pos: Point,
    top_pad: f32,
    offset_x: f32,
) -> usize {
    let effective_y = bounds.y + top_pad;
    if cursor_pos.y < bounds.y {
        return 0;
    }
    if cursor_pos.y > bounds.y + bounds.height {
        return plain_text.len();
    }

    let clamped_x = cursor_pos.x.clamp(bounds.x, bounds.x + bounds.width);
    let rel_y = (cursor_pos.y - effective_y).max(0.0);
    let rel_pos = Point::new((clamped_x - (bounds.x + offset_x)).max(0.0), rel_y);

    if let Some(cursor) = buffer.hit(rel_pos.x, rel_pos.y) {
        let line_start = line_offsets.get(cursor.line).copied().unwrap_or(0);
        let mut offset = (line_start + cursor.index).min(plain_text.len());
        if cursor_pos.x < bounds.x {
            offset = line_start;
        }
        char_boundary(plain_text, offset)
    } else {
        0
    }
}

fn is_word_delimiter(c: char) -> bool {
    c.is_whitespace()
        || c.is_ascii_punctuation()
        || matches!(
            c,
            '“' | '”'
                | '‘'
                | '’'
                | '«'
                | '»'
                | '—'
                | '–'
                | '…'
                | '·'
                | '•'
                | '、'
                | '。'
                | '〈'
                | '〉'
                | '《'
                | '》'
                | '「'
                | '」'
                | '『'
                | '』'
                | '【'
                | '】'
                | '〔'
                | '〕'
                | '，'
                | '！'
                | '？'
                | '：'
                | '；'
        )
}

fn expand_to_word_bounds(text: &str, offset: usize) -> (usize, usize) {
    if text.is_empty() {
        return (0, 0);
    }
    let offset = char_boundary(text, offset.min(text.len()));

    if offset < text.len() {
        let ch = text[offset..].chars().next().unwrap_or(' ');
        if is_word_delimiter(ch) {
            return (offset, offset);
        }
    } else if offset == text.len() {
        let prev_ch = text.chars().next_back().unwrap_or(' ');
        if is_word_delimiter(prev_ch) {
            return (offset, offset);
        }
    }

    let mut start = offset;
    for (idx, ch) in text[..offset].char_indices().rev() {
        if is_word_delimiter(ch) {
            break;
        }
        start = idx;
    }

    let mut end = offset;
    for (idx, ch) in text[offset..].char_indices() {
        if is_word_delimiter(ch) {
            break;
        }
        end = offset + idx + ch.len_utf8();
    }

    (start, end)
}

fn char_boundary(text: &str, idx: usize) -> usize {
    let mut i = idx.min(text.len());
    while i > 0 && !text.is_char_boundary(i) {
        i -= 1;
    }
    i
}

#[cfg(feature = "profile-telemetry")]
pub mod telemetry {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    static PARAGRAPH_MATERIALIZE_COUNT: AtomicUsize = AtomicUsize::new(0);
    static CACHE_HIT_COUNT: AtomicUsize = AtomicUsize::new(0);
    static CACHE_MISS_COUNT: AtomicUsize = AtomicUsize::new(0);

    #[inline]
    pub fn record_materialize() -> usize {
        PARAGRAPH_MATERIALIZE_COUNT.fetch_add(1, Ordering::Relaxed) + 1
    }

    #[inline]
    pub fn record_layout_hit(duration: Duration, spans_count: usize, text_len: usize) {
        let hits = CACHE_HIT_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
        println!(
            "[kglance::telemetry] SelectableText layout CACHE HIT ({duration:?}) | spans: {spans_count}, text_len: {text_len} | hits: {hits}"
        );
    }

    #[inline]
    pub fn record_layout_miss(
        duration: Duration,
        spans_count: usize,
        text_len: usize,
        mat_count: usize,
    ) {
        let misses = CACHE_MISS_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
        println!(
            "[kglance::telemetry] SelectableText layout CACHE MISS ({duration:?}) | spans: {spans_count}, text_len: {text_len} | materialize #{mat_count} | misses: {misses}"
        );
    }

    #[must_use]
    pub fn stats() -> (usize, usize, usize) {
        (
            CACHE_HIT_COUNT.load(Ordering::Relaxed),
            CACHE_MISS_COUNT.load(Ordering::Relaxed),
            PARAGRAPH_MATERIALIZE_COUNT.load(Ordering::Relaxed),
        )
    }

    pub fn reset() {
        CACHE_HIT_COUNT.store(0, Ordering::Relaxed);
        CACHE_MISS_COUNT.store(0, Ordering::Relaxed);
        PARAGRAPH_MATERIALIZE_COUNT.store(0, Ordering::Relaxed);
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for SelectableText<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::text::Renderer<
            Font = Font,
            Paragraph = iced::advanced::graphics::text::Paragraph,
        >,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Shrink,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        #[cfg(feature = "profile-telemetry")]
        let t_start = std::time::Instant::now();

        let state = tree.state.downcast_mut::<State>();
        let max_width = limits.max().width;
        let top_pad = self.top_padding();

        let spans_hash = {
            let mut hasher = rustc_hash::FxHasher::default();
            hasher.write_u64(super::paragraph_cache::hash_spans(&self.spans));
            for r in &self.rubies {
                hasher.write_usize(r.start);
                hasher.write_usize(r.end);
                hasher.write(r.ruby.as_bytes());
            }
            hasher.finish()
        };

        let is_cached = state.paragraph.is_some()
            && state.last_spans_hash == spans_hash
            && (state.last_bounds_width - max_width).abs() < 0.1
            && (state.last_font_size - self.font_size).abs() < 0.1;

        let size = if is_cached {
            #[cfg(feature = "profile-telemetry")]
            telemetry::record_layout_hit(
                t_start.elapsed(),
                self.spans.len(),
                state.plain_text.len(),
            );

            state.last_size
        } else {
            let align_bits = match self.align_x {
                alignment::Horizontal::Left => 0,
                alignment::Horizontal::Center => 1,
                alignment::Horizontal::Right => 2,
            };

            let key = super::paragraph_cache::ParagraphKey {
                spans_hash,
                font_size_bits: self.font_size.to_bits(),
                max_width_bits: (max_width * 2.0).round() as u32,
                align_bits,
            };

            if let Some(cached) = super::paragraph_cache::get_cached_paragraph(&key) {
                #[cfg(feature = "profile-telemetry")]
                telemetry::record_layout_hit(
                    t_start.elapsed(),
                    self.spans.len(),
                    cached.plain_text.len(),
                );

                state.paragraph = Some(cached.paragraph);
                state.plain_text = cached.plain_text;
                state.line_offsets = cached.line_offsets;
                state.last_bounds_width = max_width;
                state.last_font_size = self.font_size;
                state.last_spans_hash = spans_hash;
                state.last_size = cached.size;
                cached.size
            } else {
                state.plain_text.clear();
                for span in &self.spans {
                    state.plain_text.push_str(&span.text);
                }

                let line_height_val = if self.rubies.is_empty() { 1.5 } else { 1.85 };
                let text_input = Text {
                    content: &self.spans[..],
                    bounds: Size::new(max_width, f32::INFINITY),
                    size: Pixels(self.font_size),
                    line_height: iced::advanced::text::LineHeight::Relative(line_height_val),
                    font: Font::default(),
                    align_x: self.align_x.into(),
                    align_y: alignment::Vertical::Top,
                    shaping: iced::advanced::text::Shaping::Advanced,
                    wrapping: iced::advanced::text::Wrapping::Word,
                };

                #[cfg(feature = "profile-telemetry")]
                let mat_count = telemetry::record_materialize();

                let p = Renderer::Paragraph::with_spans(text_input);
                let p_size = p.min_bounds();
                let line_offsets = compute_line_starts(p.buffer());

                super::paragraph_cache::insert_cached_paragraph(
                    key,
                    super::paragraph_cache::CachedParagraph {
                        paragraph: p.clone(),
                        plain_text: state.plain_text.clone(),
                        line_offsets: line_offsets.clone(),
                        size: p_size,
                    },
                );

                state.paragraph = Some(p);
                state.line_offsets = line_offsets;
                state.last_bounds_width = max_width;
                state.last_font_size = self.font_size;
                state.last_spans_hash = spans_hash;
                state.last_size = p_size;

                #[cfg(feature = "profile-telemetry")]
                telemetry::record_layout_miss(
                    t_start.elapsed(),
                    self.spans.len(),
                    state.plain_text.len(),
                    mat_count,
                );

                p_size
            }
        };

        let mut final_size = limits.resolve(self.width, Length::Shrink, size);
        final_size.height += top_pad;
        layout::Node::new(final_size)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();

        let Some(paragraph) = &state.paragraph else {
            return;
        };

        let draw_anchor = bounds.anchor(
            paragraph.min_bounds(),
            self.align_x,
            alignment::Vertical::Top,
        );
        let offset_x = (draw_anchor.x - bounds.x).max(0.0);

        if self.block_index.is_some() && self.selection_range.is_none() && !self.drag_active {
            state.selection = None;
            state.drag_start = None;
            state.is_selecting = false;
            state.is_mouse_held = false;
        }

        if shell.is_event_captured() {
            return;
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                state.hovered_tooltip = None;
                if let Some(cursor_pos) = cursor.position_over(bounds) {
                    state.is_mouse_held = true;
                    state.press_pos = Some(cursor_pos);
                    state.is_drag = false;
                    if state.line_offsets.is_empty() {
                        state.line_offsets = compute_line_starts(paragraph.buffer());
                    }
                    let offset = point_to_offset(
                        paragraph.buffer(),
                        &state.line_offsets,
                        &state.plain_text,
                        bounds,
                        cursor_pos,
                        self.top_padding(),
                        offset_x,
                    );
                    state.pressed_link_url = self
                        .links
                        .iter()
                        .find(|l| offset >= l.start && offset < l.end)
                        .map(|l| l.url.clone());
                    let new_click =
                        mouse::Click::new(cursor_pos, mouse::Button::Left, state.last_click);

                    match new_click.kind() {
                        click::Kind::Double => {
                            state.pressed_link_url = None;
                            let (start, end) = expand_to_word_bounds(&state.plain_text, offset);
                            state.selection = Some((start, end));
                            state.is_selecting = false;
                            state.is_mouse_held = false;
                            if let (Some(blk), Some(on_start), Some(on_update)) =
                                (self.block_index, &self.on_drag_start, &self.on_drag_update)
                            {
                                shell.publish(on_start(blk, start));
                                shell.publish(on_update(blk, end));
                                if let Some(on_end) = &self.on_drag_end {
                                    shell.publish(on_end());
                                }
                            } else if let Some(on_change) = &self.on_selection_change {
                                let start = char_boundary(&state.plain_text, start);
                                let end = char_boundary(&state.plain_text, end);
                                let selected_str = if start < end && end <= state.plain_text.len() {
                                    Some(state.plain_text[start..end].to_string())
                                } else {
                                    None
                                };
                                shell.publish(on_change(selected_str));
                                if let Some(on_end) = &self.on_drag_end {
                                    shell.publish(on_end());
                                }
                                shell.request_redraw();
                            }
                        }
                        click::Kind::Triple => {
                            state.pressed_link_url = None;
                            state.selection = Some((0, state.plain_text.len()));
                            state.is_selecting = false;
                            state.is_mouse_held = false;
                            if let (Some(blk), Some(on_start), Some(on_update)) =
                                (self.block_index, &self.on_drag_start, &self.on_drag_update)
                            {
                                shell.publish(on_start(blk, 0));
                                shell.publish(on_update(blk, state.plain_text.len()));
                                if let Some(on_end) = &self.on_drag_end {
                                    shell.publish(on_end());
                                }
                            } else if let Some(on_change) = &self.on_selection_change {
                                let selected_str = if !state.plain_text.is_empty() {
                                    Some(state.plain_text.clone())
                                } else {
                                    None
                                };
                                shell.publish(on_change(selected_str));
                                if let Some(on_end) = &self.on_drag_end {
                                    shell.publish(on_end());
                                }
                                shell.request_redraw();
                            }
                        }
                        click::Kind::Single => {
                            state.is_selecting = true;
                            state.drag_start = Some(offset);
                            state.selection = Some((offset, offset));
                            if let (Some(blk), Some(on_start)) =
                                (self.block_index, &self.on_drag_start)
                            {
                                shell.publish(on_start(blk, offset));
                            } else {
                                shell.request_redraw();
                            }
                        }
                    }

                    state.last_click = Some(new_click);
                    shell.capture_event();
                } else {
                    let had_selection = state.selection.is_some();
                    state.selection = None;
                    state.is_selecting = false;
                    state.is_mouse_held = false;
                    state.press_pos = None;
                    state.pressed_link_url = None;
                    if had_selection {
                        shell.request_redraw();
                        if let Some(on_clear) = &self.on_clear_selection {
                            shell.publish(on_clear());
                        }
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let dragging = state.is_mouse_held || state.is_selecting || self.drag_active;
                if !dragging {
                    let mut new_hovered: Option<HoveredTooltip> = None;
                    if let Some(cursor_pos) = cursor.position_over(bounds) {
                        if state.line_offsets.is_empty() {
                            state.line_offsets = compute_line_starts(paragraph.buffer());
                        }
                        let offset = point_to_offset(
                            paragraph.buffer(),
                            &state.line_offsets,
                            &state.plain_text,
                            bounds,
                            cursor_pos,
                            self.top_padding(),
                            offset_x,
                        );

                        let matched_tooltip = self
                            .tooltips
                            .iter()
                            .find(|t| offset >= t.start && offset < t.end)
                            .map(|t| (t.start, t.end, t.content.clone()))
                            .or_else(|| {
                                self.links
                                    .iter()
                                    .find(|l| offset >= l.start && offset < l.end)
                                    .map(|l| (l.start, l.end, l.url.clone()))
                            });

                        if let Some((start, end, content)) = matched_tooltip {
                            let target_rect = compute_inline_rect(
                                paragraph.buffer(),
                                &state.line_offsets,
                                bounds,
                                self.top_padding(),
                                start,
                                end,
                                offset_x,
                            )
                            .unwrap_or(Rectangle {
                                x: cursor_pos.x,
                                y: cursor_pos.y,
                                width: 1.0,
                                height: self.font_size,
                            });

                            new_hovered = Some(HoveredTooltip {
                                content,
                                target_rect,
                            });
                        }
                    }

                    if state.hovered_tooltip != new_hovered {
                        state.hovered_tooltip = new_hovered;
                        shell.request_redraw();
                    }
                    return;
                }
                state.hovered_tooltip = None;
                if let Some(cursor_pos) = cursor.position() {
                    if state.is_mouse_held
                        && let Some(press_pos) = state.press_pos
                    {
                        let dist_sq = (cursor_pos.x - press_pos.x).powi(2)
                            + (cursor_pos.y - press_pos.y).powi(2);
                        if dist_sq > 16.0 {
                            state.is_drag = true;
                            state.pressed_link_url = None;
                        }
                    }

                    let is_over = cursor.position_over(bounds).is_some();
                    let is_in_y_range =
                        cursor_pos.y >= bounds.y && cursor_pos.y <= (bounds.y + bounds.height);
                    let is_in_x_range =
                        cursor_pos.x >= bounds.x && cursor_pos.x <= (bounds.x + bounds.width);

                    let should_process = is_over
                        || (self.drag_active
                            && is_in_y_range
                            && (self.width == Length::Fill || is_in_x_range));

                    if should_process {
                        if state.line_offsets.is_empty() {
                            state.line_offsets = compute_line_starts(paragraph.buffer());
                        }
                        let offset = point_to_offset(
                            paragraph.buffer(),
                            &state.line_offsets,
                            &state.plain_text,
                            bounds,
                            cursor_pos,
                            self.top_padding(),
                            offset_x,
                        );

                        if state.is_selecting {
                            let old_sel = state.selection;
                            if let Some(drag_start) = state.drag_start {
                                let start = drag_start.min(offset);
                                let end = drag_start.max(offset);
                                state.selection = Some((start, end));
                            } else {
                                state.drag_start = Some(offset);
                                state.selection = Some((offset, offset));
                            }
                            if state.selection != old_sel && self.block_index.is_none() {
                                shell.request_redraw();
                            }
                        } else if is_over
                            && (state.is_mouse_held
                                || (self.drag_active && self.selection_range.is_none()))
                        {
                            state.is_selecting = true;
                            state.drag_start = Some(offset);
                            state.selection = Some((offset, offset));
                            if let (Some(blk), Some(on_start)) =
                                (self.block_index, &self.on_drag_start)
                            {
                                shell.publish(on_start(blk, offset));
                            } else {
                                shell.request_redraw();
                            }
                        }

                        if state.is_selecting || self.drag_active {
                            if let (Some(blk), Some(on_update)) =
                                (self.block_index, &self.on_drag_update)
                            {
                                shell.publish(on_update(blk, offset));
                            }
                            shell.capture_event();
                        }
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let was_link_click = if !state.is_drag {
                    state.pressed_link_url.take()
                } else {
                    None
                };

                let was_selecting = state.is_selecting || state.is_mouse_held;
                state.is_mouse_held = false;
                state.is_selecting = false;
                state.press_pos = None;
                state.pressed_link_url = None;

                if let Some(url) = was_link_click {
                    state.selection = None;
                    if let Some(on_link) = &self.on_link_click {
                        shell.publish(on_link(url));
                    }
                    if let Some(on_clear) = &self.on_clear_selection {
                        shell.publish(on_clear());
                    }
                    shell.capture_event();
                    return;
                }

                if was_selecting || self.drag_active {
                    if self.block_index.is_none()
                        && let (Some(selection), Some(on_change)) =
                            (state.selection, &self.on_selection_change)
                    {
                        let (start, end) = selection;
                        let start = char_boundary(&state.plain_text, start);
                        let end = char_boundary(&state.plain_text, end);
                        let selected_str = if start < end && end <= state.plain_text.len() {
                            Some(state.plain_text[start..end].to_string())
                        } else {
                            None
                        };
                        shell.publish(on_change(selected_str));
                    }
                    if was_selecting && let Some(on_end) = &self.on_drag_end {
                        shell.publish(on_end());
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorLeft) if state.hovered_tooltip.is_some() => {
                state.hovered_tooltip = None;
                shell.request_redraw();
            }
            _ => {}
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();

        let Some(paragraph) = &state.paragraph else {
            return;
        };

        let top_pad = self.top_padding();
        let draw_anchor = bounds.anchor(
            paragraph.min_bounds(),
            self.align_x,
            alignment::Vertical::Top,
        );
        let offset_x = (draw_anchor.x - bounds.x).max(0.0);

        let effective_selection = if let Some(current_blk) = self.block_index {
            if let Some(range) = self.selection_range {
                let (start_pt, end_pt) = if (range.start.block, range.start.offset)
                    <= (range.end.block, range.end.offset)
                {
                    (range.start, range.end)
                } else {
                    (range.end, range.start)
                };

                if current_blk < start_pt.block || current_blk > end_pt.block {
                    None
                } else if current_blk > start_pt.block && current_blk < end_pt.block {
                    Some((0, state.plain_text.len()))
                } else if current_blk == start_pt.block && current_blk == end_pt.block {
                    Some((
                        start_pt.offset.min(state.plain_text.len()),
                        end_pt.offset.min(state.plain_text.len()),
                    ))
                } else if current_blk == start_pt.block {
                    Some((
                        start_pt.offset.min(state.plain_text.len()),
                        state.plain_text.len(),
                    ))
                } else {
                    Some((0, end_pt.offset.min(state.plain_text.len())))
                }
            } else if state.is_selecting {
                state.selection
            } else {
                None
            }
        } else {
            state.selection
        };

        if let Some((start, end)) = effective_selection
            && start < end
        {
            let buffer = paragraph.buffer();
            let fallback_offsets;
            let line_offsets: &[usize] = if state.line_offsets.is_empty() {
                fallback_offsets = compute_line_starts(buffer);
                &fallback_offsets
            } else {
                &state.line_offsets
            };

            for run in buffer.layout_runs() {
                let line_top = bounds.y + top_pad + run.line_top;
                let line_height = run.line_height;

                let line_start = line_offsets.get(run.line_i).copied().unwrap_or(0);
                let line_text_len = run.text.len();
                let line_end = line_start + line_text_len;

                if line_start >= end || line_end <= start {
                    if line_text_len == 0 && start <= line_start && end > line_start {
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: Rectangle {
                                    x: bounds.x,
                                    y: line_top,
                                    width: (self.font_size * 0.4).max(6.0),
                                    height: line_height,
                                },
                                border: Border::default(),
                                shadow: Shadow::default(),
                                snap: false,
                            },
                            Background::Color(self.selection_color),
                        );
                    }
                    continue;
                }

                let sel_start_in_line = start.saturating_sub(line_start).min(line_text_len);
                let sel_end_in_line = end.saturating_sub(line_start).min(line_text_len);

                if sel_start_in_line >= sel_end_in_line {
                    continue;
                }

                let mut text_min_x: Option<f32> = None;
                let mut text_max_x: Option<f32> = None;

                for glyph in run.glyphs {
                    if glyph.end > sel_start_in_line && glyph.start < sel_end_in_line {
                        let x = bounds.x + offset_x + glyph.x;
                        let x_end = x + glyph.w;
                        text_min_x = Some(text_min_x.map_or(x, |m: f32| m.min(x)));
                        text_max_x = Some(text_max_x.map_or(x_end, |m: f32| m.max(x_end)));
                    }
                }

                if let (Some(x1), Some(x2)) = (text_min_x, text_max_x) {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: Rectangle {
                                x: x1,
                                y: line_top,
                                width: (x2 - x1).max(0.0),
                                height: line_height,
                            },
                            border: Border::default(),
                            shadow: Shadow::default(),
                            snap: true,
                        },
                        Background::Color(self.selection_color),
                    );
                }
            }
        }

        renderer.fill_paragraph(
            paragraph,
            Point::new(bounds.x + offset_x, bounds.y + top_pad),
            self.default_text_color,
            bounds,
        );

        if !self.rubies.is_empty() {
            let buffer = paragraph.buffer();
            let fallback_offsets;
            let line_offsets: &[usize] = if state.line_offsets.is_empty() {
                fallback_offsets = compute_line_starts(buffer);
                &fallback_offsets
            } else {
                &state.line_offsets
            };

            let rt_size = ruby_font_size(self.font_size);
            let rt_box_h = ruby_box_height(self.font_size);
            let ruby_color = Color {
                a: self.default_text_color.a * 0.85,
                ..self.default_text_color
            };
            let ruby_font = self
                .spans
                .first()
                .and_then(|s| s.font)
                .unwrap_or(Font::DEFAULT);

            for ruby in &self.rubies {
                for run in buffer.layout_runs() {
                    let line_start = line_offsets.get(run.line_i).copied().unwrap_or(0);
                    let line_text_len = run.text.len();
                    let line_end = line_start + line_text_len;

                    if line_start >= ruby.end || line_end <= ruby.start {
                        continue;
                    }

                    let r_start_in_line = ruby.start.saturating_sub(line_start).min(line_text_len);
                    let r_end_in_line = ruby.end.saturating_sub(line_start).min(line_text_len);
                    if r_start_in_line >= r_end_in_line {
                        continue;
                    }

                    let mut k_min_x: Option<f32> = None;
                    let mut k_max_x: Option<f32> = None;

                    for glyph in run.glyphs {
                        if glyph.end > r_start_in_line && glyph.start < r_end_in_line {
                            let x = bounds.x + offset_x + glyph.x;
                            let x_end = x + glyph.w;
                            k_min_x = Some(k_min_x.map_or(x, |m: f32| m.min(x)));
                            k_max_x = Some(k_max_x.map_or(x_end, |m: f32| m.max(x_end)));
                        }
                    }
                    let kanji_top = bounds.y + top_pad + run.line_y - (self.font_size * 0.86);

                    if let (Some(min_x), Some(max_x)) = (k_min_x, k_max_x) {
                        let center_x = (min_x + max_x) * 0.5;
                        let box_w = (max_x - min_x)
                            .max(rt_size * ruby.ruby.chars().count() as f32 * 1.5)
                            .max(40.0);
                        // let rt_y = if run_idx == 0 {
                        //     bounds.y
                        // } else {
                        //     bounds.y + top_pad + run.line_top - rt_box_h - 1.0
                        // };
                        let rt_y = kanji_top - rt_box_h - 1.5;

                        renderer.fill_text(
                            Text {
                                content: ruby.ruby.clone(),
                                bounds: Size::new(box_w, rt_box_h),
                                size: Pixels(rt_size),
                                line_height: iced::widget::text::LineHeight::Relative(1.0),
                                font: ruby_font,
                                align_x: iced::alignment::Horizontal::Center.into(),
                                align_y: iced::alignment::Vertical::Top,
                                shaping: iced::widget::text::Shaping::Advanced,
                                wrapping: iced::widget::text::Wrapping::None,
                            },
                            Point::new(center_x, rt_y),
                            ruby_color,
                            *viewport,
                        );
                    }
                }
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();
        if let Some(cursor_pos) = cursor.position_over(bounds) {
            let state = tree.state.downcast_ref::<State>();
            if let Some(paragraph) = &state.paragraph {
                let draw_anchor = bounds.anchor(
                    paragraph.min_bounds(),
                    self.align_x,
                    alignment::Vertical::Top,
                );
                let offset_x = (draw_anchor.x - bounds.x).max(0.0);
                let fallback_offsets;
                let line_offsets: &[usize] = if state.line_offsets.is_empty() {
                    fallback_offsets = compute_line_starts(paragraph.buffer());
                    &fallback_offsets
                } else {
                    &state.line_offsets
                };
                let offset = point_to_offset(
                    paragraph.buffer(),
                    line_offsets,
                    &state.plain_text,
                    bounds,
                    cursor_pos,
                    self.top_padding(),
                    offset_x,
                );
                if self
                    .links
                    .iter()
                    .any(|l| offset >= l.start && offset < l.end)
                {
                    return mouse::Interaction::Pointer;
                }
                if self
                    .tooltips
                    .iter()
                    .any(|t| offset >= t.start && offset < t.end)
                {
                    return mouse::Interaction::Pointer;
                }
            }
            mouse::Interaction::Text
        } else {
            mouse::Interaction::default()
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        _layout: Layout<'b>,
        _renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        let state = tree.state.downcast_ref::<State>();
        let hovered = state.hovered_tooltip.as_ref()?;

        let mut target_rect = hovered.target_rect;
        target_rect.x += translation.x;
        target_rect.y += translation.y;

        let theme = self.app_theme.unwrap_or(AppTheme::Dark);

        Some(create_tooltip_overlay(
            hovered.content.clone(),
            target_rect,
            *viewport,
            (self.font_size * 0.85).clamp(11.0, 13.0),
            theme,
        ))
    }
}

impl<'a, Message, Theme, Renderer> From<SelectableText<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::text::Renderer<
            Font = Font,
            Paragraph = iced::advanced::graphics::text::Paragraph,
        > + 'a,
{
    fn from(selectable_text: SelectableText<'a, Message, Theme, Renderer>) -> Self {
        Element::new(selectable_text)
    }
}

#[cfg(test)]
#[path = "selectable_text_tests.rs"]
mod tests;
