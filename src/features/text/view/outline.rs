use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Border, Color, Element, Length, Padding};

use crate::app::Message;
use crate::features::text::outline::{CodeSymbol, SymbolKind};
use crate::ui::components::scroll_pane::scroll_pane;
use crate::ui::components::sidebar::sidebar_entry_style;
use crate::ui::theme::AppTheme;
use crate::ui::theme::tokens::spacing;

const OUTLINE_ITEM_SPACING: f32 = 2.0;
const OUTLINE_PADDING: Padding = Padding {
    top: 6.0,
    right: 6.0,
    bottom: 6.0,
    left: 6.0,
};
const BADGE_FONT_SIZE: f32 = 10.0;
const NAME_FONT_SIZE: f32 = 12.0;
const LINE_FONT_SIZE: f32 = 11.0;

fn symbol_badge_color(kind: SymbolKind, theme: AppTheme) -> Color {
    let p = theme.palette().base;
    match kind {
        SymbolKind::Function => Color::from_rgb(0.35, 0.65, 0.95), // Blue
        SymbolKind::Struct => Color::from_rgb(0.35, 0.85, 0.65),   // Green
        SymbolKind::Class => Color::from_rgb(0.95, 0.65, 0.35),    // Orange
        SymbolKind::Enum => Color::from_rgb(0.85, 0.45, 0.85),     // Purple
        SymbolKind::Trait => Color::from_rgb(0.95, 0.85, 0.35),    // Yellow
        SymbolKind::Module => Color::from_rgb(0.65, 0.75, 0.85),   // Slate
        SymbolKind::Type => Color::from_rgb(0.45, 0.75, 0.85),     // Cyan
        SymbolKind::Const => p.text_dim,
    }
}

pub fn render_outline_sidebar<'a>(
    symbols: &'a [CodeSymbol],
    theme: AppTheme,
    width: f32,
) -> Element<'a, Message> {
    let bg_color = theme.palette().base.bg;
    let border_color = theme.palette().base.border;

    let entries: Vec<Element<'a, Message>> = symbols
        .iter()
        .map(|sym| render_symbol_entry(sym, theme))
        .collect();

    let content = column![
        scroll_pane(
            "text_outline_scroll",
            column(entries)
                .spacing(OUTLINE_ITEM_SPACING)
                .padding(OUTLINE_PADDING)
        )
        .build()
    ]
    .width(width)
    .height(Length::Fill);

    iced::widget::opaque(
        container(content)
            .width(width)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(bg_color.into()),
                border: Border {
                    width: 1.0,
                    color: border_color,
                    radius: 0.0.into(),
                },
                ..Default::default()
            }),
    )
}

fn render_symbol_entry<'a>(sym: &'a CodeSymbol, theme: AppTheme) -> Element<'a, Message> {
    let badge_color = symbol_badge_color(sym.kind, theme);
    let line_color = theme.palette().base.text_dim;

    let indent_space = (sym.indent_level as f32) * 8.0;

    let badge = container(
        text(sym.kind.badge_label())
            .size(BADGE_FONT_SIZE)
            .color(badge_color),
    )
    .padding(Padding {
        top: 1.0,
        right: 4.0,
        bottom: 1.0,
        left: 4.0,
    })
    .style(move |_| container::Style {
        background: Some(
            Color {
                a: 0.12,
                ..badge_color
            }
            .into(),
        ),
        border: Border {
            radius: 3.0.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    let name_label = text(&sym.name).size(NAME_FONT_SIZE).width(Length::Fill);

    let line_label = text(sym.line_number.to_string())
        .size(LINE_FONT_SIZE)
        .color(line_color);

    let mut row_widgets = row![].spacing(spacing::XS).align_y(Alignment::Center);

    if indent_space > 0.0 {
        row_widgets = row_widgets.push(Space::new().width(indent_space));
    }

    row_widgets = row_widgets.push(badge).push(name_label).push(line_label);

    let line_num = sym.line_number;

    button(row_widgets)
        .on_press(crate::app::messages::TextMsg::SymbolClicked(line_num).into())
        .width(Length::Fill)
        .padding(Padding {
            top: 4.0,
            right: 6.0,
            bottom: 4.0,
            left: 6.0,
        })
        .style(move |iced_theme, status| sidebar_entry_style(iced_theme, status, false, theme))
        .into()
}
