use super::style::STYLE;
use crate::app::Message;
use crate::features::markdown::parser::AlertKind;
use crate::features::markdown::view::blocks::render_block;
use crate::parsers::markdown::Block;
use crate::ui::theme::AppTheme;
use crate::ui::types::RenderContext;
use iced::font::Weight;
use iced::widget::{column, container, row, text};
use iced::{Color, Element, Font, Length, Padding};

fn alert_colors(kind: AlertKind, theme: AppTheme) -> (Color, Color) {
    let alerts = theme.palette().alerts;
    let group = match kind {
        AlertKind::Note => alerts.note,
        AlertKind::Tip => alerts.tip,
        AlertKind::Important => alerts.info,
        AlertKind::Warning => alerts.warning,
        AlertKind::Caution => alerts.caution,
    };
    (group.fg, group.bg)
}

fn alert_title(kind: AlertKind) -> &'static str {
    match kind {
        AlertKind::Note => "Note",
        AlertKind::Tip => "Tip",
        AlertKind::Important => "Important",
        AlertKind::Warning => "Warning",
        AlertKind::Caution => "Caution",
    }
}

pub(crate) fn render_alert<'a>(
    kind: AlertKind,
    blocks: &'a [Block],
    state: &'a crate::core::MarkdownState,
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    let (accent_color, bg_color) = alert_colors(kind, ctx.theme);
    let title_str = alert_title(kind);

    let header = text(title_str)
        .size(ctx.font_size * 0.95)
        .font(Font {
            weight: Weight::Bold,
            ..Default::default()
        })
        .color(accent_color);

    let base_block_index = ctx.block_index;
    let inner: Element<'a, Message> = column(blocks.iter().enumerate().map(|(i, block)| {
        let alert_ctx = RenderContext {
            block_index: base_block_index + i + 1,
            ..*ctx
        };
        render_block(i, block, state, &alert_ctx)
    }))
    .spacing(STYLE.general.section_spacing)
    .into();

    let content_col = column![header, inner].spacing(6.0);

    let content = container(content_col)
        .padding(Padding {
            top: 10.0,
            right: 14.0,
            bottom: 10.0,
            left: 14.0,
        })
        .style(move |_: &iced::Theme| container::Style {
            background: Some(bg_color.into()),
            ..Default::default()
        })
        .width(Length::Fill);

    let bar = container(text(""))
        .width(4.0)
        .height(Length::Fill)
        .style(move |_: &iced::Theme| container::Style {
            background: Some(accent_color.into()),
            ..Default::default()
        });

    row![bar, content].spacing(0).into()
}
