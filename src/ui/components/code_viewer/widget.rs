use crate::features::text::display_map::DisplayMap;
use crate::features::text::document::CodeDocument;
use crate::features::text::highlight::HighlightedSpan;
use crate::ui::components::code_viewer::selection::{SelectionRange, TextPosition};
use crate::ui::theme::AppTheme;
use iced::advanced::graphics::core::event::Event;
use iced::advanced::graphics::core::layout::{self, Layout};
use iced::advanced::graphics::core::mouse::{self, click};
use iced::advanced::graphics::core::renderer;
use iced::advanced::graphics::core::widget::{Tree, Widget, tree};
use iced::advanced::graphics::core::{Clipboard, Element, Shell};
use iced::advanced::text::Text;
use iced::keyboard::{self, Key};
use iced::{Background, Border, Color, Font, Length, Pixels, Point, Rectangle, Shadow, Size};

/// Internal interaction state of VirtualCodeViewer stored in the Widget Tree.
#[derive(Default)]
struct State {
    is_mouse_held: bool,
    is_selecting: bool,
    drag_start: Option<TextPosition>,
    last_click: Option<mouse::Click>,
}

type SelectCallback<'a, Message> = Box<dyn Fn(Option<SelectionRange>) -> Message + 'a>;
type DragStartCallback<'a, Message> = Box<dyn Fn(TextPosition) -> Message + 'a>;
type DragEndCallback<'a, Message> = Box<dyn Fn() -> Message + 'a>;
type AutoScrollCallback<'a, Message> = Box<dyn Fn(Option<f32>, Point) -> Message + 'a>;
type CopyCallback<'a, Message> = Box<dyn Fn(String) -> Message + 'a>;

/// Custom 1-Pass virtualized source code viewer widget for Kglance.
pub struct VirtualCodeViewer<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    doc: &'a CodeDocument,
    display_map: Option<&'a DisplayMap>,
    tokens: &'a [Vec<HighlightedSpan>],
    tokens_start_line: usize,
    font_size: f32,
    font: Font,
    theme: AppTheme,
    wrap: bool,
    selection: Option<SelectionRange>,
    search_query: &'a str,
    search_matches: &'a [(usize, usize)],
    search_match_index: usize,
    on_select: Option<SelectCallback<'a, Message>>,
    on_drag_start: Option<DragStartCallback<'a, Message>>,
    on_drag_end: Option<DragEndCallback<'a, Message>>,
    on_auto_scroll: Option<AutoScrollCallback<'a, Message>>,
    on_copy: Option<CopyCallback<'a, Message>>,
    _phantom: std::marker::PhantomData<(Theme, Renderer)>,
}

impl<'a, Message, Theme, Renderer> VirtualCodeViewer<'a, Message, Theme, Renderer> {
    pub fn new(doc: &'a CodeDocument, font_size: f32, font: Font, theme: AppTheme) -> Self {
        Self {
            doc,
            display_map: None,
            tokens: &[],
            tokens_start_line: 0,
            font_size,
            font,
            theme,
            wrap: false,
            selection: None,
            search_query: "",
            search_matches: &[],
            search_match_index: 0,
            on_select: None,
            on_drag_start: None,
            on_drag_end: None,
            on_auto_scroll: None,
            on_copy: None,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn display_map(mut self, display_map: &'a DisplayMap) -> Self {
        self.display_map = Some(display_map);
        self
    }

    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn tokens(mut self, tokens: &'a [Vec<HighlightedSpan>], start_line: usize) -> Self {
        self.tokens = tokens;
        self.tokens_start_line = start_line;
        self
    }

    pub fn selection(mut self, selection: Option<SelectionRange>) -> Self {
        self.selection = selection;
        self
    }

    pub fn search(
        mut self,
        query: &'a str,
        matches: &'a [(usize, usize)],
        current_idx: usize,
    ) -> Self {
        self.search_query = query;
        self.search_matches = matches;
        self.search_match_index = current_idx;
        self
    }

    pub fn on_select<F>(mut self, f: F) -> Self
    where
        F: Fn(Option<SelectionRange>) -> Message + 'a,
    {
        self.on_select = Some(Box::new(f));
        self
    }

    pub fn on_drag_start<F>(mut self, f: F) -> Self
    where
        F: Fn(TextPosition) -> Message + 'a,
    {
        self.on_drag_start = Some(Box::new(f));
        self
    }

    pub fn on_drag_end<F>(mut self, f: F) -> Self
    where
        F: Fn() -> Message + 'a,
    {
        self.on_drag_end = Some(Box::new(f));
        self
    }

    pub fn on_auto_scroll<F>(mut self, f: F) -> Self
    where
        F: Fn(Option<f32>, Point) -> Message + 'a,
    {
        self.on_auto_scroll = Some(Box::new(f));
        self
    }

    pub fn on_copy<F>(mut self, f: F) -> Self
    where
        F: Fn(String) -> Message + 'a,
    {
        self.on_copy = Some(Box::new(f));
        self
    }

    #[inline]
    fn line_height(&self) -> f32 {
        self.display_map
            .map(|d| d.line_height)
            .unwrap_or(self.font_size * 1.35)
    }

    #[inline]
    fn char_width(&self) -> f32 {
        self.display_map
            .map(|d| d.char_width)
            .unwrap_or(self.font_size * 0.60)
    }

    #[inline]
    fn gutter_width(&self) -> f32 {
        if let Some(d) = self.display_map {
            d.gutter_width
        } else {
            let digits = self.doc.max_digits();
            ((digits as f32) * self.char_width() + 28.0).max(44.0)
        }
    }

    #[inline]
    fn total_visual_rows(&self) -> usize {
        self.display_map
            .map(|d| d.total_visual_rows)
            .unwrap_or(self.doc.total_lines())
    }

    fn hit_test_position(&self, rel_pos: Point) -> TextPosition {
        crate::ui::components::code_viewer::selection::hit_test_position(
            self.doc,
            self.display_map,
            self.wrap,
            self.font_size,
            rel_pos,
        )
    }
}

impl<'a, Message: 'a, Theme, Renderer> Widget<Message, Theme, Renderer>
    for VirtualCodeViewer<'a, Message, Theme, Renderer>
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
            width: Length::Fill,
            height: Length::Shrink,
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let total_rows = self.total_visual_rows();
        let total_h = (total_rows as f32 * self.line_height()) + 40.0;
        let size = Size::new(limits.max().width, total_h);

        layout::Node::new(limits.resolve(Length::Fill, Length::Shrink, size))
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(cursor_pos) = cursor.position_over(bounds) {
                    state.is_mouse_held = true;
                    let rel_pos = Point::new(cursor_pos.x - bounds.x, cursor_pos.y - bounds.y);
                    let pos = self.hit_test_position(rel_pos);

                    let new_click =
                        mouse::Click::new(cursor_pos, mouse::Button::Left, state.last_click);

                    match new_click.kind() {
                        click::Kind::Double => {
                            let line = self.doc.get_line(pos.line);
                            let chars: Vec<char> = line.chars().collect();
                            let mut start_col = pos.col;
                            while start_col > 0
                                && chars
                                    .get(start_col - 1)
                                    .is_some_and(|c| c.is_alphanumeric() || *c == '_')
                            {
                                start_col -= 1;
                            }
                            let mut end_col = pos.col;
                            while end_col < chars.len()
                                && chars
                                    .get(end_col)
                                    .is_some_and(|c| c.is_alphanumeric() || *c == '_')
                            {
                                end_col += 1;
                            }

                            let sel = SelectionRange::new(
                                TextPosition::new(pos.line, start_col),
                                TextPosition::new(pos.line, end_col),
                            );
                            state.is_selecting = false;
                            if let Some(on_select) = &self.on_select {
                                shell.publish(on_select(Some(sel)));
                            }
                        }
                        click::Kind::Triple => {
                            let line = self.doc.get_line(pos.line);
                            let sel = SelectionRange::new(
                                TextPosition::new(pos.line, 0),
                                TextPosition::new(pos.line, line.chars().count()),
                            );
                            state.is_selecting = false;
                            if let Some(on_select) = &self.on_select {
                                shell.publish(on_select(Some(sel)));
                            }
                        }
                        click::Kind::Single => {
                            state.is_selecting = true;
                            state.drag_start = Some(pos);
                            let sel = SelectionRange::new(pos, pos);
                            if let Some(on_drag_start) = &self.on_drag_start {
                                shell.publish(on_drag_start(pos));
                            }
                            if let Some(on_select) = &self.on_select {
                                shell.publish(on_select(Some(sel)));
                            }
                        }
                    }

                    state.last_click = Some(new_click);
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if state.is_mouse_held
                    && state.is_selecting
                    && let Some(cursor_pos) = cursor.position()
                {
                    let rel_pos = Point::new(
                        (cursor_pos.x - bounds.x).max(0.0),
                        (cursor_pos.y - bounds.y).max(0.0),
                    );
                    let current_pos = self.hit_test_position(rel_pos);

                    if let Some(start_pos) = state.drag_start {
                        let sel = SelectionRange::new(start_pos, current_pos);
                        if let Some(on_select) = &self.on_select {
                            shell.publish(on_select(Some(sel)));
                        }
                    }

                    let overflow = if cursor_pos.y < viewport.y {
                        cursor_pos.y - viewport.y
                    } else if cursor_pos.y > viewport.y + viewport.height {
                        cursor_pos.y - (viewport.y + viewport.height)
                    } else {
                        0.0
                    };

                    if let Some(on_auto_scroll) = &self.on_auto_scroll {
                        let viewport_rel = Point::new(
                            (cursor_pos.x - viewport.x).max(0.0),
                            cursor_pos.y - viewport.y,
                        );
                        if overflow != 0.0 {
                            let direction = overflow.signum();
                            let speed = (overflow.abs() * 0.8).clamp(5.0, 40.0) * direction;
                            shell.publish(on_auto_scroll(Some(speed), viewport_rel));
                        } else {
                            shell.publish(on_auto_scroll(None, viewport_rel));
                        }
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.is_mouse_held = false;
                state.is_selecting = false;
                if let Some(on_drag_end) = &self.on_drag_end {
                    shell.publish(on_drag_end());
                }
                if let Some(on_auto_scroll) = &self.on_auto_scroll {
                    shell.publish(on_auto_scroll(None, Point::ORIGIN));
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                if (modifiers.control() || modifiers.command())
                    && let Key::Character(c) = key
                    && c.eq_ignore_ascii_case("c")
                    && let Some(sel) = self.selection
                    && !sel.is_empty()
                {
                    let (start, end) = sel.normalized();
                    let copied_text = self
                        .doc
                        .extract_range(start.line, start.col, end.line, end.col);
                    clipboard.write(
                        iced::advanced::clipboard::Kind::Standard,
                        copied_text.clone(),
                    );
                    if let Some(on_copy) = &self.on_copy {
                        shell.publish(on_copy(copied_text));
                    }
                }
            }
            _ => {}
        }
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let line_h = self.line_height();
        let gutter_w = self.gutter_width();
        let char_w = self.char_width();
        let total_lines = self.doc.total_lines();
        let total_vrows = self.total_visual_rows();

        if total_lines == 0 || bounds.height <= 0.0 || total_vrows == 0 {
            return;
        }

        // 1. Compute visible boundary in viewport (O(1))
        let relative_y = (viewport.y - bounds.y).max(0.0);
        let first_vrow = (relative_y / line_h).floor() as usize;
        let start_vrow = first_vrow.saturating_sub(4);
        let visible_count = (viewport.height / line_h).ceil() as usize;
        let end_vrow = (first_vrow + visible_count + 6).min(total_vrows);

        let gutter_x = viewport.x.max(bounds.x);
        let gutter_color = self.theme.palette().base.text_dim;
        let gutter_bg = self.theme.palette().base.surface;
        let gutter_border = self.theme.palette().base.border;
        let default_text_color = self.theme.palette().base.text;
        let selection_bg = self.theme.palette().overlay.selection_bg;
        let search_match_bg = self.theme.palette().markdown.search_inactive_bg;
        let active_match_bg = self.theme.palette().markdown.search_active_bg;
        let bold_font = Font {
            weight: iced::font::Weight::Bold,
            ..self.font
        };

        // Render pinned gutter background and separator line
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: gutter_x,
                    y: viewport.y,
                    width: gutter_w,
                    height: viewport.height,
                },
                border: Border::default(),
                shadow: Shadow::default(),
                snap: false,
            },
            Background::Color(gutter_bg),
        );

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: gutter_x + gutter_w - 1.0,
                    y: viewport.y,
                    width: 1.0,
                    height: viewport.height,
                },
                border: Border::default(),
                shadow: Shadow::default(),
                snap: false,
            },
            Background::Color(gutter_border),
        );

        let (start_line, _) = if let Some(d) = self.display_map {
            d.visual_row_to_line(start_vrow)
        } else {
            (start_vrow.min(total_lines.saturating_sub(1)), 0)
        };

        let (end_line, _) = if let Some(d) = self.display_map {
            d.visual_row_to_line(end_vrow.saturating_sub(1))
        } else {
            (end_vrow.min(total_lines.saturating_sub(1)), 0)
        };

        let max_cols = self
            .display_map
            .map(|d| d.max_cols_per_row())
            .unwrap_or(usize::MAX);
        let code_x = bounds.x + gutter_w;
        let mut num_buf = [0u8; 16];

        for line_idx in start_line..=end_line.min(total_lines.saturating_sub(1)) {
            let line_start_vrow = if let Some(d) = self.display_map {
                d.line_to_visual_row(line_idx)
            } else {
                line_idx
            };

            let line_text = self.doc.get_line(line_idx);
            let line_char_count = line_text.chars().count();
            let sel_cols = self
                .selection
                .and_then(|sel| sel.line_col_range(line_idx, line_char_count));
            let sel_bytes = sel_cols.and_then(|(start_col, end_col)| {
                if start_col < end_col {
                    let sb = char_to_byte_idx(line_text, start_col, line_char_count);
                    let eb = char_to_byte_idx(line_text, end_col, line_char_count);
                    if sb < eb { Some((sb, eb)) } else { None }
                } else {
                    None
                }
            });
            let sub_slices = if self.wrap {
                crate::features::text::display_map::wrap_line_to_slices(line_text, max_cols)
            } else {
                vec![crate::features::text::display_map::VisualSubSlice {
                    start_byte: 0,
                    end_byte: line_text.len(),
                    start_col: 0,
                    end_col: line_char_count,
                }]
            };

            for (sub_idx, sub_slice) in sub_slices.iter().enumerate() {
                let vrow = line_start_vrow + sub_idx;
                if vrow < start_vrow || vrow >= end_vrow {
                    continue;
                }

                let row_y = bounds.y + (vrow as f32 * line_h);

                // a. Draw Gutter (number ONLY for sub_idx == 0)
                if sub_idx == 0 {
                    let line_num = line_idx + 1;
                    let num_len = format_number_into_buffer(line_num, &mut num_buf);
                    if let Ok(num_str) = std::str::from_utf8(&num_buf[..num_len]) {
                        let num_x = gutter_x + (gutter_w - 14.0) - (num_len as f32 * char_w);
                        renderer.fill_text(
                            Text {
                                content: num_str.to_string(),
                                bounds: Size::new(gutter_w, line_h),
                                size: Pixels(self.font_size),
                                line_height: iced::widget::text::LineHeight::Relative(1.35),
                                font: self.font,
                                align_x: iced::alignment::Horizontal::Left.into(),
                                align_y: iced::alignment::Vertical::Top,
                                shaping: iced::widget::text::Shaping::Basic,
                                wrapping: iced::widget::text::Wrapping::None,
                            },
                            Point::new(num_x, row_y),
                            gutter_color,
                            *viewport,
                        );
                    }
                }

                // b. Draw selection background if line intersects selection range
                if let Some((start_col, end_col)) = sel_cols
                    && start_col < end_col
                {
                    let row_sel_start = start_col.max(sub_slice.start_col);
                    let row_sel_end = end_col.min(sub_slice.end_col);
                    if row_sel_start < row_sel_end {
                        let sel_offset_chars = row_sel_start - sub_slice.start_col;
                        let sel_x = code_x + (sel_offset_chars as f32 * char_w);
                        let sel_w = ((row_sel_end - row_sel_start) as f32 * char_w).max(4.0);
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: Rectangle {
                                    x: sel_x,
                                    y: row_y,
                                    width: sel_w,
                                    height: line_h,
                                },
                                border: Border::default(),
                                shadow: Shadow::default(),
                                snap: false,
                            },
                            Background::Color(selection_bg),
                        );
                    }
                }

                // c. Draw search highlight matches
                if !self.search_query.is_empty() && !self.search_matches.is_empty() {
                    let query_len = self.search_query.chars().count();
                    for (match_idx, &(m_line, m_col)) in self.search_matches.iter().enumerate() {
                        if m_line == line_idx + 1 {
                            let m_col_0 = m_col.saturating_sub(1);
                            let m_end_col = m_col_0 + query_len;
                            let row_hl_start = m_col_0.max(sub_slice.start_col);
                            let row_hl_end = m_end_col.min(sub_slice.end_col);
                            if row_hl_start < row_hl_end {
                                let hl_offset_chars = row_hl_start - sub_slice.start_col;
                                let hl_x = code_x + (hl_offset_chars as f32 * char_w);
                                let hl_w = ((row_hl_end - row_hl_start) as f32 * char_w).max(4.0);
                                let is_active = match_idx == self.search_match_index;
                                let bg = if is_active {
                                    active_match_bg
                                } else {
                                    search_match_bg
                                };

                                renderer.fill_quad(
                                    renderer::Quad {
                                        bounds: Rectangle {
                                            x: hl_x,
                                            y: row_y,
                                            width: hl_w,
                                            height: line_h,
                                        },
                                        border: Border {
                                            radius: 2.0.into(),
                                            width: if is_active { 1.0 } else { 0.0 },
                                            color: Color::WHITE,
                                        },
                                        shadow: Shadow::default(),
                                        snap: false,
                                    },
                                    Background::Color(bg),
                                );
                            }
                        }
                    }
                }

                // d. Draw text spans for sub-slice
                let slice_str = if sub_slice.end_byte <= line_text.len() {
                    &line_text[sub_slice.start_byte..sub_slice.end_byte]
                } else {
                    ""
                };

                let token_spans = if line_idx >= self.tokens_start_line {
                    self.tokens.get(line_idx - self.tokens_start_line)
                } else {
                    None
                };

                let draw_piece = |renderer: &mut Renderer,
                                  piece_start: usize,
                                  piece_end: usize,
                                  color: Color,
                                  is_bold: bool| {
                    if piece_start >= piece_end || piece_end > line_text.len() {
                        return;
                    }
                    let piece_str = &line_text[piece_start..piece_end];
                    let prefix = &line_text[sub_slice.start_byte..piece_start];
                    let offset_chars = prefix.chars().count();
                    let span_x = code_x + (offset_chars as f32 * char_w);
                    let font = if is_bold { bold_font } else { self.font };

                    renderer.fill_text(
                        Text {
                            content: piece_str.to_string(),
                            bounds: Size::new((bounds.width - gutter_w).max(100.0), line_h),
                            size: Pixels(self.font_size),
                            line_height: iced::widget::text::LineHeight::Relative(1.35),
                            font,
                            align_x: iced::alignment::Horizontal::Left.into(),
                            align_y: iced::alignment::Vertical::Top,
                            shaping: iced::widget::text::Shaping::Auto,
                            wrapping: iced::widget::text::Wrapping::None,
                        },
                        Point::new(span_x, row_y),
                        color,
                        *viewport,
                    );
                };

                let draw_span_with_selection =
                    |renderer: &mut Renderer, s_start: usize, s_end: usize, color: Color| {
                        if let Some((sel_sb, sel_eb)) = sel_bytes {
                            let row_sel_sb = sel_sb.max(sub_slice.start_byte);
                            let row_sel_eb = sel_eb.min(sub_slice.end_byte);
                            if row_sel_sb < row_sel_eb {
                                if s_start < row_sel_sb {
                                    let p_end = s_end.min(row_sel_sb);
                                    draw_piece(renderer, s_start, p_end, color, false);
                                }
                                let b_start = s_start.max(row_sel_sb);
                                let b_end = s_end.min(row_sel_eb);
                                if b_start < b_end {
                                    draw_piece(renderer, b_start, b_end, color, true);
                                }
                                if s_end > row_sel_eb {
                                    let p_start = s_start.max(row_sel_eb);
                                    draw_piece(renderer, p_start, s_end, color, false);
                                }
                                return;
                            }
                        }
                        draw_piece(renderer, s_start, s_end, color, false);
                    };

                if let Some(spans) = token_spans
                    && !spans.is_empty()
                {
                    let mut drawn_any = false;
                    for span in spans {
                        if span.start_byte < sub_slice.end_byte
                            && span.end_byte > sub_slice.start_byte
                        {
                            let s_start = span.start_byte.max(sub_slice.start_byte);
                            let s_end = span.end_byte.min(sub_slice.end_byte);
                            if s_start < s_end && s_end <= line_text.len() {
                                draw_span_with_selection(renderer, s_start, s_end, span.color);
                                drawn_any = true;
                            }
                        }
                    }
                    if !drawn_any && !slice_str.is_empty() {
                        draw_span_with_selection(
                            renderer,
                            sub_slice.start_byte,
                            sub_slice.end_byte,
                            default_text_color,
                        );
                    }
                } else if !slice_str.is_empty() {
                    draw_span_with_selection(
                        renderer,
                        sub_slice.start_byte,
                        sub_slice.end_byte,
                        default_text_color,
                    );
                }
            }
        }
    }
}

#[inline]
fn char_to_byte_idx(s: &str, char_idx: usize, char_count: usize) -> usize {
    if s.len() == char_count {
        char_idx.min(s.len())
    } else {
        s.char_indices()
            .nth(char_idx)
            .map(|(idx, _)| idx)
            .unwrap_or(s.len())
    }
}

/// Formats positive integer into a byte buffer without heap allocation (Zero Allocation).
#[inline]
fn format_number_into_buffer(mut n: usize, buf: &mut [u8; 16]) -> usize {
    if n == 0 {
        buf[0] = b'0';
        return 1;
    }
    let mut temp = [0u8; 16];
    let mut len = 0;
    while n > 0 {
        temp[len] = b'0' + (n % 10) as u8;
        n /= 10;
        len += 1;
    }
    for i in 0..len {
        buf[i] = temp[len - 1 - i];
    }
    len
}

impl<'a, Message, Theme, Renderer> From<VirtualCodeViewer<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::text::Renderer<
            Font = Font,
            Paragraph = iced::advanced::graphics::text::Paragraph,
        > + 'a,
{
    fn from(viewer: VirtualCodeViewer<'a, Message, Theme, Renderer>) -> Self {
        Element::new(viewer)
    }
}
