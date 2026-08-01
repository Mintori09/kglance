use super::style::{STYLE, code_block_style, copy_button_style, language_label_style};
use crate::app::Message;
use crate::ui::theme::font::get_code_font;
use crate::ui::theme::scale_size;
use crate::ui::views::markdown_view::blocks::RenderContext;
use crate::ui::views::markdown_view::highlight::highlight_code;
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

    let line_spans_vector: Vec<Vec<iced::widget::text::Span<'a, (), iced::Font>>> = highlighted
        .iter()
        .map(|line_spans| {
            line_spans
                .iter()
                .map(|(color, span_text)| {
                    iced::widget::text::Span::new(*span_text)
                        .font(code_font)
                        .color(*color)
                })
                .collect()
        })
        .collect();

    let code_lines: Vec<Element<'a, Message>> = line_spans_vector
        .into_iter()
        .enumerate()
        .map(|(line_idx, spans)| {
            let line_block_index = ctx.block_index + line_idx + 1;
            crate::ui::components::selectable_text::SelectableText::new(
                spans,
                scale_size(STYLE.code.line_font_size, font_size),
            )
            .block_index(line_block_index)
            .selection_range(ctx.selection_range)
            .drag_active(ctx.drag_active)
            .on_selection_change(|s| crate::app::messages::MarkdownMsg::SelectionChanged(s).into())
            .on_drag_start(|block, offset| {
                crate::app::messages::MarkdownMsg::SelectionDragStart { block, offset }.into()
            })
            .on_drag_update(|block, offset| {
                crate::app::messages::MarkdownMsg::SelectionDragUpdate { block, offset }.into()
            })
            .on_drag_end(|| crate::app::messages::MarkdownMsg::SelectionDragEnd.into())
            .on_clear_selection(|| crate::app::messages::MarkdownMsg::SelectionClear.into())
            .into()
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
        .on_press(crate::app::messages::ActionMsg::CopyCode(copy_content).into())
        .style(copy_button_style)
        .padding(STYLE.code.button_padding),
    )
    .padding(0)
    .width(Length::Fill)
    .align_x(iced::alignment::Horizontal::Right)
    .into()
}
