use crate::app::Message;
use crate::features::markdown::view::components::{
    render_alert, render_code_block, render_footnote_definition, render_frontmatter,
    render_heading, render_horizontal_rule, render_html, render_inline_image, render_list,
    render_math_block, render_mermaid, render_paragraph, render_quote, render_table,
};
use crate::parsers::markdown::Block;
use crate::ui::types::RenderContext;
use iced::Element;

pub(crate) fn render_block<'a>(
    index: usize,
    block: &'a Block,
    state: &'a crate::core::MarkdownState,
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    match block {
        Block::Frontmatter(entries) => render_frontmatter(entries, ctx),
        Block::Heading { level, content } => render_heading(*level, content, ctx),
        Block::Paragraph(content) => render_paragraph(content, ctx),
        Block::CodeBlock { lang, code, .. } => render_code_block(lang, code, ctx),
        Block::Table(table) => render_table(table, ctx),
        Block::Mermaid { lines, rendered: _ } => render_mermaid(index, lines, state, ctx),
        Block::Image { link_url, .. } => render_inline_image(index, link_url.as_deref(), state),
        Block::List {
            ordered,
            start_number,
            items,
        } => render_list(*ordered, *start_number, items, state, ctx),
        Block::Quote(blocks) => render_quote(blocks, state, ctx),
        Block::Alert { kind, content } => render_alert(*kind, content, state, ctx),
        Block::FootnoteDefinition { label, content } => {
            render_footnote_definition(label, content, state, ctx)
        }
        Block::HorizontalRule => render_horizontal_rule(ctx.theme),
        Block::Html(html) => render_html(html, ctx),
        Block::Math(latex) => render_math_block(latex, ctx),
    }
}

pub(crate) use crate::parsers::markdown::block_margin;
