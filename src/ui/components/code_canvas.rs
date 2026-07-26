use iced::widget::canvas::{self, Canvas, Frame, Geometry, Text};
use iced::{Element, Font, Length, Point, Rectangle};

use crate::app::Message;
use crate::core::layout_engine::{
    LayoutConfig, LogicalDocument, TextLayoutEngine, VisualDocument, WrapMode,
};
use crate::ui::theme::{DARK_TEXT_DIM, LIGHT_TEXT_DIM};

#[derive(Debug, Clone, Default)]
pub struct SelectionState {
    pub is_selecting: bool,
    pub start: Option<Point>,
    pub end: Option<Point>,
}

pub fn code_canvas<'a>(
    content: &'a str,
    extension: &'a str,
    is_dark: bool,
    font_size: f32,
    font: Font,
) -> Element<'a, Message> {
    let doc = LogicalDocument::from_text(content);
    let config = LayoutConfig {
        viewport_width: 800.0,
        font_size,
        tab_width: 4,
        wrap_mode: WrapMode::NoWrap,
    };
    let visual_doc = TextLayoutEngine::compute_highlighted(&doc, &config, extension, is_dark);

    Canvas::new(CodeCanvasProgram {
        visual_doc,
        is_dark,
        font_size,
        font,
    })
    .width(Length::Fill)
    .height(Length::Shrink)
    .into()
}

struct CodeCanvasProgram {
    visual_doc: VisualDocument,
    is_dark: bool,
    font_size: f32,
    font: Font,
}

impl<Message> canvas::Program<Message> for CodeCanvasProgram {
    type State = SelectionState;

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let gutter_width = 40.0;
        let line_height = self.font_size * 1.4;

        let gutter_color = if self.is_dark {
            DARK_TEXT_DIM
        } else {
            LIGHT_TEXT_DIM
        };

        for (idx, vline) in self.visual_doc.visual_lines.iter().enumerate() {
            let y = idx as f32 * line_height;

            if let Some(num) = vline.line_number {
                frame.fill_text(Text {
                    content: num.to_string(),
                    position: Point::new(gutter_width - 8.0, y),
                    color: gutter_color,
                    size: self.font_size.into(),
                    font: self.font,
                    align_x: iced::alignment::Horizontal::Right.into(),
                    align_y: iced::alignment::Vertical::Top,
                    shaping: iced::widget::text::Shaping::Basic,
                    max_width: f32::INFINITY,
                    line_height: iced::widget::text::LineHeight::Relative(1.0),
                });
            }

            let mut x = gutter_width + 12.0;
            for span in &vline.spans {
                frame.fill_text(Text {
                    content: span.text.clone(),
                    position: Point::new(x, y),
                    color: span.color,
                    size: self.font_size.into(),
                    font: self.font,
                    align_x: iced::alignment::Horizontal::Left.into(),
                    align_y: iced::alignment::Vertical::Top,
                    shaping: iced::widget::text::Shaping::Basic,
                    max_width: f32::INFINITY,
                    line_height: iced::widget::text::LineHeight::Relative(1.0),
                });
                x += span.text.chars().count() as f32 * (self.font_size * 0.6);
            }
        }

        vec![frame.into_geometry()]
    }
}
