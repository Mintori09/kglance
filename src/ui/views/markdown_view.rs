use crate::app::Message;
use crate::log_debug;
use crate::parsers::markdown::{Block, TableBlock};
use crate::ui::theme::glass;
use iced::widget::{column, container, image, row, scrollable, text};
use iced::{Border, Color, Element, Length, Shadow};

fn on_scrolled(viewport: iced::widget::scrollable::Viewport) -> Message {
    Message::ContentScrolled(viewport.absolute_offset().y)
}

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

// ==========================================
// 2. BLOCK RENDERERS
// ==========================================

fn scale(s: f32, font_size: f32) -> f32 {
    (s * font_size / 14.0).round().max(8.0)
}

fn render_heading<'a>(level: u8, text_content: &'a str, font_size: f32) -> Element<'a, Message> {
    let raw: f32 = match level {
        1 => 28.0,
        2 => 22.0,
        3 => 18.0,
        _ => 16.0,
    };
    let size = scale(raw, font_size);
    let padding = match level {
        1 => 8,
        2 => 6,
        3 => 4,
        _ => 2,
    };

    container(text(text_content).size(size))
        .padding([padding, 0])
        .width(Length::Fill)
        .into()
}

fn render_paragraph<'a>(text_content: &'a str, font_size: f32) -> Element<'a, Message> {
    container(text(text_content).size(font_size))
        .padding([2, 0])
        .width(Length::Fill)
        .into()
}

fn render_code_block<'a>(lang: &'a str, code: &'a str, font_size: f32) -> Element<'a, Message> {
    let lang_bar: Element<'a, Message> = if !lang.is_empty() {
        container(text(lang).size(scale(11.0, font_size)))
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
        container(text(code).size(scale(13.0, font_size)))
            .padding(10)
            .width(Length::Fill)
            .style(code_block_style),
    ]
    .spacing(0)
    .into()
}

fn render_table<'a>(table: &'a TableBlock, font_size: f32) -> Element<'a, Message> {
    let header_cells = table.headers.iter().map(|h| {
        container(text(h).size(scale(14.0, font_size)))
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
            container(text(c).size(scale(13.0, font_size)))
                .padding(6)
                .width(Length::FillPortion(1))
                .into()
        });

        let row_widget = row(cells).spacing(1);

        let container_bg = move |theme: &iced::Theme| {
            let is_dark = matches!(theme, iced::Theme::Dark);
            let bg_color = if i % 2 == 0 {
                if is_dark {
                    glass::DARK_SURFACE
                } else {
                    glass::LIGHT_SURFACE
                }
            } else if is_dark {
                glass::DARK_BG
            } else {
                glass::LIGHT_BG
            };

            container::Style {
                background: Some(bg_color.into()),
                ..Default::default()
            }
        };

        container(row_widget).style(container_bg).into()
    });

    column(std::iter::once(header.into()).chain(body_rows))
        .spacing(0)
        .into()
}

fn render_inline_image<'a>(
    index: usize,
    alt: &'a str,
    state: &'a crate::core::MarkdownState,
) -> Element<'a, Message> {
    if let Some(handle) = state.cached_image_handles.get(&index) {
        let img = image(handle.clone()).height(Length::Shrink);
        let img = if let Some((w, _h)) = state.cached_image_sizes.get(&index) {
            if *w > 600 {
                img.width(Length::Fixed(600.0))
            } else {
                img.width(Length::Shrink)
            }
        } else {
            img.width(Length::Shrink)
        };
        container(img)
            .center_x(Length::Fill)
            .padding([4, 0])
            .width(Length::Fill)
            .into()
    } else {
        // Fallback: show alt text while loading
        container(text(if alt.is_empty() { "[image]" } else { alt }).size(13))
            .padding([2, 8])
            .style(|theme: &iced::Theme| {
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
                    border: iced::Border {
                        color: if is_dark {
                            glass::DARK_BORDER
                        } else {
                            glass::LIGHT_BORDER
                        },
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                }
            })
            .into()
    }
}

fn render_mermaid<'a>(
    index: usize,
    lines: &'a [String],
    _rendered: &Option<Vec<u8>>,
    state: &'a crate::core::MarkdownState,
    font_size: f32,
) -> Element<'a, Message> {
    let badge = container(text("Mermaid Diagram").size(scale(11.0, font_size)))
        .padding([2, 8])
        .style(|_theme: &iced::Theme| container::Style {
            background: Some(glass::ACCENT.into()),
            text_color: Some(Color::WHITE),
            ..Default::default()
        });

    if let Some(handle) = state.cached_mermaid_handles.get(&index) {
        log_debug!("render_mermaid[{}]: handle found, showing image", index);
        let img = container(
            image(handle.clone())
                .width(Length::Shrink)
                .height(Length::Shrink),
        )
        .center_x(Length::Fill)
        .padding(10);

        column![badge, img].spacing(0).into()
    } else {
        log_debug!(
            "render_mermaid[{}]: no handle, showing text fallback",
            index
        );
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
            text(display).size(scale(13.0, font_size)).into()
        });

        let content = container(column(line_widgets).spacing(2))
            .padding(10)
            .width(Length::Fill)
            .style(code_block_style);

        column![badge, content].spacing(0).into()
    }
}

// ==========================================
// 3. MAIN ROUTER & VIEW
// ==========================================

fn render_block<'a>(
    index: usize,
    block: &'a Block,
    state: &'a crate::core::MarkdownState,
    font_size: f32,
) -> Element<'a, Message> {
    match block {
        Block::Heading { level, text: t } => render_heading(*level, t, font_size),
        Block::Paragraph(t) => render_paragraph(t, font_size),
        Block::CodeBlock { lang, code } => render_code_block(lang, code, font_size),
        Block::Table(tbl) => render_table(tbl, font_size),
        Block::Mermaid { lines, rendered } => {
            render_mermaid(index, lines, rendered, state, font_size)
        }
        Block::Image { alt, .. } => render_inline_image(index, alt, state),
    }
}

pub fn view_markdown<'a>(
    blocks: &'a [Block],
    state: &'a crate::core::MarkdownState,
    font_size: f32,
) -> Element<'a, Message> {
    let content = blocks
        .iter()
        .enumerate()
        .map(|(i, block)| render_block(i, block, state, font_size));

    let inner = column(content).spacing(6).padding(15);

    scrollable(inner)
        .id("content_scroll")
        .on_scroll(on_scrolled)
        .style(crate::ui::theme::glass_scrollable)
        .height(Length::Fill)
        .into()
}
