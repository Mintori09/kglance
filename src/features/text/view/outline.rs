use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Border, Color, Element, Length, Padding};

use crate::app::Message;
use crate::features::text::display_map::DisplayMap;
use crate::features::text::outline::{CodeSymbol, SymbolKind};
use crate::ui::components::scroll_pane::scroll_pane;
use crate::ui::components::sidebar::sidebar_entry_style;
use crate::ui::theme::AppTheme;
use crate::ui::theme::tokens::{radius, spacing, typography};

const OUTLINE_ITEM_SPACING: f32 = spacing::XXS;
const OUTLINE_PADDING: Padding = Padding {
    top: 6.0,
    right: 6.0,
    bottom: 6.0,
    left: 6.0,
};
const BADGE_FONT_SIZE: f32 = typography::BADGE;
const NAME_FONT_SIZE: f32 = typography::BODY;
const LINE_FONT_SIZE: f32 = typography::CAPTION;
const SCROLL_OFFSET_MARGIN: f32 = 40.0;

fn symbol_badge_color(kind: SymbolKind, theme: AppTheme) -> Color {
    let p = theme.palette().base;
    let s = theme.palette().symbols;
    match kind {
        SymbolKind::Function => s.function,
        SymbolKind::Struct => s.r#struct,
        SymbolKind::Class => s.class,
        SymbolKind::Enum => s.r#enum,
        SymbolKind::Trait => s.r#trait,
        SymbolKind::Module => s.module,
        SymbolKind::Type => s.r#type,
        SymbolKind::Const => p.text_dim,
    }
}

pub fn render_outline_sidebar<'a>(
    symbols: &'a [CodeSymbol],
    theme: AppTheme,
    width: f32,
    scroll_y: f32,
    display_map: &DisplayMap,
) -> Element<'a, Message> {
    let bg_color = theme.palette().base.bg;
    let border_color = theme.palette().base.border;

    let active_symbol_line = find_active_symbol_line(symbols, scroll_y, display_map);

    let entries: Vec<Element<'a, Message>> = if symbols.is_empty() {
        vec![
            container(
                text("No outline available")
                    .size(NAME_FONT_SIZE)
                    .color(theme.palette().base.text_dim),
            )
            .padding(Padding {
                top: 12.0,
                right: 8.0,
                bottom: 12.0,
                left: 8.0,
            })
            .into(),
        ]
    } else {
        symbols
            .iter()
            .map(|sym| {
                let is_active = active_symbol_line == Some(sym.line_number);
                render_symbol_entry(sym, is_active, theme)
            })
            .collect()
    };

    container(
        scroll_pane(
            "text_outline_scroll",
            column(entries)
                .spacing(OUTLINE_ITEM_SPACING)
                .padding(OUTLINE_PADDING),
        )
        .build(),
    )
    .width(Length::Fixed(width))
    .height(Length::Fill)
    .style(move |_| container::Style {
        background: Some(bg_color.into()),
        border: Border {
            width: 1.0,
            color: border_color,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn find_active_symbol_line(
    symbols: &[CodeSymbol],
    scroll_y: f32,
    display_map: &DisplayMap,
) -> Option<usize> {
    symbols
        .iter()
        .rposition(|sym| {
            let line_idx = sym.line_number.saturating_sub(1);
            let sym_y = display_map.get_line_y(line_idx);
            sym_y <= scroll_y + SCROLL_OFFSET_MARGIN
        })
        .map(|idx| symbols[idx].line_number)
}

fn render_symbol_entry<'a>(
    sym: &'a CodeSymbol,
    is_active: bool,
    theme: AppTheme,
) -> Element<'a, Message> {
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
            radius: radius::XS.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    let name_label = container(
        text(&sym.name)
            .size(NAME_FONT_SIZE)
            .wrapping(iced::widget::text::Wrapping::None),
    )
    .width(Length::Fill)
    .clip(true);

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
        .style(move |iced_theme, status| sidebar_entry_style(iced_theme, status, is_active, theme))
        .into()
}
