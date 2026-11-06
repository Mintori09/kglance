use super::style::STYLE;
use crate::app::Message;
use crate::core::config::RubyDisplayMode;
use crate::features::markdown::view::blocks::render_block;
use crate::features::markdown::view::components::inline::{
    InlineStyleOptions, render_inlines_with_options, resolve_body_font_size,
};
use crate::parsers::markdown::{Block, Inline, ListItem};
use crate::ui::components::selectable_text::ruby_top_padding;
use crate::ui::types::RenderContext;
use iced::widget::{column, container, row, text};
use iced::{Border, Color, Element, Length, Padding};

struct ListPrefixArgs<'a> {
    ordered: bool,
    start_number: u64,
    index: usize,
    item: &'a ListItem,
    level: usize,
    has_stacked_ruby: bool,
    font_size: f32,
}

pub(crate) fn render_list<'a>(
    ordered: bool,
    start_number: u64,
    items: &'a [ListItem],
    block_style: Option<&crate::features::epub::parser::css::BlockStyle>,
    state: &'a crate::core::MarkdownState,
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    render_list_level(ordered, start_number, items, block_style, state, ctx, 0)
}

fn render_list_level<'a>(
    ordered: bool,
    start_number: u64,
    items: &'a [ListItem],
    block_style: Option<&crate::features::epub::parser::css::BlockStyle>,
    state: &'a crate::core::MarkdownState,
    ctx: &RenderContext<'_>,
    level: usize,
) -> Element<'a, Message> {
    let mut item_elements = Vec::with_capacity(items.len());
    let list_style_type_none = block_style.is_some_and(|s| s.list_style_type_none);
    let font_family = block_style.and_then(|s| s.font_family.as_deref());
    let font_scale = block_style.and_then(|s| s.font_scale);
    let font_size = resolve_body_font_size(font_scale, ctx);

    for (index, item) in items.iter().enumerate() {
        let has_direct_content =
            !item.content.is_empty() || item.is_task.is_some() || item.sub_blocks.is_empty();

        let mut children: Vec<Element<'a, Message>> = Vec::with_capacity(item.sub_blocks.len() + 1);

        if has_direct_content {
            let has_ruby = has_stacked_ruby(&item.content, ctx.ruby_display_mode);
            let (weight, style, color, underline, strikethrough) = match block_style {
                Some(s) => (
                    if s.is_bold {
                        Some(iced::font::Weight::Bold)
                    } else {
                        None
                    },
                    if s.is_italic {
                        Some(iced::font::Style::Italic)
                    } else {
                        None
                    },
                    if ctx.is_epub {
                        None
                    } else {
                        s.color
                            .map(|[r, g, b, a]| iced::Color::from_rgba8(r, g, b, a as f32 / 255.0))
                    },
                    s.is_underline,
                    s.is_strikethrough,
                ),
                None => (None, None, None, false, false),
            };

            let (base_weight, base_color) = if item.is_task == Some(true) {
                (None, Some(ctx.theme.palette().base.text_dim))
            } else {
                (weight, color)
            };

            let opts = InlineStyleOptions {
                base_weight,
                base_style: style,
                base_color,
                align_x: iced::alignment::Horizontal::Left,
                prefix_text: None,
                base_underline: underline,
                base_strikethrough: strikethrough,
                line_height: block_style.and_then(|s| s.line_height),
                font_family,
            };

            let content = render_inlines_with_options(&item.content, font_size, opts, ctx);

            if list_style_type_none {
                children.push(content);
            } else {
                let prefix_args = ListPrefixArgs {
                    ordered,
                    start_number,
                    index,
                    item,
                    level,
                    has_stacked_ruby: has_ruby,
                    font_size,
                };
                let prefix = create_list_prefix(prefix_args, ctx);
                children.push(
                    row![prefix, content]
                        .spacing(STYLE.list.item_spacing)
                        .into(),
                );
            }
        }

        for (sb_idx, sub_block) in item.sub_blocks.iter().enumerate() {
            let sub_element = if let crate::parsers::markdown::Block::List {
                ordered: sub_ord,
                start_number: sub_start,
                items: sub_items,
            } = sub_block
            {
                render_list_level(
                    *sub_ord,
                    *sub_start,
                    sub_items,
                    block_style,
                    state,
                    ctx,
                    level + 1,
                )
            } else {
                render_block(sb_idx, sub_block, state, ctx)
            };
            if !has_direct_content && sb_idx == 0 {
                if list_style_type_none {
                    children.push(sub_element);
                } else {
                    let has_ruby = block_starts_with_stacked_ruby(sub_block, ctx.ruby_display_mode);
                    let prefix_args = ListPrefixArgs {
                        ordered,
                        start_number,
                        index,
                        item,
                        level,
                        has_stacked_ruby: has_ruby,
                        font_size,
                    };
                    let prefix = create_list_prefix(prefix_args, ctx);
                    children.push(
                        row![prefix, sub_element]
                            .spacing(STYLE.list.item_spacing)
                            .into(),
                    );
                }
            } else {
                let left_padding = if has_direct_content {
                    STYLE.list.sub_block_left_padding
                } else {
                    0.0
                };
                children.push(
                    container(sub_element)
                        .padding(Padding {
                            top: STYLE.list.item_padding,
                            right: 0.0,
                            bottom: STYLE.list.item_padding,
                            left: left_padding,
                        })
                        .width(Length::Fill)
                        .into(),
                );
            }
        }

        item_elements.push(
            container(column(children).spacing(STYLE.general.item_spacing_small))
                .padding(Padding {
                    top: STYLE.list.item_padding,
                    right: 0.0,
                    bottom: STYLE.list.item_padding,
                    left: 0.0,
                })
                .width(Length::Fill)
                .into(),
        );
    }

    column(item_elements)
        .spacing(STYLE.list.item_spacing)
        .into()
}

fn create_list_prefix<'a>(
    args: ListPrefixArgs<'a>,
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    let base_prefix: Element<'a, Message> = if let Some(checked) = args.item.is_task {
        let box_size = (args.font_size * 0.9).round().max(14.0);
        let border_color = if checked {
            ctx.theme.palette().roles.accent
        } else {
            ctx.theme.palette().base.border
        };
        let bg_color = if checked {
            ctx.theme.palette().roles.accent
        } else {
            Color::TRANSPARENT
        };
        let checkmark_color = if checked {
            Color::WHITE
        } else {
            Color::TRANSPARENT
        };

        let checkmark = text(if checked { "✓" } else { "" })
            .size((box_size * 0.75).round())
            .color(checkmark_color)
            .shaping(iced::widget::text::Shaping::Basic);

        let checkbox = container(checkmark)
            .width(box_size)
            .height(box_size)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .style(move |_: &iced::Theme| container::Style {
                background: Some(bg_color.into()),
                border: Border {
                    color: border_color,
                    width: 1.5,
                    radius: 3.5.into(),
                },
                ..Default::default()
            });

        // Wrap with a tiny right padding to align naturally with the following text
        container(checkbox)
            .padding(Padding {
                top: 2.0,
                right: 4.0,
                bottom: 0.0,
                left: 0.0,
            })
            .into()
    } else if args.ordered {
        let color = ctx.theme.palette().roles.accent;
        text(format!("{}. ", args.start_number + args.index as u64))
            .size(args.font_size)
            .color(color)
            .shaping(iced::widget::text::Shaping::Basic)
            .into()
    } else {
        let bullet = match args.level % 3 {
            0 => "• ",
            1 => "◦ ",
            _ => "▪ ",
        };
        let color = ctx.theme.palette().roles.accent;
        text(bullet)
            .size(args.font_size)
            .color(color)
            .shaping(iced::widget::text::Shaping::Basic)
            .into()
    };

    if args.has_stacked_ruby {
        container(base_prefix)
            .padding(Padding {
                top: ruby_top_padding(args.font_size),
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            })
            .into()
    } else {
        base_prefix
    }
}

pub(crate) fn has_stacked_ruby(inlines: &[Inline], mode: RubyDisplayMode) -> bool {
    matches!(mode, RubyDisplayMode::Stacked) && inlines.iter().any(inline_has_ruby)
}

fn inline_has_ruby(inline: &Inline) -> bool {
    match inline {
        Inline::Ruby { .. } => true,
        Inline::Bold(children)
        | Inline::Italic(children)
        | Inline::Strikethrough(children)
        | Inline::Underline(children)
        | Inline::Link { text: children, .. }
        | Inline::Styled {
            content: children, ..
        } => children.iter().any(inline_has_ruby),
        Inline::Text(_)
        | Inline::Code(_)
        | Inline::Image { .. }
        | Inline::InlineMath(_)
        | Inline::DisplayMath(_)
        | Inline::FootnoteReference(_)
        | Inline::SoftBreak => false,
    }
}

fn block_starts_with_stacked_ruby(block: &Block, mode: RubyDisplayMode) -> bool {
    match block {
        Block::Paragraph(inlines)
        | Block::Heading {
            content: inlines, ..
        } => has_stacked_ruby(inlines, mode),
        Block::Frontmatter(_)
        | Block::List { .. }
        | Block::Quote(_)
        | Block::CodeBlock { .. }
        | Block::Table(_)
        | Block::Html(_)
        | Block::HorizontalRule
        | Block::Alert { .. }
        | Block::Math(_)
        | Block::Mermaid { .. }
        | Block::FootnoteDefinition { .. }
        | Block::Image { .. } => false,
    }
}

#[cfg(test)]
#[path = "list_tests.rs"]
mod tests;
