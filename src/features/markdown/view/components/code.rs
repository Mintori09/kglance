use super::style::{STYLE, code_block_style, copy_button_style, language_label_style};
use crate::app::Message;
use crate::features::markdown::view::build_selectable;
use crate::features::markdown::view::highlight::highlight_code;
use crate::ui::theme::font::get_code_font;
use crate::ui::theme::{AppTheme, scale_size};
use crate::ui::types::RenderContext;
use iced::widget::{button, column, container, row, text};
use iced::{Element, Length};

pub(crate) fn render_code_block<'a>(
    lang: &'a Option<String>,
    code: &'a str,
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    let font_size = ctx.font_size;
    let code_font = get_code_font(ctx.font_family_mono);
    let language = lang.as_deref().unwrap_or("");
    let copy_content = code.to_string();

    let highlighted = highlight_code(lang, code, ctx.theme);

    let theme = ctx.theme;
    let default_text_color = ctx.theme.palette().base.text;

    let mut all_spans: Vec<iced::widget::text::Span<'a, (), iced::Font>> = Vec::new();
    for (line_idx, line_spans) in highlighted.iter().enumerate() {
        for (color, span_text) in line_spans {
            all_spans.push(
                iced::widget::text::Span::new(*span_text)
                    .font(code_font)
                    .color(*color),
            );
        }
        if line_idx + 1 < highlighted.len() {
            all_spans.push(
                iced::widget::text::Span::new("\n")
                    .font(code_font)
                    .color(default_text_color),
            );
        }
    }

    let code_content = build_selectable(
        all_spans,
        scale_size(STYLE.code.line_font_size, font_size),
        ctx.block_index + 1,
        Length::Fill,
        default_text_color,
        ctx.selection_range,
        ctx.drag_active,
    );

    let top_bar: Element<'a, Message> = if !language.is_empty() {
        let language_label = container(
            text(language)
                .font(code_font)
                .size(scale_size(STYLE.code.label_button_font_size, font_size)),
        )
        .padding(STYLE.code.top_bar_padding)
        .style(move |_: &iced::Theme| language_label_style(theme));
        row![
            language_label,
            copy_button_inline(code_font, copy_content, font_size, theme),
        ]
        .into()
    } else {
        row![copy_button_inline(
            code_font,
            copy_content,
            font_size,
            theme
        ),]
        .into()
    };

    column![
        top_bar,
        container(code_content)
            .padding(STYLE.code.padding)
            .width(Length::Fill)
            .style(move |_: &iced::Theme| code_block_style(theme)),
    ]
    .spacing(0)
    .into()
}

fn copy_button_inline<'a>(
    code_font: iced::Font,
    copy_content: String,
    font_size: f32,
    theme: AppTheme,
) -> Element<'a, Message> {
    container(
        button(
            text("⎘")
                .font(code_font)
                .size(scale_size(STYLE.code.label_button_font_size, font_size)),
        )
        .on_press(crate::app::messages::ActionMsg::CopyCode(copy_content).into())
        .style(move |_: &iced::Theme, status| copy_button_style(theme, status))
        .padding(STYLE.code.button_padding),
    )
    .padding(0)
    .width(Length::Fill)
    .align_x(iced::alignment::Horizontal::Right)
    .into()
}
