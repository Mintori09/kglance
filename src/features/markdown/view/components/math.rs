use crate::app::Message;
use crate::ui::types::RenderContext;
use iced::widget::container;
use iced::{Element, Length};

fn sanitize_for_iced_math(latex: &str) -> String {
    let mut out = String::with_capacity(latex.len());
    let mut rest = latex;

    while let Some(pos) = rest.find("\\text{") {
        let (before, after) = rest.split_at(pos);
        out.push_str(&before.replace("&&", r"\quad "));

        let body = &after["\\text{".len()..];
        let mut depth = 1usize;
        let mut end = 0usize;
        for (i, ch) in body.char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = i;
                        break;
                    }
                }
                _ => {}
            }
        }

        let text_content = &body[..end];
        if text_content.is_ascii() {
            out.push_str(r"\text{");
            out.push_str(text_content);
            out.push('}');
        }
        rest = &body[end + 1..];
    }

    out.push_str(&rest.replace("&&", r"\quad "));
    out
}

pub(crate) fn render_math_block<'a>(
    latex: &'a str,
    _ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    let sanitized = sanitize_for_iced_math(latex);
    container(iced_math::block(&sanitized))
        .width(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .into()
}
