use crate::app::Message;
use crate::ui::types::RenderContext;
use iced::font::Weight;
use iced::widget::{column, container, row, text};
use iced::{Color, Element, Font, Length, Padding};

pub(crate) fn render_frontmatter<'a>(
    entries: &'a [(String, String)],
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    let base_color = ctx.theme.palette().base;

    let rows: Vec<Element<'a, Message>> = entries
        .iter()
        .map(|(key, value)| {
            let key_label = text(format!("{key}:"))
                .size(ctx.font_size * 0.9)
                .font(Font {
                    weight: Weight::Bold,
                    ..Default::default()
                })
                .color(base_color.text_dim);

            let val_label = text(value.as_str())
                .size(ctx.font_size * 0.9)
                .color(base_color.text);

            row![key_label, val_label].spacing(8.0).into()
        })
        .collect();

    let col = column(rows).spacing(6.0);

    container(col)
        .padding(Padding {
            top: 12.0,
            right: 16.0,
            bottom: 12.0,
            left: 16.0,
        })
        .style(move |_: &iced::Theme| container::Style {
            background: Some(Color::from_rgba(0.5, 0.5, 0.5, 0.05).into()),
            border: iced::Border {
                color: base_color.border,
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        })
        .width(Length::Fill)
        .into()
}
