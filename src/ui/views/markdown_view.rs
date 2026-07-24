use std::sync::OnceLock;

use crate::app::Message;
use crate::core::TocEntry;
use crate::log_debug;
use crate::parsers::markdown::{Block, Inline, ListItem, TableBlock, flatten_inlines};
use crate::ui::theme::glass;
use iced::font::Weight;
use iced::widget::text::{Rich, Span};
use iced::widget::{button, column, container, image, row, scrollable, text, tooltip};
use iced::{Border, Color, Element, Font, Length, Padding, Pixels, Shadow};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

const CODE_FONT: Font = Font::with_name("JetBrainsMonoNL Nerd Font");

fn syntax_set() -> &'static SyntaxSet {
    static SS: OnceLock<SyntaxSet> = OnceLock::new();
    SS.get_or_init(SyntaxSet::load_defaults_newlines)
}

fn theme_set() -> &'static ThemeSet {
    static TS: OnceLock<ThemeSet> = OnceLock::new();
    TS.get_or_init(ThemeSet::load_defaults)
}

fn syntect_to_iced_color(c: syntect::highlighting::Color) -> Color {
    Color::from_rgba(
        c.r as f32 / 255.0,
        c.g as f32 / 255.0,
        c.b as f32 / 255.0,
        c.a as f32 / 255.0,
    )
}

fn highlight_code<'a>(
    lang: &Option<String>,
    code: &'a str,
    is_dark: bool,
) -> Vec<Vec<(Color, &'a str)>> {
    let ss = syntax_set();
    let ts = theme_set();

    let syntax = lang
        .as_deref()
        .and_then(|l| ss.find_syntax_by_token(l))
        .unwrap_or_else(|| ss.find_syntax_plain_text());

    let theme_name = if is_dark {
        "base16-eighties.dark"
    } else {
        "InspiredGitHub"
    };
    let theme = ts
        .themes
        .get(theme_name)
        .unwrap_or_else(|| &ts.themes["base16-ocean.dark"]);

    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut result = Vec::new();

    for line in LinesWithEndings::from(code) {
        let ranges = highlighter
            .highlight_line(line, ss)
            .unwrap_or_else(|_| vec![]);

        if ranges.is_empty() {
            let fg = theme
                .settings
                .foreground
                .map(syntect_to_iced_color)
                .unwrap_or_else(|| {
                    if is_dark {
                        Color::from_rgb(0.8, 0.8, 0.8)
                    } else {
                        Color::from_rgb(0.2, 0.2, 0.2)
                    }
                });
            let t = line.strip_suffix('\n').unwrap_or(line);
            result.push(vec![(fg, t)]);
            continue;
        }

        let line_spans: Vec<(Color, &'a str)> = ranges
            .iter()
            .map(|(style, text)| {
                let t = text.strip_suffix('\n').unwrap_or(text);
                (syntect_to_iced_color(style.foreground), t)
            })
            .collect();
        result.push(line_spans);
    }

    result
}

fn inlines_to_spans<'a>(inlines: &'a [Inline]) -> Vec<Span<'a, (), Font>> {
    let mut spans = Vec::new();
    for inline in inlines {
        match inline {
            Inline::Text(t) => {
                spans.push(Span::new(t.as_str()));
            }
            Inline::Bold(children) => {
                for s in inlines_to_spans(children) {
                    spans.push(s.font(Font {
                        weight: Weight::Bold,
                        ..Default::default()
                    }));
                }
            }
            Inline::Italic(children) => {
                for s in inlines_to_spans(children) {
                    spans.push(s.font(Font {
                        style: iced::font::Style::Italic,
                        ..Default::default()
                    }));
                }
            }
            Inline::Strikethrough(children) => {
                for s in inlines_to_spans(children) {
                    spans.push(s.strikethrough(true));
                }
            }
            Inline::Code(code) => {
                spans.push(
                    Span::new(code.as_str())
                        .font(CODE_FONT)
                        .color(Color::from_rgb(0.8, 0.35, 0.35)),
                );
            }
            Inline::Link {
                text: link_text, ..
            } => {
                for s in inlines_to_spans(link_text) {
                    spans.push(s.color(Color::from_rgb(0.3, 0.5, 0.9)).underline(true));
                }
            }
            Inline::SoftBreak => {
                spans.push(Span::new(" "));
            }
            Inline::Image { alt, .. } => {
                spans.push(Span::new(format!("[{alt}]")).color(Color::from_rgb(0.5, 0.5, 0.5)));
            }
            Inline::InlineMath(latex) => {
                spans.push(Span::new(latex.as_str()).color(Color::from_rgb(0.5, 0.2, 0.7)));
            }
            Inline::DisplayMath(latex) => {
                spans.push(Span::new(latex.as_str()).color(Color::from_rgb(0.5, 0.2, 0.7)));
            }
        }
    }
    spans
}

fn link_button_style(theme: &iced::Theme, status: button::Status) -> button::Style {
    let is_dark = matches!(theme, iced::Theme::Dark);
    let base = if is_dark {
        Color::from_rgb(0.4, 0.6, 1.0)
    } else {
        Color::from_rgb(0.3, 0.5, 0.9)
    };
    button::Style {
        background: None,
        text_color: match status {
            button::Status::Hovered | button::Status::Pressed => base,
            _ => base,
        },
        border: Border {
            width: 0.0,
            color: Color::TRANSPARENT,
            radius: 0.0.into(),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

fn render_inlines<'a>(inlines: &'a [Inline], font_size: f32) -> Element<'a, Message> {
    let has_link = inlines.iter().any(|i| matches!(i, Inline::Link { .. }));
    let has_math = inlines
        .iter()
        .any(|i| matches!(i, Inline::InlineMath(_) | Inline::DisplayMath(_)));

    if !has_link && !has_math {
        return Rich::with_spans(inlines_to_spans(inlines))
            .size(Pixels(font_size))
            .width(Length::Fill)
            .into();
    }

    let mut elements: Vec<Element<'a, Message>> = Vec::new();
    let mut start = 0;

    for (i, inline) in inlines.iter().enumerate() {
        if let Inline::Link {
            text: link_text,
            url,
        } = inline
        {
            if start < i {
                elements.push(
                    Rich::with_spans(inlines_to_spans(&inlines[start..i]))
                        .size(Pixels(font_size))
                        .width(Length::Shrink)
                        .into(),
                );
            }

            let display = flatten_inlines(link_text);
            let url_clone = url.clone();
            let btn = button(
                iced::widget::text(display)
                    .size(font_size)
                    .color(Color::from_rgb(0.3, 0.5, 0.9)),
            )
            .on_press(Message::OpenLink(url_clone))
            .style(link_button_style)
            .padding(0);

            let tooltip_label = iced::widget::text(url.as_str())
                .size(12)
                .color(Color::WHITE);
            let tooltip_wrapped = iced::widget::container(tooltip_label)
                .padding([4, 8])
                .style(crate::ui::theme::glass::glass_tooltip);
            elements.push(
                tooltip(btn, tooltip_wrapped, tooltip::Position::Top)
                    .gap(6)
                    .into(),
            );

            start = i + 1;
        } else if let Inline::InlineMath(latex) = inline {
            if start < i {
                elements.push(
                    Rich::with_spans(inlines_to_spans(&inlines[start..i]))
                        .size(Pixels(font_size))
                        .width(Length::Shrink)
                        .into(),
                );
            }
            elements.push(iced_math::inline(latex.as_str()));
            start = i + 1;
        } else if let Inline::DisplayMath(latex) = inline {
            if start < i {
                elements.push(
                    Rich::with_spans(inlines_to_spans(&inlines[start..i]))
                        .size(Pixels(font_size))
                        .width(Length::Shrink)
                        .into(),
                );
            }
            elements.push(iced_math::block(latex.as_str()));
            start = i + 1;
        }
    }

    if start < inlines.len() {
        elements.push(
            Rich::with_spans(inlines_to_spans(&inlines[start..]))
                .size(Pixels(font_size))
                .width(Length::Shrink)
                .into(),
        );
    }

    let mut wrap = iced_aw::Wrap::new().spacing(4).line_spacing(4);
    for el in elements {
        wrap = wrap.push(el);
    }
    wrap.into()
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

fn render_heading<'a>(level: u8, content: &'a [Inline], font_size: f32) -> Element<'a, Message> {
    let raw: f32 = match level {
        1 => 32.0,
        2 => 24.0,
        3 => 20.0,
        _ => 16.0,
    };
    let size = scale(raw, font_size);
    let (pt, pb) = match level {
        1 => (24.0, 12.0),
        2 => (20.0, 8.0),
        3 => (12.0, 4.0),
        _ => (8.0, 4.0),
    };

    let label = render_inlines(content, size);
    let heading = container(label)
        .padding(Padding {
            top: pt,
            right: 0.0,
            bottom: pb,
            left: 0.0,
        })
        .width(Length::Fill);

    if level == 1 || level == 2 {
        let div = container(text(""))
            .style(move |theme: &iced::Theme| {
                let d = matches!(theme, iced::Theme::Dark);
                let color = if d {
                    glass::DARK_BORDER
                } else {
                    glass::LIGHT_BORDER
                };
                container::Style {
                    background: Some(color.into()),
                    ..Default::default()
                }
            })
            .height(1)
            .width(Length::Fill);
        column![heading, div].spacing(4).into()
    } else {
        heading.into()
    }
}

fn render_paragraph<'a>(content: &'a [Inline], font_size: f32) -> Element<'a, Message> {
    let rich = render_inlines(content, font_size);
    container(rich).padding([2, 0]).width(Length::Fill).into()
}

fn render_code_block<'a>(
    lang: &'a Option<String>,
    code: &'a str,
    font_size: f32,
    is_dark: bool,
) -> Element<'a, Message> {
    let lang_str = lang.as_deref().unwrap_or("");
    let copy_code = code.to_string();
    let copy_btn = button(text("Copy").font(CODE_FONT).size(scale(11.0, font_size)))
        .on_press(Message::CopyCode(copy_code))
        .style(|theme: &iced::Theme, status: button::Status| {
            let d = matches!(theme, iced::Theme::Dark);
            let bg = if d { glass::DARK_BG } else { glass::LIGHT_BG };
            button::Style {
                background: Some(match status {
                    button::Status::Hovered | button::Status::Pressed => {
                        iced::Background::Color(Color { a: 0.3, ..bg })
                    }
                    _ => iced::Background::Color(Color { a: 0.0, ..bg }),
                }),
                text_color: match status {
                    button::Status::Hovered => {
                        if d {
                            glass::DARK_TEXT
                        } else {
                            glass::LIGHT_TEXT
                        }
                    }
                    _ => {
                        if d {
                            glass::DARK_TEXT_DIM
                        } else {
                            glass::LIGHT_TEXT_DIM
                        }
                    }
                },
                border: Border {
                    radius: 4.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                shadow: Shadow::default(),
                snap: false,
            }
        })
        .padding([2, 8]);

    let top_bar = if !lang_str.is_empty() {
        row![
            container(text(lang_str).font(CODE_FONT).size(scale(11.0, font_size)))
                .padding([2, 8])
                .style(|theme: &iced::Theme| {
                    let d = matches!(theme, iced::Theme::Dark);
                    container::Style {
                        background: Some((if d { glass::DARK_BG } else { glass::LIGHT_BG }).into()),
                        text_color: Some(if d {
                            glass::DARK_TEXT_DIM
                        } else {
                            glass::LIGHT_TEXT_DIM
                        }),
                        ..Default::default()
                    }
                }),
            container(copy_btn)
                .padding(0)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Right),
        ]
        .into()
    } else {
        Element::from(row![
            container(copy_btn)
                .padding(0)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Right)
        ])
    };

    let highlighted = highlight_code(lang, code, is_dark);

    let code_lines: Vec<Element<'a, Message>> = highlighted
        .iter()
        .map(|line_spans| {
            let spans: Vec<Element<'a, Message>> = line_spans
                .iter()
                .map(|(color, span_text)| {
                    text(*span_text)
                        .font(CODE_FONT)
                        .size(scale(13.0, font_size))
                        .color(*color)
                        .into()
                })
                .collect();
            row(spans).into()
        })
        .collect();

    column![
        top_bar,
        container(column(code_lines))
            .padding(10)
            .width(Length::Fill)
            .style(code_block_style),
    ]
    .spacing(0)
    .into()
}

fn render_table<'a>(table: &'a TableBlock, font_size: f32, _is_dark: bool) -> Element<'a, Message> {
    let hdr_size = scale(14.0, font_size);
    let cel_size = scale(13.0, font_size);

    let col_weights: Vec<u16> = {
        let n = table.headers.len();
        if n == 0 {
            vec![]
        } else {
            let mut max_lens = vec![0usize; n];
            for (i, h) in table.headers.iter().enumerate() {
                max_lens[i] = flatten_inlines(&h.content).len();
            }
            for row in &table.rows {
                for (i, c) in row.iter().enumerate().take(n) {
                    max_lens[i] = max_lens[i].max(flatten_inlines(&c.content).len());
                }
            }
            max_lens
                .iter()
                .map(|&l| (l.max(3) as u16).min(20))
                .collect()
        }
    };

    let header_cells: Vec<Element<'a, Message>> = table
        .headers
        .iter()
        .enumerate()
        .map(|(i, h)| {
            let w = if i < col_weights.len() {
                Length::FillPortion(col_weights[i])
            } else {
                Length::FillPortion(1)
            };
            let cell = render_inlines(&h.content, hdr_size);
            container(cell).padding([8, 12]).width(w).into()
        })
        .collect();

    let header = container(row(header_cells).spacing(0)).style(move |theme: &iced::Theme| {
        let d = matches!(theme, iced::Theme::Dark);
        let bg = if d {
            Color::from_rgb(0.2, 0.22, 0.25)
        } else {
            Color::from_rgb(0.9, 0.91, 0.93)
        };
        container::Style {
            background: Some(bg.into()),
            text_color: Some(if d {
                Color::from_rgb(0.95, 0.95, 0.95)
            } else {
                Color::from_rgb(0.2, 0.2, 0.2)
            }),
            ..Default::default()
        }
    });

    let mut children: Vec<Element<'a, Message>> = Vec::new();
    children.push(header.into());

    if !table.rows.is_empty() {
        children.push(
            container(text(""))
                .style(move |theme: &iced::Theme| {
                    let d = matches!(theme, iced::Theme::Dark);
                    let color = if d {
                        Color::from_rgba(0.45, 0.47, 0.5, 0.6)
                    } else {
                        Color::from_rgba(0.6, 0.62, 0.65, 0.5)
                    };
                    container::Style {
                        background: Some(color.into()),
                        ..Default::default()
                    }
                })
                .height(1)
                .width(Length::Fill)
                .into(),
        );
    }

    for (i, row_data) in table.rows.iter().enumerate() {
        let cells: Vec<Element<'a, Message>> = row_data
            .iter()
            .enumerate()
            .map(|(j, c)| {
                let w = if j < col_weights.len() {
                    Length::FillPortion(col_weights[j])
                } else {
                    Length::FillPortion(1)
                };
                let cell = render_inlines(&c.content, cel_size);
                container(cell).padding([8, 12]).width(w).into()
            })
            .collect();

        let row_widget = row(cells).spacing(0);

        let bg_style = move |theme: &iced::Theme| {
            let d = matches!(theme, iced::Theme::Dark);
            let bg = if i % 2 == 0 {
                if d {
                    glass::DARK_SURFACE
                } else {
                    glass::LIGHT_SURFACE
                }
            } else if d {
                glass::DARK_BG
            } else {
                glass::LIGHT_BG
            };
            container::Style {
                background: Some(bg.into()),
                ..Default::default()
            }
        };

        children.push(container(row_widget).style(bg_style).into());
    }

    let table_content = column(children).spacing(0);

    container(table_content)
        .width(Length::Fill)
        .style(move |theme: &iced::Theme| {
            let d = matches!(theme, iced::Theme::Dark);
            container::Style {
                border: Border {
                    color: if d {
                        Color::from_rgb(0.45, 0.47, 0.5)
                    } else {
                        Color::from_rgb(0.6, 0.62, 0.65)
                    },
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            }
        })
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
        .padding([4, 10])
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
                text_color: Some(if is_dark {
                    glass::DARK_TEXT_DIM
                } else {
                    glass::LIGHT_TEXT_DIM
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
                ..Default::default()
            }
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
            text(display)
                .font(CODE_FONT)
                .size(scale(13.0, font_size))
                .into()
        });

        let content = container(column(line_widgets).spacing(2))
            .padding(10)
            .width(Length::Fill)
            .style(code_block_style);

        column![badge, content].spacing(0).into()
    }
}

fn render_list<'a>(
    ordered: bool,
    start_number: u64,
    items: &'a [ListItem],
    state: &'a crate::core::MarkdownState,
    font_size: f32,
    is_dark: bool,
) -> Element<'a, Message> {
    let item_elements = items.iter().enumerate().map(|(idx, item)| {
        let prefix_el: Element<'a, Message> = if let Some(checked) = item.is_task {
            let symbol = if checked { "[x] " } else { "[ ] " };
            let color = if checked {
                if is_dark {
                    Color::from_rgb(0.4, 0.8, 0.4)
                } else {
                    Color::from_rgb(0.1, 0.6, 0.2)
                }
            } else if is_dark {
                glass::DARK_TEXT_DIM
            } else {
                glass::LIGHT_TEXT_DIM
            };
            text(symbol)
                .font(CODE_FONT)
                .size(font_size)
                .color(color)
                .into()
        } else if ordered {
            text(format!("{}. ", start_number + idx as u64))
                .size(font_size)
                .color(Color::from_rgb(0.5, 0.5, 0.5))
                .into()
        } else {
            text("• ")
                .size(font_size)
                .color(Color::from_rgb(0.5, 0.5, 0.5))
                .into()
        };

        let content_el = render_inlines(&item.content, font_size);

        let mut children: Vec<Element<'a, Message>> =
            vec![row![prefix_el, content_el].spacing(6).into()];
        for (bi, sub) in item.sub_blocks.iter().enumerate() {
            let sub_el = render_block(idx * 1000 + bi, sub, state, font_size, is_dark);
            children.push(
                container(sub_el)
                    .padding(Padding {
                        top: 2.0,
                        right: 0.0,
                        bottom: 2.0,
                        left: 24.0,
                    })
                    .width(Length::Fill)
                    .into(),
            );
        }

        container(column(children).spacing(2))
            .padding(Padding {
                top: 2.0,
                right: 0.0,
                bottom: 2.0,
                left: 0.0,
            })
            .width(Length::Fill)
            .into()
    });

    column(item_elements).spacing(4).into()
}

fn render_quote<'a>(
    blocks: &'a [Block],
    state: &'a crate::core::MarkdownState,
    font_size: f32,
    is_dark: bool,
) -> Element<'a, Message> {
    let inner: Element<'a, Message> = column(
        blocks
            .iter()
            .enumerate()
            .map(|(i, b)| render_block(i, b, state, font_size, is_dark)),
    )
    .spacing(4)
    .into();

    let accent_color = if is_dark {
        Color::from_rgb(0.45, 0.5, 0.65)
    } else {
        Color::from_rgb(0.6, 0.5, 0.8)
    };

    let bg = if is_dark {
        glass::DARK_SURFACE
    } else {
        glass::LIGHT_SURFACE
    };

    let content = container(inner)
        .padding([8, 12])
        .style(move |_| container::Style {
            background: Some(bg.into()),
            ..Default::default()
        })
        .width(Length::Fill);

    let bar = container(text(""))
        .width(4)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(accent_color.into()),
            ..Default::default()
        });

    row![bar, content].spacing(0).into()
}

fn render_horizontal_rule<'a>(_font_size: f32) -> Element<'a, Message> {
    container(
        container(text("").size(1))
            .style(move |theme: &iced::Theme| {
                let is_dark = matches!(theme, iced::Theme::Dark);
                container::Style {
                    background: Some(
                        (if is_dark {
                            glass::DARK_BORDER
                        } else {
                            glass::LIGHT_BORDER
                        })
                        .into(),
                    ),
                    ..Default::default()
                }
            })
            .height(1)
            .width(Length::Fill),
    )
    .padding([8, 0])
    .width(Length::Fill)
    .into()
}

fn render_html<'a>(html: &'a str, _font_size: f32) -> Element<'a, Message> {
    container(
        text(format!(
            "[HTML: {}]",
            html.chars().take(80).collect::<String>()
        ))
        .size(12)
        .color(Color::from_rgb(0.5, 0.5, 0.5)),
    )
    .padding([2, 0])
    .width(Length::Fill)
    .into()
}

// ==========================================
// 3. MAIN ROUTER & VIEW
// ==========================================

fn render_block<'a>(
    index: usize,
    block: &'a Block,
    state: &'a crate::core::MarkdownState,
    font_size: f32,
    is_dark: bool,
) -> Element<'a, Message> {
    match block {
        Block::Heading { level, content } => render_heading(*level, content, font_size),
        Block::Paragraph(content) => render_paragraph(content, font_size),
        Block::CodeBlock { lang, code, .. } => render_code_block(lang, code, font_size, is_dark),
        Block::Table(tbl) => render_table(tbl, font_size, is_dark),
        Block::Mermaid { lines, rendered } => {
            render_mermaid(index, lines, rendered, state, font_size)
        }
        Block::Image { alt, .. } => render_inline_image(index, alt, state),
        Block::List {
            ordered,
            start_number,
            items,
        } => render_list(*ordered, *start_number, items, state, font_size, is_dark),
        Block::Quote(blocks) => render_quote(blocks, state, font_size, is_dark),
        Block::HorizontalRule => render_horizontal_rule(font_size),
        Block::Html(html) => render_html(html, font_size),
    }
}

fn block_margin(block: &Block) -> f32 {
    match block {
        Block::Heading { level, .. } if *level == 1 => 24.0,
        Block::Heading { level, .. } if *level == 2 => 20.0,
        Block::Heading { .. } => 16.0,
        Block::HorizontalRule => 24.0,
        Block::CodeBlock { .. } => 16.0,
        Block::Table(_) => 16.0,
        Block::List { .. } => 12.0,
        Block::Quote(_) => 16.0,
        Block::Image { .. } => 16.0,
        Block::Mermaid { .. } => 16.0,
        Block::Paragraph(_) | Block::Html(_) => 8.0,
    }
}

fn render_toc_tooltip_style(theme: &iced::Theme) -> container::Style {
    let d = matches!(theme, iced::Theme::Dark);
    container::Style {
        background: Some(
            (if d {
                glass::DARK_SURFACE
            } else {
                glass::LIGHT_SURFACE
            })
            .into(),
        ),
        text_color: Some(if d {
            glass::DARK_TEXT
        } else {
            glass::LIGHT_TEXT
        }),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: if d {
                glass::DARK_BORDER
            } else {
                glass::LIGHT_BORDER
            },
        },
        shadow: Shadow {
            offset: iced::Vector::new(0.0, 3.0),
            blur_radius: 10.0,
            color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3),
        },
        ..Default::default()
    }
}

fn render_toc_sidebar<'a>(
    toc: &'a [TocEntry],
    state: &'a crate::core::MarkdownState,
    scroll_y: f32,
    is_dark: bool,
) -> Element<'a, Message> {
    let active_idx = toc.iter().rposition(|e| e.y_offset <= scroll_y + 50.0);

    let mut visible_entries: Vec<&TocEntry> = Vec::new();
    let mut collapsed_depth: Option<u8> = None;

    for entry in toc {
        if matches!(collapsed_depth, Some(depth) if entry.level <= depth) {
            collapsed_depth = None;
        }
        if collapsed_depth.is_none() {
            visible_entries.push(entry);
            if state.collapsed_headings.contains(&entry.block_index) {
                collapsed_depth = Some(entry.level);
            }
        }
    }

    let entries: Vec<Element<'a, Message>> = visible_entries
        .iter()
        .map(|entry| {
            let indent = (entry.level as f32 - 1.0) * 12.0;
            let is_active = active_idx
                .map(|i| toc[i].block_index == entry.block_index)
                .unwrap_or(false);

            // Only show collapse button if this heading has child sub-headings
            let has_children = toc
                .iter()
                .skip_while(|e| e.block_index != entry.block_index)
                .nth(1)
                .map(|next| next.level > entry.level)
                .unwrap_or(false);

            let is_collapsed = state.collapsed_headings.contains(&entry.block_index);

            let item_row: Element<'a, Message> = if has_children {
                let arrow_icon = if is_collapsed { "▶ " } else { "▼ " };
                let collapse_btn = button(text(arrow_icon).size(9).style(move |_| text::Style {
                    color: Some(if is_dark {
                        glass::DARK_TEXT_DIM
                    } else {
                        glass::LIGHT_TEXT_DIM
                    }),
                }))
                .on_press(Message::TocToggleCollapse(entry.block_index))
                .style(|_, _| button::Style {
                    background: None,
                    border: Border::default(),
                    shadow: Shadow::default(),
                    ..Default::default()
                })
                .padding([2, 4]);

                let label = text(&entry.text).size(12);
                let btn = button(label)
                    .on_press(Message::TocHeadingClicked(entry.block_index))
                    .width(Length::Fill)
                    .style(move |theme: &iced::Theme, status: button::Status| {
                        let d = matches!(theme, iced::Theme::Dark);
                        let bg = match status {
                            button::Status::Hovered | button::Status::Pressed => Some(
                                (if d {
                                    Color::from_rgba(1.0, 1.0, 1.0, 0.08)
                                } else {
                                    Color::from_rgba(0.0, 0.0, 0.0, 0.06)
                                })
                                .into(),
                            ),
                            _ => {
                                if is_active {
                                    Some(
                                        (if d {
                                            Color::from_rgba(0.4, 0.7, 1.0, 0.15)
                                        } else {
                                            Color::from_rgba(0.1, 0.4, 0.8, 0.1)
                                        })
                                        .into(),
                                    )
                                } else {
                                    None
                                }
                            }
                        };
                        let text_color = if is_active {
                            if d {
                                Color::from_rgb(0.5, 0.8, 1.0)
                            } else {
                                Color::from_rgb(0.1, 0.45, 0.85)
                            }
                        } else if d {
                            Color::from_rgb(0.8, 0.82, 0.85)
                        } else {
                            Color::from_rgb(0.3, 0.32, 0.35)
                        };
                        button::Style {
                            background: bg,
                            text_color,
                            border: Border {
                                width: 0.0,
                                color: Color::TRANSPARENT,
                                radius: 4.0.into(),
                            },
                            shadow: Shadow::default(),
                            snap: false,
                        }
                    })
                    .padding([4, 4]);

                row![collapse_btn, btn]
                    .align_y(iced::Alignment::Center)
                    .spacing(2)
                    .into()
            } else {
                let label = text(&entry.text).size(12);
                let btn = button(label)
                    .on_press(Message::TocHeadingClicked(entry.block_index))
                    .width(Length::Fill)
                    .style(move |theme: &iced::Theme, status: button::Status| {
                        let d = matches!(theme, iced::Theme::Dark);
                        let bg = match status {
                            button::Status::Hovered | button::Status::Pressed => Some(
                                (if d {
                                    Color::from_rgba(1.0, 1.0, 1.0, 0.08)
                                } else {
                                    Color::from_rgba(0.0, 0.0, 0.0, 0.06)
                                })
                                .into(),
                            ),
                            _ => {
                                if is_active {
                                    Some(
                                        (if d {
                                            Color::from_rgba(0.4, 0.7, 1.0, 0.15)
                                        } else {
                                            Color::from_rgba(0.1, 0.4, 0.8, 0.1)
                                        })
                                        .into(),
                                    )
                                } else {
                                    None
                                }
                            }
                        };
                        let text_color = if is_active {
                            if d {
                                Color::from_rgb(0.5, 0.8, 1.0)
                            } else {
                                Color::from_rgb(0.1, 0.45, 0.85)
                            }
                        } else if d {
                            Color::from_rgb(0.8, 0.82, 0.85)
                        } else {
                            Color::from_rgb(0.3, 0.32, 0.35)
                        };
                        button::Style {
                            background: bg,
                            text_color,
                            border: Border {
                                width: 0.0,
                                color: Color::TRANSPARENT,
                                radius: 4.0.into(),
                            },
                            shadow: Shadow::default(),
                            snap: false,
                        }
                    })
                    .padding([4, 4]);

                // Space placeholder to align with headings that have collapse buttons
                let placeholder = iced::widget::Space::new().width(15);
                row![placeholder, btn]
                    .align_y(iced::Alignment::Center)
                    .spacing(2)
                    .into()
            };

            let wrapped = container(item_row)
                .padding(Padding {
                    top: 0.0,
                    right: 0.0,
                    bottom: 0.0,
                    left: indent,
                })
                .width(Length::Fill);

            wrapped.into()
        })
        .collect();

    let (bg, border_color) = if is_dark {
        (glass::DARK_BG, glass::DARK_BORDER)
    } else {
        (glass::LIGHT_BG, glass::LIGHT_BORDER)
    };

    let title_text = text("Table of Contents")
        .size(12)
        .style(move |_| text::Style {
            color: Some(if is_dark {
                glass::DARK_TEXT
            } else {
                glass::LIGHT_TEXT
            }),
        });

    let tip_badge = container(text("g t").size(10).style(move |_| text::Style {
        color: Some(if is_dark {
            glass::DARK_TEXT_DIM
        } else {
            glass::LIGHT_TEXT_DIM
        }),
    }))
    .padding([2, 6])
    .style(render_toc_tooltip_style);

    let header = row![
        title_text,
        iced::widget::Space::new().width(Length::Fill),
        tip_badge
    ]
    .align_y(iced::Alignment::Center)
    .padding([8, 12]);

    let toc_list = scrollable(column(entries).spacing(2).padding(8))
        .id("toc_scroll")
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new().width(4).margin(2),
        ))
        .style(crate::ui::theme::glass_scrollable)
        .height(Length::Fill);

    container(column![header, toc_list].width(220).height(Length::Fill))
        .width(220)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(bg.into()),
            border: Border {
                width: 1.0,
                color: border_color,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

pub fn view_markdown<'a>(
    blocks: &'a [Block],
    state: &'a crate::core::MarkdownState,
    font_size: f32,
    is_dark: bool,
) -> Element<'a, Message> {
    let content = blocks.iter().enumerate().map(|(i, block)| {
        let inner = render_block(i, block, state, font_size, is_dark);
        let mb = block_margin(block);
        container(inner)
            .padding(Padding {
                top: 0.0,
                right: 0.0,
                bottom: mb,
                left: 0.0,
            })
            .width(Length::Fill)
            .into()
    });

    let inner = column(content).spacing(0).padding(15);

    let scroll = scrollable(inner)
        .id("content_scroll")
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new().width(4).margin(2),
        ))
        .style(crate::ui::theme::glass_scrollable)
        .height(Length::Fill)
        .on_scroll(|v| Message::MarkdownScrolled(v.absolute_offset().y));

    if state.toc_visible && !state.toc.is_empty() {
        let sidebar = render_toc_sidebar(&state.toc, state, state.scroll_y, is_dark);
        row![sidebar, scroll].spacing(0).height(Length::Fill).into()
    } else {
        scroll.into()
    }
}
