use crate::app::Message;
use crate::parsers::markdown::Block;
use crate::ui::types::RenderContext;
use crate::ui::views::markdown_view::components::style::STYLE;
use crate::ui::views::markdown_view::components::{
    render_code_block, render_heading, render_horizontal_rule, render_html, render_inline_image,
    render_list, render_mermaid, render_paragraph, render_quote, render_table,
};
use iced::Element;

pub(crate) fn render_block<'a>(
    index: usize,
    block: &'a Block,
    state: &'a crate::core::MarkdownState,
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    match block {
        Block::Heading { level, content } => render_heading(*level, content, ctx),
        Block::Paragraph(content) => render_paragraph(content, ctx),
        Block::CodeBlock { lang, code, .. } => render_code_block(lang, code, ctx),
        Block::Table(table) => render_table(table, ctx),
        Block::Mermaid { lines, rendered: _ } => render_mermaid(index, lines, state, ctx),
        Block::Image { .. } => render_inline_image(index, state),
        Block::List {
            ordered,
            start_number,
            items,
        } => render_list(*ordered, *start_number, items, state, ctx),
        Block::Quote(blocks) => render_quote(blocks, state, ctx),
        Block::HorizontalRule => render_horizontal_rule(ctx.is_dark),
        Block::Html(html) => render_html(html, ctx),
    }
}

pub(crate) fn block_margin(block: &Block) -> f32 {
    match block {
        Block::Heading { level, .. } if *level == 1 => STYLE.block.heading_h1,
        Block::Heading { level, .. } if *level == 2 => STYLE.block.heading_h2,
        Block::Heading { .. } => STYLE.block.heading_default,
        Block::HorizontalRule => STYLE.block.horizontal_rule,
        Block::CodeBlock { .. } => STYLE.block.code,
        Block::Table(_) => STYLE.block.table,
        Block::Quote(_) => STYLE.block.quote,
        Block::Image { .. } => STYLE.block.image,
        Block::Mermaid { .. } => STYLE.block.mermaid,
        Block::List { .. } => STYLE.block.list,
        Block::Paragraph(_) => STYLE.block.paragraph,
        Block::Html(_) => STYLE.block.html,
    }
}
