use std::cell::Cell;

use crate::app::Message;
use crate::app::messages::ActionMsg;
use crate::features::markdown::view::blocks::render_block;
use crate::features::markdown::view::highlight::highlight_code;
use crate::features::notebook::types::NotebookCell;
use crate::features::notebook::view::output::render_cell_output;
use crate::ui::components::selectable_text::SelectableText;
use crate::ui::theme::AppTheme;
use crate::ui::theme::font::{get_code_font, get_main_font};
use crate::ui::types::RenderContext;
use iced::widget::{button, column, container, row, text};
use iced::{Element, Length, Padding};

#[allow(clippy::too_many_arguments)]
pub fn render_cell<'a>(
    cell_index: usize,
    cell: &'a NotebookCell,
    md_state: &'a crate::core::MarkdownState,
    font_size: f32,
    theme: AppTheme,
    font_family: Option<&'a str>,
    font_family_mono: Option<&'a str>,
) -> Element<'a, Message> {
    let base = theme.palette().base;
    let roles = theme.palette().roles;
    let code_font = get_code_font(font_family_mono);
    let main_font = get_main_font(font_family);

    match cell {
        NotebookCell::Markdown { blocks, .. } => {
            let next_block = Cell::new(0);
            let counter = Cell::new(0);

            let ctx = RenderContext {
                next_block_index: &next_block,
                selection_range: None,
                drag_active: false,
                search_query: "",
                active_match: 0,
                counter: &counter,
                theme,
                font_size,
                font_family,
                font_family_mono,
                ruby_display_mode: crate::core::config::RubyDisplayMode::default(),
                footnotes: None,
                result_popup: false,
                show_heading_divider: false,
                is_epub: false,
                epub_override_book_font: false,
                content_width: 800.0,
            };

            let mut col = column![].spacing(8);
            for (idx, block) in blocks.iter().enumerate() {
                col = col.push(render_block(idx, block, md_state, &ctx));
            }

            container(col)
                .padding(Padding::from([6, 12]))
                .width(Length::Fill)
                .into()
        }
        NotebookCell::Code {
            source,
            execution_count,
            outputs,
            language,
        } => {
            // Prompt badge: In [n]:
            let prompt_str = if let Some(ec) = execution_count {
                format!("In [{ec}]:")
            } else {
                "In [ ]:".to_string()
            };

            let prompt_badge = text(prompt_str)
                .font(code_font)
                .size((font_size * 0.85).max(11.0))
                .color(roles.link);

            let lang_str = language.as_deref().unwrap_or("code");
            let lang_label = text(lang_str)
                .font(main_font)
                .size((font_size * 0.8).max(11.0))
                .color(base.text_dim);

            let copy_btn = button(
                text("⎘")
                    .font(code_font)
                    .size((font_size * 0.9).max(12.0))
                    .color(base.text),
            )
            .on_press(ActionMsg::CopyText(source.clone()).into())
            .padding(Padding::from([2, 6]))
            .style(move |_: &iced::Theme, status| {
                let is_hovered = matches!(status, iced::widget::button::Status::Hovered);
                button::Style {
                    background: if is_hovered {
                        Some(base.surface_raised.into())
                    } else {
                        None
                    },
                    border: iced::Border {
                        color: if is_hovered {
                            base.border
                        } else {
                            iced::Color::TRANSPARENT
                        },
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                }
            });

            let top_bar = row![
                prompt_badge,
                container(row![lang_label, copy_btn].spacing(8))
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Right)
            ]
            .align_y(iced::Alignment::Center)
            .padding(Padding::from([2, 4]));

            // Code highlight body
            let highlighted = highlight_code(language, source, theme);

            let code_content: Element<'a, Message> = if highlighted.is_empty() {
                text(source.as_str())
                    .font(code_font)
                    .size(font_size)
                    .color(base.text)
                    .into()
            } else {
                let mut all_spans = Vec::new();
                for (line_idx, line) in highlighted.iter().enumerate() {
                    for (color, segment) in line {
                        all_spans.push(
                            iced::widget::text::Span::new(*segment)
                                .font(code_font)
                                .color(*color),
                        );
                    }
                    if line_idx + 1 < highlighted.len() {
                        all_spans.push(
                            iced::widget::text::Span::new("\n")
                                .font(code_font)
                                .color(base.text),
                        );
                    }
                }
                SelectableText::new(all_spans, font_size)
                    .default_text_color(base.text)
                    .width(Length::Fill)
                    .into()
            };

            let code_container = container(code_content)
                .padding(Padding::from([10, 14]))
                .width(Length::Fill)
                .style(move |_: &iced::Theme| container::Style {
                    background: Some(base.surface.into()),
                    border: iced::Border {
                        color: base.border,
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                });

            let mut cell_col = column![top_bar, code_container].spacing(4);

            // Outputs
            if !outputs.is_empty() {
                let mut out_col = column![].spacing(8);
                for (out_idx, out) in outputs.iter().enumerate() {
                    out_col = out_col.push(render_cell_output(
                        out,
                        *execution_count,
                        font_size,
                        theme,
                        font_family_mono,
                        cell_index,
                        out_idx,
                    ));
                }
                let out_container = container(out_col)
                    .padding(Padding {
                        top: 4.0,
                        right: 8.0,
                        bottom: 4.0,
                        left: 18.0,
                    })
                    .width(Length::Fill);
                cell_col = cell_col.push(out_container);
            }

            container(cell_col)
                .padding(Padding::from([4, 6]))
                .width(Length::Fill)
                .into()
        }
        NotebookCell::Raw { source } => {
            let raw_badge = text("Raw")
                .font(code_font)
                .size((font_size * 0.8).max(11.0))
                .color(base.text_dim);

            let raw_container = container(
                text(source.as_str())
                    .font(code_font)
                    .size(font_size)
                    .color(base.text),
            )
            .padding(Padding::from([8, 12]))
            .width(Length::Fill)
            .style(move |_: &iced::Theme| container::Style {
                background: Some(base.surface.into()),
                border: iced::Border {
                    color: base.border,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            });

            container(column![raw_badge, raw_container].spacing(4))
                .padding(Padding::from([4, 6]))
                .width(Length::Fill)
                .into()
        }
    }
}
