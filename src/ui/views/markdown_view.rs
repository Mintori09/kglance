use crate::app::Message;
use crate::parsers::markdown::{Block, TableBlock};
use crate::ui::theme::glass;
use iced::widget::{column, container, image, row, scrollable, text};
use iced::{Border, Color, Element, Length, Shadow};

fn code_block_style(theme: &iced::Theme) -> container::Style {
    let is_dark = matches!(theme, iced::Theme::Dark);
    container::Style {
        background: Some(
            (if is_dark {
                glass::DARK_SURFACE
            } else {
                glass::LIGHT_SURFACE
            })
            .into(),
        ),
        text_color: Some(if is_dark {
            glass::DARK_TEXT
        } else {
            glass::LIGHT_TEXT
        }),
        border: Border {
            color: if is_dark {
                glass::DARK_BORDER
            } else {
                glass::LIGHT_BORDER
            },
            width: 1.0,
            radius: 4.0.into(),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

fn render_table<'a>(table: &'a TableBlock) -> Element<'a, Message> {
    let header_cells = table.headers.iter().map(|h| {
        container(text(h).size(14))
            .padding(8)
            .width(Length::FillPortion(1))
            .into()
    });

    let header_row = row(header_cells).spacing(1);

    let header = container(header_row).style(|_theme: &iced::Theme| container::Style {
        background: Some(glass::ACCENT.into()),
        text_color: Some(Color::WHITE),
        ..Default::default()
    });

    if table.rows.is_empty() {
        return column![header].into();
    }

    let body_rows = table.rows.iter().enumerate().map(|(i, row_data)| {
        let cells = row_data.iter().map(|c| {
            container(text(c).size(13))
                .padding(6)
                .width(Length::FillPortion(1))
                .into()
        });

        let row_widget = row(cells).spacing(1);

        let container_bg = if i % 2 == 0 {
            move |theme: &iced::Theme| {
                let is_dark = matches!(theme, iced::Theme::Dark);
                container::Style {
                    background: Some(
                        (if is_dark {
                            glass::DARK_SURFACE
                        } else {
                            glass::LIGHT_SURFACE
                        })
                        .into(),
                    ),
                    ..Default::default()
                }
            }
        } else {
            move |theme: &iced::Theme| {
                let is_dark = matches!(theme, iced::Theme::Dark);
                container::Style {
                    background: Some(
                        (if is_dark {
                            glass::DARK_BG
                        } else {
                            glass::LIGHT_BG
                        })
                        .into(),
                    ),
                    ..Default::default()
                }
            }
        };

        container(row_widget).style(container_bg).into()
    });

    column(std::iter::once(header.into()).chain(body_rows))
        .spacing(0)
        .into()
}

pub fn view_markdown<'a>(
    blocks: &'a [Block],
    state: &'a crate::core::MarkdownState,
) -> Element<'a, Message> {
    let content = blocks.iter().enumerate().map(|(i, block)| match block {
        Block::Heading { level, text: t } => {
            let size: f32 = match level {
                1 => 28.0,
                2 => 22.0,
                3 => 18.0,
                _ => 16.0,
            };
            let padding = match level {
                1 => 8,
                2 => 6,
                3 => 4,
                _ => 2,
            };
            container(text(t).size(size))
                .padding([padding, 0])
                .width(Length::Fill)
                .into()
        }
        Block::Paragraph(t) => container(text(t).size(14))
            .padding([2, 0])
            .width(Length::Fill)
            .into(),
        Block::CodeBlock { lang, code } => {
            let lang_bar: Element<'a, Message> = if !lang.is_empty() {
                container(text(lang).size(11))
                    .padding([2, 8])
                    .style(|theme: &iced::Theme| {
                        let is_dark = matches!(theme, iced::Theme::Dark);
                        container::Style {
                            background: Some(
                                (if is_dark {
                                    glass::DARK_BG
                                } else {
                                    glass::LIGHT_BG
                                })
                                .into(),
                            ),
                            text_color: Some(if is_dark {
                                glass::DARK_TEXT_DIM
                            } else {
                                glass::LIGHT_TEXT_DIM
                            }),
                            ..Default::default()
                        }
                    })
                    .into()
            } else {
                Element::from(container(text("")).padding(0))
            };

            column![
                lang_bar,
                container(text(code).size(13))
                    .padding(10)
                    .width(Length::Fill)
                    .style(code_block_style),
            ]
            .spacing(0)
            .into()
        }
        Block::Table(tbl) => render_table(tbl),
        Block::Mermaid { lines, rendered } => {
            let badge = container(text("Mermaid Diagram").size(11))
                .padding([2, 8])
                .style(|_theme: &iced::Theme| container::Style {
                    background: Some(glass::ACCENT.into()),
                    text_color: Some(Color::WHITE),
                    ..Default::default()
                });

            if rendered.is_some() {
                if let Some(Some(handle)) = state.cached_mermaid_handles.get(i) {
                    let img = container(
                        image(handle.clone())
                            .width(Length::Shrink)
                            .height(Length::Shrink),
                    )
                    .center_x(Length::Fill)
                    .padding(10);
                    column![badge, img].spacing(0).into()
                } else {
                    column![badge].spacing(0).into()
                }
            } else {
                let line_widgets = lines.iter().map(|line| {
                    let display = if line.contains("-->") {
                        line.replace("-->", " → ")
                    } else if line.contains("==>") {
                        line.replace("==>", " ⇒ ")
                    } else if line.contains("---") {
                        line.replace("---", " ── ")
                    } else {
                        line.clone()
                    };
                    text(display).size(13).into()
                });

                let content = container(column(line_widgets).spacing(2))
                    .padding(10)
                    .width(Length::Fill)
                    .style(code_block_style);

                column![badge, content].spacing(0).into()
            }
        }
    });

    let inner = column(content).spacing(6).padding(15);

    scrollable(inner)
        .id("content_scroll")
        .style(crate::ui::theme::glass_scrollable)
        .height(Length::Fill)
        .into()
}
