use super::style::STYLE;
use crate::app::Message;
use crate::core::config::RubyDisplayMode;
use crate::features::markdown::view::blocks::render_block;
use crate::features::markdown::view::components::render_inlines;
use crate::parsers::markdown::{Block, Inline, ListItem};
use crate::ui::components::selectable_text::ruby_top_padding;
use crate::ui::theme::font::get_code_font;
use crate::ui::types::RenderContext;
use iced::widget::{column, container, row, text};
use iced::{Color, Element, Length, Padding};

pub(crate) fn render_list<'a>(
    ordered: bool,
    start_number: u64,
    items: &'a [ListItem],
    state: &'a crate::core::MarkdownState,
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    render_list_level(ordered, start_number, items, state, ctx, 0)
}

fn render_list_level<'a>(
    ordered: bool,
    start_number: u64,
    items: &'a [ListItem],
    state: &'a crate::core::MarkdownState,
    ctx: &RenderContext<'_>,
    level: usize,
) -> Element<'a, Message> {
    let mut item_elements = Vec::with_capacity(items.len());

    for (index, item) in items.iter().enumerate() {
        let has_direct_content =
            !item.content.is_empty() || item.is_task.is_some() || item.sub_blocks.is_empty();

        let mut children: Vec<Element<'a, Message>> = Vec::with_capacity(item.sub_blocks.len() + 1);

        if has_direct_content {
            let has_ruby = has_stacked_ruby(&item.content, ctx.ruby_display_mode);
            let prefix =
                create_list_prefix(ordered, start_number, index, item, ctx, level, has_ruby);
            let content = render_inlines(&item.content, ctx.font_size, ctx);
            children.push(
                row![prefix, content]
                    .spacing(STYLE.list.item_spacing)
                    .into(),
            );
        }

        for (sb_idx, sub_block) in item.sub_blocks.iter().enumerate() {
            let sub_element = if let crate::parsers::markdown::Block::List {
                ordered: sub_ord,
                start_number: sub_start,
                items: sub_items,
            } = sub_block
            {
                render_list_level(*sub_ord, *sub_start, sub_items, state, ctx, level + 1)
            } else {
                render_block(sb_idx, sub_block, state, ctx)
            };
            if !has_direct_content && sb_idx == 0 {
                let has_ruby = block_starts_with_stacked_ruby(sub_block, ctx.ruby_display_mode);
                let prefix =
                    create_list_prefix(ordered, start_number, index, item, ctx, level, has_ruby);
                children.push(
                    row![prefix, sub_element]
                        .spacing(STYLE.list.item_spacing)
                        .into(),
                );
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
    ordered: bool,
    start_number: u64,
    index: usize,
    item: &ListItem,
    ctx: &RenderContext<'_>,
    level: usize,
    has_stacked_ruby: bool,
) -> Element<'a, Message> {
    let base_prefix: Element<'a, Message> = if let Some(checked) = item.is_task {
        let symbol = if checked { "[x] " } else { "[ ] " };
        let color: Color = if checked {
            ctx.theme.palette().roles.success
        } else {
            ctx.theme.palette().base.text_dim
        };
        text(symbol)
            .font(get_code_font(ctx.font_family_mono))
            .size(ctx.font_size)
            .color(color)
            .shaping(iced::widget::text::Shaping::Basic)
            .into()
    } else if ordered {
        let color = ctx.theme.palette().roles.accent;
        text(format!("{}. ", start_number + index as u64))
            .size(ctx.font_size)
            .color(color)
            .shaping(iced::widget::text::Shaping::Basic)
            .into()
    } else {
        let bullet = match level % 3 {
            0 => "• ",
            1 => "◦ ",
            _ => "▪ ",
        };
        let color = ctx.theme.palette().roles.accent;
        text(bullet)
            .size(ctx.font_size)
            .color(color)
            .shaping(iced::widget::text::Shaping::Basic)
            .into()
    };

    if has_stacked_ruby {
        container(base_prefix)
            .padding(Padding {
                top: ruby_top_padding(ctx.font_size),
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
        | Inline::Link { text: children, .. } => children.iter().any(inline_has_ruby),
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
