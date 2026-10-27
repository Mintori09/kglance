use iced::advanced::graphics::text::cosmic_text;
use iced::advanced::renderer;
use iced::advanced::text::Text;
use iced::{Background, Border, Color, Font, Pixels, Point, Rectangle, Shadow, Size, Vector};

use crate::ui::theme::AppTheme;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextTooltip {
    pub start: usize,
    pub end: usize,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HoveredTooltip {
    pub content: String,
    pub target_rect: Rectangle,
}

pub struct TooltipOverlay<Theme, Renderer> {
    content: String,
    target_rect: Rectangle,
    viewport: Rectangle,
    font_size: f32,
    theme: AppTheme,
    _phantom: std::marker::PhantomData<(Theme, Renderer)>,
}

pub fn create_tooltip_overlay<'a, Message: 'a, Theme: 'a, Renderer>(
    content: String,
    target_rect: Rectangle,
    viewport: Rectangle,
    font_size: f32,
    theme: AppTheme,
) -> iced::advanced::overlay::Element<'a, Message, Theme, Renderer>
where
    Renderer: 'a
        + iced::advanced::Renderer
        + iced::advanced::text::Renderer<
            Font = Font,
            Paragraph = iced::advanced::graphics::text::Paragraph,
        >,
{
    iced::advanced::overlay::Element::new(Box::new(TooltipOverlay {
        content,
        target_rect,
        viewport,
        font_size,
        theme,
        _phantom: std::marker::PhantomData,
    }))
}

impl<Message, Theme, Renderer> iced::advanced::overlay::Overlay<Message, Theme, Renderer>
    for TooltipOverlay<Theme, Renderer>
where
    Renderer: iced::advanced::Renderer
        + iced::advanced::text::Renderer<
            Font = Font,
            Paragraph = iced::advanced::graphics::text::Paragraph,
        >,
{
    fn layout(&mut self, _renderer: &Renderer, _bounds: Size) -> iced::advanced::layout::Node {
        let char_count = self.content.chars().count();
        let est_char_width = self.font_size * 0.62;
        let padding_h = 16.0;
        let padding_v = 10.0;
        let max_content_width = (self.viewport.width - 32.0).clamp(100.0, 520.0);
        let raw_width = (char_count as f32 * est_char_width) + padding_h;
        let width = raw_width.min(max_content_width).max(40.0);

        let lines = (raw_width / max_content_width).ceil().max(1.0);
        let height = (lines * self.font_size * 1.3) + padding_v;

        let mut x = self.target_rect.x + (self.target_rect.width - width) * 0.5;
        let min_x = self.viewport.x + 8.0;
        let max_x = (self.viewport.x + self.viewport.width - width - 8.0).max(min_x);
        x = x.clamp(min_x, max_x);

        let mut y = self.target_rect.y - height - 6.0;
        if y < self.viewport.y + 4.0 {
            y = self.target_rect.y + self.target_rect.height + 6.0;
        }

        iced::advanced::layout::Node::new(Size::new(width, height)).move_to(Point::new(x, y))
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        _cursor: iced::mouse::Cursor,
    ) {
        let bounds = layout.bounds();
        let (bg, border_color, text_color) = if self.theme.is_dark() {
            (
                Color::from_rgb8(28, 30, 36),
                Color::from_rgba(1.0, 1.0, 1.0, 0.15),
                Color::from_rgb8(235, 238, 245),
            )
        } else {
            (
                Color::from_rgb8(250, 250, 252),
                Color::from_rgba(0.0, 0.0, 0.0, 0.15),
                Color::from_rgb8(30, 32, 40),
            )
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    color: border_color,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
                    offset: Vector::new(0.0, 3.0),
                    blur_radius: 8.0,
                },
                snap: false,
            },
            Background::Color(bg),
        );

        let padding_h = 8.0;
        let padding_v = 5.0;
        let text_pos = Point::new(bounds.x + padding_h, bounds.y + padding_v);
        let text_bounds = Size::new(
            (bounds.width - padding_h * 2.0).max(0.0),
            (bounds.height - padding_v * 2.0).max(0.0),
        );

        renderer.fill_text(
            Text {
                content: self.content.clone(),
                bounds: text_bounds,
                size: Pixels(self.font_size),
                line_height: iced::widget::text::LineHeight::Relative(1.3),
                font: Font::default(),
                align_x: iced::alignment::Horizontal::Left.into(),
                align_y: iced::alignment::Vertical::Top,
                shaping: iced::widget::text::Shaping::Advanced,
                wrapping: iced::widget::text::Wrapping::Word,
            },
            text_pos,
            text_color,
            Rectangle::with_size(Size::INFINITE),
        );
    }
}

pub fn compute_inline_rect(
    buffer: &cosmic_text::Buffer,
    line_offsets: &[usize],
    bounds: Rectangle,
    top_pad: f32,
    start: usize,
    end: usize,
    offset_x: f32,
) -> Option<Rectangle> {
    let mut min_x: Option<f32> = None;
    let mut max_x: Option<f32> = None;
    let mut target_y: Option<f32> = None;
    let mut target_h = 16.0;

    for (run_idx, run) in buffer.layout_runs().enumerate() {
        let line_start = line_offsets.get(run.line_i).copied().unwrap_or(0);
        let line_text_len = run.text.len();
        let line_end = line_start + line_text_len;

        if line_start >= end || line_end <= start {
            continue;
        }

        let r_start_in_line = start.saturating_sub(line_start).min(line_text_len);
        let r_end_in_line = end.saturating_sub(line_start).min(line_text_len);
        if r_start_in_line >= r_end_in_line {
            continue;
        }

        let run_top = if run_idx == 0 {
            bounds.y
        } else {
            bounds.y + top_pad + run.line_top
        };
        target_y = Some(target_y.map_or(run_top, |y| y.min(run_top)));
        target_h = run.line_height;

        for glyph in run.glyphs {
            if glyph.end > r_start_in_line && glyph.start < r_end_in_line {
                let x = bounds.x + offset_x + glyph.x;
                let x_end = x + glyph.w;
                min_x = Some(min_x.map_or(x, |m| m.min(x)));
                max_x = Some(max_x.map_or(x_end, |m| m.max(x_end)));
            }
        }
    }

    if let (Some(x1), Some(x2), Some(y)) = (min_x, max_x, target_y) {
        Some(Rectangle {
            x: x1,
            y,
            width: (x2 - x1).max(1.0),
            height: target_h,
        })
    } else {
        None
    }
}
