use crate::app::Message;
use crate::features::markdown::view::build_selectable;
use crate::ui::theme::tokens::{radius, spacing};
use crate::ui::types::RenderContext;
use iced::font::Weight;
use iced::widget::{column, container};
use iced::{Element, Font, Length, Padding};

pub(crate) fn render_frontmatter<'a>(
    entries: &'a [(String, String)],
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    let base_color = ctx.theme.palette().base;

    let rows: Vec<Element<'a, Message>> = entries
        .iter()
        .map(|(key, value)| {
            let key_span = iced::widget::text::Span::new(format!("{key}: "))
                .font(Font {
                    weight: Weight::Bold,
                    ..Default::default()
                })
                .color(base_color.text_dim);
            let val_span = iced::widget::text::Span::new(value.clone()).color(base_color.text);
            let blk = ctx.alloc_block_index();

            let selectable = build_selectable(
                vec![key_span, val_span],
                ctx.font_size * 0.9,
                blk,
                Length::Fill,
                base_color.text,
                ctx.selection_range,
                ctx.drag_active,
            );
            selectable.into()
        })
        .collect();

    let col = column(rows).spacing(spacing::XS + 2.0);

    container(col)
        .padding(Padding {
            top: spacing::M,
            right: spacing::L,
            bottom: spacing::M,
            left: spacing::L,
        })
        .style(move |_: &iced::Theme| container::Style {
            background: Some(base_color.surface_raised.into()),
            border: iced::Border {
                color: base_color.border,
                width: 1.0,
                radius: radius::MD.into(),
            },
            ..Default::default()
        })
        .width(Length::Fill)
        .into()
}
