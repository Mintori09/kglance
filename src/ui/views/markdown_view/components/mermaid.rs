use super::style::{STYLE, code_block_style, mermaid_badge_style};
use crate::app::Message;
use crate::log_debug;
use crate::ui::views::markdown_view::blocks::RenderContext;
use crate::ui::views::shared::font::get_code_font;
use crate::ui::views::shared::theme::scale_size;
use iced::widget::{column, container, image, text};
use iced::{Element, Length};

pub(crate) fn render_mermaid<'a>(
    index: usize,
    lines: &'a [String],
    state: &'a crate::core::MarkdownState,
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    let badge = container(
        text("Mermaid Diagram").size(scale_size(STYLE.mermaid.badge_font_size, ctx.font_size)),
    )
    .padding(STYLE.mermaid.badge_padding)
    .style(mermaid_badge_style);

    if let Some(handle) = state.cached_mermaid_handles.get(&index) {
        log_debug!("render_mermaid[{}]: handle found, showing image", index);
        let image_container = container(
            image(handle.clone())
                .width(Length::Shrink)
                .height(Length::Shrink),
        )
        .center_x(Length::Fill)
        .width(Length::Fill)
        .padding(STYLE.mermaid.image_padding)
        .style(code_block_style);

        column![badge, image_container]
            .spacing(STYLE.general.section_spacing)
            .into()
    } else {
        log_debug!(
            "render_mermaid[{}]: no handle, showing text fallback",
            index
        );
        let code_font = get_code_font(ctx.font_family_mono);
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
                .font(code_font)
                .size(scale_size(STYLE.code.line_font_size, ctx.font_size))
                .into()
        });

        let content = container(column(line_widgets).spacing(STYLE.general.item_spacing_small))
            .padding(STYLE.code.padding)
            .width(Length::Fill)
            .style(code_block_style);

        column![badge, content].spacing(0).into()
    }
}
