use crate::app::Message;
use crate::features::markdown::view::components::inline_spans::{
    SpanCtx, inlines_to_spans_with_links,
};
use crate::parsers::markdown::Inline;
use crate::ui::types::RenderContext;
use iced::Element;

#[derive(Clone, Copy)]
pub struct InlineStyleOptions<'a> {
    pub base_weight: Option<iced::font::Weight>,
    pub base_style: Option<iced::font::Style>,
    pub base_color: Option<iced::Color>,
    pub align_x: iced::alignment::Horizontal,
    pub prefix_text: Option<&'a str>,
}

impl Default for InlineStyleOptions<'_> {
    fn default() -> Self {
        Self {
            base_weight: None,
            base_style: None,
            base_color: None,
            align_x: iced::alignment::Horizontal::Left,
            prefix_text: None,
        }
    }
}

pub fn render_inlines<'a>(
    inlines: &[Inline],
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
    inlines: &[Inline],
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
    inlines: &[Inline],
    font_size: f32,
    base_weight: Option<iced::font::Weight>,
    base_color: Option<iced::Color>,
    align_x: iced::alignment::Horizontal,
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    render_inlines_aligned_with_prefix(
        inlines,
        font_size,
        base_weight,
        base_color,
        align_x,
        None,
        ctx,
    )
}

pub fn render_inlines_aligned_with_prefix<'a>(
    inlines: &[Inline],
    font_size: f32,
    base_weight: Option<iced::font::Weight>,
    base_color: Option<iced::Color>,
    align_x: iced::alignment::Horizontal,
    prefix_text: Option<&str>,
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    let opts = InlineStyleOptions {
        base_weight,
        base_style: None,
        base_color,
        align_x,
        prefix_text,
    };
    render_inlines_with_options(inlines, font_size, opts, ctx)
}

pub fn render_inlines_with_options<'a>(
    inlines: &[Inline],
    font_size: f32,
    opts: InlineStyleOptions<'_>,
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    let span_ctx = SpanCtx {
        font_family: ctx.font_family,
        font_family_mono: ctx.font_family_mono,
        search_query: ctx.search_query,
        active_match: ctx.active_match,
        counter: ctx.counter,
        theme: ctx.theme,
        base_weight: opts.base_weight,
        base_style: opts.base_style,
        base_color: opts.base_color,
        font_size,
        ruby_display_mode: ctx.ruby_display_mode,
        footnotes: ctx.footnotes,
    };
    let default_text_color = opts.base_color.unwrap_or(ctx.theme.palette().base.text);
    let (mut spans, links, rubies, tooltips) = inlines_to_spans_with_links(inlines, &span_ctx);

    if let Some(prefix) = opts.prefix_text {
        let prefix_font = iced::Font {
            weight: opts.base_weight.unwrap_or(iced::font::Weight::Normal),
            style: opts.base_style.unwrap_or(iced::font::Style::Normal),
            ..crate::ui::theme::font::get_main_font(span_ctx.font_family)
        };
        spans.insert(
            0,
            iced::widget::text::Span::new(prefix.to_string())
                .font(prefix_font)
                .color(default_text_color)
                .size(iced::Pixels(font_size)),
        );
    }

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
        opts.align_x,
    )
    .into()
}
