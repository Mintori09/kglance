use crate::app::Message;
use crate::features::markdown::view::components::inline_spans::{
    SpanCtx, inlines_to_spans_with_links,
};
use crate::parsers::markdown::Inline;
use crate::ui::types::RenderContext;
use iced::Element;

pub fn render_inlines<'a>(
    inlines: &'a [Inline],
    font_size: f32,
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    render_inlines_aligned(
        inlines,
        font_size,
        None,
        None,
        iced::alignment::Horizontal::Left,
        ctx,
    )
}

pub fn render_inlines_styled<'a>(
    inlines: &'a [Inline],
    font_size: f32,
    base_weight: Option<iced::font::Weight>,
    base_color: Option<iced::Color>,
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    render_inlines_aligned(
        inlines,
        font_size,
        base_weight,
        base_color,
        iced::alignment::Horizontal::Left,
        ctx,
    )
}

pub fn render_inlines_aligned<'a>(
    inlines: &'a [Inline],
    font_size: f32,
    base_weight: Option<iced::font::Weight>,
    base_color: Option<iced::Color>,
    align_x: iced::alignment::Horizontal,
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    let span_ctx = SpanCtx {
        font_family: ctx.font_family,
        font_family_mono: ctx.font_family_mono,
        search_query: ctx.search_query,
        active_match: ctx.active_match,
        counter: ctx.counter,
        theme: ctx.theme,
        base_weight,
        base_color,
        font_size,
        ruby_display_mode: ctx.ruby_display_mode,
        footnotes: ctx.footnotes,
    };
    let default_text_color = base_color.unwrap_or(ctx.theme.palette().base.text);
    let (spans, links, rubies, tooltips) = inlines_to_spans_with_links(inlines, &span_ctx);
    let blk = ctx.alloc_block_index();
    crate::features::markdown::view::build_selectable_with_links(
        spans,
        links,
        rubies,
        tooltips,
        font_size,
        blk,
        iced::Length::Fill,
        default_text_color,
        ctx.selection_range,
        ctx.drag_active,
        ctx.theme,
        ctx.result_popup,
        align_x,
    )
    .into()
}
