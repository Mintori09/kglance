use super::style::{STYLE, code_block_style, copy_button_style, language_label_style};
use crate::app::Message;
use crate::features::markdown::view::build_selectable;
use crate::features::markdown::view::highlight::highlight_code;
use crate::ui::theme::font::{get_code_font, get_main_font};
use crate::ui::theme::{AppTheme, scale_size};
use crate::ui::types::RenderContext;
use iced::widget::{button, column, container, row, text};
use iced::{Element, Length};

fn is_circled_number(c: char) -> bool {
    let u = c as u32;
    (0x2460..=0x2473).contains(&u) || (0x3251..=0x32BF).contains(&u)
}

fn push_code_span<'a>(
    spans: &mut Vec<iced::widget::text::Span<'a, (), iced::Font>>,
    text: &'a str,
    color: iced::Color,
    code_font: iced::Font,
    badge_font: iced::Font,
    badge_color: iced::Color,
) {
    if text.is_empty() {
        return;
    }
    if !text.chars().any(is_circled_number) {
        spans.push(
            iced::widget::text::Span::new(text)
                .font(code_font)
                .color(color),
        );
        return;
    }

    let mut start = 0;
    let mut in_badge = false;
    for (idx, ch) in text.char_indices() {
        let is_b = is_circled_number(ch);
        if idx == 0 {
            in_badge = is_b;
        } else if is_b != in_badge {
            let seg = &text[start..idx];
            if !seg.is_empty() {
                if in_badge {
                    spans.push(
                        iced::widget::text::Span::new(seg)
                            .font(badge_font)
                            .color(badge_color),
                    );
                } else {
                    spans.push(
                        iced::widget::text::Span::new(seg)
                            .font(code_font)
                            .color(color),
                    );
                }
            }
            start = idx;
            in_badge = is_b;
        }
    }
    let last_seg = &text[start..];
    if !last_seg.is_empty() {
        if in_badge {
            spans.push(
                iced::widget::text::Span::new(last_seg)
                    .font(badge_font)
                    .color(badge_color),
            );
        } else {
            spans.push(
                iced::widget::text::Span::new(last_seg)
                    .font(code_font)
                    .color(color),
            );
        }
    }
}

pub(crate) fn render_code_block<'a>(
    lang: &'a Option<String>,
    code: &'a str,
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    let font_size = ctx.font_size;
    let code_font = get_code_font(ctx.font_family_mono);
    let mut badge_font = get_main_font(ctx.font_family);
    badge_font.weight = iced::font::Weight::Bold;
    let badge_color = ctx.theme.palette().roles.link;
    let language = lang.as_deref().unwrap_or("");
    let copy_content = code.to_string();

    let highlighted = highlight_code(lang, code, ctx.theme);

    let theme = ctx.theme;
    let default_text_color = ctx.theme.palette().base.text;

    let is_plain = highlighted.is_empty()
        || highlighted
            .iter()
            .all(|line| line.is_empty() || (line.len() == 1 && line[0].0 == default_text_color));

    let all_spans: Vec<iced::widget::text::Span<'a, (), iced::Font>> = if is_plain {
        let mut spans = Vec::new();
        push_code_span(
            &mut spans,
            code,
            default_text_color,
            code_font,
            badge_font,
            badge_color,
        );
        spans
    } else {
        let mut spans = Vec::new();
        for (line_idx, line_spans) in highlighted.iter().enumerate() {
            for (color, span_text) in line_spans {
                if !span_text.is_empty() {
                    push_code_span(
                        &mut spans,
                        span_text,
                        *color,
                        code_font,
                        badge_font,
                        badge_color,
                    );
                }
            }
            if line_idx + 1 < highlighted.len() {
                spans.push(
                    iced::widget::text::Span::new("\n")
                        .font(code_font)
                        .color(default_text_color),
                );
            }
        }
        spans
    };

    let _fence_open = ctx.alloc_block_index();
    let code_idx = ctx.alloc_block_index();
    let _fence_close = ctx.alloc_block_index();

    let code_content = build_selectable(
        all_spans,
        scale_size(STYLE.code.line_font_size, font_size),
        code_idx,
        Length::Fill,
        default_text_color,
        ctx.selection_range,
        ctx.drag_active,
        ctx.result_popup,
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
        .on_press(crate::app::messages::ActionMsg::CopyText(copy_content).into())
        .style(move |_: &iced::Theme, status| copy_button_style(theme, status))
        .padding(STYLE.code.button_padding),
    )
    .padding(0)
    .width(Length::Fill)
    .align_x(iced::alignment::Horizontal::Right)
    .into()
}
