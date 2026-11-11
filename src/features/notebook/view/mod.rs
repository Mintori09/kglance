pub mod cell;
pub mod output;

use crate::app::Message;
use crate::features::markdown::view::toc::render_toc_sidebar;
use crate::features::notebook::types::NotebookData;
use crate::features::notebook::view::cell::render_cell;
use crate::ui::components::content_layout::scrollable_content;
use crate::ui::components::sidebar::with_sidebar_layout;
use crate::ui::theme::AppTheme;
use iced::widget::{container, text};
use iced::{Element, Length, Padding};

pub const NOTEBOOK_SCROLL_ID: &str = "content_scroll";
const CONTENT_PADDING: f32 = 24.0;
const CELL_SPACING: f32 = 16.0;

#[allow(clippy::too_many_arguments)]
pub fn view_notebook<'a>(
    data: &'a NotebookData,
    md_state: &'a crate::core::MarkdownState,
    font_size: f32,
    theme: AppTheme,
    font_family: Option<&'a str>,
    font_family_mono: Option<&'a str>,
    max_text_width: Option<f32>,
    window_width: f32,
) -> Element<'a, Message> {
    if data.cells.is_empty() {
        let base = theme.palette().base;
        return container(
            text("Notebook is empty (0 cells)")
                .size(font_size)
                .color(base.text_dim),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into();
    }

    let elements: Vec<Element<'a, Message>> = data
        .cells
        .iter()
        .enumerate()
        .map(|(cell_index, cell)| {
            let el = render_cell(
                cell_index,
                cell,
                md_state,
                font_size,
                theme,
                font_family,
                font_family_mono,
            );
            container(el)
                .padding(Padding {
                    top: 0.0,
                    right: 0.0,
                    bottom: CELL_SPACING,
                    left: 0.0,
                })
                .width(Length::Fill)
                .into()
        })
        .collect();

    let scroll = scrollable_content(
        elements,
        max_text_width,
        CONTENT_PADDING,
        NOTEBOOK_SCROLL_ID,
    )
    .filter_wheel(true)
    .on_wheel(|delta| crate::app::messages::MarkdownMsg::SmoothWheelScrolled(delta).into())
    .on_scroll(|v| {
        crate::app::messages::MarkdownMsg::Scrolled {
            y: v.absolute_offset().y,
            viewport_height: v.bounds().height,
            content_height: v.content_bounds().height,
        }
        .into()
    })
    .build();

    let _ = window_width;

    with_sidebar_layout(
        md_state.toc_visible && !md_state.toc.is_empty(),
        md_state.sidebar_resizing,
        theme,
        || {
            render_toc_sidebar(
                &md_state.toc,
                md_state,
                md_state.scroll_y,
                theme,
                font_family,
            )
        },
        scroll,
        Message::SidebarDragStarted,
    )
}
