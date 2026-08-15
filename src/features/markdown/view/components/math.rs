use super::style::STYLE;
use crate::app::Message;
use crate::ui::theme::scale_size;
use crate::ui::types::RenderContext;
use iced::widget::{container, svg as svg_widget, text};
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

fn enhance_svg_stroke(svg_bytes: &[u8], hex_color: &str, stroke_width: f32) -> Vec<u8> {
    if let Ok(mut s) = String::from_utf8(svg_bytes.to_vec()) {
        let target_open = format!(r#"<g fill="{hex_color}">"#);
        let enhanced_open = format!(
            r#"<g fill="{hex_color}" stroke="{hex_color}" stroke-width="{stroke_width:.2}" stroke-linejoin="round" stroke-linecap="round">"#
        );
        if s.contains(&target_open) {
            s = s.replace(&target_open, &enhanced_open);
            return s.into_bytes();
        }
    }
    svg_bytes.to_vec()
}

pub(crate) fn render_math_block<'a>(
    latex: &'a str,
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    let sanitized = sanitize_for_iced_math(latex);
    let text_color = ctx.theme.palette().base.text;
    let r = (text_color.r * 255.0).round() as u8;
    let g = (text_color.g * 255.0).round() as u8;
    let b = (text_color.b * 255.0).round() as u8;
    let math_color = iced_math::Color::rgb(r, g, b);
    let hex_color = format!("#{r:02x}{g:02x}{b:02x}");

    let math_font_size = scale_size(ctx.font_size * STYLE.math.font_scale, ctx.font_size);

    let renderer = iced_math::MathRenderer::new()
        .font_size(math_font_size)
        .display_style(true)
        .color(math_color);

    let math_element: Element<'a, Message> = match renderer.to_svg(&sanitized) {
        Ok(bytes) => {
            let final_bytes = enhance_svg_stroke(&bytes, &hex_color, STYLE.math.stroke_width);
            svg_widget::Svg::new(svg_widget::Handle::from_memory(final_bytes))
                .width(Length::Shrink)
                .height(Length::Shrink)
                .into()
        }
        Err(_) => text::Text::new(sanitized)
            .font(iced::Font::MONOSPACE)
            .color(iced::Color::from_rgb8(0xc0, 0x39, 0x2b))
            .into(),
    };

    container(math_element)
        .center_x(Length::Fill)
        .padding(STYLE.math.padding)
        .into()
}
