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
    on_select: Option<Box<dyn Fn(Option<SelectionRange>) -> Message + 'a>>,
    on_copy: Option<Box<dyn Fn(String) -> Message + 'a>>,
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
        let line_h = self.line_height();
        let gutter_w = self.gutter_width();
        let char_w = self.char_width();

        let vrow = (rel_pos.y / line_h).floor() as usize;
        let (line_idx, sub_row_idx) = if let Some(d) = self.display_map {
            d.visual_row_to_line(vrow)
        } else {
            (vrow.min(self.doc.total_lines().saturating_sub(1)), 0)
        };

        let x_in_code = (rel_pos.x - gutter_w).max(0.0);
        let line_text = self.doc.get_line(line_idx);
        let max_cols = self
            .display_map
            .map(|d| d.max_cols_per_row())
            .unwrap_or(usize::MAX);

        let sub_slices = if self.wrap {
            crate::features::text::display_map::wrap_line_to_slices(line_text, max_cols)
        } else {
            vec![crate::features::text::display_map::VisualSubSlice {
                start_byte: 0,
                end_byte: line_text.len(),
                start_col: 0,
                end_col: line_text.chars().count(),
            }]
        };

        let sub_slice = sub_slices.get(sub_row_idx).copied().unwrap_or(
            crate::features::text::display_map::VisualSubSlice {
                start_byte: 0,
                end_byte: line_text.len(),
                start_col: 0,
                end_col: line_text.chars().count(),
            },
        );

        let col_in_sub = (x_in_code / char_w).round() as usize;
        let col_idx = (sub_slice.start_col + col_in_sub).min(sub_slice.end_col);

        TextPosition::new(line_idx, col_idx)
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
        _viewport: &Rectangle,
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
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.is_mouse_held = false;
                state.is_selecting = false;
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
        let selection_bg = Color::from_rgba(0.2, 0.4, 0.8, 0.35);
        let search_match_bg = Color::from_rgba(0.9, 0.7, 0.1, 0.4);
        let active_match_bg = Color::from_rgba(0.95, 0.45, 0.1, 0.7);

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
            let sub_slices = if self.wrap {
                crate::features::text::display_map::wrap_line_to_slices(line_text, max_cols)
            } else {
                vec![crate::features::text::display_map::VisualSubSlice {
                    start_byte: 0,
                    end_byte: line_text.len(),
                    start_col: 0,
                    end_col: line_text.chars().count(),
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
                if let Some(sel) = self.selection
                    && let Some((start_col, end_col)) =
                        sel.line_col_range(line_idx, line_text.chars().count())
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
                                let span_text = &line_text[s_start..s_end];
                                let prefix = &line_text[sub_slice.start_byte..s_start];
                                let offset_chars = prefix.chars().count();
                                let span_x = code_x + (offset_chars as f32 * char_w);

                                renderer.fill_text(
                                    Text {
                                        content: span_text.to_string(),
                                        bounds: Size::new(
                                            (bounds.width - gutter_w).max(100.0),
                                            line_h,
                                        ),
                                        size: Pixels(self.font_size),
                                        line_height: iced::widget::text::LineHeight::Relative(1.35),
                                        font: self.font,
                                        align_x: iced::alignment::Horizontal::Left.into(),
                                        align_y: iced::alignment::Vertical::Top,
                                        shaping: iced::widget::text::Shaping::Basic,
                                        wrapping: iced::widget::text::Wrapping::None,
                                    },
                                    Point::new(span_x, row_y),
                                    span.color,
                                    *viewport,
                                );
                                drawn_any = true;
                            }
                        }
                    }
                    if !drawn_any && !slice_str.is_empty() {
                        renderer.fill_text(
                            Text {
                                content: slice_str.to_string(),
                                bounds: Size::new((bounds.width - gutter_w).max(100.0), line_h),
                                size: Pixels(self.font_size),
                                line_height: iced::widget::text::LineHeight::Relative(1.35),
                                font: self.font,
                                align_x: iced::alignment::Horizontal::Left.into(),
                                align_y: iced::alignment::Vertical::Top,
                                shaping: iced::widget::text::Shaping::Basic,
                                wrapping: iced::widget::text::Wrapping::None,
                            },
                            Point::new(code_x, row_y),
                            default_text_color,
                            *viewport,
                        );
                    }
                } else if !slice_str.is_empty() {
                    renderer.fill_text(
                        Text {
                            content: slice_str.to_string(),
                            bounds: Size::new((bounds.width - gutter_w).max(100.0), line_h),
                            size: Pixels(self.font_size),
                            line_height: iced::widget::text::LineHeight::Relative(1.35),
                            font: self.font,
                            align_x: iced::alignment::Horizontal::Left.into(),
                            align_y: iced::alignment::Vertical::Top,
                            shaping: iced::widget::text::Shaping::Basic,
                            wrapping: iced::widget::text::Wrapping::None,
                        },
                        Point::new(code_x, row_y),
                        default_text_color,
                        *viewport,
                    );
                }
            }
        }
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
