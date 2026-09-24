use iced::advanced::graphics::core::event::Event;
use iced::advanced::graphics::core::layout::{self, Layout};
use iced::advanced::graphics::core::mouse::{self, click};
use iced::advanced::graphics::core::renderer;
use iced::advanced::graphics::core::widget::{Tree, Widget, tree};
use iced::advanced::graphics::core::{Clipboard, Element, Shell};
use iced::advanced::text::{Paragraph, Text};
use iced::widget::text::Span;
use iced::{
    Background, Border, Color, Font, Length, Pixels, Point, Rectangle, Shadow, Size, alignment,
};

pub struct SelectableText<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Renderer: iced::advanced::text::Renderer<Paragraph = iced::advanced::graphics::text::Paragraph>,
    Renderer::Font: From<Font>,
{
    spans: Vec<Span<'a, (), Font>>,
    font_size: f32,
    default_text_color: Color,
    selection_color: Color,
    block_index: Option<usize>,
    selection_range: Option<crate::core::SelectionRange>,
    on_selection_change: Option<Box<dyn Fn(Option<String>) -> Message + 'a>>,
    on_drag_start: Option<Box<dyn Fn(usize, usize) -> Message + 'a>>,
    on_drag_update: Option<Box<dyn Fn(usize, usize) -> Message + 'a>>,
    on_drag_end: Option<Box<dyn Fn() -> Message + 'a>>,
    on_clear_selection: Option<Box<dyn Fn() -> Message + 'a>>,
    drag_active: bool,
    width: Length,
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
            font_size,
            default_text_color: Color::BLACK,
            selection_color: Color::from_rgba(0.2, 0.4, 0.8, 0.3),
            block_index: None,
            selection_range: None,
            on_selection_change: None,
            on_drag_start: None,
            on_drag_update: None,
            on_drag_end: None,
            on_clear_selection: None,
            drag_active: false,
            width: Length::Fill,
            _phantom: std::marker::PhantomData,
        }
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
    last_size: Size,
    selection: Option<(usize, usize)>,
    is_selecting: bool,

    is_mouse_held: bool,
    drag_start: Option<usize>,
    last_click: Option<mouse::Click>,
    plain_text: String,
}

fn expand_to_word_bounds(text: &str, offset: usize) -> (usize, usize) {
    if text.is_empty() {
        return (0, 0);
    }
    let offset = offset.min(text.len());

    if offset < text.len() {
        let b = text.as_bytes()[offset];
        if b.is_ascii_whitespace() || b.is_ascii_punctuation() {
            return (offset, offset);
        }
    }

    if offset == text.len() {
        let b = text.as_bytes()[offset - 1];
        if b.is_ascii_whitespace() || b.is_ascii_punctuation() {
            return (offset, offset);
        }
    }

    let bytes = text.as_bytes();

    let mut start = offset;
    while start > 0 {
        let prev = start - 1;
        if bytes[prev].is_ascii_whitespace() || bytes[prev].is_ascii_punctuation() {
            break;
        }
        start = prev;
    }

    let mut end = offset;
    while end < text.len() {
        if bytes[end].is_ascii_whitespace() || bytes[end].is_ascii_punctuation() {
            break;
        }
        end += 1;
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

        let mut spans_changed = false;
        let mut total_len = 0;
        for span in &self.spans {
            total_len += span.text.len();
        }
        if total_len != state.plain_text.len() {
            spans_changed = true;
        } else {
            let mut offset = 0;
            for span in &self.spans {
                let end = offset + span.text.len();
                if state.plain_text.get(offset..end) != Some(&span.text) {
                    spans_changed = true;
                    break;
                }
                offset = end;
            }
        }

        let is_cached = !spans_changed
            && state.paragraph.is_some()
            && (state.last_bounds_width - max_width).abs() < 0.1
            && (state.last_font_size - self.font_size).abs() < 0.1;

        let size = if is_cached {
            #[cfg(feature = "profile-telemetry")]
            telemetry::record_layout_hit(t_start.elapsed(), self.spans.len(), total_len);

            state.last_size
        } else {
            let spans_hash = super::paragraph_cache::hash_spans(&self.spans);
            let key = super::paragraph_cache::ParagraphKey {
                spans_hash,
                font_size_bits: self.font_size.to_bits(),
                max_width_bits: (max_width * 2.0).round() as u32,
            };

            if let Some(cached) = super::paragraph_cache::get_cached_paragraph(&key) {
                #[cfg(feature = "profile-telemetry")]
                telemetry::record_layout_hit(t_start.elapsed(), self.spans.len(), total_len);

                state.paragraph = Some(cached.paragraph);
                state.plain_text = cached.plain_text;
                state.last_bounds_width = max_width;
                state.last_font_size = self.font_size;
                state.last_size = cached.size;
                cached.size
            } else {
                state.plain_text.clear();
                for span in &self.spans {
                    state.plain_text.push_str(&span.text);
                }

                let text_input = Text {
                    content: &self.spans[..],
                    bounds: Size::new(max_width, f32::INFINITY),
                    size: Pixels(self.font_size),
                    line_height: iced::advanced::text::LineHeight::default(),
                    font: Font::default(),
                    align_x: alignment::Horizontal::Left.into(),
                    align_y: alignment::Vertical::Top,
                    shaping: iced::advanced::text::Shaping::Advanced,
                    wrapping: iced::advanced::text::Wrapping::Word,
                };

                #[cfg(feature = "profile-telemetry")]
                let mat_count = telemetry::record_materialize();

                let p = Renderer::Paragraph::with_spans(text_input);
                let p_size = p.min_bounds();

                super::paragraph_cache::insert_cached_paragraph(
                    key,
                    super::paragraph_cache::CachedParagraph {
                        paragraph: p.clone(),
                        plain_text: state.plain_text.clone(),
                        size: p_size,
                    },
                );

                state.paragraph = Some(p);
                state.last_bounds_width = max_width;
                state.last_font_size = self.font_size;
                state.last_size = p_size;

                #[cfg(feature = "profile-telemetry")]
                telemetry::record_layout_miss(
                    t_start.elapsed(),
                    self.spans.len(),
                    total_len,
                    mat_count,
                );

                p_size
            }
        };

        layout::Node::new(limits.resolve(self.width, Length::Shrink, size))
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

        if self.selection_range.is_none() && !state.is_selecting && !state.is_mouse_held {
            state.selection = None;
            state.drag_start = None;
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(cursor_pos) = cursor.position_over(bounds) {
                    state.is_mouse_held = true;
                    let rel_pos = Point::new(cursor_pos.x - bounds.x, cursor_pos.y - bounds.y);
                    if let Some(hit) = paragraph.hit_test(rel_pos) {
                        let offset = hit.cursor();
                        let new_click =
                            mouse::Click::new(cursor_pos, mouse::Button::Left, state.last_click);

                        match new_click.kind() {
                            click::Kind::Double => {
                                let (start, end) = expand_to_word_bounds(&state.plain_text, offset);
                                state.selection = Some((start, end));
                                state.is_selecting = false;
                                if let (Some(blk), Some(on_start), Some(on_update)) =
                                    (self.block_index, &self.on_drag_start, &self.on_drag_update)
                                {
                                    shell.publish(on_start(blk, start));
                                    shell.publish(on_update(blk, end));
                                }
                            }
                            click::Kind::Triple => {
                                state.selection = Some((0, state.plain_text.len()));
                                state.is_selecting = false;
                                if let (Some(blk), Some(on_start), Some(on_update)) =
                                    (self.block_index, &self.on_drag_start, &self.on_drag_update)
                                {
                                    shell.publish(on_start(blk, 0));
                                    shell.publish(on_update(blk, state.plain_text.len()));
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
                                }
                            }
                        }

                        state.last_click = Some(new_click);
                        shell.capture_event();
                    }
                } else {
                    state.selection = None;
                    state.is_selecting = false;
                    state.is_mouse_held = false;
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let dragging = state.is_mouse_held || state.is_selecting || self.drag_active;
                if !dragging {
                    return;
                }
                if let Some(cursor_pos) = cursor.position() {
                    let is_over = cursor.position_over(bounds).is_some();
                    let is_in_y_range =
                        cursor_pos.y >= bounds.y && cursor_pos.y <= (bounds.y + bounds.height);

                    if is_over || is_in_y_range {
                        let clamped_x = cursor_pos.x.clamp(bounds.x, bounds.x + bounds.width);
                        let clamped_y = cursor_pos.y.clamp(bounds.y, bounds.y + bounds.height);
                        let rel_pos = Point::new(clamped_x - bounds.x, clamped_y - bounds.y);

                        if let Some(hit) = paragraph.hit_test(rel_pos) {
                            let mut offset = hit.cursor();
                            // If cursor is to the right of the line/bounds, select to end
                            if cursor_pos.x > bounds.x + bounds.width {
                                offset = state.plain_text.len();
                            } else if cursor_pos.x < bounds.x {
                                offset = 0;
                            }

                            if state.is_selecting {
                                if let Some(drag_start) = state.drag_start {
                                    let start = drag_start.min(offset);
                                    let end = drag_start.max(offset);
                                    state.selection = Some((start, end));
                                } else {
                                    state.drag_start = Some(offset);
                                    state.selection = Some((offset, offset));
                                }
                            } else if state.is_mouse_held
                                || (self.drag_active && self.selection_range.is_none())
                            {
                                state.is_selecting = true;
                                state.drag_start = Some(offset);
                                state.selection = Some((offset, offset));
                                if let (Some(blk), Some(on_start)) =
                                    (self.block_index, &self.on_drag_start)
                                {
                                    shell.publish(on_start(blk, offset));
                                }
                            }

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
                let was_selecting = state.is_selecting || state.is_mouse_held;
                state.is_mouse_held = false;
                state.is_selecting = false;
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
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();

        let Some(paragraph) = &state.paragraph else {
            return;
        };

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
            let text = &state.plain_text;

            for run in buffer.layout_runs() {
                let line_top = bounds.y + run.line_top;
                let line_height = run.line_height;

                let mut text_min_x: Option<f32> = None;
                let mut text_max_x: Option<f32> = None;

                for glyph in run.glyphs {
                    if glyph.end > start && glyph.start < end {
                        let glyph_str = text.get(glyph.start..glyph.end).unwrap_or("");

                        if !glyph_str.chars().all(|c| c.is_whitespace()) {
                            let x = bounds.x + glyph.x;
                            let x_end = x + glyph.w;
                            text_min_x = Some(text_min_x.map_or(x, |m: f32| m.min(x)));
                            text_max_x = Some(text_max_x.map_or(x_end, |m: f32| m.max(x_end)));
                        }
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
                            snap: false,
                        },
                        Background::Color(self.selection_color),
                    );
                }
            }
        }

        renderer.fill_paragraph(
            paragraph,
            bounds.position(),
            self.default_text_color,
            bounds,
        );
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
