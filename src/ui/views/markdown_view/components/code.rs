use super::style::{STYLE, code_block_style, copy_button_style, language_label_style};
use crate::app::Message;
use crate::ui::views::markdown_view::blocks::RenderContext;
use crate::ui::views::markdown_view::highlight::highlight_code;
use crate::ui::views::shared::font::get_code_font;
use crate::ui::views::shared::theme::scale_size;
use iced::widget::{button, column, container, row, text};
use iced::{Element, Length};

pub(crate) fn render_code_block<'a>(
    lang: &'a Option<String>,
    code: &'a str,
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    let is_dark = ctx.is_dark;
    let font_size = ctx.font_size;
    let code_font = get_code_font(ctx.font_family_mono);
    let language = lang.as_deref().unwrap_or("");
    let copy_content = code.to_string();

    let highlighted = highlight_code(lang, code, is_dark);

    let code_lines: Vec<Element<'a, Message>> = highlighted
        .iter()
        .map(|line_spans| {
            let spans: Vec<Element<'a, Message>> = line_spans
                .iter()
                .map(|(color, span_text)| {
                    text(*span_text)
                        .font(code_font)
                        .size(scale_size(STYLE.code.line_font_size, font_size))
                        .color(*color)
                        .into()
                })
                .collect();
            row(spans).into()
        })
        .collect();

    let top_bar: Element<'a, Message> = if !language.is_empty() {
        let language_label = container(
            text(language)
                .font(code_font)
                .size(scale_size(STYLE.code.label_button_font_size, font_size)),
        )
        .padding(STYLE.code.top_bar_padding)
        .style(language_label_style);
        row![
            language_label,
            copy_button_inline(code_font, copy_content, font_size),
        ]
        .into()
    } else {
        row![copy_button_inline(code_font, copy_content, font_size),].into()
    };

    column![
        top_bar,
        container(column(code_lines))
            .padding(STYLE.code.padding)
            .width(Length::Fill)
            .style(code_block_style),
    ]
    .spacing(0)
    .into()
}

fn copy_button_inline<'a>(
    code_font: iced::Font,
    copy_content: String,
    font_size: f32,
) -> Element<'a, Message> {
    container(
        button(
            text("Copy")
                .font(code_font)
                .size(scale_size(STYLE.code.label_button_font_size, font_size)),
        )
        .on_press(Message::CopyCode(copy_content))
        .style(copy_button_style)
        .padding(STYLE.code.button_padding),
    )
    .padding(0)
    .width(Length::Fill)
    .align_x(iced::alignment::Horizontal::Right)
    .into()
}
